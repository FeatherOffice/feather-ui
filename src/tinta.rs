//! Aba Desenhar (caneta) compartilhada: a camada transparente que captura
//! mouse/caneta sobre o documento e o grupo de controles da faixa.
//!
//! Sem motor: a camada não sabe o que é célula, slide ou página. Ela recebe
//! os traços já em **coordenadas do documento** ([`TracoTela`]) e a
//! [`Transformacao`] documento → tela, desenha, e devolve [`EventoCamada`]
//! em coordenadas do documento. Guardar, desfazer, apagar e reconhecer
//! forma é com o aplicativo (o motor está em `featheroffice-drawing::tinta`).
//! Esc é do aplicativo (sai do modo desenho), não da camada.
use std::time::Instant;

use iced::mouse;
use iced::touch;
use iced::widget::canvas::{self, event, Frame, Geometry, LineCap, LineDash, LineJoin, Path, Program, Stroke};
use iced::widget::{button, column, container, mouse_area, opaque, row, stack, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Point, Rectangle, Renderer, Size, Theme};

use crate::componentes::{botao_grande, botao_pequeno, grupo_faixa_compacto};
use crate::icones::icone;
use crate::tema::*;
pub use crate::caneta::FontePressao;

// ------------------------------------------------------------------ estado

/// Cor e espessura de uma caneta (espessura na unidade do documento).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pena {
    pub cor: [u8; 4],
    pub espessura: f32,
}

/// Ferramenta escolhida na aba Desenhar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ferramenta {
    /// Uma das três canetas (0, 1 ou 2).
    Caneta(u8),
    MarcaTexto,
    BorrachaTraco,
    BorrachaPonto,
    Laco,
}

impl Ferramenta {
    pub fn desenha(self) -> bool {
        matches!(self, Self::Caneta(_) | Self::MarcaTexto)
    }
    pub fn apaga(self) -> bool {
        matches!(self, Self::BorrachaTraco | Self::BorrachaPonto)
    }
    pub fn rotulo(self) -> &'static str {
        match self {
            Self::Caneta(0) => "Caneta 1",
            Self::Caneta(1) => "Caneta 2",
            Self::Caneta(_) => "Caneta 3",
            Self::MarcaTexto => "Marca-texto",
            Self::BorrachaTraco => "Borracha de traço",
            Self::BorrachaPonto => "Borracha de ponto",
            Self::Laco => "Seleção por laço",
        }
    }
}

/// Paleta das canetas (cores de conteúdo, não do tema).
pub const PALETA_CANETA: [[u8; 4]; 12] = [
    [0x20, 0x21, 0x24, 255],
    [0x5f, 0x63, 0x68, 255],
    [0xd9, 0x30, 0x25, 255],
    [0xe8, 0x71, 0x0a, 255],
    [0xf9, 0xab, 0x00, 255],
    [0x18, 0x80, 0x38, 255],
    [0x12, 0xb5, 0xcb, 255],
    [0x1a, 0x73, 0xe8, 255],
    [0x7b, 0x1f, 0xa2, 255],
    [0xe5, 0x2d, 0x8a, 255],
    [0x79, 0x55, 0x48, 255],
    [0xff, 0xff, 0xff, 255],
];
/// Paleta do marca-texto (cores claras, desenhadas translúcidas).
pub const PALETA_MARCA_TEXTO: [[u8; 4]; 6] = [
    [0xff, 0xeb, 0x3b, 255],
    [0x76, 0xff, 0x03, 255],
    [0x18, 0xff, 0xff, 255],
    [0xff, 0x40, 0x81, 255],
    [0xff, 0x9e, 0x40, 255],
    [0xb3, 0x88, 0xff, 255],
];
pub const ESPESSURAS_CANETA: [f32; 6] = [1.0, 1.5, 2.0, 3.0, 4.5, 6.0];
pub const ESPESSURAS_MARCA_TEXTO: [f32; 4] = [8.0, 12.0, 16.0, 22.0];

/// Estado da aba Desenhar. `ativo` = modo desenho ligado (o clique no
/// documento vira tinta, não seleção).
#[derive(Debug, Clone, PartialEq)]
pub struct Estado {
    pub ativo: bool,
    pub ferramenta: Ferramenta,
    pub canetas: [Pena; 3],
    pub marca_texto: Pena,
    pub tinta_para_forma: bool,
    /// Menu flutuante de cor/espessura aberto, e de qual ferramenta.
    pub menu: Option<Ferramenta>,
}

impl Default for Estado {
    fn default() -> Self {
        Self {
            ativo: false,
            ferramenta: Ferramenta::Caneta(0),
            canetas: [
                Pena { cor: PALETA_CANETA[0], espessura: 2.0 },
                Pena { cor: PALETA_CANETA[2], espessura: 2.0 },
                Pena { cor: PALETA_CANETA[7], espessura: 3.0 },
            ],
            marca_texto: Pena { cor: PALETA_MARCA_TEXTO[0], espessura: 12.0 },
            tinta_para_forma: false,
            menu: None,
        }
    }
}

/// Eventos do grupo de controles (o app chama [`Estado::atualizar`]).
#[derive(Debug, Clone, PartialEq)]
pub enum Evento {
    /// Escolhe a ferramenta e liga o modo desenho. Escolher de novo a
    /// caneta já ativa abre o menu dela (como no Office).
    Escolher(Ferramenta),
    AlternarMenu(Ferramenta),
    FecharMenu,
    Cor(Ferramenta, [u8; 4]),
    Espessura(Ferramenta, f32),
    AlternarFormas,
    /// Parar de desenhar (volta à seleção normal). Também pelo Esc.
    Parar,
}

impl Estado {
    /// Pena da ferramenta (só canetas e marca-texto).
    pub fn pena_de(&self, f: Ferramenta) -> Option<Pena> {
        match f {
            Ferramenta::Caneta(i) => self.canetas.get(usize::from(i)).copied(),
            Ferramenta::MarcaTexto => Some(self.marca_texto),
            _ => None,
        }
    }
    fn pena_mut(&mut self, f: Ferramenta) -> Option<&mut Pena> {
        match f {
            Ferramenta::Caneta(i) => self.canetas.get_mut(usize::from(i)),
            Ferramenta::MarcaTexto => Some(&mut self.marca_texto),
            _ => None,
        }
    }
    /// Pena em uso agora (None fora do modo desenho ou com borracha/laço).
    pub fn pena(&self) -> Option<Pena> {
        if self.ativo { self.pena_de(self.ferramenta) } else { None }
    }
    /// Ferramenta em uso, se o modo desenho estiver ligado.
    pub fn ferramenta_ativa(&self) -> Option<Ferramenta> {
        self.ativo.then_some(self.ferramenta)
    }

