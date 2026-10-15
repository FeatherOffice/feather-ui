//! Teste da caneta falando DIRETO com o compositor Wayland — sem Iced e sem winit.
//!
//! Abre uma janela xdg-toplevel com buffer shm, faz bind de `wl_seat`,
//! `wl_pointer` e `zwp_tablet_manager_v2` (protocolo tablet-v2) e imprime cada
//! evento da mesa e do ponteiro. Desenha o traço da caneta em azul (espessura
//! pela pressão, 1..10 px) e o do mouse em preto (botão esquerdo apertado).
//! Ao fim de N segundos (argumento, padrão 60) imprime um resumo.
//!
//! Uso: `cargo run --example caneta_wayland -- 90`
//!
//! Backend Rust puro (`wayland-backend` sem a feature `client_system`): não
//! carrega a libwayland-client do sistema.

#[cfg(target_os = "linux")]
fn main() {
    linux::main();
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("caneta_wayland: só existe no Linux (Wayland).");
}

#[cfg(target_os = "linux")]
mod linux {
    use std::collections::BTreeMap;
    use std::fs::File;
    use std::os::fd::{AsFd, AsRawFd};
    use std::os::unix::fs::FileExt;
    use std::time::{Duration, Instant};

    use wayland_client::globals::{registry_queue_init, GlobalListContents};
    use wayland_client::protocol::{
        wl_buffer, wl_callback, wl_compositor, wl_pointer, wl_registry, wl_seat, wl_shm,
        wl_shm_pool, wl_surface,
    };
    use wayland_client::{
        delegate_noop, event_created_child, Connection, Dispatch, Proxy, QueueHandle, WEnum,
    };
    use wayland_protocols::wp::tablet::zv2::client::{
        zwp_tablet_manager_v2, zwp_tablet_pad_group_v2, zwp_tablet_pad_ring_v2,
        zwp_tablet_pad_strip_v2, zwp_tablet_pad_v2, zwp_tablet_seat_v2, zwp_tablet_tool_v2,
        zwp_tablet_v2,
    };
    use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

    const LARGURA: usize = 900;
    const ALTURA: usize = 600;
    const FUNDO: u32 = 0xFF_F7_F7_F2;
    const AZUL: u32 = 0xFF_1A_4D_E6;
    const PRETO: u32 = 0xFF_10_10_10;

    struct Estado {
        inicio: Instant,
        contagem: BTreeMap<&'static str, u64>,
        pressao_min: f64,
        pressao_max: f64,
        // Caneta
        caneta_perto: bool,
        caneta_baixa: bool,
        caneta_pos: (f64, f64),
        caneta_pressao: f64,
        caneta_ultimo: Option<(f64, f64)>,
        ponteiro_com_caneta_perto: u64,
        // Ponteiro
        ponteiro: Option<wl_pointer::WlPointer>,
        mouse_pos: (f64, f64),
        mouse_baixo: bool,
        // Superfície
        superficie: Option<wl_surface::WlSurface>,
        buffer: Option<wl_buffer::WlBuffer>,
        arquivo: Option<File>,
        tela: Vec<u32>,
        sujo: bool,
        esperando_quadro: bool,
        configurado: bool,
        fechar: bool,
    }

