//! Pressão da caneta (mesa digitalizadora) para a camada de tinta.
//!
//! O Iced 0.13 / winit 0.30 entregam a caneta como mouse, sem a força.
//! Este módulo lê a pressão por fora, pelo sistema de janelas, e guarda
//! amostras com o instante em que chegaram ([`FontePressao`]); a camada
//! ([`crate::tinta::Camada`]) casa cada ponto do traço com a amostra mais
//! próxima no tempo ([`casar_por_tempo`]).
//!
//! - **Linux**: XInput2 pelo `x11rb` (Rust puro, conexão X própria). Lê
//!   só os eventos *crus* dos dispositivos que têm o eixo de pressão —
//!   nunca o teclado. Para o Wayland entregar a mesa a uma janela X, o
//!   aplicativo precisa rodar pelo XWayland (o app decide; ver
//!   [`deve_usar_x11`] e [`mesas_conectadas`]). Não lê `/dev/input`.
//! - **Windows**: `WM_POINTER*` + `GetPointerPenInfo`, por subclasse da
//!   janela ([`windows::ligar_na_janela`]).
//!
//! Também o **diagnóstico**: com ele ligado, a camada e os leitores anotam
//! os eventos crus num diário que o aplicativo mostra (Console F12).
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use iced::futures::channel::mpsc;
use iced::{Point, Rectangle, Size};

/// Distância máxima no tempo entre o ponto do Iced e a amostra de pressão.
/// Os dois chegam por conexões diferentes ao mesmo servidor, quase juntos.
pub const JANELA_CASAMENTO: Duration = Duration::from_millis(60);
/// Amostras guardadas (algumas centenas de ms de caneta a ~200 Hz).
const MAX_AMOSTRAS: usize = 512;

/// Uma leitura de pressão da ponta, já normalizada para `0..=1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmostraPressao {
    pub instante: Instant,
    pub pressao: f32,
}

#[derive(Default)]
struct Interno {
    amostras: Mutex<VecDeque<AmostraPressao>>,
    descricao: Mutex<String>,
    /// Janela X do aplicativo (0 = desconhecida): a posição da ponta é
    /// lida relativa a ela.
    janela_x11: AtomicU32,
    /// Tamanho lógico (Iced) da janela, para converter pixel físico em lógico.
    tamanho_logico: Mutex<Option<Size>>,
    /// Onde a camada de tinta captura, em coordenadas lógicas da janela.
    /// `None` = modo desenho desligado.
    zona_tinta: Mutex<Option<ZonaTinta>>,
    /// Toques da caneta lidos pelo XInput2, ainda não entregues ao app.
    toques: Mutex<VecDeque<ToqueCaneta>>,
    /// Acorda a assinatura do Iced quando chega toque.
    despertar: Mutex<Option<mpsc::UnboundedSender<()>>>,
    /// Instante em que a ponta encostou (None = levantada).
    contato: Mutex<Option<Instant>>,
    /// Instante do último encostar (fica depois de levantar).
    ultimo_contato: Mutex<Option<Instant>>,
    /// Último sinal da caneta: toque, levantar ou clique sintético. Diz se
    /// um clique que o Iced recebeu veio dela ou do mouse.
    ultimo_sinal: Mutex<Option<Instant>>,
}

/// Onde a camada de tinta está, em coordenadas lógicas da janela: a área
/// que captura e a origem do widget (os pontos da camada são relativos a ela).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZonaTinta {
    pub area: Rectangle,
    pub origem: Point,
}

/// Fase de um toque da ponta da caneta lido por fora do Iced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaseToque {
    Encostou,
    Moveu,
    Levantou,
}

/// Um evento da ponta lido pelo XInput2: posição em coordenadas **lógicas
/// da janela** (as mesmas do Iced), pressão crua `0..=1` (quando veio) e
/// se começou dentro da zona da camada de tinta (`na_tinta`; vale o do
/// encostar para o toque inteiro).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToqueCaneta {
    pub fase: FaseToque,
    pub posicao: Point,
    pub pressao: Option<f32>,
    pub instante: Instant,
    pub na_tinta: bool,
    /// Ponta de borracha da caneta (dispositivo "eraser"), não a de escrever.
    pub borracha: bool,
}

/// Teto da fila de toques (se o app não consumir, os mais antigos saem).
const MAX_TOQUES: usize = 4096;

/// Fonte compartilhada de pressão (barata de clonar). Quem lê o
/// dispositivo chama [`FontePressao::registrar`]; a camada pergunta
/// [`FontePressao::pressao_em`].
#[derive(Clone, Default)]
pub struct FontePressao {
    interno: Arc<Interno>,
}

impl std::fmt::Debug for FontePressao {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontePressao").field("descricao", &self.descricao()).finish()
    }
}

impl PartialEq for FontePressao {
    fn eq(&self, outro: &Self) -> bool {
        Arc::ptr_eq(&self.interno, &outro.interno)
    }
}

impl FontePressao {
    pub fn nova() -> Self {
        Self::default()
    }

    /// Guarda uma leitura. Só a ponta encostada conta: pressão 0 (caneta
    /// pairando ou levantada) não é registrada, para um clique de mouse
    /// perto da caneta parada não virar traço fino.
    pub fn registrar(&self, instante: Instant, pressao: f32) {
        if !(pressao.is_finite() && pressao > 0.0) {
            return;
        }
        let Ok(mut fila) = self.interno.amostras.lock() else { return };
        // Mantém a ordem por instante (a leitura chega de uma thread só).
        if fila.back().is_some_and(|u| u.instante > instante) {
            return;
        }
        if fila.len() >= MAX_AMOSTRAS {
            fila.pop_front();
        }
        fila.push_back(AmostraPressao { instante, pressao: pressao.min(1.0) });
    }

    /// Pressão casada com o instante `t` (ver [`casar_por_tempo`]).
    pub fn pressao_em(&self, t: Instant) -> Option<f32> {
        let fila = self.interno.amostras.lock().ok()?;
        let (a, b) = fila.as_slices();
        // VecDeque em duas fatias: casar em cada uma e ficar com a mais perto.
        [casar_com_distancia(a, t, JANELA_CASAMENTO), casar_com_distancia(b, t, JANELA_CASAMENTO)]
            .into_iter()
            .flatten()
            .min_by_key(|(d, _)| *d)
            .map(|(_, p)| p)
    }

    /// Última leitura registrada (para o indicador ao vivo).
    pub fn ultima(&self) -> Option<AmostraPressao> {
        self.interno.amostras.lock().ok()?.back().copied()
    }

