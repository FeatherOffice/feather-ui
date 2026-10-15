//! Diagnóstico dos CLIQUES da caneta numa janela X (XWayland ou X11).
//!
//! Abre uma janela X com três colunas e imprime o que o servidor entrega:
//!
//! - **coluna esquerda (clara)**: XInput2 de *todos os mestres*
//!   (`XI_ButtonPress/Release`, `XI_Motion`, `XI_Enter/Leave`,
//!   `XI_TouchBegin/Update/End`) — o mesmo que o winit 0.30 (Iced 0.13)
//!   seleciona na janela dele;
//! - **coluna do meio (clara)**: XInput2 de *todos os dispositivos*
//!   (inclui os escravos: a caneta, o mouse físico);
//! - **coluna direita (escura)**: eventos *core* (`ButtonPress`,
//!   `ButtonRelease`, `MotionNotify`), sem XInput2.
//!
//! Para cada evento: tipo, `deviceid`, `sourceid` (com o nome), botão
//! (`detail`), `flags` (marca `EMULADO` quando há `XIPointerEmulated`, que o
//! winit descarta), posição e a pressão, quando a origem tem esse eixo.
//! Movimentos são resumidos (o primeiro de cada origem e um a cada 40).
//!
//! A janela também DESENHA o traço: nas colunas claras, preto = mouse, azul =
//! caneta com botão apertado (largura pela pressão), vermelho = caneta com
//! pressão mas SEM botão (o toque não virou clique); na coluna escura
//! (core), branco com o botão 1 apertado.
//!
//! `cargo run --example caneta_cliques -- 120` (segundos; padrão 120). A
//! janela abre perto do ponteiro (no monitor em uso).
//! `cargo run --example caneta_cliques -- --autoteste`: arrasto sintético
//! pelo XTest na coluna esquerda, confere se chegou ButtonPress e traço.
//! Encoste o MOUSE e depois a CANETA nas três colunas e compare.
fn main() {
    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
    linux::rodar();
    #[cfg(not(all(unix, not(target_os = "macos"), not(target_os = "android"))))]
    println!("Este diagnóstico é do Linux (X11/XWayland).");
}

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
mod linux {
    use feather_ui::caneta::x11::{rotulo_de_pressao, valor_do_eixo};
    use std::collections::HashMap;
    use std::time::{Duration, Instant};
    use x11rb::connection::Connection;
    use x11rb::properties::{WmSizeHints, WmSizeHintsSpecification};
    use x11rb::protocol::xinput::{self, ConnectionExt as _, DeviceClassData, XIEventMask};
    use x11rb::protocol::xproto::{
        AtomEnum, CapStyle, ChangeGCAux, ConnectionExt as _, CoordMode, CreateGCAux, CreateWindowAux, EventMask,
        ButtonPressEvent, JoinStyle, KeyButMask, Point, PropMode, WindowClass,
    };
    use x11rb::protocol::xproto::{BUTTON_PRESS_EVENT, BUTTON_RELEASE_EVENT, MOTION_NOTIFY_EVENT};
    use x11rb::protocol::xtest::ConnectionExt as _;
    use x11rb::protocol::Event;
    use x11rb::rust_connection::RustConnection;
    use x11rb::wrapper::ConnectionExt as _;

    const LARGURA: u16 = 1200;
    const ALTURA: u16 = 450;

    /// Dispositivo conhecido: nome e, se houver, o eixo de pressão.
    struct Disp {
        nome: String,
        pressao: Option<(u16, f64, f64)>,
    }

    fn fp(v: xinput::Fp3232) -> f64 {
        f64::from(v.integral) + f64::from(v.frac) / 4_294_967_296.0
    }

    fn fp16(v: i32) -> f64 {
        f64::from(v) / 65536.0
    }

    fn nome_atomo(conn: &RustConnection, atomo: u32) -> String {
        if atomo == 0 {
            return String::new();
        }
        conn.get_atom_name(atomo)
            .ok()
            .and_then(|c| c.reply().ok())
            .map(|r| String::from_utf8_lossy(&r.name).into_owned())
            .unwrap_or_default()
    }