    impl Estado {
        fn log(&self, texto: impl AsRef<str>) {
            println!("{:8.3} {}", self.inicio.elapsed().as_secs_f64(), texto.as_ref());
        }
        fn conta(&mut self, nome: &'static str) {
            *self.contagem.entry(nome).or_insert(0) += 1;
        }
        fn disco(&mut self, cx: f64, cy: f64, raio: f64, cor: u32) {
            let r = raio.max(0.5);
            let (x0, x1) = ((cx - r).floor() as i64, (cx + r).ceil() as i64);
            let (y0, y1) = ((cy - r).floor() as i64, (cy + r).ceil() as i64);
            for y in y0.max(0)..=y1.min(ALTURA as i64 - 1) {
                for x in x0.max(0)..=x1.min(LARGURA as i64 - 1) {
                    let (dx, dy) = (x as f64 + 0.5 - cx, y as f64 + 0.5 - cy);
                    if dx * dx + dy * dy <= r * r {
                        self.tela[y as usize * LARGURA + x as usize] = cor;
                    }
                }
            }
        }
        fn segmento(&mut self, a: (f64, f64), b: (f64, f64), espessura: f64, cor: u32) {
            let dist = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            let passos = (dist / 0.5).ceil().max(1.0) as usize;
            for i in 0..=passos {
                let t = i as f64 / passos as f64;
                self.disco(a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, espessura / 2.0, cor);
            }
            self.sujo = true;
        }
        fn apresentar(&mut self, qh: &QueueHandle<Estado>) {
            if !self.configurado || !self.sujo || self.esperando_quadro {
                return;
            }
            let (Some(sup), Some(buf), Some(arq)) = (&self.superficie, &self.buffer, &self.arquivo)
            else {
                return;
            };
            let bytes: Vec<u8> = self.tela.iter().flat_map(|p| p.to_le_bytes()).collect();
            arq.write_all_at(&bytes, 0).expect("gravar no buffer shm");
            sup.attach(Some(buf), 0, 0);
            sup.damage_buffer(0, 0, LARGURA as i32, ALTURA as i32);
            sup.frame(qh, ());
            sup.commit();
            self.sujo = false;
            self.esperando_quadro = true;
        }
    }

    pub fn main() {
        let segundos: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(60);
        let conn = Connection::connect_to_env().expect("WAYLAND_DISPLAY: sem compositor Wayland");
        let (globais, mut fila) = registry_queue_init::<Estado>(&conn).expect("registro");
        let qh = fila.handle();

        let mut estado = Estado {
            inicio: Instant::now(),
            contagem: BTreeMap::new(),
            pressao_min: f64::MAX,
            pressao_max: f64::MIN,
            caneta_perto: false,
            caneta_baixa: false,
            caneta_pos: (0.0, 0.0),
            caneta_pressao: 0.0,
            caneta_ultimo: None,
            ponteiro_com_caneta_perto: 0,
            ponteiro: None,
            mouse_pos: (0.0, 0.0),
            mouse_baixo: false,
            superficie: None,
            buffer: None,
            arquivo: None,
            tela: vec![FUNDO; LARGURA * ALTURA],
            sujo: true,
            esperando_quadro: false,
            configurado: false,
            fechar: false,
        };

        // Globais anunciadas pelo compositor (para o relatório).
        let mut nomes: Vec<String> = globais
            .contents()
            .clone_list()
            .into_iter()
            .map(|g| format!("{} v{}", g.interface, g.version))
            .collect();
        nomes.sort();
        estado.log(format!("globais: {}", nomes.join(", ")));

        let compositor: wl_compositor::WlCompositor =
            globais.bind(&qh, 4..=6, ()).expect("wl_compositor");
        let shm: wl_shm::WlShm = globais.bind(&qh, 1..=1, ()).expect("wl_shm");
        let wm: xdg_wm_base::XdgWmBase = globais.bind(&qh, 1..=5, ()).expect("xdg_wm_base");
        let seat: wl_seat::WlSeat = globais.bind(&qh, 1..=7, ()).expect("wl_seat");
        match globais.bind::<zwp_tablet_manager_v2::ZwpTabletManagerV2, _, _>(&qh, 1..=1, ()) {
            Ok(gerente) => {
                estado.log("zwp_tablet_manager_v2: ANUNCIADO, bind v1 → get_tablet_seat");
                gerente.get_tablet_seat(&seat, &qh, ());
            }
            Err(e) => estado.log(format!("zwp_tablet_manager_v2: AUSENTE ({e})")),
        }

        // Buffer shm num arquivo em XDG_RUNTIME_DIR (apagado logo depois de aberto).
        let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
        let caminho = format!("{dir}/caneta_wayland_{}", std::process::id());
        let arquivo = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&caminho)
            .expect("arquivo shm");
        let _ = std::fs::remove_file(&caminho);
        let tamanho = LARGURA * ALTURA * 4;
        arquivo.set_len(tamanho as u64).expect("tamanho shm");
        let pool = shm.create_pool(arquivo.as_fd(), tamanho as i32, &qh, ());
        let buffer = pool.create_buffer(
            0,
            LARGURA as i32,
            ALTURA as i32,
            (LARGURA * 4) as i32,
            wl_shm::Format::Argb8888,
            &qh,
            (),
        );

