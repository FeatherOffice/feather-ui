//! Diálogo interno padrão da suíte: painel sobreposto dentro da janela do
//! aplicativo (não é janela do sistema operacional).
//!
//! Anatomia (ver `GUIA-JANELAS.md`, seção "Diálogo interno"):
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │ [ic] Título                             [x] │  cabeçalho
//! │ subtítulo apagado (opcional)                │
//! │                                             │
//! │ corpo (altura pelo conteúdo; rola acima     │
//! │        do teto)                             │
//! │                                             │
//! │ [esquerda]            [Cancelar] [Primário] │  rodapé opcional
//! └─────────────────────────────────────────────┘
//! ```
//!
//! O painel fica centralizado na horizontal, logo abaixo da faixa (`topo`),
//! sobre um véu escurecido que bloqueia a tela de trás. Clicar no véu fecha
//! (desligável). A tecla Esc é tratada pelo aplicativo, que manda a mesma
//! mensagem de fechar.
//!
//! ```ignore
//! use feather_ui::dialogo::{self, Dialogo};
//! let corpo = column![
//!     dialogo::secao("Visão geral"),
//!     dialogo::linha_rotulo_valor("Linhas", "1.234"),
//! ];
//! Dialogo::new("Estatísticas", corpo)
//!     .icone("chart-bar")
//!     .ao_fechar(Msg::Fechar)
//!     .topo(207.0)
//!     .altura_janela(tamanho.height)
//!     .secundario("Cancelar", Some(Msg::Fechar))
//!     .primario("Aplicar", Some(Msg::Aplicar))
//!     .sobre(base)
//! ```

use iced::widget::{button, column, container, mouse_area, opaque, row, stack, text, Space};
use iced::{Alignment, Background, Border, Element, Font, Length, Shadow, Vector};

use crate::tema::{
    estilo_botao_pequeno, estilo_botao_primario, estilo_botao_secundario, COR_BORDA,
    COR_BORDA_SUAVE, COR_PAINEL, COR_PAINEL_FAIXA, COR_SOMBRA, COR_TEXTO, COR_TEXTO_MUTED,
    COR_VEU,
};

/// Largura padrão (dentro da faixa 420–600 do guia).
pub const LARGURA_PADRAO: f32 = 480.0;
/// Teto padrão da altura do painel; acima dele o corpo rola.
pub const ALTURA_MAXIMA_PADRAO: f32 = 620.0;
/// Distância padrão do topo da janela quando o aplicativo não informa a faixa.
pub const TOPO_PADRAO: f32 = 96.0;
/// Margem mínima entre o painel e as bordas da janela.
pub const MARGEM: f32 = 12.0;
/// Espaço reservado ao cabeçalho (título + subtítulo + espaçamentos) e ao
/// rodapé ao calcular quanto o corpo pode crescer antes de rolar.
const RESERVA_CABECALHO: f32 = 64.0;
const RESERVA_SUBTITULO: f32 = 22.0;
const RESERVA_RODAPE: f32 = 54.0;
/// Espaçamento padrão entre os blocos do diálogo.
pub const ESPACO: f32 = 12.0;

fn semibold() -> Font {
    Font { weight: iced::font::Weight::Semibold, ..Font::default() }
}

/// Estilo do painel do diálogo: fundo do painel, borda fina, raio 12, sombra.
pub fn estilo_painel(_tema: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COR_PAINEL())),
        text_color: Some(COR_TEXTO()),
        border: Border { color: COR_BORDA(), width: 1.0, radius: 12.0.into() },
        shadow: Shadow { color: COR_SOMBRA(), offset: Vector::new(0.0, 6.0), blur_radius: 22.0 },
    }
}

/// Estilo do véu atrás do diálogo.
pub fn estilo_veu(_tema: &iced::Theme) -> container::Style {
    container::Style { background: Some(Background::Color(COR_VEU())), ..container::Style::default() }
}

/// Bloco discreto (fundo da faixa, raio 9) para agrupar linhas no corpo.
pub fn estilo_bloco(_tema: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COR_PAINEL_FAIXA())),
        border: Border { color: COR_BORDA_SUAVE(), width: 1.0, radius: 9.0.into() },
        ..container::Style::default()
    }
}