    /// De onde vem a pressão (ex.: "XInput2: xwayland-tablet stylus").
    pub fn descricao(&self) -> String {
        self.interno.descricao.lock().map(|d| d.clone()).unwrap_or_default()
    }

    pub fn definir_descricao(&self, texto: impl Into<String>) {
        if let Ok(mut d) = self.interno.descricao.lock() {
            *d = texto.into();
        }
    }

    // ---------------------------------------------- toques fora do Iced

    /// Janela X do aplicativo (o id que o winit criou). Sem ela a leitura
    /// não sabe onde a ponta está e não gera toques para a tinta.
    pub fn definir_janela_x11(&self, janela: u32) {
        self.interno.janela_x11.store(janela, Ordering::Relaxed);
    }

    pub fn janela_x11(&self) -> Option<u32> {
        Some(self.interno.janela_x11.load(Ordering::Relaxed)).filter(|&j| j != 0)
    }

    /// Tamanho lógico da janela (o que o Iced informa no redimensionar).
    pub fn definir_tamanho_logico(&self, tamanho: Size) {
        if let Ok(mut t) = self.interno.tamanho_logico.lock() {
            *t = Some(tamanho);
        }
    }

    pub fn tamanho_logico(&self) -> Option<Size> {
        *self.interno.tamanho_logico.lock().ok()?
    }

    /// Onde a camada de tinta captura (coordenadas lógicas da janela);
    /// `None` fora do modo desenho. A camada atualiza ao desenhar.
    pub fn definir_zona_tinta(&self, zona: Option<ZonaTinta>) {
        if let Ok(mut z) = self.interno.zona_tinta.lock() {
            if *z != zona {
                *z = zona;
            }
        }
    }

    pub fn zona_tinta(&self) -> Option<ZonaTinta> {
        *self.interno.zona_tinta.lock().ok()?
    }

    /// A leitura por fora entrega toques (há janela X conhecida)? Com ela,
    /// a camada deixa a ponta da caneta só com esse caminho.
    pub fn entrega_toques(&self) -> bool {
        self.janela_x11().is_some()
    }

    /// Guarda um toque e acorda a assinatura do Iced.
    pub fn enviar_toque(&self, toque: ToqueCaneta) {
        if let Ok(mut fila) = self.interno.toques.lock() {
            if fila.len() >= MAX_TOQUES {
                fila.pop_front();
            }
            fila.push_back(toque);
        }
        self.marcar_sinal();
        match toque.fase {
            FaseToque::Encostou => self.marcar_contato(toque.instante),
            FaseToque::Levantou => self.marcar_soltura(),
            FaseToque::Moveu => {}
        }
        if let Ok(mut d) = self.interno.despertar.lock() {
            if d.as_ref().is_some_and(|tx| tx.unbounded_send(()).is_err()) {
                *d = None;
            }
        }
    }

    /// Ponta encostou sem posição conhecida (sem janela): só o estado.
    pub fn marcar_contato(&self, instante: Instant) {
        if let Ok(mut c) = self.interno.contato.lock() {
            *c = Some(instante);
        }
        if let Ok(mut u) = self.interno.ultimo_contato.lock() {
            *u = Some(instante);
        }
    }

    pub fn marcar_soltura(&self) {
        if let Ok(mut c) = self.interno.contato.lock() {
            *c = None;
        }
    }

    /// Tira os toques pendentes (em ordem).
    pub fn recolher_toques(&self) -> Vec<ToqueCaneta> {
        self.interno.toques.lock().map(|mut f| f.drain(..).collect()).unwrap_or_default()
    }

    /// A ponta está encostada agora (pela leitura por fora)?
    pub fn em_contato(&self) -> bool {
        self.interno.contato.lock().is_ok_and(|c| c.is_some())
    }

    /// Algum sinal da caneta (toque ou clique sintético) agora.
    pub fn marcar_sinal(&self) {
        if let Ok(mut u) = self.interno.ultimo_sinal.lock() {
            *u = Some(Instant::now());
        }
    }

    /// A caneta está em uso: encostada, ou deu sinal há até `janela`. Um
    /// clique que o Iced recebe fora disso é do mouse.
    pub fn caneta_em_uso(&self, janela: Duration) -> bool {
        self.em_contato()
            || self.interno.ultimo_sinal.lock().is_ok_and(|u| u.is_some_and(|u| u.elapsed() <= janela))
    }

    /// A ponta encostou em algum momento a partir de `t`?
    pub fn encostou_desde(&self, t: Instant) -> bool {
        self.interno.ultimo_contato.lock().is_ok_and(|u| u.is_some_and(|u| u >= t))
    }

    fn registrar_despertar(&self, tx: mpsc::UnboundedSender<()>) {
        if let Ok(mut d) = self.interno.despertar.lock() {
            *d = Some(tx);
        }
    }
}

/// Assinatura do Iced com os toques da caneta lidos por fora (lotes, em
/// ordem). Uma por fonte; o app transforma cada lote numa mensagem.
pub fn assinatura_toques(fonte: &FontePressao) -> iced::Subscription<Vec<ToqueCaneta>> {
    use iced::futures::{SinkExt as _, StreamExt as _};
    let fonte = fonte.clone();
    let id = Arc::as_ptr(&fonte.interno) as usize;
    iced::Subscription::run_with_id(
        ("feather-ui-caneta-toques", id),
        iced::stream::channel(64, move |mut saida: mpsc::Sender<Vec<ToqueCaneta>>| async move {
            let (tx, mut rx) = mpsc::unbounded();
            fonte.registrar_despertar(tx);
            // Toque que chegou antes de a assinatura existir também vai.
            let pendentes = fonte.recolher_toques();
            if !pendentes.is_empty() && saida.send(pendentes).await.is_err() {
                return;
            }
            while rx.next().await.is_some() {
                let lote = fonte.recolher_toques();
                if !lote.is_empty() && saida.send(lote).await.is_err() {
                    return;
                }
            }
        }),
    )
}

/// Cliques de mouse que o Iced recebeu (o app conta todos, capturados ou
/// não). A leitura por fora compara antes e depois de um toque para saber
/// se o clique da caneta chegou ao aplicativo.
static CLIQUES_ICED: AtomicU64 = AtomicU64::new(0);

/// O app chama a cada `mouse::Event::ButtonPressed` (ver [`ouvir_cliques_iced`]).
pub fn registrar_clique_iced() {
    CLIQUES_ICED.fetch_add(1, Ordering::Relaxed);
}