    /// Aplica o evento. Devolve se algo mudou.
    pub fn atualizar(&mut self, evento: Evento) -> bool {
        let antes = self.clone();
        match evento {
            Evento::Escolher(f) => {
                if self.ativo && self.ferramenta == f && f.desenha() {
                    self.menu = if self.menu == Some(f) { None } else { Some(f) };
                } else {
                    self.ativo = true;
                    self.ferramenta = f;
                    self.menu = None;
                }
            }
            Evento::AlternarMenu(f) => {
                self.menu = if self.menu == Some(f) { None } else { Some(f) };
            }
            Evento::FecharMenu => self.menu = None,
            Evento::Cor(f, cor) => {
                if let Some(p) = self.pena_mut(f) {
                    p.cor = cor;
                }
                self.ativo = true;
                self.ferramenta = f;
            }
            Evento::Espessura(f, e) => {
                if let Some(p) = self.pena_mut(f) {
                    if e.is_finite() && e > 0.0 {
                        p.espessura = e;
                    }
                }
                self.ativo = true;
                self.ferramenta = f;
            }
            Evento::AlternarFormas => self.tinta_para_forma = !self.tinta_para_forma,
            Evento::Parar => {
                self.ativo = false;
                self.menu = None;
            }
        }
        *self != antes
    }
}

fn cor_iced(c: [u8; 4]) -> Color {
    Color::from_rgba8(c[0], c[1], c[2], f32::from(c[3]) / 255.0)
}

// ------------------------------------------------------------------ faixa

/// Botão de caneta: ícone com a cor e a espessura embaixo, e a seta que
/// abre o menu de cor/espessura.
fn botao_pena<'a>(estado: &Estado, f: Ferramenta) -> Element<'a, Evento> {
    let Some(pena) = estado.pena_de(f) else { return Space::with_width(0).into() };
    let ativo = estado.ativo && estado.ferramenta == f;
    let nome_icone = if f == Ferramenta::MarcaTexto { "highlight" } else { "pencil" };
    let cor = cor_iced(pena.cor);
    let altura = (pena.espessura / if f == Ferramenta::MarcaTexto { 4.0 } else { 1.2 }).clamp(3.0, 6.0);
    let amostra = container(Space::with_height(altura)).width(22.0).style(move |_| container::Style {
        background: Some(Background::Color(cor)),
        border: Border { color: COR_BORDA(), width: 0.6, radius: 1.0.into() },
        ..Default::default()
    });
    let principal = button(column![icone(nome_icone, 22.0), amostra].spacing(3).align_x(Alignment::Center))
        .width(Length::Fixed(44.0))
        .padding([5, 6])
        .style(estilo_botao_grande(ativo))
        .on_press(Evento::Escolher(f));
    let seta = button(container(icone("chevron-down", 12.0)).center_x(Length::Fill))
        .width(Length::Fixed(44.0))
        .padding([1, 4])
        .style(estilo_botao_pequeno(estado.menu == Some(f)))
        .on_press(Evento::AlternarMenu(f));
    let dica = format!("{} (clique de novo para cor e espessura)", f.rotulo());
    crate::formatacao::dica(column![principal, seta].spacing(1), &dica)
}

/// Grupos da aba Desenhar, já com legenda (para entrar na faixa como os outros).
pub fn grupos_faixa<'a>(estado: &Estado) -> Vec<Element<'a, Evento>> {
    let canetas = row![
        botao_pena(estado, Ferramenta::Caneta(0)),
        botao_pena(estado, Ferramenta::Caneta(1)),
        botao_pena(estado, Ferramenta::Caneta(2)),
        botao_pena(estado, Ferramenta::MarcaTexto),
    ]
    .spacing(3)
    .align_y(Alignment::Center);
    let ativo = |f: Ferramenta| estado.ativo && estado.ferramenta == f;
    let borrachas = column![
        botao_pequeno(icone("eraser", 17.0), "Borracha de traço", Some(Evento::Escolher(Ferramenta::BorrachaTraco)), ativo(Ferramenta::BorrachaTraco)),
        botao_pequeno(icone("eraser", 17.0), "Borracha de ponto", Some(Evento::Escolher(Ferramenta::BorrachaPonto)), ativo(Ferramenta::BorrachaPonto)),
        botao_pequeno(icone("lasso", 17.0), "Laço", Some(Evento::Escolher(Ferramenta::Laco)), ativo(Ferramenta::Laco)),
    ]
    .spacing(3);
    let parar: Element<'a, Evento> = if estado.ativo {
        crate::formatacao::dica(
            botao_grande(icone("hand-stop", 23.0), "Parar de desenhar", Some(Evento::Parar), false),
            "Volta à seleção normal (Esc)",
        )
    } else {
        crate::formatacao::dica(
            botao_grande(icone("hand-stop", 23.0), "Parar de desenhar", None, false),
            "Escolha uma caneta para desenhar; aqui volta à seleção",
        )
    };
    vec![
        grupo_faixa_compacto(row![canetas, borrachas].spacing(8).align_y(Alignment::Center).into(), "Ferramentas de desenho"),
        grupo_faixa_compacto(
            crate::formatacao::dica(
                botao_grande(icone("wand", 23.0), "Tinta para forma", Some(Evento::AlternarFormas), estado.tinta_para_forma),
                "Linha, seta, retângulo, triângulo e elipse viram formas perfeitas",
            ),
            "Converter",
        ),
        grupo_faixa_compacto(parar, "Modo"),
    ]
}

/// Larguras aproximadas dos grupos (para a faixa responsiva do app).
pub const LARGURAS_GRUPOS: [f32; 3] = [370.0, 120.0, 135.0];