/// Botão "fechar" do cabeçalho: ícone `x` do Tabler, dica "Fechar (Esc)".
pub fn botao_fechar<'a, M: Clone + 'a + 'static>(msg: M) -> Element<'a, M> {
    crate::formatacao::dica(
        button(crate::icones::icone("x", 15.0))
            .padding([5, 7])
            .style(estilo_botao_pequeno(false))
            .on_press(msg),
        "Fechar (Esc)",
    )
}

/// Botão de rodapé no estilo primário (ação principal, uma por diálogo).
/// Sem mensagem, fica desabilitado.
pub fn botao_primario<'a, M: Clone + 'a>(rotulo: impl Into<String>, msg: Option<M>) -> Element<'a, M> {
    button(text(rotulo.into()).size(13))
        .padding([7, 16])
        .style(estilo_botao_primario)
        .on_press_maybe(msg)
        .into()
}

/// Botão de rodapé no estilo secundário. Sem mensagem, fica desabilitado.
pub fn botao_secundario<'a, M: Clone + 'a>(rotulo: impl Into<String>, msg: Option<M>) -> Element<'a, M> {
    button(text(rotulo.into()).size(13))
        .padding([7, 14])
        .style(estilo_botao_secundario)
        .on_press_maybe(msg)
        .into()
}

/// Título de seção dentro do corpo: caixa alta, 10,5, semibold, apagado.
pub fn secao<'a, M: 'a>(titulo: &str) -> Element<'a, M> {
    text(titulo.to_uppercase())
        .size(10.5)
        .font(semibold())
        .color(COR_TEXTO_MUTED())
        .into()
}

/// Uma linha rótulo / valor: rótulo apagado à esquerda, valor à direita.
/// Compacta (linha de ~24 px) para listas de números.
pub fn linha_rotulo_valor<'a, M: 'a>(rotulo: impl Into<String>, valor: impl Into<String>) -> Element<'a, M> {
    container(
        row![
            text(rotulo.into()).size(12.5).color(COR_TEXTO_MUTED()).width(Length::Fill),
            text(valor.into()).size(13).color(COR_TEXTO()),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding([3, 0])
    .width(Length::Fill)
    .into()
}

/// Grade de pares rótulo / valor em `colunas` colunas, dentro de um bloco
/// discreto. A ordem é por linha: (0,1) na primeira linha, (2,3) na segunda…
pub fn grade_rotulo_valor<'a, M: 'a>(pares: Vec<(String, String)>, colunas: usize) -> Element<'a, M> {
    let colunas = colunas.max(1);
    let mut linhas = column![].spacing(2);
    let mut iter = pares.into_iter().peekable();
    while iter.peek().is_some() {
        let mut linha = row![].spacing(22);
        for _ in 0..colunas {
            linha = match iter.next() {
                Some((r, v)) => linha.push(container(linha_rotulo_valor(r, v)).width(Length::Fill)),
                None => linha.push(Space::with_width(Length::Fill)),
            };
        }
        linhas = linhas.push(linha);
    }
    container(linhas).padding([6, 12]).width(Length::Fill).style(estilo_bloco).into()
}

/// Aviso de conteúdo vazio: ícone `info-circle` apagado + frase, num bloco
/// discreto. Usar no lugar de várias seções vazias.
pub fn aviso_vazio<'a, M: 'a>(texto: impl Into<String>) -> Element<'a, M> {
    aviso("info-circle", texto)
}

/// Aviso com um ícone do catálogo (ex.: `alert-triangle` para erro).
pub fn aviso<'a, M: 'a>(icone: &str, texto: impl Into<String>) -> Element<'a, M> {
    container(
        row![
            crate::icones::icone(icone, 17.0),
            text(texto.into()).size(12.5).color(COR_TEXTO_MUTED()).width(Length::Fill),
        ]
        .spacing(9)
        .align_y(Alignment::Center),
    )
    .padding([10, 12])
    .width(Length::Fill)
    .style(estilo_bloco)
    .into()
}

/// Folga à direita do corpo, onde a barra de rolagem (6 px) fica sem cobrir
/// o conteúdo. O painel tem padding direito menor na mesma medida, então o
/// conteúdo continua alinhado com o `x` e com os botões do rodapé.
const CANAL_BARRA: f32 = 10.0;

