//! Diagnóstico da caneta: lista os dispositivos XInput e mostra a pressão
//! que chega durante alguns segundos (Linux/X11 ou XWayland).
//!
//! `cargo run --example caneta_diag -- 15` (segundos; padrão 10).
//! Sob Wayland, a mesa só entrega pressão a janelas X: encoste a caneta
//! numa janela X (ex. o FeatherSheet com a pressão ligada) enquanto roda.
fn main() {
    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
    {
        use feather_ui::caneta;
        let segundos: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(10);
        match caneta::x11::listar_dispositivos() {
            Ok(linhas) => {
                println!("Dispositivos XInput ({}):", linhas.len());
                for l in linhas {
                    println!("  {l}");
                }
            }
            Err(e) => {
                println!("XInput2 indisponível: {e}");
                return;
            }
        }
        caneta::definir_diagnostico(true);
        let fonte = caneta::FontePressao::nova();
        match caneta::x11::iniciar(fonte.clone()) {
            Ok(d) => println!("Ouvindo por {segundos} s: {d}"),
            Err(e) => {
                println!("Falhou: {e}");
                return;
            }
        }
        let fim = std::time::Instant::now() + std::time::Duration::from_secs(segundos);
        while std::time::Instant::now() < fim {
            std::thread::sleep(std::time::Duration::from_millis(200));
            for linha in caneta::recolher_diagnostico() {
                println!("{linha}");
            }
        }
        println!("Fim. Última amostra: {:?}", fonte.ultima().map(|a| a.pressao));
    }
    #[cfg(not(all(unix, not(target_os = "macos"), not(target_os = "android"))))]
    println!("Este diagnóstico é do Linux (XInput2). No Windows use o FeatherSheet com FEATHERSHEET_CANETA_DIAG=1.");
}
