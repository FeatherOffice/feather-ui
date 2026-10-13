//! Barra de edição e superfícies de navegação comuns aos aplicativos Iced.
//! Eventos e operações continuam pertencendo a cada aplicativo.
use iced::widget::{button, column, container, row, text, tooltip, mouse_area, svg, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};

pub const BARRA_ALTURA: f32 = 42.0;

/// Os eventos continuam sendo do programa hospedeiro.
pub struct AcoesJanela<M> {
    pub arrastar: M,
    pub minimizar: M,
    pub maximizar: M,
    pub fechar: M,
}

use crate::{icones::icone, marca, tema::*};

pub struct Acao<M> {
    pub icone: &'static str,
    pub dica: &'static str,
    pub mensagem: Option<M>,
}

/// Pena vetorial do produto ativo — cor de fábrica ou cor personalizada.
pub fn emblema<'a, M: 'a>() -> Element<'a, M> {
    marca::icone_atual(27.).into()
}

/// Ação compacta com descrição acessível ao passar o ponteiro.
pub fn acao<'a, M: Clone + 'static>(nome: &str, dica: &str, mensagem: Option<M>) -> Element<'a, M> {
    acao_icone(icone(nome, 19.), dica, mensagem)
}

/// Mesmo botão da barra, aceitando ícones próprios do aplicativo (Slide/CAD).
pub fn acao_icone<'a, M: Clone + 'static>(
    desenho: iced::widget::svg::Svg<'static, iced::Theme>,
    dica: &str,
    mensagem: Option<M>,
) -> Element<'a, M> {
    tooltip(
        button(desenho).padding([7, 9])
            .style(estilo_botao_pequeno(false))
            .on_press_maybe(mensagem),
        container(text(dica.to_owned()).size(12))
            .padding([4, 8]).style(estilo_dica),
        tooltip::Position::Bottom,
    ).into()
}

/// Linha superior do editor, compatível com mensagens próprias de cada app.
/// Controles da janela (minimizar/arrastar) ficam na aplicação hospedeira.
pub fn barra_editor<'a, M: Clone + 'static>(
    arquivo: &str,
    alterado: bool,
    abrir_arquivos: M,
    acoes: impl IntoIterator<Item = Acao<M>>,
    abrir_configuracoes: M,
) -> Element<'a, M> {
    let mut itens = row![
        emblema(),
        button(text("Arquivos").size(12.5)).padding([7, 10])
            .style(estilo_botao_pequeno(false)).on_press(abrir_arquivos),
        Space::with_width(6),
    ].spacing(2).align_y(Alignment::Center);
    for item in acoes {
        itens = itens.push(acao(item.icone, item.dica, item.mensagem));
    }
    itens = itens.push(Space::with_width(10))
        .push(row![
            text(arquivo.to_owned()).size(13),
            text(if alterado { "• não salvo" } else { "salvo" })
                .size(11.5).color(COR_TEXTO_MUTED()),
        ].spacing(8).align_y(Alignment::Center))
        .push(Space::with_width(Length::Fill))
        .push(acao("settings", "Configurações", Some(abrir_configuracoes)));
    column![
        container(itens).padding([0, 12]).width(Length::Fill)
            .height(Length::Fixed(BARRA_ALTURA)).style(estilo_container_topo),
        container(Space::with_height(1)).width(Length::Fill).style(estilo_linha),
    ].into()
}

/// Ícones de janela com os mesmos traços Tabler usados pelo FeatherSlide.
fn icone_janela(caminho: &str) -> svg::Svg<'static, iced::Theme> {
    let data = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#5f6368\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{caminho}</svg>"
    );
    svg::Svg::new(svg::Handle::from_memory(data.into_bytes())).width(16).height(16)
}