/// Rolagem do corpo: altura pelo conteúdo (Shrink) e barra que só aparece
/// quando o conteúdo passa do teto (a barra "embutida" de `tema::rolagem`
/// desenha o trilho sempre). As folgas de 3 px em cima e embaixo evitam que o
/// recorte da camada coma o acento de uma letra maiúscula (Ã) da primeira linha
/// e a borda do último bloco.
pub fn rolagem<'a, M: 'a>(corpo: impl Into<Element<'a, M>>) -> Element<'a, M> {
    use iced::widget::scrollable::{Direction, Scrollbar};
    iced::widget::scrollable(
        container(corpo.into())
            .width(Length::Fill)
            .padding(iced::Padding { top: 3.0, bottom: 3.0, right: CANAL_BARRA, left: 0.0 }),
    )
    .direction(Direction::Vertical(Scrollbar::new().width(6).scroller_width(6)))
    .width(Length::Fill)
    .into()
}

/// Diálogo interno em construção (padrão builder). Termina em
/// [`Dialogo::sobre`] (sobre a tela, com véu) ou [`Dialogo::painel`] (só o
/// painel, para testes ou para encaixar noutra camada).
pub struct Dialogo<'a, M> {
    titulo: String,
    icone: Option<String>,
    subtitulo: Option<String>,
    corpo: Element<'a, M>,
    fechar: Option<M>,
    esquerda: Vec<Element<'a, M>>,
    botoes: Vec<Element<'a, M>>,
    largura: f32,
    altura_maxima: f32,
    topo: f32,
    altura_janela: Option<f32>,
    largura_janela: Option<f32>,
    fora_fecha: bool,
    veu: bool,
    rolar: bool,
    modal: bool,
    a_direita: bool,
}

impl<'a, M: Clone + 'a + 'static> Dialogo<'a, M> {
    pub fn new(titulo: impl Into<String>, corpo: impl Into<Element<'a, M>>) -> Self {
        Self {
            titulo: titulo.into(),
            icone: None,
            subtitulo: None,
            corpo: corpo.into(),
            fechar: None,
            esquerda: Vec::new(),
            botoes: Vec::new(),
            largura: LARGURA_PADRAO,
            altura_maxima: ALTURA_MAXIMA_PADRAO,
            topo: TOPO_PADRAO,
            altura_janela: None,
            largura_janela: None,
            fora_fecha: true,
            veu: true,
            rolar: true,
            modal: true,
            a_direita: false,
        }
    }

    /// Ícone do catálogo Tabler antes do título.
    pub fn icone(mut self, nome: &str) -> Self {
        self.icone = Some(nome.to_owned());
        self
    }

    /// Uma frase apagada logo abaixo do título.
    pub fn subtitulo(mut self, texto: impl Into<String>) -> Self {
        self.subtitulo = Some(texto.into());
        self
    }

    /// Mensagem de fechar: mostra o `x` no cabeçalho e fecha ao clicar fora.
    pub fn ao_fechar(mut self, msg: M) -> Self {
        self.fechar = Some(msg);
        self
    }

    /// Largura preferida do painel (fica menor se a janela não couber).
    pub fn largura(mut self, largura: f32) -> Self {
        self.largura = largura;
        self
    }

    /// Teto da altura do painel; acima dele o corpo rola.
    pub fn altura_maxima(mut self, altura: f32) -> Self {
        self.altura_maxima = altura;
        self
    }

    /// Distância do topo da janela até o painel (normalmente, o fim da faixa).
    pub fn topo(mut self, topo: f32) -> Self {
        self.topo = topo;
        self
    }

    /// Altura da janela, para o teto respeitar o espaço abaixo do `topo`.
    pub fn altura_janela(mut self, altura: f32) -> Self {
        self.altura_janela = Some(altura);
        self
    }

    /// Largura da janela, para o painel encolher em telas estreitas.
    pub fn largura_janela(mut self, largura: f32) -> Self {
        self.largura_janela = Some(largura);
        self
    }

    /// Se clicar no véu fecha o diálogo (padrão: sim, quando há `ao_fechar`).
    pub fn clique_fora_fecha(mut self, sim: bool) -> Self {
        self.fora_fecha = sim;
        self
    }

    /// Se desenha o véu escurecido atrás (padrão: sim).
    pub fn veu(mut self, sim: bool) -> Self {
        self.veu = sim;
        self
    }

    /// Diálogo não modal: sem véu e sem camada que bloqueia a tela de trás,
    /// para a pessoa poder clicar na planilha (escolher células) com ele
    /// aberto. Clicar fora não fecha.
    pub fn nao_modal(mut self) -> Self {
        self.modal = false;
        self
    }

    /// Encosta o painel à direita (com margem) em vez de centralizar.
    pub fn a_direita(mut self) -> Self {
        self.a_direita = true;
        self
    }