pub fn cliques_iced() -> u64 {
    CLIQUES_ICED.load(Ordering::Relaxed)
}

/// Assinatura que conta os cliques do Iced (nunca gera mensagem).
pub fn ouvir_cliques_iced<M: Send + 'static>() -> iced::Subscription<M> {
    iced::event::listen_with(|evento, _status, _janela| {
        if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(_)) = evento {
            registrar_clique_iced();
        }
        None
    })
}

/// Pixel físico da janela X → coordenada lógica do Iced, pela razão entre
/// as larguras física e lógica. Sem tamanho lógico (ou inválido), 1:1.
pub fn fisico_para_logico(x: f32, y: f32, largura_fisica: f32, largura_logica: Option<f32>) -> Point {
    let escala = match largura_logica {
        Some(l) if l > 0.0 && largura_fisica > 0.0 && l.is_finite() => l / largura_fisica,
        _ => 1.0,
    };
    Point::new(x * escala, y * escala)
}

/// O toque que levantou precisa de clique sintético? Só quando não foi
/// para a tinta e o Iced não recebeu clique nenhum desde que a ponta
/// encostou (o servidor X não entregou o botão da caneta à janela).
pub fn precisa_clique_sintetico(na_tinta: bool, cliques_ao_encostar: u64, cliques_agora: u64) -> bool {
    !na_tinta && cliques_agora == cliques_ao_encostar
}

fn casar_com_distancia(amostras: &[AmostraPressao], t: Instant, janela: Duration) -> Option<(Duration, f32)> {
    if amostras.is_empty() {
        return None;
    }
    // Primeira amostra com instante >= t; a mais perto é ela ou a anterior.
    let i = amostras.partition_point(|a| a.instante < t);
    let distancia = |a: &AmostraPressao| if a.instante >= t { a.instante - t } else { t - a.instante };
    [i.checked_sub(1), (i < amostras.len()).then_some(i)]
        .into_iter()
        .flatten()
        .map(|k| (distancia(&amostras[k]), amostras[k].pressao))
        .filter(|(d, _)| *d <= janela)
        .min_by_key(|(d, _)| *d)
}

/// A pressão da amostra mais próxima de `t` (antes ou depois), se estiver
/// a até `janela`. `amostras` em ordem crescente de instante. `None` =
/// o ponto veio do mouse (ou a caneta não está encostada).
pub fn casar_por_tempo(amostras: &[AmostraPressao], t: Instant, janela: Duration) -> Option<f32> {
    casar_com_distancia(amostras, t, janela).map(|(_, p)| p)
}

/// Normaliza o valor do eixo de pressão (`valuator`) para `0..=1` pela
/// faixa que o dispositivo anuncia. Faixa inválida = `None`.
pub fn normalizar_valuator(valor: f64, minimo: f64, maximo: f64) -> Option<f32> {
    if !(valor.is_finite() && minimo.is_finite() && maximo.is_finite()) || maximo <= minimo {
        return None;
    }
    Some(((valor - minimo) / (maximo - minimo)).clamp(0.0, 1.0) as f32)
}

/// Prévia ao vivo da pressão sintética pela velocidade (unidades por ms).
/// **Mesma fórmula** de `featheroffice_drawing::tinta::pressao_da_velocidade`
/// (o motor é quem grava; aqui é só o desenho durante o traço). O
/// aplicativo tem teste comparando as duas.
pub fn previa_pressao_da_velocidade(velocidade: f32, velocidade_fina: f32) -> f32 {
    const MINIMA: f32 = 0.3;
    let vf = if velocidade_fina.is_finite() && velocidade_fina > 0.0 { velocidade_fina } else { 2.0 };
    let v = if velocidade.is_finite() { velocidade.max(0.0) } else { 0.0 };
    1.0 - (v / vf).min(1.0) * (1.0 - MINIMA)
}

/// Onde a janela do aplicativo abre no Linux com Wayland.
///
/// O winit 0.30 não fala o protocolo de mesa do Wayland (tablet-v2): numa
/// janela Wayland nativa a ponta da caneta move o cursor mas **não clica**
/// em nada. Pelo XWayland a mesa vira mouse (clique em todo o app) e a
/// pressão chega pelo XInput2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferenciaJanela {
    /// X11 (XWayland) se houver mesa digitalizadora conectada; senão Wayland.
    Automatica,
    X11,
    Wayland,
}

impl PreferenciaJanela {
    /// `auto`, `x11` ou `wayland` (configuração e `FEATHER*_JANELA`).
    pub fn de_texto(texto: &str) -> Option<Self> {
        match texto.trim().to_ascii_lowercase().as_str() {
            "auto" | "automatica" | "automática" => Some(Self::Automatica),
            "x11" | "xwayland" => Some(Self::X11),
            "wayland" => Some(Self::Wayland),
            _ => None,
        }
    }
    pub fn texto(self) -> &'static str {
        match self {
            Self::Automatica => "auto",
            Self::X11 => "x11",
            Self::Wayland => "wayland",
        }
    }
}

/// O aplicativo deve abrir a janela pelo X11 (XWayland)? Só numa sessão
/// Wayland com servidor X (`DISPLAY`) disponível; `ha_mesa` decide o modo
/// automático. Recebe as variáveis para ser testável.
pub fn deve_usar_x11(preferencia: PreferenciaJanela, ha_mesa: bool, wayland_display: Option<&str>, display: Option<&str>) -> bool {
    let tem = |v: Option<&str>| v.is_some_and(|s| !s.is_empty());
    let escolhe = match preferencia {
        PreferenciaJanela::X11 => true,
        PreferenciaJanela::Wayland => false,
        PreferenciaJanela::Automatica => ha_mesa,
    };
    cfg!(all(unix, not(target_os = "macos"))) && escolhe && tem(wayland_display) && tem(display)
}

/// ABS_PRESSURE e BTN_TOOL_PEN do evdev (linux/input-event-codes.h).
const ABS_PRESSURE: usize = 0x18;
const BTN_TOOL_PEN: usize = 0x140;

/// O bit `bit` está ligado no texto de capacidades do sysfs
/// (`/sys/class/input/inputN/capabilities/*`): palavras hexadecimais de
/// 64 bits separadas por espaço, a mais significativa primeiro.
pub fn capacidade_tem_bit(texto: &str, bit: usize) -> bool {
    let palavras: Vec<&str> = texto.split_whitespace().collect();
    let Some(indice) = palavras.len().checked_sub(1 + bit / 64) else { return false };
    u64::from_str_radix(palavras[indice], 16).is_ok_and(|w| w & (1u64 << (bit % 64)) != 0)
}