/// Copiado do FeatherSlide: largura 48, altura 42, hover neutro e fechamento vermelho.
pub fn controles_janela<'a, M: Clone + 'static>(
    acoes: AcoesJanela<M>,
) -> Element<'a, M> {
    let botao = |desenho: svg::Svg<'static, iced::Theme>, msg: M, fechar: bool| -> Element<'a, M> {
        button(container(desenho).width(Length::Fill).height(Length::Fill)
            .align_x(Alignment::Center).align_y(Alignment::Center))
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(BARRA_ALTURA))
            .padding(0)
            .style(move |_, status| {
                let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
                button::Style {
                    background: Some(Background::Color(if hover {
                        if fechar { Color::from_rgb8(0xe8, 0x11, 0x23) } else { COR_HOVER() }
                    } else { Color::TRANSPARENT })),
                    text_color: COR_TEXTO(),
                    border: Border::default(),
                    shadow: iced::Shadow::default(),
                }
            })
            .on_press(msg).into()
    };
    container(row![
        botao(icone_janela("<path d=\"M5 12h14\"/>"), acoes.minimizar, false),
        botao(icone_janela("<rect x=\"5\" y=\"5\" width=\"14\" height=\"14\" rx=\"1\"/>"), acoes.maximizar, false),
        botao(icone_janela("<path d=\"M18 6L6 18M6 6l12 12\"/>"), acoes.fechar, true),
    ]).style(estilo_container_topo).into()
}

/// Aplica a barra personalizada ao corpo de qualquer janela auxiliar (Configurações, F1, F12).
pub fn janela_com_barra<'a, M: Clone + 'static>(
    titulo: &str,
    conteudo: Element<'a, M>,
    acoes: AcoesJanela<M>,
) -> Element<'a, M> {
    let nome = container(row![
        text(titulo.to_owned()).size(13),
        Space::with_width(Length::Fill),
    ].align_y(Alignment::Center))
        .padding([0, 14]).align_y(Alignment::Center)
        .width(Length::Fill).height(Length::Fixed(BARRA_ALTURA))
        .style(estilo_container_topo);
    let arrastar = acoes.arrastar.clone();
    let barra = row![
        mouse_area(nome).on_press(arrastar),
        controles_janela(acoes),
    ].align_y(Alignment::Center);
    let linha = container(Space::with_height(1)).width(Length::Fill).style(estilo_linha);
    column![barra, linha, container(conteudo).width(Length::Fill).height(Length::Fill)].into()
}

/// Combina uma barra de edição do app com os controles nativos desenhados.
pub fn barra_integrada<'a, M: Clone + 'static>(
    barra: Element<'a, M>,
    acoes: AcoesJanela<M>,
) -> Element<'a, M> {
    let arrastar = acoes.arrastar.clone();
    column![
        row![
            mouse_area(barra).on_press(arrastar),
            controles_janela(acoes),
        ].align_y(Alignment::Center),
        container(Space::with_height(1)).width(Length::Fill).style(estilo_linha),
    ].into()
}

/// Menu de arquivos da suíte, com as ações disponíveis do aplicativo.
pub fn menu_arquivos<'a, M: Clone + 'a>(
    aplicativo: &str,
    opcoes: impl IntoIterator<Item = (&'static str, &'static str, M)>,
) -> Element<'a, M> {
    let mut itens = column![
        text(aplicativo.to_owned()).size(18),
        text("Arquivos e ferramentas").size(12).color(COR_TEXTO_MUTED()),
        Space::with_height(8),
    ].spacing(6);
    for (icone_nome, rotulo, mensagem) in opcoes {
        itens = itens.push(
            button(row![icone(icone_nome, 17.), text(rotulo).size(13)]
                .spacing(10).align_y(Alignment::Center))
                .width(Length::Fill).padding([9, 12])
                .style(estilo_botao_pequeno(false)).on_press(mensagem),
        );
    }
    container(itens.padding(14)).width(280)
        .style(estilo_painel_miniaturas).into()
}

/// Cabeçalho usado por Ajuda (F1), Monitor (F12) e outros painéis utilitários.
pub fn painel_utilitario<'a, M: Clone + 'a>(
    titulo: &str,
    conteudo: Element<'a, M>,
    fechar: M,
) -> Element<'a, M> {
    container(column![
        row![
            text(titulo.to_owned()).size(20),
            Space::with_width(Length::Fill),
            button(text("Fechar").size(12)).padding([6, 12])
                .style(estilo_botao_secundario).on_press(fechar),
        ].align_y(Alignment::Center),
        conteudo,
    ].spacing(14)).padding(22).width(Length::Fill)
        .style(estilo_painel_miniaturas).into()
}