/// Conteúdo do menu flutuante de cor e espessura (None se fechado).
pub fn menu_pena<'a>(estado: &Estado) -> Option<Element<'a, Evento>> {
    let f = estado.menu?;
    let pena = estado.pena_de(f)?;
    let (paleta, espessuras): (&[[u8; 4]], &[f32]) = if f == Ferramenta::MarcaTexto {
        (&PALETA_MARCA_TEXTO, &ESPESSURAS_MARCA_TEXTO)
    } else {
        (&PALETA_CANETA, &ESPESSURAS_CANETA)
    };
    let amostra = |c: [u8; 4]| {
        let escolhida = c == pena.cor;
        button(Space::new(Length::Fixed(20.0), Length::Fixed(20.0)))
            .padding(0)
            .on_press(Evento::Cor(f, c))
            .style(move |_: &Theme, status| button::Style {
                background: Some(Background::Color(cor_iced(c))),
                border: Border {
                    color: if escolhida || matches!(status, button::Status::Hovered) { COR_AZUL_ACENTO() } else { COR_BORDA() },
                    width: if escolhida { 2.5 } else { 1.0 },
                    radius: 5.0.into(),
                },
                ..Default::default()
            })
    };
    let mut cores = column![].spacing(6);
    for linha in paleta.chunks(6) {
        cores = cores.push(row(linha.iter().map(|&c| amostra(c).into())).spacing(6));
    }
    let cor_linha = cor_iced(pena.cor);
    let mut grossuras = column![].spacing(2);
    for &e in espessuras {
        let traco = container(Space::with_height((e * 0.8).clamp(1.0, 14.0))).width(70.0).style(move |_| container::Style {
            background: Some(Background::Color(cor_linha)),
            border: Border { radius: 3.0.into(), ..Default::default() },
            ..Default::default()
        });
        grossuras = grossuras.push(
            button(row![container(traco).width(80.0).center_y(16.0), text(format!("{e} px")).size(12)].spacing(8).align_y(Alignment::Center))
                .width(Length::Fill)
                .padding([4, 8])
                .style(estilo_botao_pequeno(e == pena.espessura))
                .on_press(Evento::Espessura(f, e)),
        );
    }
    let corpo = column![
        row![text(f.rotulo()).size(14), Space::with_width(Length::Fill), botao_fechar(Evento::FecharMenu)]
            .align_y(Alignment::Center),
        text("COR").size(10.5).color(COR_TEXTO_MUTED()),
        cores,
        text("ESPESSURA").size(10.5).color(COR_TEXTO_MUTED()),
        grossuras,
    ]
    .spacing(8)
    .padding(12)
    .width(Length::Fixed(196.0));
    Some(container(corpo).style(estilo_popup_menu).into())
}

/// Põe o menu da caneta flutuando em (`x`, `y`) da janela, sem empurrar o
/// layout; clicar fora fecha. `mapear` converte o evento na mensagem do app.
pub fn sobrepor_menu<'a, M: Clone + 'a>(
    base: Element<'a, M>,
    estado: &Estado,
    x: f32,
    y: f32,
    janela: Size,
    mapear: fn(Evento) -> M,
) -> Element<'a, M> {
    let Some(menu) = menu_pena(estado) else { return base };
    let x = x.min(janela.width - 208.0).max(8.0);
    let y = y.min(janela.height - 420.0).max(8.0);
    let fora = mouse_area(Space::new(Length::Fill, Length::Fill)).on_press(mapear(Evento::FecharMenu));
    let posicionado = column![Space::with_height(y), row![Space::with_width(x), opaque(menu.map(mapear))]];
    stack![base, fora, posicionado].width(Length::Fill).height(Length::Fill).into()
}

// ------------------------------------------------------------------ camada

/// Documento → tela: `tela = documento × escala + deslocamento`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transformacao {
    pub escala: f32,
    pub dx: f32,
    pub dy: f32,
}

impl Default for Transformacao {
    fn default() -> Self {
        Self { escala: 1.0, dx: 0.0, dy: 0.0 }
    }
}

impl Transformacao {
    pub fn para_tela(self, p: Point) -> Point {
        Point::new(p.x * self.escala + self.dx, p.y * self.escala + self.dy)
    }
    pub fn para_documento(self, p: Point) -> Point {
        let e = if self.escala.abs() > f32::EPSILON { self.escala } else { 1.0 };
        Point::new((p.x - self.dx) / e, (p.y - self.dy) / e)
    }
}

/// Traço a desenhar, em coordenadas do documento. A cor já leva a
/// opacidade no alfa.
#[derive(Debug, Clone, PartialEq)]
pub struct TracoTela {
    pub id: u64,
    pub pontos: Vec<Point>,
    pub cor: Color,
    pub espessura: f32,
    /// Segmentos retos (forma) em vez de curva suave.
    pub reto: bool,
    /// Ponta quadrada (marca-texto) em vez de redonda.
    pub marca_texto: bool,
    /// Contorno de largura variável (polígonos fechados no documento,
    /// calculados pelo motor). Vazio = linha de `espessura` constante.
    pub contorno: Vec<Vec<Point>>,
}

/// Ponto capturado (documento) com o instante desde o início do traço e
/// a pressão **real** da caneta (`0..=1`, crua, sem curva), quando a
/// [`FontePressao`] da camada a casou com o ponto. `None` = mouse/dedo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PontoCaptura {
    pub x: f32,
    pub y: f32,
    pub tempo_ms: u32,
    pub pressao: Option<f32>,
}

impl PontoCaptura {
    pub const fn novo(x: f32, y: f32, tempo_ms: u32) -> Self {
        Self { x, y, tempo_ms, pressao: None }
    }
}

/// O que a camada devolve ao app, sempre em coordenadas do documento.
#[derive(Debug, Clone, PartialEq)]
pub enum EventoCamada {
    /// Caneta/marca-texto soltou: o traço completo (um Ctrl+Z no app).
    TracoTerminado { ferramenta: Ferramenta, pontos: Vec<PontoCaptura> },
    /// Borracha passando: apague em volta do ponto (`raio` no documento).
    Apagar { ferramenta: Ferramenta, ponto: Point, raio: f32 },
    /// Borracha soltou: feche o apagar numa etapa de desfazer.
    ApagarFim,
    /// Laço fechado (polígono no documento).
    Laco(Vec<Point>),
}

/// Raio da borracha na tela, em pixels.
pub const RAIO_BORRACHA_TELA: f32 = 8.0;

/// Camada transparente de tinta (`canvas`). Fora do modo desenho
/// (`ferramenta = None`) só desenha e deixa todo evento passar.
pub struct Camada<M> {
    pub tracos: Vec<TracoTela>,
    pub transformacao: Transformacao,
    /// Área útil (relativa ao widget) onde desenha e captura. `None` = o
    /// widget inteiro menos as `margens` (cabeçalhos, barras de rolagem).
    pub area: Option<Rectangle>,
    pub margens: Padding,
    pub ferramenta: Option<Ferramenta>,
    /// Pena da prévia ao vivo (cor com opacidade, espessura no documento).
    pub pena: Option<(Color, f32)>,
    /// Ids realçados (seleção do laço).
    pub selecionados: Vec<u64>,
    /// De onde vem a pressão real (mesa). `None` = só mouse/toque.
    pub fonte_pressao: Option<FontePressao>,
    /// Curva da prévia ao vivo: `(mínimo, gama)`, a mesma que o app usa
    /// ao gravar (`espessura × (mínimo + (1 − mínimo) × pressão^gama)`).
    pub curva_previa: (f32, f32),
    /// Sem pressão real: prévia da largura pela velocidade (unidades do
    /// documento por ms em que chega ao mais fino). `None` = largura fixa.
    pub velocidade_previa: Option<f32>,
    /// Indicador ao vivo da pressão durante o traço (modo de teste).
    pub mostrar_pressao: bool,
    pub mapear: fn(EventoCamada) -> M,
}