        let superficie = compositor.create_surface(&qh, ());
        let xdg_sup = wm.get_xdg_surface(&superficie, &qh, ());
        let topo = xdg_sup.get_toplevel(&qh, ());
        topo.set_title(format!("Caneta Wayland — escreva aqui ({segundos} s)"));
        topo.set_app_id("org.featheroffice.CanetaWayland".into());
        topo.set_min_size(LARGURA as i32, ALTURA as i32);
        topo.set_max_size(LARGURA as i32, ALTURA as i32);
        superficie.commit();
        estado.superficie = Some(superficie);
        estado.buffer = Some(buffer);
        estado.arquivo = Some(arquivo);

        let limite = Duration::from_secs(segundos);
        while estado.inicio.elapsed() < limite && !estado.fechar {
            fila.dispatch_pending(&mut estado).expect("despachar");
            estado.apresentar(&qh);
            fila.flush().expect("flush");
            let Some(guarda) = fila.prepare_read() else { continue };
            let restante = limite.saturating_sub(estado.inicio.elapsed());
            let mut pfd = libc::pollfd {
                fd: guarda.connection_fd().as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            let ms = restante.as_millis().min(i32::MAX as u128) as i32;
            let r = unsafe { libc::poll(&mut pfd, 1, ms) };
            if r > 0 {
                let _ = guarda.read();
            } else {
                drop(guarda);
            }
        }

        let e = &estado;
        println!("\n===== RESUMO ({:.1} s) =====", e.inicio.elapsed().as_secs_f64());
        for (nome, n) in &e.contagem {
            println!("  {nome:<28} {n}");
        }
        if e.pressao_max >= 0.0 {
            println!("  pressão (0..1): mín {:.4}  máx {:.4}", e.pressao_min, e.pressao_max);
        } else {
            println!("  pressão: nenhum evento");
        }
        println!(
            "  eventos de ponteiro com a caneta em proximidade: {} ({})",
            e.ponteiro_com_caneta_perto,
            if e.ponteiro_com_caneta_perto > 0 {
                "o compositor TAMBÉM emulou ponteiro para a caneta"
            } else {
                "a caneta NÃO gerou ponteiro nesta superfície"
            }
        );
        drop(pool);
    }

    // ---------- registro, compositor, shm, xdg ----------

    impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Estado {
        fn event(
            _: &mut Self,
            _: &wl_registry::WlRegistry,
            _: wl_registry::Event,
            _: &GlobalListContents,
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
        }
    }
    delegate_noop!(Estado: wl_compositor::WlCompositor);
    delegate_noop!(Estado: wl_surface::WlSurface);
    delegate_noop!(Estado: ignore wl_shm::WlShm);
    delegate_noop!(Estado: wl_shm_pool::WlShmPool);
    delegate_noop!(Estado: ignore wl_buffer::WlBuffer);
    delegate_noop!(Estado: zwp_tablet_manager_v2::ZwpTabletManagerV2);

    impl Dispatch<wl_callback::WlCallback, ()> for Estado {
        fn event(
            s: &mut Self,
            _: &wl_callback::WlCallback,
            _: wl_callback::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            s.esperando_quadro = false;
        }
    }

    impl Dispatch<xdg_wm_base::XdgWmBase, ()> for Estado {
        fn event(
            _: &mut Self,
            wm: &xdg_wm_base::XdgWmBase,
            ev: xdg_wm_base::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            if let xdg_wm_base::Event::Ping { serial } = ev {
                wm.pong(serial);
            }
        }
    }

