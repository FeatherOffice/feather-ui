//! Controles de formatação da Ribbon, independentes de Slide/Docs/Sheet.
//!
//! Este módulo fornece APENAS estado e visual de popovers, seletores e prévias.
//! A escolha de fonte, tamanho, cor ou borda é sempre uma mensagem do consumidor.
//! Não guarda estilos de células, documentos ou formas e não importa motores.

use iced::widget::{button, column, container, row, text, tooltip, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Vector};

use crate::{componentes::lista_tipada, icones, seletor_cor, tema};

/// Um único popover aberto por vez. IDs são exclusivos da interface, nunca do documento.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EstadoPopover {
    aberto: Option<String>,
}

impl EstadoPopover {
    pub fn esta_aberto(&self, id: &str) -> bool {
        self.aberto.as_deref() == Some(id)
    }

    /// Repetir a ativação do mesmo controle fecha o popover.
    pub fn alternar(&mut self, id: impl Into<String>) {
        let id = id.into();
        self.aberto = if self.esta_aberto(&id) { None } else { Some(id) };
    }

    pub fn fechar(&mut self) {
        self.aberto = None;
    }

    pub fn aberto(&self) -> Option<&str> {
        self.aberto.as_deref()
    }
}

/// Dica acessível visualmente, com fundo, borda e tema unificados.
pub fn dica<'a, M: Clone + 'a>(
    conteudo: impl Into<Element<'a, M>>,
    descricao: &str,
) -> Element<'a, M> {
    tooltip(
        conteudo,
        container(text(descricao.to_owned()).size(12))
            .padding([4, 8])
            .style(tema::estilo_dica),
        tooltip::Position::Bottom,
    )
    .into()
}

/// Superfície de popover. Posicionamento e overlay permanecem com o aplicativo
/// para preservar recorte, foco e limites da janela Iced em cada editor.
pub fn popover<'a, M: 'a>(
    titulo: &str,
    conteudo: Element<'a, M>,
) -> Element<'a, M> {
    container(
        column![text(titulo.to_owned()).size(12), conteudo]
            .spacing(8)
            .align_x(Alignment::Start),
    )
    .padding(10)
    .width(Length::Shrink)
    .style(|_| container::Style {
        background: Some(Background::Color(tema::COR_PAINEL())),
        text_color: Some(tema::COR_TEXTO()),
        border: Border {
            color: tema::COR_BORDA(),
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.22),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
    })
    .into()
}

/// Botão com amostra RGBA/sem cor e mensagem tipada do editor.
/// A cor é somente pré-visualização; o seletor HSV/alfa completo permanece em
/// `seletor_cor::painel`. Entradas hex inválidas não são exibidas como preto.
pub fn botao_cor<'a, M: Clone + 'a>(
    rotulo: &str,
    hex: &str,
    mensagem: Option<M>,
    ativo: bool,
) -> Element<'a, M> {
    let valido = seletor_cor::hex_valido(hex);
    let cor = if valido {
        let [r, g, b, a] = seletor_cor::rgba_de(hex);
        Some(Color::from_rgba8(r, g, b, a as f32 / 255.0))
    } else {
        None
    };
    let amostra = container(
        if hex.is_empty() {
            text("×").size(11).color(tema::COR_TEXTO_MUTED())
        } else if !valido {
            text("?").size(11).color(tema::COR_TEXTO_MUTED())
        } else {
            text("").size(11)
        },
    )
    .width(Length::Fixed(16.0))
    .height(Length::Fixed(16.0))
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(move |_| container::Style {
        background: cor.map(Background::Color),
        border: Border {
            color: tema::COR_BORDA(),
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    });
    button(row![amostra, text(rotulo.to_owned()).size(11)]
        .spacing(5).align_y(Alignment::Center))
        .padding([4, 7])
        .style(tema::estilo_botao_pequeno(ativo))
        .on_press_maybe(mensagem)
        .into()
}


/// Ícone sobre uma pequena faixa com a cor atual, no padrão da Ribbon do Excel.
/// O controle não aplica estilo: comunica-se com o seletor do aplicativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoCorCompacta {
    /// Balde de tinta: preenchimento da célula.
    Preenchimento,
    /// Letra A: cor do texto.
    Texto,
}

/// Controle estreito sem rótulo ao lado: balde/A em cima, retângulo colorido
/// embaixo. O tooltip preserva o nome acessível e o clique abre o seletor
/// existente, que continua responsável pela escolha da cor.
pub fn botao_cor_compacto<'a, M: Clone + 'a>(
    tipo: TipoCorCompacta,
    hex: &str,
    mensagem: Option<M>,
    ativo: bool,
) -> Element<'a, M> {
    let cor = if seletor_cor::hex_valido(hex) {
        let [r, g, b, a] = seletor_cor::rgba_de(hex);
        Color::from_rgba8(r, g, b, a as f32 / 255.0)
    } else {
        // Sem estilo direto: mostrar o fundo e o texto padrões do tema.
        // Não gravar estas cores como formatação da célula.
        match tipo {
            TipoCorCompacta::Preenchimento => tema::COR_PAINEL(),
            TipoCorCompacta::Texto => tema::COR_TEXTO(),
        }
    };
    let amostra = container(Space::with_height(Length::Fixed(4.0)))
        .width(Length::Fixed(20.0))
        .height(Length::Fixed(4.0))
        .style(move |_| container::Style {
            background: Some(Background::Color(cor)),
            border: Border {
                color: tema::COR_BORDA(),
                width: 0.6,
                radius: 1.0.into(),
            },
            ..Default::default()
        });
    let (icone, descricao): (Element<'a, M>, &str) = match tipo {
        TipoCorCompacta::Preenchimento => (
            icones::icone("bucket", 18.0).into(),
            "Cor de preenchimento da célula",
        ),
        TipoCorCompacta::Texto => (
            text("A")
                .size(18)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..iced::Font::DEFAULT
                })
                .into(),
            "Cor do texto",
        ),
    };
    let controle = button(
        column![icone, amostra]
            .spacing(1)
            .align_x(Alignment::Center),
    )
    .width(Length::Fixed(32.0))
    .padding([2, 5])
    .style(tema::estilo_botao_pequeno(ativo))
    .on_press_maybe(mensagem);
    dica(controle, descricao)
}