impl<M> Camada<M> {
    pub fn nova(tracos: Vec<TracoTela>, transformacao: Transformacao, mapear: fn(EventoCamada) -> M) -> Self {
        Self {
            tracos,
            transformacao,
            area: None,
            margens: Padding::ZERO,
            ferramenta: None,
            pena: None,
            selecionados: Vec::new(),
            fonte_pressao: None,
            curva_previa: (0.2, 1.0),
            velocidade_previa: None,
            mostrar_pressao: false,
            mapear,
        }
    }
    fn area_em(&self, bounds: Rectangle) -> Rectangle {
        self.area.unwrap_or_else(|| {
            let m = self.margens;
            Rectangle::new(
                Point::new(m.left, m.top),
                Size::new((bounds.width - m.left - m.right).max(0.0), (bounds.height - m.top - m.bottom).max(0.0)),
            )
        })
    }
}

/// Gesto em andamento (pontos no documento).
#[derive(Debug, Default)]
pub struct EstadoCamada {
    gesto: Option<Gesto>,
    /// Diagnóstico: movimentos do cursor desde o último evento anotado.
    movimentos_soltos: u32,
}

/// Diagnóstico da caneta: anota os eventos crus de mouse e toque que a
/// camada recebe (movimentos só contados, para não inundar o diário).
fn anotar_evento_cru(evento: &canvas::Event, estado: &mut EstadoCamada, cursor: mouse::Cursor) {
    if !crate::caneta::diagnostico_ligado() {
        return;
    }
    let descricao = match evento {
        canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
            estado.movimentos_soltos += 1;
            return;
        }
        canvas::Event::Mouse(mouse::Event::WheelScrolled { .. }) => return,
        canvas::Event::Mouse(e) => format!("mouse {e:?}"),
        canvas::Event::Touch(e) => format!("toque {e:?}"),
        _ => return,
    };
    let movimentos = std::mem::take(&mut estado.movimentos_soltos);
    let posicao = cursor.position().map(|p| format!("({:.0}, {:.0})", p.x, p.y)).unwrap_or_else(|| "fora".into());
    crate::caneta::anotar(|| format!("[camada] {descricao} no cursor {posicao}; {movimentos} movimentos antes"));
}

/// Número com vírgula decimal (pt-BR), 2 casas.
fn decimal_br(v: f32) -> String {
    format!("{v:.2}").replace('.', ",")
}

#[derive(Debug)]
struct Gesto {
    ferramenta: Ferramenta,
    inicio: Instant,
    pontos: Vec<PontoCaptura>,
    /// Quando cada ponto chegou (para casar a pressão de novo ao soltar:
    /// a amostra da mesa pode chegar um pouco depois do evento do Iced).
    instantes: Vec<Instant>,
    ultimo_tela: Point,
    /// Botão que começou o traço (a ponta da caneta pode chegar como outro botão).
    botao: Option<mouse::Button>,
    /// Movimentos recebidos desde o último registro do diagnóstico.
    movimentos: u32,
}

impl EstadoCamada {
    /// Há um traço/apagar/laço em andamento.
    pub fn em_gesto(&self) -> bool {
        self.gesto.is_some()
    }
}

impl<M> Camada<M> {
    fn pressao_agora(&self, instante: Instant) -> Option<f32> {
        self.fonte_pressao.as_ref().and_then(|f| f.pressao_em(instante))
    }

    fn comecar(&self, estado: &mut EstadoCamada, p_tela: Point, ferramenta: Ferramenta, botao: Option<mouse::Button>) -> Option<M> {
        let doc = self.transformacao.para_documento(p_tela);
        let agora = Instant::now();
        estado.gesto = Some(Gesto {
            ferramenta,
            inicio: agora,
            pontos: vec![PontoCaptura { x: doc.x, y: doc.y, tempo_ms: 0, pressao: self.pressao_agora(agora) }],
            instantes: vec![agora],
            ultimo_tela: p_tela,
            botao,
            movimentos: 0,
        });
        ferramenta.apaga().then(|| (self.mapear)(EventoCamada::Apagar {
            ferramenta,
            ponto: doc,
            raio: RAIO_BORRACHA_TELA / self.transformacao.escala.max(0.01),
        }))
    }

    fn mover(&self, estado: &mut EstadoCamada, p_tela: Point) -> Option<M> {
        let fonte = self.fonte_pressao.as_ref();
        let gesto = estado.gesto.as_mut()?;
        gesto.movimentos += 1;
        if gesto.ultimo_tela.distance(p_tela) < 0.75 {
            return None;
        }
        gesto.ultimo_tela = p_tela;
        let doc = self.transformacao.para_documento(p_tela);
        let agora = Instant::now();
        let tempo_ms = agora.duration_since(gesto.inicio).as_millis().min(u128::from(u32::MAX)) as u32;
        let pressao = fonte.and_then(|f| f.pressao_em(agora));
        gesto.pontos.push(PontoCaptura { x: doc.x, y: doc.y, tempo_ms, pressao });
        gesto.instantes.push(agora);
        gesto.ferramenta.apaga().then(|| (self.mapear)(EventoCamada::Apagar {
            ferramenta: gesto.ferramenta,
            ponto: doc,
            raio: RAIO_BORRACHA_TELA / self.transformacao.escala.max(0.01),
        }))
    }

    fn terminar(&self, estado: &mut EstadoCamada) -> Option<M> {
        let mut gesto = estado.gesto.take()?;
        // Casa a pressão de novo com tudo o que já chegou da mesa.
        if let Some(fonte) = &self.fonte_pressao {
            for (p, &t) in gesto.pontos.iter_mut().zip(&gesto.instantes) {
                if let Some(pr) = fonte.pressao_em(t) {
                    p.pressao = Some(pr);
                }
            }
        }
        crate::caneta::anotar(|| {
            let com: Vec<f32> = gesto.pontos.iter().filter_map(|p| p.pressao).collect();
            let resumo = if com.is_empty() {
                "sem pressão real".to_string()
            } else {
                let min = com.iter().copied().fold(f32::INFINITY, f32::min);
                let max = com.iter().copied().fold(0.0f32, f32::max);
                let med = com.iter().sum::<f32>() / com.len() as f32;
                format!("pressão em {} de {} pontos: mín {min:.2} méd {med:.2} máx {max:.2}", com.len(), gesto.pontos.len())
            };
            format!("[camada] fim do gesto ({:?}, botão {:?}): {} pontos, {} movimentos; {resumo}",
                gesto.ferramenta, gesto.botao, gesto.pontos.len(), gesto.movimentos)
        });
        let evento = match gesto.ferramenta {
            f if f.desenha() => EventoCamada::TracoTerminado { ferramenta: f, pontos: gesto.pontos },
            f if f.apaga() => EventoCamada::ApagarFim,
            _ => {
                if gesto.pontos.len() < 3 {
                    EventoCamada::Laco(Vec::new())
                } else {
                    EventoCamada::Laco(gesto.pontos.iter().map(|p| Point::new(p.x, p.y)).collect())
                }
            }
        };
        Some((self.mapear)(evento))
    }
}