    fn dispositivos(conn: &RustConnection) -> HashMap<u16, Disp> {
        let mut mapa = HashMap::new();
        let Some(r) = conn.xinput_xi_query_device(0u16).ok().and_then(|c| c.reply().ok()) else { return mapa };
        for info in r.infos {
            let mut pressao = None;
            for classe in &info.classes {
                if let DeviceClassData::Valuator(v) = &classe.data {
                    if pressao.is_none() && rotulo_de_pressao(&nome_atomo(conn, v.label)) {
                        pressao = Some((v.number, fp(v.min), fp(v.max)));
                    }
                }
            }
            println!(
                "  id {:>2} anexo {:>2} {:?} \"{}\"{}",
                info.deviceid,
                info.attachment,
                info.type_,
                String::from_utf8_lossy(&info.name),
                pressao.map(|(e, a, b)| format!(" pressão no eixo {e} ({a}..{b})")).unwrap_or_default()
            );
            mapa.insert(info.deviceid, Disp { nome: String::from_utf8_lossy(&info.name).into_owned(), pressao });
        }
        mapa
    }

    fn nome(mapa: &HashMap<u16, Disp>, id: u16) -> String {
        mapa.get(&id).map(|d| d.nome.clone()).unwrap_or_else(|| "?".into())
    }

    fn valor_pressao(mapa: &HashMap<u16, Disp>, origem: u16, mascara: &[u32], valores: &[xinput::Fp3232]) -> Option<f32> {
        let (eixo, min, max) = mapa.get(&origem).and_then(|d| d.pressao)?;
        let v = valor_do_eixo(mascara, valores, eixo)?;
        (max > min).then(|| ((v - min) / (max - min)).clamp(0.0, 1.0) as f32)
    }

    fn pressao(mapa: &HashMap<u16, Disp>, origem: u16, mascara: &[u32], valores: &[xinput::Fp3232]) -> String {
        let Some((eixo, min, max)) = mapa.get(&origem).and_then(|d| d.pressao) else { return String::new() };
        match valor_do_eixo(mascara, valores, eixo) {
            Some(v) if max > min => format!(" pressão {v} ({:.3})", (v - min) / (max - min)),
            Some(v) => format!(" pressão {v}"),
            None => " pressão (sem eixo no evento)".into(),
        }
    }