    /// Desliga a rolagem automática do corpo (quando o próprio corpo já tem
    /// uma área rolável com altura própria).
    pub fn sem_rolagem(mut self) -> Self {
        self.rolar = false;
        self
    }

    /// Botão primário no rodapé (vai por último, à direita).
    pub fn primario(mut self, rotulo: impl Into<String>, msg: Option<M>) -> Self {
        self.botoes.push(botao_primario(rotulo, msg));
        self
    }

    /// Botão secundário no rodapé, à esquerda dos que vierem depois.
    pub fn secundario(mut self, rotulo: impl Into<String>, msg: Option<M>) -> Self {
        self.botoes.push(botao_secundario(rotulo, msg));
        self
    }

    /// Qualquer elemento no grupo da direita do rodapé (na ordem de chamada).
    pub fn botao(mut self, elemento: impl Into<Element<'a, M>>) -> Self {
        self.botoes.push(elemento.into());
        self
    }

    /// Elemento no canto esquerdo do rodapé (ex.: ação destrutiva, contagem).
    pub fn rodape_esquerda(mut self, elemento: impl Into<Element<'a, M>>) -> Self {
        self.esquerda.push(elemento.into());
        self
    }

    /// Largura efetiva do painel.
    pub fn largura_efetiva(&self) -> f32 {
        match self.largura_janela {
            Some(j) => self.largura.min((j - 2.0 * MARGEM).max(260.0)),
            None => self.largura,
        }
    }

    /// Teto efetivo da altura do painel inteiro.
    pub fn teto_efetivo(&self) -> f32 {
        match self.altura_janela {
            Some(j) => self.altura_maxima.min((j - self.topo - MARGEM).max(160.0)),
            None => self.altura_maxima,
        }
    }

    /// Quanto o corpo pode crescer antes de rolar.
    pub fn teto_corpo(&self) -> f32 {
        let mut reserva = RESERVA_CABECALHO;
        if self.subtitulo.is_some() {
            reserva += RESERVA_SUBTITULO;
        }
        if !self.botoes.is_empty() || !self.esquerda.is_empty() {
            reserva += RESERVA_RODAPE;
        }
        (self.teto_efetivo() - reserva).max(60.0)
    }

    /// Só o painel (cabeçalho, corpo e rodapé), sem véu nem posição.
    pub fn painel(self) -> Element<'a, M> {
        let largura = self.largura_efetiva();
        let teto_corpo = self.teto_corpo();
        let teto = self.teto_efetivo();
        let Dialogo { titulo, icone, subtitulo, corpo, fechar, esquerda, botoes, rolar, .. } = self;

        let mut cabecalho = row![].spacing(9).align_y(Alignment::Center);
        if let Some(nome) = icone {
            cabecalho = cabecalho.push(crate::icones::icone(&nome, 19.0));
        }
        cabecalho = cabecalho
            .push(text(titulo).size(17).font(semibold()).color(COR_TEXTO()).width(Length::Fill));
        if let Some(msg) = fechar {
            cabecalho = cabecalho.push(botao_fechar(msg));
        }
        let mut topo = column![cabecalho].spacing(4);
        if let Some(sub) = subtitulo {
            topo = topo.push(text(sub).size(12).color(COR_TEXTO_MUTED()));
        }

