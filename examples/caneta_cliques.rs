//! Diagnóstico dos CLIQUES da caneta numa janela X (XWayland ou X11).
//!
//! Abre uma janela X com duas metades e imprime o que o servidor entrega:
//!
//! - **metade esquerda (clara)**: eventos XInput2 — `XI_ButtonPress/Release`,
//!   `XI_Motion`, `XI_Enter/Leave` de *todos os mestres* e de *todos os
//!   dispositivos*, e `XI_TouchBegin/Update/End` dos mestres. É o que o
//!   winit 0.30 (Iced 0.13) seleciona na janela dele, mais os escravos;
//! - **metade direita (escura)**: eventos *core* (`ButtonPress`,
//!   `ButtonRelease`, `MotionNotify`), sem XInput2.
//!
//! Para cada evento: tipo, `deviceid`, `sourceid` (com o nome), botão
//! (`detail`), `flags` (marca `EMULADO` quando há `XIPointerEmulated`, que o
//! winit descarta), posição e a pressão, quando a origem tem esse eixo.
//! Movimentos são resumidos (o primeiro de cada origem e um a cada 40).
//!
//! `cargo run --example caneta_cliques -- 20` (segundos; padrão 20).
//! Encoste o MOUSE e depois a CANETA nas duas metades e compare.
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
    use x11rb::protocol::xinput::{self, ConnectionExt as _, DeviceClassData, XIEventMask};
    use x11rb::protocol::xproto::{
        AtomEnum, ConnectionExt as _, CreateWindowAux, EventMask, PropMode, WindowClass,
    };
    use x11rb::protocol::Event;
    use x11rb::rust_connection::RustConnection;
    use x11rb::wrapper::ConnectionExt as _;

    const LARGURA: u16 = 900;
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

    fn pressao(mapa: &HashMap<u16, Disp>, origem: u16, mascara: &[u32], valores: &[xinput::Fp3232]) -> String {
        let Some((eixo, min, max)) = mapa.get(&origem).and_then(|d| d.pressao) else { return String::new() };
        match valor_do_eixo(mascara, valores, eixo) {
            Some(v) if max > min => format!(" pressão {v} ({:.3})", (v - min) / (max - min)),
            Some(v) => format!(" pressão {v}"),
            None => " pressão (sem eixo no evento)".into(),
        }
    }

    pub fn rodar() {
        let segundos: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(20);
        if let Err(e) = executar(segundos) {
            println!("Falhou: {e}");
        }
    }

    fn executar(segundos: u64) -> Result<(), Box<dyn std::error::Error>> {
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

        // Janela principal com duas filhas: esquerda XInput2, direita core.
        let principal = conn.generate_id()?;
        conn.create_window(
            0,
            principal,
            raiz,
            0,
            0,
            LARGURA,
            ALTURA,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().background_pixel(preto),
        )?;
        let titulo = b"caneta_cliques: esquerda XInput2 | direita core";
        conn.change_property8(PropMode::REPLACE, principal, AtomEnum::WM_NAME, AtomEnum::STRING, titulo)?;
        let metade = LARGURA / 2;
        let esquerda = conn.generate_id()?;
        conn.create_window(
            0,
            esquerda,
            principal,
            0,
            0,
            metade - 2,
            ALTURA,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().background_pixel(branco),
        )?;
        let direita = conn.generate_id()?;
        conn.create_window(
            0,
            direita,
            principal,
            i16::try_from(metade + 2)?,
            0,
            metade - 2,
            ALTURA,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().background_pixel(preto).event_mask(
                EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::POINTER_MOTION
                    | EventMask::ENTER_WINDOW
                    | EventMask::LEAVE_WINDOW,
            ),
        )?;

        // XInput2 na metade esquerda. Mestres (1) = o que o winit seleciona;
        // todos (0) = também os escravos (a caneta, o mouse físico).
        let ponteiro = XIEventMask::BUTTON_PRESS | XIEventMask::BUTTON_RELEASE | XIEventMask::MOTION;
        let toque = XIEventMask::TOUCH_BEGIN | XIEventMask::TOUCH_UPDATE | XIEventMask::TOUCH_END;
        let mestres = ponteiro | XIEventMask::ENTER | XIEventMask::LEAVE;
        let com_toque = [
            xinput::EventMask { deviceid: 1, mask: vec![mestres | toque] },
            xinput::EventMask { deviceid: 0, mask: vec![ponteiro] },
        ];
        match conn.xinput_xi_select_events(esquerda, &com_toque)?.check() {
            Ok(()) => println!("XInput2 selecionado (com toque) na metade esquerda"),
            Err(e) => {
                println!("Toque recusado ({e:?}); seguindo sem toque");
                let sem_toque = [
                    xinput::EventMask { deviceid: 1, mask: vec![mestres] },
                    xinput::EventMask { deviceid: 0, mask: vec![ponteiro] },
                ];
                conn.xinput_xi_select_events(esquerda, &sem_toque)?.check()?;
            }
        }
        // Troca de dispositivos (a caneta entra/sai de proximidade no XWayland).
        conn.xinput_xi_select_events(raiz, &[xinput::EventMask { deviceid: 0, mask: vec![XIEventMask::HIERARCHY] }])?
            .check()?;

        conn.map_window(esquerda)?;
        conn.map_window(direita)?;
        conn.map_window(principal)?;
        conn.flush()?;
        println!("Janela aberta por {segundos} s. Encoste o MOUSE e a CANETA nas duas metades.\n");

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
                    *contagem
                        .entry(format!("{tipo} {alvo} origem {} \"{}\"{}", e.sourceid, nome(&mapa, e.sourceid), if emulado { " EMULADO" } else { "" }))
                        .or_default() += 1;
                    if tipo == "XI_Motion" {
                        let n = movimentos.entry((tipo, e.deviceid, e.sourceid)).or_default();
                        *n += 1;
                        if *n % 40 != 1 {
                            continue;
                        }
                    }
                    println!(
                        "{t:8.3}s {tipo:<16} disp {:>2} ({}) origem {:>2} \"{}\" botão {} flags {flags:#x}{} em ({:.1}, {:.1}){}",
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