/// Pelas capacidades do sysfs: é uma caneta de mesa (pressão + ferramenta caneta)?
/// Mouse e teclado da própria mesa, touchpad e tela de toque ficam de fora.
pub fn capacidades_de_caneta(abs: &str, key: &str) -> bool {
    capacidade_tem_bit(abs, ABS_PRESSURE) && capacidade_tem_bit(key, BTN_TOOL_PEN)
}

/// Nomes das canetas de mesa conectadas (Linux). Lê só os metadados em
/// `/sys/class/input/input*/` (nome e capacidades, legíveis por qualquer
/// usuário) — nunca abre `/dev/input`, nunca lê evento.
pub fn mesas_conectadas() -> Vec<String> {
    #[cfg(target_os = "linux")]
    {
        let Ok(entradas) = std::fs::read_dir("/sys/class/input") else { return Vec::new() };
        let mut nomes = Vec::new();
        for e in entradas.flatten() {
            let nome_pasta = e.file_name();
            if !nome_pasta.to_string_lossy().starts_with("input") {
                continue;
            }
            let base = e.path();
            let ler = |f: &str| std::fs::read_to_string(base.join(f)).unwrap_or_default();
            if capacidades_de_caneta(&ler("capabilities/abs"), &ler("capabilities/key")) {
                nomes.push(ler("name").trim().to_string());
            }
        }
        nomes.sort();
        nomes
    }
    #[cfg(not(target_os = "linux"))]
    {
        Vec::new()
    }
}

// ------------------------------------------------------------ diagnóstico

static DIAGNOSTICO: AtomicBool = AtomicBool::new(false);
static DIARIO: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Liga/desliga o diário de eventos crus da caneta.
pub fn definir_diagnostico(ligado: bool) {
    DIAGNOSTICO.store(ligado, Ordering::Relaxed);
}

pub fn diagnostico_ligado() -> bool {
    DIAGNOSTICO.load(Ordering::Relaxed)
}

/// Anota uma linha no diário (só com o diagnóstico ligado; teto de 2000
/// linhas pendentes para nunca crescer sem limite).
pub fn anotar(linha: impl FnOnce() -> String) {
    if !diagnostico_ligado() {
        return;
    }
    if let Ok(mut d) = DIARIO.lock() {
        if d.len() < 2000 {
            d.push(linha());
        }
    }
}

/// Tira as linhas pendentes do diário (o aplicativo leva ao Console F12).
pub fn recolher_diagnostico() -> Vec<String> {
    DIARIO.lock().map(|mut d| std::mem::take(&mut *d)).unwrap_or_default()
}

// ------------------------------------------------------------ Linux (X11)

/// XInput2 pelo `x11rb` (Rust puro). Só eventos crus (`XI_RawMotion`,
/// `XI_RawButtonPress/Release`) dos dispositivos com eixo de pressão, mais
/// o aviso de troca de dispositivos (`XI_HierarchyChanged`).
#[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
pub mod x11 {
    use super::{
        anotar, cliques_iced, fisico_para_logico, normalizar_valuator, precisa_clique_sintetico, FaseToque, FontePressao,
        ToqueCaneta,
    };
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use x11rb::connection::{Connection, RequestConnection as _};
    use x11rb::protocol::xinput::{self, ConnectionExt as _, DeviceClassData, DeviceType, XIEventMask};
    use x11rb::protocol::xproto::{ConnectionExt as _, BUTTON_PRESS_EVENT, BUTTON_RELEASE_EVENT, MOTION_NOTIFY_EVENT};
    use x11rb::protocol::xtest::{self, ConnectionExt as _};
    use x11rb::protocol::Event;
    use x11rb::rust_connection::RustConnection;