    impl Dispatch<xdg_surface::XdgSurface, ()> for Estado {
        fn event(
            s: &mut Self,
            xs: &xdg_surface::XdgSurface,
            ev: xdg_surface::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            if let xdg_surface::Event::Configure { serial } = ev {
                xs.ack_configure(serial);
                if !s.configurado {
                    s.log("janela configurada (xdg_surface.configure) — visível");
                }
                s.configurado = true;
                s.sujo = true;
            }
        }
    }

    impl Dispatch<xdg_toplevel::XdgToplevel, ()> for Estado {
        fn event(
            s: &mut Self,
            _: &xdg_toplevel::XdgToplevel,
            ev: xdg_toplevel::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            if let xdg_toplevel::Event::Close = ev {
                s.log("janela fechada pelo usuário");
                s.fechar = true;
            }
        }
    }

    // ---------- seat e ponteiro ----------

    impl Dispatch<wl_seat::WlSeat, ()> for Estado {
        fn event(
            s: &mut Self,
            seat: &wl_seat::WlSeat,
            ev: wl_seat::Event,
            _: &(),
            _: &Connection,
            qh: &QueueHandle<Self>,
        ) {
            match ev {
                wl_seat::Event::Capabilities { capabilities: WEnum::Value(c) } => {
                    s.log(format!("wl_seat capacidades: {c:?}"));
                    if c.contains(wl_seat::Capability::Pointer) && s.ponteiro.is_none() {
                        s.ponteiro = Some(seat.get_pointer(qh, ()));
                    }
                }
                wl_seat::Event::Name { name } => s.log(format!("wl_seat nome: {name}")),
                _ => {}
            }
        }
    }

    impl Dispatch<wl_pointer::WlPointer, ()> for Estado {
        fn event(
            s: &mut Self,
            _: &wl_pointer::WlPointer,
            ev: wl_pointer::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            use wl_pointer::Event as E;
            let nome = match &ev {
                E::Enter { .. } => "ponteiro.enter",
                E::Leave { .. } => "ponteiro.leave",
                E::Motion { .. } => "ponteiro.motion",
                E::Button { .. } => "ponteiro.button",
                E::Axis { .. } => "ponteiro.axis",
                E::Frame => "ponteiro.frame",
                _ => "ponteiro.outro",
            };
            s.conta(nome);
            if s.caneta_perto && nome != "ponteiro.frame" {
                s.ponteiro_com_caneta_perto += 1;
            }
            match ev {
                E::Enter { surface_x, surface_y, .. } => {
                    s.mouse_pos = (surface_x, surface_y);
                    s.log(format!("ponteiro enter ({surface_x:.1}, {surface_y:.1})"));
                }
                E::Leave { .. } => {
                    s.mouse_baixo = false;
                    s.log("ponteiro leave");
                }
                E::Motion { surface_x, surface_y, .. } => {
                    let novo = (surface_x, surface_y);
                    if s.mouse_baixo {
                        let antigo = s.mouse_pos;
                        s.segmento(antigo, novo, 2.0, PRETO);
                    }
                    s.mouse_pos = novo;
                    s.log(format!(
                        "ponteiro motion ({surface_x:.1}, {surface_y:.1}){}",
                        if s.caneta_perto { "  [caneta em proximidade]" } else { "" }
                    ));
                }
                E::Button { button, state, .. } => {
                    let apertado = matches!(state, WEnum::Value(wl_pointer::ButtonState::Pressed));
                    if button == 0x110 {
                        s.mouse_baixo = apertado;
                    }
                    s.log(format!(
                        "ponteiro button 0x{button:x} {}{}",
                        if apertado { "apertado" } else { "solto" },
                        if s.caneta_perto { "  [caneta em proximidade]" } else { "" }
                    ));
                }
                _ => {}
            }
        }
    }

    // ---------- tablet-v2 ----------