impl<M> Camada<M> {
    /// Fração da espessura em cada ponto da prévia: pressão real pela
    /// `curva_previa`; sem ela, pela velocidade (se ligada). `None` = fixa.
    fn fatores_previa(&self, pontos: &[PontoCaptura]) -> Option<Vec<f32>> {
        let (minimo, gama) = self.curva_previa;
        let minimo = minimo.clamp(0.0, 1.0);
        let fator = |p: f32| minimo + (1.0 - minimo) * p.clamp(0.0, 1.0);
        if pontos.iter().any(|p| p.pressao.is_some()) {
            let gama = if gama.is_finite() { gama.clamp(0.1, 10.0) } else { 1.0 };
            return Some(pontos.iter().map(|p| p.pressao.map_or(1.0, |pr| fator(pr.clamp(0.0, 1.0).powf(gama)))).collect());
        }
        let vf = self.velocidade_previa?;
        if pontos.len() < 2 {
            return None;
        }
        let mut suave: Option<f32> = None;
        let mut saida = Vec::with_capacity(pontos.len());
        for s in pontos.windows(2) {
            let dt = s[1].tempo_ms.saturating_sub(s[0].tempo_ms).max(1) as f32;
            let v = (s[1].x - s[0].x).hypot(s[1].y - s[0].y) / dt;
            let atual = suave.map_or(v, |a| a + 0.35 * (v - a));
            suave = Some(atual);
            saida.push(fator(crate::caneta::previa_pressao_da_velocidade(atual, vf)));
        }
        saida.insert(0, saida[0]);
        Some(saida)
    }

    /// "pressão 0,63" (mesa), "velocidade 0,45" (sem mesa) ou "sem pressão".
    fn desenhar_indicador(&self, f: &mut Frame, gesto: &Gesto) {
        let ultimo = gesto.pontos.last().copied();
        let texto = match ultimo.and_then(|p| p.pressao) {
            Some(p) => format!("pressão {}", decimal_br(p)),
            None => match self.fatores_previa(&gesto.pontos).and_then(|v| v.last().copied()) {
                Some(fator) => format!("sem mesa: largura pela velocidade {}", decimal_br(fator)),
                None => "sem pressão (mouse)".into(),
            },
        };
        let origem = self.fonte_pressao.as_ref().map(|f| f.descricao()).filter(|d| !d.is_empty());
        let linha = match origem {
            Some(o) => format!("{texto}  ·  {o}"),
            None => texto,
        };
        let largura = linha.chars().count() as f32 * 6.6 + 16.0;
        f.fill_rectangle(Point::new(6.0, 6.0), Size::new(largura, 22.0), COR_PAINEL());
        f.stroke(
            &Path::rectangle(Point::new(6.0, 6.0), Size::new(largura, 22.0)),
            Stroke::default().with_color(COR_BORDA()).with_width(1.0),
        );
        f.fill_text(canvas::Text {
            content: linha,
            position: Point::new(14.0, 11.0),
            color: COR_TEXTO(),
            size: 12.0.into(),
            ..canvas::Text::default()
        });
    }
}

impl<M> Program<M> for Camada<M> {
    type State = EstadoCamada;

    fn update(&self, estado: &mut EstadoCamada, evento: canvas::Event, bounds: Rectangle, cursor: mouse::Cursor) -> (event::Status, Option<M>) {
        let Some(ferramenta) = self.ferramenta else {
            estado.gesto = None;
            return (event::Status::Ignored, None);
        };
        anotar_evento_cru(&evento, estado, cursor);
        let area = self.area_em(bounds);
        let local = |p: Point| Point::new(p.x - bounds.x, p.y - bounds.y);
        match evento {
            // A ponta da caneta chega como botão esquerdo (XWayland, Windows)
            // ou, conforme o compositor, como outro botão (BTN_TOUCH/BTN_STYLUS
            // = `Other(n)`): os dois começam o traço. Direito/meio/voltar não.
            canvas::Event::Mouse(mouse::Event::ButtonPressed(botao @ (mouse::Button::Left | mouse::Button::Other(_))))
                if estado.gesto.is_none() =>
            {
                match cursor.position_in(bounds) {
                    Some(p) if area.contains(p) => {
                        let msg = self.comecar(estado, p, ferramenta, Some(botao));
                        (event::Status::Captured, msg)
                    }
                    _ => (event::Status::Ignored, None),
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { position }) if estado.gesto.is_some() => {
                let msg = self.mover(estado, local(position));
                (event::Status::Captured, msg)
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(botao))
                if estado.gesto.as_ref().is_some_and(|g| g.botao == Some(botao)) =>
            {
                let msg = self.terminar(estado);
                (event::Status::Captured, msg)
            }
            canvas::Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                let p = local(position);
                if area.contains(p) && bounds.contains(position) && estado.gesto.is_none() {
                    let msg = self.comecar(estado, p, ferramenta, None);
                    (event::Status::Captured, msg)
                } else {
                    (event::Status::Ignored, None)
                }
            }
            canvas::Event::Touch(touch::Event::FingerMoved { position, .. }) if estado.gesto.is_some() => {
                let msg = self.mover(estado, local(position));
                (event::Status::Captured, msg)
            }
            canvas::Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. })
                if estado.gesto.is_some() =>
            {
                let msg = self.terminar(estado);
                (event::Status::Captured, msg)
            }
            _ => (event::Status::Ignored, None),
        }
    }

    fn draw(&self, estado: &EstadoCamada, renderer: &Renderer, _: &Theme, bounds: Rectangle, cursor: mouse::Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let area = self.area_em(bounds);
        let t = self.transformacao;
        frame.with_clip(area, |f| {
            // O recorte desloca a origem para o canto da área.
            let ajuste = |p: Point| {
                let q = t.para_tela(p);
                Point::new(q.x - area.x, q.y - area.y)
            };
            for traco in &self.tracos {
                if traco.contorno.is_empty() {
                    desenhar_traco(f, &traco.pontos, ajuste, traco.cor, traco.espessura * t.escala, traco.reto, traco.marca_texto);
                } else {
                    desenhar_contorno(f, &traco.contorno, ajuste, traco.cor);
                }
            }
            if !self.selecionados.is_empty() {
                let pts = self
                    .tracos
                    .iter()
                    .filter(|tr| self.selecionados.contains(&tr.id))
                    .flat_map(|tr| tr.pontos.iter().map(|&p| ajuste(p)));
                if let Some(caixa) = caixa_de(pts) {
                    let margem = 4.0;
                    let r = Path::rectangle(
                        Point::new(caixa.x - margem, caixa.y - margem),
                        Size::new(caixa.width + 2.0 * margem, caixa.height + 2.0 * margem),
                    );
                    f.stroke(&r, tracejado(1.2, &[5.0, 4.0]));
                }
            }
            if let Some(gesto) = &estado.gesto {
                let pts: Vec<Point> = gesto.pontos.iter().map(|p| Point::new(p.x, p.y)).collect();
                if gesto.ferramenta.desenha() {
                    if let Some((cor, espessura)) = self.pena {
                        let marca = gesto.ferramenta == Ferramenta::MarcaTexto;
                        match (!marca).then(|| self.fatores_previa(&gesto.pontos)).flatten() {
                            Some(fatores) => desenhar_previa_variavel(f, &pts, &fatores, ajuste, cor, espessura * t.escala),
                            None => desenhar_traco(f, &pts, ajuste, cor, espessura * t.escala, false, marca),
                        }
                    }
                    if self.mostrar_pressao {
                        self.desenhar_indicador(f, gesto);
                    }
                } else if gesto.ferramenta == Ferramenta::Laco && pts.len() > 1 {
                    let caminho = Path::new(|b| {
                        b.move_to(ajuste(pts[0]));
                        for &p in &pts[1..] {
                            b.line_to(ajuste(p));
                        }
                    });
                    f.stroke(&caminho, tracejado(1.4, &[6.0, 4.0]));
                }
            }
            // Borracha: círculo do alcance sob o cursor.
            if self.ferramenta.is_some_and(Ferramenta::apaga) {
                if let Some(p) = cursor.position_in(bounds) {
                    if area.contains(p) {
                        let c = Path::circle(Point::new(p.x - area.x, p.y - area.y), RAIO_BORRACHA_TELA);
                        f.stroke(&c, Stroke::default().with_color(COR_TEXTO_MUTED()).with_width(1.2));
                    }
                }
            }
        });
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(&self, estado: &EstadoCamada, bounds: Rectangle, cursor: mouse::Cursor) -> mouse::Interaction {
        match self.ferramenta {
            Some(_) if estado.gesto.is_some() => mouse::Interaction::Crosshair,
            Some(_) if cursor.position_in(bounds).is_some_and(|p| self.area_em(bounds).contains(p)) => mouse::Interaction::Crosshair,
            _ => mouse::Interaction::default(),
        }
    }
}