        let corpo: Element<'a, M> = if rolar {
            container(rolagem(corpo)).max_height(teto_corpo).into()
        } else {
            container(corpo).padding(iced::Padding { right: CANAL_BARRA, ..iced::Padding::ZERO }).into()
        };
        let topo = container(topo).padding(iced::Padding { right: CANAL_BARRA, ..iced::Padding::ZERO });
        let mut tudo = column![topo, corpo].spacing(ESPACO + 2.0);
        if !botoes.is_empty() || !esquerda.is_empty() {
            let mut rodape = row![].spacing(8).align_y(Alignment::Center);
            for e in esquerda {
                rodape = rodape.push(e);
            }
            rodape = rodape.push(Space::with_width(Length::Fill));
            for b in botoes {
                rodape = rodape.push(b);
            }
            tudo = tudo.push(container(rodape).padding(iced::Padding { right: CANAL_BARRA, ..iced::Padding::ZERO }));
        }
        container(tudo)
            .padding(iced::Padding { top: 18.0, bottom: 18.0, left: 20.0, right: 20.0 - CANAL_BARRA })
            .width(Length::Fixed(largura))
            .max_height(teto)
            .style(estilo_painel)
            .into()
    }

    /// O diálogo sobre `base`: véu, painel centralizado na horizontal e
    /// colado ao `topo`. Cliques não atravessam para a tela de trás.
    pub fn sobre(self, base: impl Into<Element<'a, M>>) -> Element<'a, M> {
        let topo = self.topo;
        let veu = self.veu;
        let modal = self.modal;
        let a_direita = self.a_direita;
        let fechar_fora = if self.fora_fecha { self.fechar.clone() } else { None };
        let painel = self.painel();

        let posicionado = container(opaque(painel)).width(Length::Fill);
        let posicionado = if a_direita {
            posicionado.align_x(iced::alignment::Horizontal::Right).padding([0.0, 2.0 * MARGEM])
        } else {
            posicionado.align_x(iced::alignment::Horizontal::Center)
        };
        let camada = column![Space::with_height(Length::Fixed(topo.max(0.0))), posicionado]
            .width(Length::Fill)
            .height(Length::Fill);
        let base: Element<'a, M> = base.into();
        if !modal {
            return stack![base, camada].width(Length::Fill).height(Length::Fill).into();
        }
        let fundo = container(Space::new(Length::Fill, Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill);
        let fundo = if veu { fundo.style(estilo_veu) } else { fundo };
        let mut area_fora = mouse_area(fundo);
        if let Some(msg) = fechar_fora {
            area_fora = area_fora.on_press(msg);
        }
        stack![base, opaque(area_fora), camada]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use iced::widget::column;

    #[derive(Debug, Clone, PartialEq)]
    enum Msg {
        Fechar,
        Ok,
    }

    fn montar(escuro: bool) {
        crate::tema::definir_tema(escuro, 1);
        let corpo = column![
            secao::<Msg>("Visão geral"),
            linha_rotulo_valor("Linhas", "10"),
            grade_rotulo_valor(vec![("A".into(), "1".into()), ("B".into(), "2".into()), ("C".into(), "3".into())], 2),
            aviso_vazio("Nada aqui"),
        ];
        let _: Element<'_, Msg> = Dialogo::new("Teste", corpo)
            .icone("chart-bar")
            .subtitulo("Somente leitura")
            .ao_fechar(Msg::Fechar)
            .secundario("Cancelar", Some(Msg::Fechar))
            .primario("Aplicar", Some(Msg::Ok))
            .rodape_esquerda(text("3 itens"))
            .topo(200.0)
            .altura_janela(700.0)
            .sobre(Space::new(Length::Fill, Length::Fill));
        let _: Element<'_, Msg> = Dialogo::new("Não modal", text("x"))
            .ao_fechar(Msg::Fechar)
            .nao_modal()
            .a_direita()
            .sobre(Space::new(Length::Fill, Length::Fill));
    }

    #[test]
    fn monta_nos_dois_temas() {
        montar(false);
        montar(true);
        crate::tema::definir_tema(false, 0);
    }

    #[test]
    fn teto_respeita_janela_e_reserva_cabecalho_e_rodape() {
        let d: Dialogo<'_, Msg> = Dialogo::new("T", text("x")).topo(200.0).altura_janela(500.0);
        assert_eq!(d.teto_efetivo(), 500.0 - 200.0 - MARGEM);
        let sem_rodape = d.teto_corpo();
        let d = d.primario("Ok", Some(Msg::Ok));
        assert!(d.teto_corpo() < sem_rodape, "rodapé tira espaço do corpo");
        let d: Dialogo<'_, Msg> = Dialogo::new("T", text("x")).altura_maxima(400.0).altura_janela(2000.0);
        assert_eq!(d.teto_efetivo(), 400.0, "janela grande não passa do teto pedido");
    }

    #[test]
    fn largura_encolhe_em_janela_estreita() {
        let d: Dialogo<'_, Msg> = Dialogo::new("T", text("x")).largura(560.0).largura_janela(400.0);
        assert_eq!(d.largura_efetiva(), 400.0 - 2.0 * MARGEM);
        let d: Dialogo<'_, Msg> = Dialogo::new("T", text("x")).largura(560.0).largura_janela(1600.0);
        assert_eq!(d.largura_efetiva(), 560.0);
    }

    #[test]
    fn icones_usados_existem_no_catalogo() {
        for nome in ["x", "info-circle", "chart-bar"] {
            assert!(crate::icones::existe(nome), "{nome}");
        }
    }
}