    impl Dispatch<zwp_tablet_seat_v2::ZwpTabletSeatV2, ()> for Estado {
        fn event(
            s: &mut Self,
            _: &zwp_tablet_seat_v2::ZwpTabletSeatV2,
            ev: zwp_tablet_seat_v2::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            use zwp_tablet_seat_v2::Event as E;
            match ev {
                E::TabletAdded { id } => {
                    s.conta("mesa.tablet_added");
                    s.log(format!("tablet_added id={}", id.id()));
                }
                E::ToolAdded { id } => {
                    s.conta("mesa.tool_added");
                    s.log(format!("tool_added id={}", id.id()));
                }
                E::PadAdded { id } => {
                    s.conta("mesa.pad_added");
                    s.log(format!("pad_added id={}", id.id()));
                }
                _ => {}
            }
        }

        event_created_child!(Estado, zwp_tablet_seat_v2::ZwpTabletSeatV2, [
            zwp_tablet_seat_v2::EVT_TABLET_ADDED_OPCODE => (zwp_tablet_v2::ZwpTabletV2, ()),
            zwp_tablet_seat_v2::EVT_TOOL_ADDED_OPCODE => (zwp_tablet_tool_v2::ZwpTabletToolV2, ()),
            zwp_tablet_seat_v2::EVT_PAD_ADDED_OPCODE => (zwp_tablet_pad_v2::ZwpTabletPadV2, ()),
        ]);
    }

    impl Dispatch<zwp_tablet_v2::ZwpTabletV2, ()> for Estado {
        fn event(
            s: &mut Self,
            mesa: &zwp_tablet_v2::ZwpTabletV2,
            ev: zwp_tablet_v2::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            use zwp_tablet_v2::Event as E;
            let id = mesa.id();
            match ev {
                E::Name { name } => s.log(format!("  mesa {id}: nome \"{name}\"")),
                E::Id { vid, pid } => s.log(format!("  mesa {id}: vid=0x{vid:04x} pid=0x{pid:04x}")),
                E::Path { path } => s.log(format!("  mesa {id}: caminho {path}")),
                E::Done => s.log(format!("  mesa {id}: done")),
                E::Removed => s.log(format!("  mesa {id}: removida")),
                outro => s.log(format!("  mesa {id}: {outro:?}")),
            }
        }
    }