    /// Um dispositivo com eixo de pressão.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Caneta {
        pub id: u16,
        pub nome: String,
        /// Número do eixo (`valuator`) da pressão.
        pub eixo: u16,
        pub minimo: f64,
        pub maximo: f64,
    }

    fn fp(v: xinput::Fp3232) -> f64 {
        f64::from(v.integral) + f64::from(v.frac) / 4_294_967_296.0
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

    /// Rótulo de eixo que é pressão ("Abs Pressure" no evdev/libinput e no XWayland).
    pub fn rotulo_de_pressao(rotulo: &str) -> bool {
        rotulo.to_ascii_lowercase().contains("pressure")
    }

    fn conectar() -> Result<(RustConnection, u32), String> {
        let (conn, tela) = RustConnection::connect(None).map_err(|e| format!("sem servidor X (DISPLAY): {e}"))?;
        let raiz = conn.setup().roots.get(tela).map(|s| s.root).ok_or("tela X inexistente")?;
        let versao = conn
            .xinput_xi_query_version(2, 2)
            .map_err(|e| format!("XInput2 indisponível: {e}"))?
            .reply()
            .map_err(|e| format!("XInput2 indisponível: {e}"))?;
        if versao.major_version < 2 {
            return Err(format!("XInput {}.{} (precisa 2.0)", versao.major_version, versao.minor_version));
        }
        Ok((conn, raiz))
    }

    fn tipo(t: DeviceType) -> &'static str {
        match t {
            DeviceType::MASTER_POINTER => "ponteiro mestre",
            DeviceType::MASTER_KEYBOARD => "teclado mestre",
            DeviceType::SLAVE_POINTER => "ponteiro",
            DeviceType::SLAVE_KEYBOARD => "teclado",
            DeviceType::FLOATING_SLAVE => "solto",
            _ => "?",
        }
    }

    /// Lista os dispositivos XInput (diagnóstico): id, tipo, nome e eixos.
    /// Teclados aparecem só pelo nome (nenhum evento deles é lido).
    pub fn listar_dispositivos() -> Result<Vec<String>, String> {
        let (conn, _) = conectar()?;
        let infos = conn
            .xinput_xi_query_device(0u16)
            .map_err(|e| e.to_string())?
            .reply()
            .map_err(|e| e.to_string())?
            .infos;
        let mut linhas = Vec::new();
        for info in infos {
            let mut eixos = Vec::new();
            for classe in &info.classes {
                if let DeviceClassData::Valuator(v) = &classe.data {
                    let rotulo = nome_atomo(&conn, v.label);
                    eixos.push(format!(
                        "{}:{} {}..{}{}",
                        v.number,
                        if rotulo.is_empty() { "?".into() } else { rotulo.clone() },
                        fp(v.min),
                        fp(v.max),
                        if rotulo_de_pressao(&rotulo) { " (PRESSÃO)" } else { "" }
                    ));
                }
            }
            linhas.push(format!(
                "id {} {} \"{}\"{}",
                info.deviceid,
                tipo(info.type_),
                String::from_utf8_lossy(&info.name),
                if eixos.is_empty() { String::new() } else { format!(" eixos [{}]", eixos.join("; ")) }
            ));
        }
        Ok(linhas)
    }

    fn procurar(conn: &RustConnection) -> Vec<Caneta> {
        let Some(infos) = conn.xinput_xi_query_device(0u16).ok().and_then(|c| c.reply().ok()) else { return Vec::new() };
        let mut canetas = Vec::new();
        for info in infos.infos {
            if !matches!(info.type_, DeviceType::SLAVE_POINTER | DeviceType::FLOATING_SLAVE) || !info.enabled {
                continue;
            }
            for classe in &info.classes {
                if let DeviceClassData::Valuator(v) = &classe.data {
                    if rotulo_de_pressao(&nome_atomo(conn, v.label)) && fp(v.max) > fp(v.min) {
                        canetas.push(Caneta {
                            id: info.deviceid,
                            nome: String::from_utf8_lossy(&info.name).into_owned(),
                            eixo: v.number,
                            minimo: fp(v.min),
                            maximo: fp(v.max),
                        });
                        break;
                    }
                }
            }
        }
        canetas
    }

    fn selecionar(conn: &RustConnection, raiz: u32, canetas: &[Caneta]) {
        let mut mascaras = vec![xinput::EventMask { deviceid: 0, mask: vec![XIEventMask::HIERARCHY] }];
        mascaras.extend(canetas.iter().map(|c| xinput::EventMask {
            deviceid: c.id,
            mask: vec![XIEventMask::RAW_MOTION | XIEventMask::RAW_BUTTON_PRESS | XIEventMask::RAW_BUTTON_RELEASE],
        }));
        let _ = conn.xinput_xi_select_events(raiz, &mascaras);
        let _ = conn.flush();
    }

    fn descrever(canetas: &[Caneta]) -> String {
        if canetas.is_empty() {
            "XInput2: nenhuma caneta com pressão encontrada (ligue a mesa)".into()
        } else {
            let nomes: Vec<String> = canetas.iter().map(|c| format!("{} ({}..{})", c.nome, c.minimo, c.maximo)).collect();
            format!("XInput2: {}", nomes.join(", "))
        }
    }

    /// Valor do eixo `eixo` num evento cru (a máscara diz quais vieram, em ordem).
    pub fn valor_do_eixo(mascara: &[u32], valores: &[xinput::Fp3232], eixo: u16) -> Option<f64> {
        let mut k = 0usize;
        for (palavra, bits) in mascara.iter().enumerate() {
            for b in 0..32 {
                if bits & (1 << b) != 0 {
                    if palavra * 32 + b == usize::from(eixo) {
                        return valores.get(k).map(|v| fp(*v));
                    }
                    k += 1;
                }
            }
        }
        None
    }

    /// Abre a conexão, acha as canetas e começa a ler numa thread própria.
    /// Devolve a descrição (também gravada na fonte). Erro = sem servidor X
    /// ou sem XInput2.
    pub fn iniciar(fonte: FontePressao) -> Result<String, String> {
        let (conn, raiz) = conectar()?;
        let canetas = procurar(&conn);
        selecionar(&conn, raiz, &canetas);
        let descricao = descrever(&canetas);
        fonte.definir_descricao(descricao.clone());
        anotar(|| format!("[caneta-x11] {descricao}"));
        std::thread::Builder::new()
            .name("caneta-x11".into())
            .spawn(move || ler(conn, raiz, canetas, fonte))
            .map_err(|e| e.to_string())?;
        Ok(descricao)
    }

    /// Onde a ponta está: posição física relativa à janela do app (se
    /// conhecida), largura física dela e posição na raiz (para o XTest).
    struct Ponto {
        janela: Option<(f32, f32, f32)>,
        raiz: (i16, i16),
    }

    fn onde_esta(conn: &RustConnection, raiz: u32, janela: Option<u32>) -> Option<Ponto> {
        let alvo = janela.unwrap_or(raiz);
        let q = conn.query_pointer(alvo).ok()?.reply().ok()?;
        let na_janela = janela.and_then(|j| {
            let largura = conn.get_geometry(j).ok()?.reply().ok()?.width;
            Some((f32::from(q.win_x), f32::from(q.win_y), f32::from(largura)))
        });
        Some(Ponto { janela: na_janela, raiz: (q.root_x, q.root_y) })
    }

    /// Toque em andamento (a ponta encostada).
    struct Contato {
        na_tinta: bool,
        borracha: bool,
        cliques_ao_encostar: u64,
        raiz: (i16, i16),
    }

    #[allow(clippy::too_many_arguments)]
    fn toque(
        fonte: &FontePressao,
        fase: FaseToque,
        ponto: &Ponto,
        pressao: Option<f32>,
        na_tinta: Option<bool>,
        borracha: bool,
        agora: Instant,
    ) -> Option<ToqueCaneta> {
        let (x, y, largura) = ponto.janela?;
        let posicao = fisico_para_logico(x, y, largura, fonte.tamanho_logico().map(|t| t.width));
        let na_tinta = na_tinta.unwrap_or_else(|| fonte.zona_tinta().is_some_and(|z| z.area.contains(posicao)));
        Some(ToqueCaneta { fase, posicao, pressao, instante: agora, na_tinta, borracha })
    }

    /// Ponta de borracha (o XWayland chama de "xwayland-tablet eraser").
    pub fn e_borracha(nome: &str) -> bool {
        nome.to_ascii_lowercase().contains("eraser")
    }

    /// Clique de mouse sintético (XTest) onde a ponta encostou: o servidor
    /// leu a caneta mas não entregou o botão à janela do app. Espera um
    /// pouco antes, para o clique do próprio Iced (se veio) ser contado.
    fn clique_sintetico(conn: Arc<RustConnection>, raiz: u32, contato: Contato, fonte: FontePressao) {
        let _ = std::thread::Builder::new().name("caneta-clique".into()).spawn(move || {
            std::thread::sleep(Duration::from_millis(80));
            if !precisa_clique_sintetico(false, contato.cliques_ao_encostar, cliques_iced()) {
                anotar(|| "[caneta-x11] o Iced recebeu o clique da caneta; sem clique sintético".into());
                return;
            }
            let (x, y) = contato.raiz;
            fonte.marcar_sinal();
            let r = (|| -> Result<(), x11rb::errors::ConnectionError> {
                conn.xtest_fake_input(MOTION_NOTIFY_EVENT, 0, 0, raiz, x, y, 0)?;
                conn.xtest_fake_input(BUTTON_PRESS_EVENT, 1, 0, raiz, 0, 0, 0)?;
                conn.xtest_fake_input(BUTTON_RELEASE_EVENT, 1, 0, raiz, 0, 0, 0)?;
                conn.flush()
            })();
            match r {
                Ok(()) => anotar(|| format!("[caneta-x11] o Iced não recebeu o toque: clique sintético (XTest) na raiz ({x}, {y})")),
                Err(e) => anotar(|| format!("[caneta-x11] clique sintético falhou: {e}")),
            }
        });
    }

    fn ler(conn: RustConnection, raiz: u32, mut canetas: Vec<Caneta>, fonte: FontePressao) {
        let conn = Arc::new(conn);
        let xtest = conn.extension_information(xtest::X11_EXTENSION_NAME).ok().flatten().is_some();
        let mut contagem = 0u64;
        let mut contato: Option<Contato> = None;
        loop {
            let evento = match conn.wait_for_event() {
                Ok(e) => e,
                Err(e) => {
                    anotar(|| format!("[caneta-x11] conexão encerrada: {e}"));
                    return;
                }
            };
            let agora = Instant::now();
            match evento {
                Event::XinputRawMotion(e) => {
                    let Some(c) = canetas.iter().find(|c| c.id == e.deviceid) else { continue };
                    let pressao = valor_do_eixo(&e.valuator_mask, &e.axisvalues, c.eixo)
                        .and_then(|bruto| normalizar_valuator(bruto, c.minimo, c.maximo).map(|p| (bruto, p)));
                    if let Some((bruto, p)) = pressao {
                        fonte.registrar(agora, p);
                        contagem += 1;
                        if p > 0.0 && contagem % 20 == 1 {
                            anotar(|| format!("[caneta-x11] {}: pressão bruta {bruto} -> {p:.3}", c.nome));
                        }
                    }
                    // Ponta encostada: o traço segue pela leitura daqui.
                    if let Some(ct) = &contato {
                        if let Some(t) = onde_esta(&conn, raiz, fonte.janela_x11())
                            .and_then(|pt| toque(&fonte, FaseToque::Moveu, &pt, pressao.map(|(_, p)| p), Some(ct.na_tinta), ct.borracha, agora))
                        {
                            fonte.enviar_toque(t);
                        }
                    }
                }
                Event::XinputRawButtonPress(e) | Event::XinputRawButtonRelease(e) => {
                    let solto = e.event_type == xinput::RAW_BUTTON_RELEASE_EVENT;
                    let Some(c) = canetas.iter().find(|c| c.id == e.deviceid) else { continue };
                    let nome = c.nome.clone();
                    let borracha = e_borracha(&nome);
                    anotar(|| format!("[caneta-x11] {nome}: botão {} {}", e.detail, if solto { "solto" } else { "apertado" }));
                    if e.detail != 1 {
                        continue;
                    }
                    let Some(pt) = onde_esta(&conn, raiz, fonte.janela_x11()) else { continue };
                    if !solto {
                        // A amostra de pressão do encostar chega no movimento
                        // seguinte; a mais recente serve de começo.
                        let pressao = fonte.pressao_em(agora);
                        let t = toque(&fonte, FaseToque::Encostou, &pt, pressao, None, borracha, agora);
                        let na_tinta = t.is_some_and(|t| t.na_tinta);
                        anotar(|| match t {
                            Some(t) => format!(
                                "[caneta-x11] encostou em ({:.0}, {:.0}) lógico, {}",
                                t.posicao.x,
                                t.posicao.y,
                                if na_tinta { "na camada de tinta: traço pela leitura X11" } else { "fora da tinta: clique" }
                            ),
                            None => "[caneta-x11] encostou (janela do app desconhecida)".into(),
                        });
                        contato = Some(Contato { na_tinta, borracha, cliques_ao_encostar: cliques_iced(), raiz: pt.raiz });
                        match t {
                            Some(t) => fonte.enviar_toque(t),
                            None => fonte.marcar_contato(agora),
                        }
                    } else if let Some(ct) = contato.take() {
                        match toque(&fonte, FaseToque::Levantou, &pt, None, Some(ct.na_tinta), ct.borracha, agora) {
                            Some(t) => fonte.enviar_toque(t),
                            None => fonte.marcar_soltura(),
                        }
                        if !ct.na_tinta && xtest {
                            clique_sintetico(Arc::clone(&conn), raiz, ct, fonte.clone());
                        }
                    }
                }
                Event::XinputHierarchy(_) => {
                    canetas = procurar(&conn);
                    selecionar(&conn, raiz, &canetas);
                    let d = descrever(&canetas);
                    fonte.definir_descricao(d.clone());
                    anotar(|| format!("[caneta-x11] dispositivos mudaram: {d}"));
                }
                _ => {}
            }
        }
    }
}