    pub fn rodar() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let autoteste = args.iter().any(|a| a == "--autoteste");
        let segundos: u64 =
            args.iter().find_map(|s| s.parse().ok()).unwrap_or(if autoteste { 3 } else { 120 });
        if let Err(e) = executar(segundos, autoteste) {
            println!("Falhou: {e}");
        }
    }

    fn executar(segundos: u64, autoteste: bool) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "WAYLAND_DISPLAY={:?} DISPLAY={:?}",
            std::env::var("WAYLAND_DISPLAY").ok(),
            std::env::var("DISPLAY").ok()
        );
        let (conn, tela) = RustConnection::connect(None)?;
        let v = conn.xinput_xi_query_version(2, 2)?.reply()?;
        println!("XInput {}.{}", v.major_version, v.minor_version);
        let screen = &conn.setup().roots[tela];
        let raiz = screen.root;
        let (branco, preto) = (screen.white_pixel, screen.black_pixel);

        println!("Dispositivos:");
        let mut mapa = dispositivos(&conn);

        // Janela principal com três filhas (XInput2 mestres, XInput2 todos, core).
        // Nasce onde o ponteiro está: com dois monitores o GNOME a abria no
        // outro e a pessoa não achava a janela dentro do tempo.
        let ponteiro_raiz = conn.query_pointer(raiz)?.reply()?;
        let px = (i32::from(ponteiro_raiz.root_x) - i32::from(LARGURA) / 2).max(0);
        let py = (i32::from(ponteiro_raiz.root_y) - i32::from(ALTURA) / 2).max(0);
        let principal = conn.generate_id()?;
        conn.create_window(
            0,
            principal,
            raiz,
            i16::try_from(px).unwrap_or(0),
            i16::try_from(py).unwrap_or(0),
            LARGURA,
            ALTURA,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().background_pixel(preto),
        )?;
        // USPosition: o gerenciador de janelas respeita a posição pedida.
        let mut dicas = WmSizeHints::new();
        dicas.position = Some((WmSizeHintsSpecification::UserSpecified, px, py));
        dicas.set_normal_hints(&conn, principal)?;
        println!("Janela em ({px}, {py}) na raiz, perto do ponteiro ({}, {}).", ponteiro_raiz.root_x, ponteiro_raiz.root_y);
        let titulo = format!("Escreva com a CANETA e com o MOUSE — {segundos} s");
        let net_wm_name = conn.intern_atom(false, b"_NET_WM_NAME")?.reply()?.atom;
        let utf8 = conn.intern_atom(false, b"UTF8_STRING")?.reply()?.atom;
        conn.change_property8(PropMode::REPLACE, principal, net_wm_name, utf8, titulo.as_bytes())?;
        let ascii = format!("Escreva com a CANETA e com o MOUSE - {segundos} s");
        conn.change_property8(PropMode::REPLACE, principal, AtomEnum::WM_NAME, AtomEnum::STRING, ascii.as_bytes())?;
        // Três colunas: esquerda = XI2 dos mestres (o que o winit seleciona),
        // meio = XI2 de todos os dispositivos, direita = eventos core.
        let terco = LARGURA / 3;
        let filha = |x: u16, fundo: u32, mascara: EventMask| -> Result<u32, Box<dyn std::error::Error>> {
            let id = conn.generate_id()?;
            conn.create_window(
                0,
                id,
                principal,
                i16::try_from(x)?,
                0,
                terco - 3,
                ALTURA,
                0,
                WindowClass::INPUT_OUTPUT,
                0,
                &CreateWindowAux::new().background_pixel(fundo).event_mask(mascara),
            )?;
            Ok(id)
        };
        let esquerda = filha(0, branco, EventMask::EXPOSURE)?;
        let meio = filha(terco, branco, EventMask::EXPOSURE)?;
        let direita = filha(
            2 * terco,
            preto,
            EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE
                | EventMask::POINTER_MOTION
                | EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW
                | EventMask::EXPOSURE,
        )?;

        // Pincéis: nas colunas claras, preto = mouse, azul = caneta com botão
        // apertado, vermelho = caneta com pressão MAS sem botão (o toque não
        // virou clique). Na coluna escura (core, sem origem): branco.
        let cmap = screen.default_colormap;
        let cor = |r: u16, g: u16, b: u16| -> Result<u32, Box<dyn std::error::Error>> {
            Ok(conn.alloc_color(cmap, r, g, b)?.reply()?.pixel)
        };
        let azul = cor(0x1000, 0x4000, 0xf000)?;
        let vermelho = cor(0xe000, 0x1000, 0x1000)?;
        let novo_gc = |pixel: u32| -> Result<u32, Box<dyn std::error::Error>> {
            let gc = conn.generate_id()?;
            conn.create_gc(
                gc,
                principal,
                &CreateGCAux::new().foreground(pixel).line_width(2).cap_style(CapStyle::ROUND).join_style(JoinStyle::ROUND),
            )?;
            Ok(gc)
        };
        let gc_mouse = novo_gc(preto)?;
        let gc_caneta = novo_gc(azul)?;
        let gc_sem_botao = novo_gc(vermelho)?;
        let gc_core = novo_gc(branco)?;
        let mut ultimo: HashMap<(u32, u16), (i16, i16)> = HashMap::new();
        let mut ultimo_core: Option<(i16, i16)> = None;
        let mut tracos = 0u64;

        // XInput2: cada seleção conferida com .check(), erro impresso.
        let ponteiro = XIEventMask::BUTTON_PRESS | XIEventMask::BUTTON_RELEASE | XIEventMask::MOTION;
        let toque = XIEventMask::TOUCH_BEGIN | XIEventMask::TOUCH_UPDATE | XIEventMask::TOUCH_END;
        let mestres = ponteiro | XIEventMask::ENTER | XIEventMask::LEAVE;
        let selecionar = |janela: u32, deviceid: u16, mascara: XIEventMask, rotulo: &str| -> bool {
            let r = conn
                .xinput_xi_select_events(janela, &[xinput::EventMask { deviceid, mask: vec![mascara] }])
                .map_err(|e| format!("{e:?}"))
                .and_then(|c| c.check().map_err(|e| format!("{e:?}")));
            match r {
                Ok(()) => {
                    println!("Seleção XI2 OK: {rotulo}");
                    true
                }
                Err(e) => {
                    println!("Seleção XI2 FALHOU: {rotulo}: {e}");
                    false
                }
            }
        };
        if !selecionar(esquerda, 1, mestres | toque, "coluna esquerda, XIAllMasterDevices, ponteiro+toque") {
            selecionar(esquerda, 1, mestres, "coluna esquerda, XIAllMasterDevices, só ponteiro");
        }
        selecionar(meio, 0, ponteiro | XIEventMask::ENTER | XIEventMask::LEAVE, "coluna do meio, XIAllDevices, ponteiro");
        // Troca de dispositivos (a caneta entra/sai de proximidade no XWayland).
        selecionar(raiz, 0, XIEventMask::HIERARCHY, "raiz, XIAllDevices, hierarquia");

        conn.map_window(esquerda)?;
        conn.map_window(meio)?;
        conn.map_window(direita)?;
        conn.map_window(principal)?;
        conn.flush()?;
        println!("Janela aberta por {segundos} s: escreva com a CANETA e com o MOUSE nas três colunas.");
        println!("Colunas: esquerda = XI2 mestres (igual ao winit), meio = XI2 todos os dispositivos, direita = core.");
        println!("Traço preto = mouse, azul = caneta com botão, vermelho = caneta com pressão SEM botão.\n");

        // Autoteste: clique-arraste sintético pelo XTest na coluna esquerda.
        if autoteste {
            std::thread::sleep(Duration::from_millis(800));
            while conn.poll_for_event()?.is_some() {}
            let base = conn.translate_coordinates(esquerda, raiz, 40, 60)?.reply()?;
            let (x0, y0) = (base.dst_x, base.dst_y);
            match conn.xtest_get_version(2, 2).map(|c| c.reply()) {
                Ok(Ok(v)) => println!("XTEST {}.{}", v.major_version, v.minor_version),
                Ok(Err(e)) => println!("XTEST indisponível: {e:?}"),
                Err(e) => println!("XTEST indisponível: {e:?}"),
            }
            println!("Autoteste: XTest arrasta de ({x0}, {y0}) na raiz (coluna esquerda)");
            conn.xtest_fake_input(MOTION_NOTIFY_EVENT, 0, 0, raiz, x0, y0, 0)?;
            conn.flush()?;
            std::thread::sleep(Duration::from_millis(60));
            conn.xtest_fake_input(BUTTON_PRESS_EVENT, 1, 0, raiz, 0, 0, 0)?;
            for i in 1..=20i16 {
                conn.xtest_fake_input(MOTION_NOTIFY_EVENT, 0, 0, raiz, x0 + i * 8, y0 + i * 5, 0)?;
                conn.flush()?;
                std::thread::sleep(Duration::from_millis(15));
            }
            conn.xtest_fake_input(BUTTON_RELEASE_EVENT, 1, 0, raiz, 0, 0, 0)?;
            conn.flush()?;
            // Sob o GNOME o XWayland pode não aplicar o XTest. Para conferir ao
            // menos o desenho, manda um arrasto core sintético (SendEvent) à
            // coluna direita: chega marcado como "enviado", sem XInput2.
            println!("Autoteste: SendEvent core (ButtonPress + 10 MotionNotify com botão 1 + ButtonRelease) na coluna direita");
            let evento = |tipo: u8, botao: u8, x: i16, y: i16, estado: KeyButMask| ButtonPressEvent {
                response_type: tipo,
                detail: botao,
                sequence: 0,
                time: 0,
                root: raiz,
                event: direita,
                child: 0,
                root_x: 0,
                root_y: 0,
                event_x: x,
                event_y: y,
                state: estado,
                same_screen: true,
            };
            conn.send_event(false, direita, EventMask::NO_EVENT, evento(BUTTON_PRESS_EVENT, 1, 40, 60, KeyButMask::default()))?;
            for i in 1..=10i16 {
                let m = evento(MOTION_NOTIFY_EVENT, 0, 40 + i * 12, 60 + i * 8, KeyButMask::BUTTON1);
                conn.send_event(false, direita, EventMask::NO_EVENT, m)?;
            }
            conn.send_event(false, direita, EventMask::NO_EVENT, evento(BUTTON_RELEASE_EVENT, 1, 160, 140, KeyButMask::BUTTON1))?;
            conn.flush()?;
        }

        let inicio = Instant::now();
        let fim = inicio + Duration::from_secs(segundos);
        let mut movimentos: HashMap<(&'static str, u16, u16), u64> = HashMap::new();
        let mut contagem: HashMap<String, u64> = HashMap::new();
        let mut core_mov = 0u64;
        while Instant::now() < fim {
            let Some(evento) = conn.poll_for_event()? else {
                std::thread::sleep(Duration::from_millis(4));
                continue;
            };
            let t = inicio.elapsed().as_secs_f64();
            match evento {
                Event::XinputButtonPress(e) | Event::XinputButtonRelease(e) | Event::XinputMotion(e) => {
                    let tipo = match e.event_type {
                        xinput::BUTTON_PRESS_EVENT => "XI_ButtonPress",
                        xinput::BUTTON_RELEASE_EVENT => "XI_ButtonRelease",
                        _ => "XI_Motion",
                    };
                    let flags = u32::from(e.flags);
                    let emulado = flags & u32::from(xinput::PointerEventFlags::POINTER_EMULATED) != 0;
                    let alvo = if e.deviceid == e.sourceid { "escravo" } else { "mestre" };
                    let coluna = if e.event == esquerda { "esq" } else if e.event == meio { "meio" } else { "?" };
                    *contagem
                        .entry(format!("[{coluna}] {tipo} {alvo} origem {} \"{}\"{}", e.sourceid, nome(&mapa, e.sourceid), if emulado { " EMULADO" } else { "" }))
                        .or_default() += 1;
                    // Traço: na coluna esquerda pelo evento do mestre, na do
                    // meio pelo do escravo (o outro é o mesmo movimento).
                    if (e.event == esquerda && alvo == "mestre") || (e.event == meio && alvo == "escravo") {
                        let ponto = (fp16(e.event_x) as i16, fp16(e.event_y) as i16);
                        let apertado = e.button_mask.first().is_some_and(|m| m & 0b10 != 0);
                        let p = valor_pressao(&mapa, e.sourceid, &e.valuator_mask, &e.axisvalues);
                        let caneta = mapa.get(&e.sourceid).is_some_and(|d| d.pressao.is_some());
                        let escrevendo = match e.event_type {
                            xinput::BUTTON_PRESS_EVENT => e.detail == 1,
                            xinput::BUTTON_RELEASE_EVENT => false,
                            _ => apertado || (caneta && p.is_some_and(|p| p > 0.0)),
                        };
                        if escrevendo {
                            if let Some(antes) = ultimo.get(&(e.event, e.sourceid)).copied() {
                                let gc = if !caneta {
                                    gc_mouse
                                } else if apertado || e.event_type == xinput::BUTTON_PRESS_EVENT {
                                    gc_caneta
                                } else {
                                    gc_sem_botao
                                };
                                let largura = p.map(|p| (1.0 + 7.0 * p).round() as u32).unwrap_or(2);
                                conn.change_gc(gc, &ChangeGCAux::new().line_width(largura))?;
                                tracos += 1;
                                conn.poly_line(
                                    CoordMode::ORIGIN,
                                    e.event,
                                    gc,
                                    &[Point { x: antes.0, y: antes.1 }, Point { x: ponto.0, y: ponto.1 }],
                                )?;
                                conn.flush()?;
                            }
                            ultimo.insert((e.event, e.sourceid), ponto);
                        } else {
                            ultimo.remove(&(e.event, e.sourceid));
                        }
                    }
                    if tipo == "XI_Motion" {
                        let n = movimentos.entry((tipo, e.deviceid, e.sourceid)).or_default();
                        *n += 1;
                        if *n % 40 != 1 {
                            continue;
                        }
                    }
                    println!(
                        "{t:8.3}s [{coluna}] {tipo:<16} disp {:>2} ({}) origem {:>2} \"{}\" botão {} flags {flags:#x}{} em ({:.1}, {:.1}){}",
                        e.deviceid,
                        alvo,
                        e.sourceid,
                        nome(&mapa, e.sourceid),
                        e.detail,
                        if emulado { " EMULADO(XIPointerEmulated: o winit DESCARTA)" } else { "" },
                        fp16(e.event_x),
                        fp16(e.event_y),
                        pressao(&mapa, e.sourceid, &e.valuator_mask, &e.axisvalues)
                    );
                }
                Event::XinputTouchBegin(e) | Event::XinputTouchUpdate(e) | Event::XinputTouchEnd(e) => {
                    let tipo = match e.event_type {
                        xinput::TOUCH_BEGIN_EVENT => "XI_TouchBegin",
                        xinput::TOUCH_UPDATE_EVENT => "XI_TouchUpdate",
                        _ => "XI_TouchEnd",
                    };
                    *contagem.entry(format!("{tipo} origem {} \"{}\"", e.sourceid, nome(&mapa, e.sourceid))).or_default() += 1;
                    println!(
                        "{t:8.3}s {tipo:<16} disp {:>2} origem {:>2} \"{}\" toque {} flags {:#x} em ({:.1}, {:.1})",
                        e.deviceid,
                        e.sourceid,
                        nome(&mapa, e.sourceid),
                        e.detail,
                        u32::from(e.flags),
                        fp16(e.event_x),
                        fp16(e.event_y)
                    );
                }
                Event::XinputEnter(e) | Event::XinputLeave(e) => {
                    let tipo = if e.event_type == xinput::ENTER_EVENT { "XI_Enter" } else { "XI_Leave" };
                    println!(
                        "{t:8.3}s {tipo:<16} disp {:>2} origem {:>2} \"{}\" modo {:?} em ({:.1}, {:.1})",
                        e.deviceid,
                        e.sourceid,
                        nome(&mapa, e.sourceid),
                        e.mode,
                        fp16(e.event_x),
                        fp16(e.event_y)
                    );
                }
                Event::XinputHierarchy(_) => {
                    println!("{t:8.3}s XI_HierarchyChanged — dispositivos agora:");
                    mapa = dispositivos(&conn);
                }
                Event::ButtonPress(e) | Event::ButtonRelease(e) => {
                    let tipo = if e.response_type & 0x7f == x11rb::protocol::xproto::BUTTON_PRESS_EVENT {
                        "core ButtonPress"
                    } else {
                        "core ButtonRelease"
                    };
                    *contagem.entry(tipo.into()).or_default() += 1;
                    ultimo_core = (tipo == "core ButtonPress" && e.detail == 1).then_some((e.event_x, e.event_y));
                    println!(
                        "{t:8.3}s {tipo:<16} botão {} estado {:#x} em ({}, {})",
                        e.detail,
                        u16::from(e.state),
                        e.event_x,
                        e.event_y
                    );
                }
                Event::MotionNotify(e) => {
                    core_mov += 1;
                    *contagem.entry("core MotionNotify".into()).or_default() += 1;
                    let ponto = (e.event_x, e.event_y);
                    if u16::from(e.state) & u16::from(KeyButMask::BUTTON1) != 0 {
                        if let Some(antes) = ultimo_core {
                            tracos += 1;
                            conn.poly_line(
                                CoordMode::ORIGIN,
                                direita,
                                gc_core,
                                &[Point { x: antes.0, y: antes.1 }, Point { x: ponto.0, y: ponto.1 }],
                            )?;
                            conn.flush()?;
                        }
                        ultimo_core = Some(ponto);
                    } else {
                        ultimo_core = None;
                    }
                    if core_mov % 40 == 1 {
                        println!("{t:8.3}s core MotionNotify estado {:#x} em ({}, {})", u16::from(e.state), e.event_x, e.event_y);
                    }
                }
                Event::EnterNotify(e) => println!("{t:8.3}s core EnterNotify em ({}, {})", e.event_x, e.event_y),
                Event::LeaveNotify(_) => println!("{t:8.3}s core LeaveNotify"),
                Event::Error(e) => println!("{t:8.3}s erro X: {e:?}"),
                _ => {}
            }
        }

        println!("\nTraços desenhados: {tracos}");
        if autoteste {
            let apertos: u64 = contagem.iter().filter(|(k, _)| k.contains("ButtonPress")).map(|(_, n)| n).sum();
            println!(
                "AUTOTESTE: {} (ButtonPress recebidos: {apertos}, traços: {tracos})",
                if apertos > 0 && tracos > 0 { "OK" } else { "FALHOU" }
            );
        }
        println!("\nResumo (eventos por tipo e origem):");
        let mut linhas: Vec<_> = contagem.into_iter().collect();
        linhas.sort();
        for (chave, n) in linhas {
            println!("  {n:>6}  {chave}");
        }
        println!(
            "\nComo ler: se a CANETA só aparece em XI_Motion/Enter, sem XI_ButtonPress de botão 1,\n\
             o toque não vira botão no X. Se aparece XI_ButtonPress com EMULADO, o winit descarta.\n\
             Se aparece XI_TouchBegin, o Iced recebe toque (não clique)."
        );
        Ok(())
    }
}