    impl Dispatch<zwp_tablet_tool_v2::ZwpTabletToolV2, ()> for Estado {
        fn event(
            s: &mut Self,
            ferramenta: &zwp_tablet_tool_v2::ZwpTabletToolV2,
            ev: zwp_tablet_tool_v2::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            use zwp_tablet_tool_v2::Event as E;
            let id = ferramenta.id();
            match ev {
                E::Type { tool_type } => s.log(format!("  ferramenta {id}: tipo {tool_type:?}")),
                E::HardwareSerial { hardware_serial_hi, hardware_serial_lo } => s.log(format!(
                    "  ferramenta {id}: serial 0x{hardware_serial_hi:x}{hardware_serial_lo:08x}"
                )),
                E::HardwareIdWacom { hardware_id_hi, hardware_id_lo } => s.log(format!(
                    "  ferramenta {id}: id wacom 0x{hardware_id_hi:x}{hardware_id_lo:08x}"
                )),
                E::Capability { capability } => {
                    s.log(format!("  ferramenta {id}: capacidade {capability:?}"))
                }
                E::Done => s.log(format!("  ferramenta {id}: done")),
                E::Removed => s.log(format!("  ferramenta {id}: removida")),
                E::ProximityIn { serial, tablet, surface } => {
                    s.conta("caneta.proximity_in");
                    s.caneta_perto = true;
                    // Cursor nulo: a ponta não mostra seta (só para o teste).
                    ferramenta.set_cursor(serial, None, 0, 0);
                    s.log(format!(
                        "caneta proximity_in (mesa {}, superfície {})",
                        tablet.id(),
                        surface.id()
                    ));
                }
                E::ProximityOut => {
                    s.conta("caneta.proximity_out");
                    s.caneta_perto = false;
                    s.caneta_baixa = false;
                    s.caneta_ultimo = None;
                    s.log("caneta proximity_out");
                }
                E::Down { .. } => {
                    s.conta("caneta.down");
                    s.caneta_baixa = true;
                    s.caneta_ultimo = Some(s.caneta_pos);
                    s.log(format!("caneta DOWN em ({:.1}, {:.1})", s.caneta_pos.0, s.caneta_pos.1));
                }
                E::Up => {
                    s.conta("caneta.up");
                    s.caneta_baixa = false;
                    s.caneta_ultimo = None;
                    s.log("caneta UP");
                }
                E::Motion { x, y } => {
                    s.conta("caneta.motion");
                    s.caneta_pos = (x, y);
                    s.log(format!("caneta motion ({x:.1}, {y:.1})"));
                }
                E::Pressure { pressure } => {
                    s.conta("caneta.pressure");
                    let p = pressure as f64 / 65535.0;
                    s.caneta_pressao = p;
                    s.pressao_min = s.pressao_min.min(p);
                    s.pressao_max = s.pressao_max.max(p);
                    s.log(format!("caneta pressure {pressure} → {p:.4}"));
                }
                E::Distance { distance } => {
                    s.conta("caneta.distance");
                    s.log(format!("caneta distance {distance}"));
                }
                E::Tilt { tilt_x, tilt_y } => {
                    s.conta("caneta.tilt");
                    s.log(format!("caneta tilt ({tilt_x:.1}°, {tilt_y:.1}°)"));
                }
                E::Button { button, state, .. } => {
                    s.conta("caneta.button");
                    s.log(format!("caneta button 0x{button:x} {state:?}"));
                }
                E::Frame { time } => {
                    s.conta("caneta.frame");
                    if s.caneta_baixa {
                        if let Some(antes) = s.caneta_ultimo {
                            let esp = 1.0 + 9.0 * s.caneta_pressao;
                            let agora = s.caneta_pos;
                            s.segmento(antes, agora, esp, AZUL);
                        }
                        s.caneta_ultimo = Some(s.caneta_pos);
                    }
                    s.log(format!("caneta frame t={time}"));
                }
                outro => {
                    s.conta("caneta.outro");
                    s.log(format!("caneta {outro:?}"));
                }
            }
        }
    }

    impl Dispatch<zwp_tablet_pad_v2::ZwpTabletPadV2, ()> for Estado {
        fn event(
            s: &mut Self,
            _: &zwp_tablet_pad_v2::ZwpTabletPadV2,
            ev: zwp_tablet_pad_v2::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            s.conta("pad.evento");
            s.log(format!("  pad: {ev:?}"));
        }

        event_created_child!(Estado, zwp_tablet_pad_v2::ZwpTabletPadV2, [
            zwp_tablet_pad_v2::EVT_GROUP_OPCODE => (zwp_tablet_pad_group_v2::ZwpTabletPadGroupV2, ()),
        ]);
    }

    impl Dispatch<zwp_tablet_pad_group_v2::ZwpTabletPadGroupV2, ()> for Estado {
        fn event(
            s: &mut Self,
            _: &zwp_tablet_pad_group_v2::ZwpTabletPadGroupV2,
            ev: zwp_tablet_pad_group_v2::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            s.log(format!("  pad grupo: {ev:?}"));
        }

        event_created_child!(Estado, zwp_tablet_pad_group_v2::ZwpTabletPadGroupV2, [
            zwp_tablet_pad_group_v2::EVT_RING_OPCODE => (zwp_tablet_pad_ring_v2::ZwpTabletPadRingV2, ()),
            zwp_tablet_pad_group_v2::EVT_STRIP_OPCODE => (zwp_tablet_pad_strip_v2::ZwpTabletPadStripV2, ()),
        ]);
    }

    delegate_noop!(Estado: ignore zwp_tablet_pad_ring_v2::ZwpTabletPadRingV2);
    delegate_noop!(Estado: ignore zwp_tablet_pad_strip_v2::ZwpTabletPadStripV2);
}