/// Dropdown de fontes que preserva tipos e callbacks locais do aplicativo.
pub fn seletor_fonte<'a, M, T>(
    opcoes: &'a [T],
    selecionada: Option<T>,
    escolher: impl Fn(T) -> M + 'a,
    largura: f32,
) -> Element<'a, M>
where
    M: Clone + 'a,
    T: Clone + PartialEq + ToString + 'a,
{
    lista_tipada("Fonte", opcoes, selecionada, escolher, largura)
}

/// Dropdown de tamanho. Não limita a tipografia do motor: cada app define
/// sua escala, seus itens e o tipo da mensagem.
pub fn seletor_tamanho<'a, M, T>(
    opcoes: &'a [T],
    selecionado: Option<T>,
    escolher: impl Fn(T) -> M + 'a,
    largura: f32,
) -> Element<'a, M>
where
    M: Clone + 'a,
    T: Clone + PartialEq + ToString + 'a,
{
    lista_tipada("Tamanho", opcoes, selecionado, escolher, largura)
}

/// Dropdown visual de bordas; nunca aplica mutações em células ou tabelas.
pub fn seletor_bordas<'a, M, T>(
    opcoes: &'a [T],
    selecionada: Option<T>,
    escolher: impl Fn(T) -> M + 'a,
    largura: f32,
) -> Element<'a, M>
where
    M: Clone + 'a,
    T: Clone + PartialEq + ToString + 'a,
{
    lista_tipada("Bordas", opcoes, selecionada, escolher, largura)
}

/// Prévia neutra da tipografia selecionada; não altera objetos do documento.
pub fn previa_estilo<'a, M: 'a>(
    amostra: &str,
    cor_hex: &str,
    tamanho: f32,
    negrito: bool,
) -> Element<'a, M> {
    let cor = if seletor_cor::hex_valido(cor_hex) {
        let [r, g, b, a] = seletor_cor::rgba_de(cor_hex);
        Color::from_rgba8(r, g, b, a as f32 / 255.0)
    } else {
        tema::COR_TEXTO()
    };
    let fonte = iced::Font {
        weight: if negrito {
            iced::font::Weight::Bold
        } else {
            iced::font::Weight::Normal
        },
        ..iced::Font::DEFAULT
    };
    container(
        column![
            text(amostra.to_owned())
                .size(if tamanho.is_finite() { tamanho.clamp(6.0, 96.0) } else { 12.0 })
                .font(fonte)
                .color(cor),
            Space::with_height(1),
        ]
        .spacing(2),
    )
    .padding([6, 8])
    .style(|_| container::Style {
        background: Some(Background::Color(tema::COR_PAINEL())),
        border: Border {
            color: tema::COR_BORDA(),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .into()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn popover_exclusivo_alterna_e_fecha_sem_comandos_de_documento() {
        let mut estado = EstadoPopover::default();
        assert_eq!(estado.aberto(), None);
        estado.alternar("fonte");
        assert!(estado.esta_aberto("fonte"));
        estado.alternar("cor");
        assert!(estado.esta_aberto("cor"));
        assert!(!estado.esta_aberto("fonte"));
        estado.alternar("cor");
        assert_eq!(estado.aberto(), None);
        estado.alternar("borda");
        estado.fechar();
        assert_eq!(estado.aberto(), None);
    }

    #[test]
    fn controles_aceitam_mensagens_proprias_sem_motor() {
        let _: Element<'_, u8> = botao_cor("Texto", "#33445580", Some(1), true);
        let _: Element<'_, u8> = botao_cor("Fundo", "", None, false);
        let _: Element<'_, u8> = botao_cor("Inválida", "#12xx90", Some(2), false);
        let _: Element<'_, u8> = botao_cor_compacto(TipoCorCompacta::Preenchimento, "#5f6368", Some(1), true);
        let _: Element<'_, u8> = botao_cor_compacto(TipoCorCompacta::Texto, "", Some(2), false);
        let _: Element<'_, u8> = botao_cor_compacto(TipoCorCompacta::Preenchimento, "#ff00ff80", None, false);
        assert!(icones::existe("bucket"), "o balde deve ser um SVG real, não o fallback");
        let _: Element<'_, u8> = seletor_fonte(&["sans", "mono"], Some("sans"), |_| 3, 112.0);
        let _: Element<'_, u8> = seletor_tamanho(&[10u16, 12, 14], Some(12), |_| 4, 64.0);
        let _: Element<'_, u8> = seletor_bordas(&["Todas", "Nenhuma"], None, |_| 5, 91.0);
        let _: Element<'_, u8> = previa_estilo("Exemplo", "#33445580", f32::NAN, true);
        let _: Element<'_, u8> = popover("Cor", text("Testando").into());
        let _: Element<'_, u8> = dica(text("Ajuda"), "Dica");
    }
}