/// Contorno tracejado na cor de destaque (seleção e laço).
fn tracejado(largura: f32, segmentos: &'static [f32]) -> Stroke<'static> {
    Stroke {
        line_dash: LineDash { segments: segmentos, offset: 0 },
        ..Stroke::default().with_color(COR_AZUL_ACENTO()).with_width(largura)
    }
}

fn caixa_de(pontos: impl Iterator<Item = Point>) -> Option<Rectangle> {
    let mut caixa: Option<(f32, f32, f32, f32)> = None;
    for p in pontos {
        let c = caixa.get_or_insert((p.x, p.y, p.x, p.y));
        c.0 = c.0.min(p.x);
        c.1 = c.1.min(p.y);
        c.2 = c.2.max(p.x);
        c.3 = c.3.max(p.y);
    }
    caixa.map(|(x0, y0, x1, y1)| Rectangle::new(Point::new(x0, y0), Size::new(x1 - x0, y1 - y0)))
}

/// Desenha a linha central: curva pelos pontos médios (igual ao motor) ou
/// segmentos retos. Um ponto só vira um disco.
fn desenhar_traco(f: &mut Frame, pontos: &[Point], ajuste: impl Fn(Point) -> Point, cor: Color, largura: f32, reto: bool, marca: bool) {
    let pts: Vec<Point> = pontos.iter().map(|&p| ajuste(p)).collect();
    let Some(&p0) = pts.first() else { return };
    let largura = largura.max(0.5);
    if pts.len() == 1 {
        f.fill(&Path::circle(p0, largura / 2.0), cor);
        return;
    }
    let caminho = Path::new(|b| {
        b.move_to(p0);
        if reto || pts.len() == 2 {
            for &p in &pts[1..] {
                b.line_to(p);
            }
        } else {
            for i in 1..pts.len() - 1 {
                let fim = Point::new((pts[i].x + pts[i + 1].x) / 2.0, (pts[i].y + pts[i + 1].y) / 2.0);
                b.quadratic_curve_to(pts[i], fim);
            }
            b.line_to(pts[pts.len() - 1]);
        }
    });
    f.stroke(
        &caminho,
        Stroke::default()
            .with_color(cor)
            .with_width(largura)
            .with_line_cap(if marca { LineCap::Square } else { LineCap::Round })
            .with_line_join(LineJoin::Round),
    );
}

/// Contorno de largura variável (polígonos do motor), preenchido com a
/// regra não-zero (os pedaços se somam).
fn desenhar_contorno(f: &mut Frame, poligonos: &[Vec<Point>], ajuste: impl Fn(Point) -> Point, cor: Color) {
    let caminho = Path::new(|b| {
        for pol in poligonos.iter().filter(|p| p.len() >= 3) {
            b.move_to(ajuste(pol[0]));
            for &p in &pol[1..] {
                b.line_to(ajuste(p));
            }
            b.close();
        }
    });
    f.fill(&caminho, cor);
}

/// Prévia ao vivo com largura variável: um trecho reto por par de pontos,
/// pontas redondas, largura média dos dois pontos.
fn desenhar_previa_variavel(f: &mut Frame, pontos: &[Point], fatores: &[f32], ajuste: impl Fn(Point) -> Point, cor: Color, largura: f32) {
    let pts: Vec<Point> = pontos.iter().map(|&p| ajuste(p)).collect();
    if pts.len() == 1 {
        f.fill(&Path::circle(pts[0], (largura * fatores[0] / 2.0).max(0.25)), cor);
        return;
    }
    for (i, s) in pts.windows(2).enumerate() {
        let l = largura * (fatores[i] + fatores[i + 1]) / 2.0;
        f.stroke(
            &Path::line(s[0], s[1]),
            Stroke::default().with_color(cor).with_width(l.max(0.5)).with_line_cap(LineCap::Round),
        );
    }
}

/// Widget pronto: a camada ocupando todo o espaço (para pôr num `stack`).
pub fn camada<'a, M: 'a>(c: Camada<M>) -> Element<'a, M> {
    canvas::Canvas::new(c).width(Length::Fill).height(Length::Fill).into()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum Msg {
        Tinta(EventoCamada),
    }

    fn bounds() -> Rectangle {
        Rectangle::new(Point::new(100.0, 50.0), Size::new(800.0, 600.0))
    }

    fn mouse(e: mouse::Event) -> canvas::Event {
        canvas::Event::Mouse(e)
    }

    fn em(x: f32, y: f32) -> mouse::Cursor {
        mouse::Cursor::Available(Point::new(x, y))
    }

    #[test]
    fn traco_vira_evento_em_coordenadas_do_documento() {
        let mut camada = Camada::nova(Vec::new(), Transformacao { escala: 2.0, dx: 10.0, dy: 20.0 }, Msg::Tinta);
        camada.ferramenta = Some(Ferramenta::Caneta(0));
        // Cabeçalhos (90 × 30) e barras de rolagem (13) ficam fora.
        camada.margens = Padding { top: 30.0, right: 13.0, bottom: 13.0, left: 90.0 };
        let mut estado = EstadoCamada::default();
        // Fora da área útil (cabeçalho): deixa passar.
        let (s, m) = camada.update(&mut estado, mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), bounds(), em(120.0, 60.0));
        assert_eq!((s, m), (event::Status::Ignored, None));
        // Na barra de rolagem de baixo também.
        let (s, _) = camada.update(&mut estado, mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), bounds(), em(300.0, 645.0));
        assert_eq!(s, event::Status::Ignored);
        // Dentro: captura sem mensagem até soltar.
        let (s, m) = camada.update(&mut estado, mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), bounds(), em(210.0, 120.0));
        assert_eq!(s, event::Status::Captured);
        assert!(m.is_none() && estado.em_gesto());
        for (x, y) in [(230.0, 140.0), (230.2, 140.1), (250.0, 160.0)] {
            let (s, m) = camada.update(&mut estado, mouse(mouse::Event::CursorMoved { position: Point::new(x, y) }), bounds(), em(x, y));
            assert_eq!((s, m), (event::Status::Captured, None));
        }
        let (_, m) = camada.update(&mut estado, mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), bounds(), em(250.0, 160.0));
        match m {
            Some(Msg::Tinta(EventoCamada::TracoTerminado { ferramenta, pontos })) => {
                assert_eq!(ferramenta, Ferramenta::Caneta(0));
                // (210,120) na janela = (110,70) no widget = ((110-10)/2, (70-20)/2) no documento.
                assert_eq!(pontos.len(), 3, "o tremido abaixo de 0,75 px não entra");
                assert_eq!((pontos[0].x, pontos[0].y), (50.0, 25.0));
                assert_eq!((pontos[2].x, pontos[2].y), (70.0, 45.0));
            }
            outro => panic!("esperava TracoTerminado, veio {outro:?}"),
        }
        assert!(!estado.em_gesto());
    }

    #[test]
    fn borracha_emite_apagar_ao_vivo_e_fim_ao_soltar() {
        let mut camada = Camada::nova(Vec::new(), Transformacao { escala: 2.0, dx: 0.0, dy: 0.0 }, Msg::Tinta);
        camada.ferramenta = Some(Ferramenta::BorrachaPonto);
        let mut estado = EstadoCamada::default();
        let (_, m) = camada.update(&mut estado, mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), bounds(), em(140.0, 90.0));
        assert_eq!(m, Some(Msg::Tinta(EventoCamada::Apagar {
            ferramenta: Ferramenta::BorrachaPonto,
            ponto: Point::new(20.0, 20.0),
            raio: RAIO_BORRACHA_TELA / 2.0,
        })));
        let (_, m) = camada.update(&mut estado, mouse(mouse::Event::CursorMoved { position: Point::new(160.0, 90.0) }), bounds(), em(160.0, 90.0));
        assert!(matches!(m, Some(Msg::Tinta(EventoCamada::Apagar { ponto, .. })) if ponto == Point::new(30.0, 20.0)));
        let (_, m) = camada.update(&mut estado, mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), bounds(), em(160.0, 90.0));
        assert_eq!(m, Some(Msg::Tinta(EventoCamada::ApagarFim)));
    }

    #[test]
    fn laco_devolve_poligono_e_fora_do_modo_nada_e_capturado() {
        let mut camada = Camada::nova(Vec::new(), Transformacao::default(), Msg::Tinta);
        camada.ferramenta = Some(Ferramenta::Laco);
        let mut estado = EstadoCamada::default();
        camada.update(&mut estado, mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), bounds(), em(110.0, 60.0));
        for (x, y) in [(200.0, 60.0), (200.0, 150.0), (110.0, 150.0)] {
            camada.update(&mut estado, mouse(mouse::Event::CursorMoved { position: Point::new(x, y) }), bounds(), em(x, y));
        }
        let (_, m) = camada.update(&mut estado, mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), bounds(), em(110.0, 150.0));
        assert_eq!(m, Some(Msg::Tinta(EventoCamada::Laco(vec![
            Point::new(10.0, 10.0), Point::new(100.0, 10.0), Point::new(100.0, 100.0), Point::new(10.0, 100.0),
        ]))));
        // Sem ferramenta (modo normal): o clique vai para a grade de baixo.
        camada.ferramenta = None;
        let (s, m) = camada.update(&mut estado, mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), bounds(), em(300.0, 300.0));
        assert_eq!((s, m), (event::Status::Ignored, None));
        assert_eq!(camada.mouse_interaction(&estado, bounds(), em(300.0, 300.0)), mouse::Interaction::default());
        // A roda nunca é capturada (a planilha continua rolando no modo desenho).
        camada.ferramenta = Some(Ferramenta::Caneta(1));
        let roda = mouse(mouse::Event::WheelScrolled { delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 } });
        assert_eq!(camada.update(&mut estado, roda, bounds(), em(300.0, 300.0)).0, event::Status::Ignored);
    }

    #[test]
    fn estado_escolhe_abre_menu_e_para() {
        let mut e = Estado::default();
        assert!(e.pena().is_none());
        assert!(e.atualizar(Evento::Escolher(Ferramenta::Caneta(1))));
        assert!(e.ativo && e.menu.is_none());
        assert_eq!(e.pena().unwrap().cor, PALETA_CANETA[2]);
        // Clicar de novo na caneta ativa abre o menu dela.
        e.atualizar(Evento::Escolher(Ferramenta::Caneta(1)));
        assert_eq!(e.menu, Some(Ferramenta::Caneta(1)));
        e.atualizar(Evento::Cor(Ferramenta::Caneta(1), PALETA_CANETA[5]));
        e.atualizar(Evento::Espessura(Ferramenta::Caneta(1), 4.5));
        assert_eq!(e.pena(), Some(Pena { cor: PALETA_CANETA[5], espessura: 4.5 }));
        assert!(!e.atualizar(Evento::Espessura(Ferramenta::Caneta(1), f32::NAN)));
        e.atualizar(Evento::Escolher(Ferramenta::BorrachaTraco));
        assert!(e.pena().is_none() && e.ativo);
        e.atualizar(Evento::AlternarFormas);
        assert!(e.tinta_para_forma);
        e.atualizar(Evento::Parar);
        assert!(!e.ativo && e.ferramenta_ativa().is_none());
    }

    #[test]
    fn faixa_e_menu_montam_nos_dois_temas_com_icones_do_catalogo() {
        for nome in ["pencil", "highlight", "eraser", "lasso", "wand", "hand-stop", "chevron-down"] {
            assert!(crate::icones::existe(nome), "{nome}");
        }
        // Os estilos leem os tokens do tema na hora de desenhar; não trocar o
        // tema global aqui (o teste das paletas de componentes.rs o usa em paralelo).
        {
            let mut e = Estado::default();
            let grupos = grupos_faixa(&e);
            assert_eq!(grupos.len(), LARGURAS_GRUPOS.len());
            assert!(menu_pena(&e).is_none());
            e.atualizar(Evento::AlternarMenu(Ferramenta::MarcaTexto));
            assert!(menu_pena(&e).is_some());
            let _: Element<'_, Evento> = sobrepor_menu(text("base").into(), &e, 50.0, 120.0, Size::new(1280.0, 800.0), |ev| ev);
            let tracos = vec![TracoTela {
                id: 1, pontos: vec![Point::ORIGIN, Point::new(5.0, 5.0)], cor: Color::BLACK,
                espessura: 2.0, reto: false, marca_texto: false, contorno: Vec::new(),
            }, TracoTela {
                id: 2, pontos: vec![Point::ORIGIN], cor: Color::BLACK, espessura: 2.0, reto: false, marca_texto: false,
                contorno: vec![vec![Point::ORIGIN, Point::new(5.0, 0.0), Point::new(5.0, 5.0)]],
            }];
            let _: Element<'_, Msg> = camada(Camada::nova(tracos, Transformacao::default(), Msg::Tinta));
        }
    }

    #[test]
    fn ponta_da_caneta_como_outro_botao_tambem_risca() {
        // No Wayland a ponta pode chegar como BTN_TOUCH/BTN_STYLUS (`Other`).
        let mut camada = Camada::nova(Vec::new(), Transformacao::default(), Msg::Tinta);
        camada.ferramenta = Some(Ferramenta::Caneta(0));
        let mut estado = EstadoCamada::default();
        let toque = mouse::Button::Other(0x14a);
        let (s, _) = camada.update(&mut estado, mouse(mouse::Event::ButtonPressed(toque)), bounds(), em(150.0, 100.0));
        assert_eq!(s, event::Status::Captured);
        camada.update(&mut estado, mouse(mouse::Event::CursorMoved { position: Point::new(180.0, 120.0) }), bounds(), em(180.0, 120.0));
        // Soltar outro botão não encerra; soltar o mesmo, sim.
        let (_, m) = camada.update(&mut estado, mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), bounds(), em(180.0, 120.0));
        assert!(m.is_none() && estado.em_gesto());
        let (_, m) = camada.update(&mut estado, mouse(mouse::Event::ButtonReleased(toque)), bounds(), em(180.0, 120.0));
        assert!(matches!(m, Some(Msg::Tinta(EventoCamada::TracoTerminado { ref pontos, .. })) if pontos.len() == 2));
        // Botão direito nunca começa traço.
        let (s, _) = camada.update(&mut estado, mouse(mouse::Event::ButtonPressed(mouse::Button::Right)), bounds(), em(150.0, 100.0));
        assert_eq!(s, event::Status::Ignored);
    }

    #[test]
    fn pressao_da_mesa_e_casada_com_cada_ponto_e_mouse_fica_sem() {
        let fonte = FontePressao::nova();
        let mut camada = Camada::nova(Vec::new(), Transformacao::default(), Msg::Tinta);
        camada.ferramenta = Some(Ferramenta::Caneta(0));
        camada.fonte_pressao = Some(fonte.clone());
        let mut estado = EstadoCamada::default();
        // Mouse: nenhuma amostra recente da mesa.
        camada.update(&mut estado, mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), bounds(), em(150.0, 100.0));
        camada.update(&mut estado, mouse(mouse::Event::CursorMoved { position: Point::new(160.0, 100.0) }), bounds(), em(160.0, 100.0));
        let (_, m) = camada.update(&mut estado, mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), bounds(), em(160.0, 100.0));
        let Some(Msg::Tinta(EventoCamada::TracoTerminado { pontos, .. })) = m else { panic!("traço") };
        assert!(pontos.iter().all(|p| p.pressao.is_none()));
        // Caneta: a amostra chega junto (ou logo depois) do evento do Iced.
        fonte.registrar(Instant::now(), 0.3);
        camada.update(&mut estado, mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), bounds(), em(150.0, 100.0));
        camada.update(&mut estado, mouse(mouse::Event::CursorMoved { position: Point::new(170.0, 100.0) }), bounds(), em(170.0, 100.0));
        fonte.registrar(Instant::now(), 0.8);
        let (_, m) = camada.update(&mut estado, mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), bounds(), em(170.0, 100.0));
        let Some(Msg::Tinta(EventoCamada::TracoTerminado { pontos, .. })) = m else { panic!("traço") };
        assert!(pontos.iter().all(|p| p.pressao.is_some()), "{pontos:?}");
        assert_eq!(pontos.last().unwrap().pressao, Some(0.8), "casado de novo ao soltar");
    }

    #[test]
    fn previa_da_largura_segue_a_pressao_ou_a_velocidade() {
        let mut camada = Camada::nova(Vec::new(), Transformacao::default(), Msg::Tinta);
        camada.curva_previa = (0.2, 1.0);
        let pc = |x: f32, t: u32, p: Option<f32>| PontoCaptura { x, y: 0.0, tempo_ms: t, pressao: p };
        let f = camada.fatores_previa(&[pc(0.0, 0, Some(0.0)), pc(1.0, 1, Some(1.0))]).unwrap();
        assert!((f[0] - 0.2).abs() < 1e-6 && (f[1] - 1.0).abs() < 1e-6);
        // Sem pressão e sem velocidade ligada: largura fixa.
        assert!(camada.fatores_previa(&[pc(0.0, 0, None), pc(1.0, 10, None)]).is_none());
        camada.velocidade_previa = Some(2.0);
        let devagar = camada.fatores_previa(&[pc(0.0, 0, None), pc(1.0, 10, None)]).unwrap();
        let rapido = camada.fatores_previa(&[pc(0.0, 0, None), pc(30.0, 3, None)]).unwrap();
        assert!(devagar[1] > rapido[1]);
        assert_eq!(decimal_br(0.63), "0,63");
    }
}