// ------------------------------------------------------------ Windows

/// `WM_POINTER*` + `GetPointerPenInfo` por subclasse da janela do Iced.
/// A mensagem segue para o winit normalmente (`DefSubclassProc`).
#[cfg(windows)]
pub mod windows {
    use super::{anotar, FontePressao};
    use iced::window::raw_window_handle::RawWindowHandle;
    use std::time::Instant;
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::UI::Input::Pointer::{GetPointerPenInfo, GetPointerType, POINTER_PEN_INFO};
    use windows_sys::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        PEN_MASK_PRESSURE, POINTER_INPUT_TYPE, PT_PEN, WM_POINTERDOWN, WM_POINTERUP, WM_POINTERUPDATE,
    };

    const ID_SUBCLASSE: usize = 0x4645_4e41; // "FENA"

    /// Liga a leitura da pressão na janela. Chamar na thread da janela
    /// (o `iced::window::run_with_handle` já roda nela).
    pub fn ligar_na_janela(janela: RawWindowHandle, fonte: FontePressao) -> Result<(), String> {
        let RawWindowHandle::Win32(h) = janela else { return Err("janela que não é Win32".into()) };
        let hwnd = h.hwnd.get() as HWND;
        fonte.definir_descricao("WM_POINTER (Windows Ink)");
        // A fonte vive enquanto a janela existir (uma por janela, nunca solta).
        let dados = Box::into_raw(Box::new(fonte)) as usize;
        // SAFETY: hwnd veio do winit e é válido nesta thread; `dados` aponta
        // para uma FontePressao que nunca é liberada.
        let ok = unsafe { SetWindowSubclass(hwnd, Some(procedimento), ID_SUBCLASSE, dados) };
        if ok == 0 {
            // SAFETY: a subclasse não foi instalada; ninguém mais usa `dados`.
            drop(unsafe { Box::from_raw(dados as *mut FontePressao) });
            return Err("SetWindowSubclass falhou".into());
        }
        anotar(|| "[caneta-windows] leitura de WM_POINTER ligada".into());
        Ok(())
    }

    unsafe extern "system" fn procedimento(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _id: usize,
        dados: usize,
    ) -> LRESULT {
        if matches!(msg, WM_POINTERDOWN | WM_POINTERUPDATE | WM_POINTERUP) {
            let id = (wparam & 0xFFFF) as u32;
            let mut tipo: POINTER_INPUT_TYPE = 0;
            // SAFETY: chamadas da API com ponteiros para variáveis locais.
            if unsafe { GetPointerType(id, &mut tipo) } != 0 && tipo == PT_PEN {
                let mut info: POINTER_PEN_INFO = unsafe { std::mem::zeroed() };
                if unsafe { GetPointerPenInfo(id, &mut info) } != 0 && info.penMask & PEN_MASK_PRESSURE != 0 {
                    // SAFETY: `dados` é a FontePressao instalada em `ligar_na_janela`.
                    let fonte = unsafe { &*(dados as *const FontePressao) };
                    let p = if msg == WM_POINTERUP { 0.0 } else { info.pressure as f32 / 1024.0 };
                    fonte.registrar(Instant::now(), p);
                    anotar(|| format!("[caneta-windows] msg {msg:#x} pressão {}", info.pressure));
                }
            }
        }
        // SAFETY: repassa ao próximo procedimento da cadeia (o do winit).
        unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn amostras(base: Instant, lista: &[(u64, f32)]) -> Vec<AmostraPressao> {
        lista
            .iter()
            .map(|&(ms, p)| AmostraPressao { instante: base + Duration::from_millis(ms), pressao: p })
            .collect()
    }

    #[test]
    fn casa_o_ponto_com_a_amostra_mais_proxima_no_tempo() {
        let t0 = Instant::now();
        let a = amostras(t0, &[(0, 0.1), (10, 0.2), (20, 0.3), (40, 0.6)]);
        let em = |ms: u64| t0 + Duration::from_millis(ms);
        assert_eq!(casar_por_tempo(&a, em(11), JANELA_CASAMENTO), Some(0.2));
        assert_eq!(casar_por_tempo(&a, em(16), JANELA_CASAMENTO), Some(0.3), "a seguinte também vale");
        assert_eq!(casar_por_tempo(&a, em(33), JANELA_CASAMENTO), Some(0.6));
        assert_eq!(casar_por_tempo(&a, em(0), JANELA_CASAMENTO), Some(0.1));
        // Longe demais de qualquer amostra = mouse.
        assert_eq!(casar_por_tempo(&a, em(40 + 61), JANELA_CASAMENTO), None);
        assert_eq!(casar_por_tempo(&[], em(0), JANELA_CASAMENTO), None);
    }

    #[test]
    fn fonte_ignora_ponta_levantada_e_guarda_em_ordem() {
        let f = FontePressao::nova();
        let t0 = Instant::now();
        f.registrar(t0, 0.0);
        f.registrar(t0, f32::NAN);
        assert!(f.ultima().is_none(), "pressão 0 (pairando) não conta");
        assert_eq!(f.pressao_em(t0), None);
        f.registrar(t0 + Duration::from_millis(5), 0.4);
        f.registrar(t0 + Duration::from_millis(15), 1.7);
        f.registrar(t0 + Duration::from_millis(1), 0.9); // fora de ordem: descartada
        assert_eq!(f.pressao_em(t0 + Duration::from_millis(14)), Some(1.0));
        assert_eq!(f.pressao_em(t0 + Duration::from_millis(6)), Some(0.4));
        // Muitas amostras: a fila não cresce sem limite e continua casando.
        for i in 0..2000u64 {
            f.registrar(t0 + Duration::from_millis(100 + i), 0.5);
        }
        assert_eq!(f.pressao_em(t0 + Duration::from_millis(2099)), Some(0.5));
        assert_eq!(f.clone(), f);
    }

    #[test]
    fn normaliza_a_faixa_do_eixo() {
        assert_eq!(normalizar_valuator(0.0, 0.0, 65535.0), Some(0.0));
        assert_eq!(normalizar_valuator(65535.0, 0.0, 65535.0), Some(1.0));
        assert!((normalizar_valuator(4096.0, 0.0, 8191.0).unwrap() - 0.50006).abs() < 1e-4);
        assert_eq!(normalizar_valuator(9000.0, 0.0, 8191.0), Some(1.0));
        assert_eq!(normalizar_valuator(-5.0, 0.0, 1.0), Some(0.0));
        assert_eq!(normalizar_valuator(12.0, 10.0, 20.0), Some(0.2));
        assert_eq!(normalizar_valuator(1.0, 5.0, 5.0), None);
        assert_eq!(normalizar_valuator(f64::NAN, 0.0, 1.0), None);
    }

    #[test]
    fn escolhe_x11_com_mesa_ou_forcado_e_nunca_sem_xwayland() {
        use PreferenciaJanela::*;
        let linux = cfg!(all(unix, not(target_os = "macos")));
        let (w, d) = (Some("wayland-0"), Some(":0"));
        assert_eq!(deve_usar_x11(Automatica, true, w, d), linux, "mesa conectada: XWayland");
        assert!(!deve_usar_x11(Automatica, false, w, d), "sem mesa: Wayland nativo");
        assert_eq!(deve_usar_x11(X11, false, w, d), linux);
        assert!(!deve_usar_x11(Wayland, true, w, d));
        assert!(!deve_usar_x11(X11, true, None, d), "já é X11: nada a trocar");
        assert!(!deve_usar_x11(X11, true, w, None), "sem XWayland: não quebra a abertura");
        assert!(!deve_usar_x11(X11, true, w, Some("")));
        assert_eq!(PreferenciaJanela::de_texto(" X11 "), Some(X11));
        assert_eq!(PreferenciaJanela::de_texto("auto"), Some(Automatica));
        assert_eq!(PreferenciaJanela::de_texto("wayland").map(PreferenciaJanela::texto), Some("wayland"));
        assert_eq!(PreferenciaJanela::de_texto("outro"), None);
    }

    #[test]
    fn reconhece_a_caneta_da_mesa_pelas_capacidades_do_sysfs() {
        // Valores reais da XP-Pen Deco 01 V3 (kernel 7.0).
        assert!(capacidades_de_caneta("d000003", "c03 0 0 0 0 0"), "UGTABLET Deco 01 V3 Pen");
        assert!(!capacidades_de_caneta("1000003", "400 70000 0 0 0 0"), "o 'Mouse' da mesa não tem BTN_TOOL_PEN");
        assert!(!capacidades_de_caneta("0", "1000000000007 ff9f207ac14057ff febeffdfffefffff fffffffffffffffe"));
        assert!(!capacidades_de_caneta("", ""));
        assert!(capacidade_tem_bit("1 0", 64) && !capacidade_tem_bit("1 0", 0));
        assert!(!capacidade_tem_bit("zz", 3));
    }

    #[test]
    fn diario_so_anota_com_diagnostico_ligado() {
        // O diário é global: este teste é o único que liga o diagnóstico.
        definir_diagnostico(false);
        anotar(|| "nao".into());
        definir_diagnostico(true);
        anotar(|| "sim".into());
        definir_diagnostico(false);
        let linhas = recolher_diagnostico();
        assert!(linhas.contains(&"sim".to_string()) && !linhas.contains(&"nao".to_string()));
    }

    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
    #[test]
    fn eixo_cru_e_achado_pela_mascara() {
        use x11rb::protocol::xinput::Fp3232;
        let v = |i: i32| Fp3232 { integral: i, frac: 0 };
        // Eixos 0, 1 e 2 presentes; pressão no eixo 2.
        assert_eq!(x11::valor_do_eixo(&[0b111], &[v(10), v(20), v(3000)], 2), Some(3000.0));
        // Só os eixos 0 e 2: o valor do 2 é o segundo da lista.
        assert_eq!(x11::valor_do_eixo(&[0b101], &[v(10), v(3000)], 2), Some(3000.0));
        // Eixo 33 na segunda palavra da máscara.
        assert_eq!(x11::valor_do_eixo(&[0b1, 0b10], &[v(1), v(77)], 33), Some(77.0));
        assert_eq!(x11::valor_do_eixo(&[0b011], &[v(1), v(2)], 2), None, "pressão não veio");
        assert!((x11::valor_do_eixo(&[1], &[Fp3232 { integral: 1, frac: 1 << 31 }], 0).unwrap() - 1.5).abs() < 1e-9);
        assert!(x11::rotulo_de_pressao("Abs Pressure") && !x11::rotulo_de_pressao("Abs Tilt X"));
        assert!(x11::e_borracha("xwayland-tablet eraser:17") && !x11::e_borracha("xwayland-tablet stylus:17"));
    }

    #[test]
    fn toques_da_caneta_vao_em_ordem_e_acordam_a_assinatura() {
        let f = FontePressao::nova();
        assert!(!f.entrega_toques() && !f.em_contato());
        let (tx, mut rx) = mpsc::unbounded();
        f.registrar_despertar(tx);
        let t0 = Instant::now();
        let toque = |fase, x: f32| ToqueCaneta { fase, posicao: Point::new(x, 0.0), pressao: Some(0.5), instante: t0, na_tinta: true, borracha: false };
        f.enviar_toque(toque(FaseToque::Encostou, 1.0));
        assert!(f.em_contato() && f.encostou_desde(t0) && !f.encostou_desde(t0 + Duration::from_millis(1)));
        f.enviar_toque(toque(FaseToque::Moveu, 2.0));
        f.enviar_toque(toque(FaseToque::Levantou, 3.0));
        assert!(!f.em_contato() && f.encostou_desde(t0), "o último encostar fica lembrado");
        let xs: Vec<f32> = f.recolher_toques().iter().map(|t| t.posicao.x).collect();
        assert_eq!(xs, vec![1.0, 2.0, 3.0]);
        assert!(f.recolher_toques().is_empty());
        let mut acordou = 0;
        while rx.try_recv().is_ok() {
            acordou += 1;
        }
        assert_eq!(acordou, 3);
        assert!(f.caneta_em_uso(Duration::from_millis(500)), "acabou de tocar");
        assert!(!FontePressao::nova().caneta_em_uso(Duration::from_millis(500)), "sem caneta: é o mouse");
        f.definir_janela_x11(77);
        assert_eq!(f.janela_x11(), Some(77));
        assert!(f.entrega_toques());
        let area = Rectangle::new(Point::new(10.0, 10.0), Size::new(5.0, 5.0));
        f.definir_zona_tinta(Some(ZonaTinta { area, origem: Point::ORIGIN }));
        assert!(f.zona_tinta().is_some_and(|z| z.area.contains(Point::new(12.0, 12.0))));
        f.definir_zona_tinta(None);
        assert!(f.zona_tinta().is_none());
        // A fila tem teto.
        for i in 0..(MAX_TOQUES + 10) {
            f.enviar_toque(toque(FaseToque::Moveu, i as f32));
        }
        let fila = f.recolher_toques();
        assert_eq!(fila.len(), MAX_TOQUES);
        assert_eq!(fila[0].posicao.x, 10.0);
    }

    #[test]
    fn pixel_fisico_vira_logico_pela_razao_das_larguras() {
        // Janela de 2560 px físicos com 1280 lógicos (escala 2).
        assert_eq!(fisico_para_logico(400.0, 300.0, 2560.0, Some(1280.0)), Point::new(200.0, 150.0));
        assert_eq!(fisico_para_logico(400.0, 300.0, 1280.0, Some(1280.0)), Point::new(400.0, 300.0));
        // Sem o tamanho lógico (ou inválido): 1:1.
        assert_eq!(fisico_para_logico(400.0, 300.0, 1280.0, None), Point::new(400.0, 300.0));
        assert_eq!(fisico_para_logico(400.0, 300.0, 0.0, Some(1280.0)), Point::new(400.0, 300.0));
    }

    #[test]
    fn clique_sintetico_so_quando_o_iced_nao_recebeu_e_fora_da_tinta() {
        assert!(precisa_clique_sintetico(false, 5, 5), "faixa/célula sem clique no Iced: sintetiza");
        assert!(!precisa_clique_sintetico(false, 5, 6), "o Iced recebeu: não duplica");
        assert!(!precisa_clique_sintetico(true, 5, 5), "na camada de tinta o traço é pela leitura X11");
        let antes = cliques_iced();
        registrar_clique_iced();
        assert!(cliques_iced() > antes);
    }
}
