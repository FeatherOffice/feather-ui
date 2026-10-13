//! Estrutura visual da área de configurações do FeatherOffice.
//! Não conhece documentos nem preferências: recebe apenas eventos do app.
use iced::{Alignment, Background, Border, Element, Length};
use iced::widget::{button, column, container, row, text, Space};
use crate::{icones, tema::*};

pub fn linha<'a, M: 'a>(
    titulo: &str,
    dica: &str,
    controle: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    column![
        row![
            column![
                text(titulo.to_owned()).size(13.5).color(COR_TEXTO()),
                text(dica.to_owned()).size(11.5).color(COR_TEXTO_MUTED()),
            ].spacing(3).width(Length::Fill),
            controle.into(),
        ].spacing(14).align_y(Alignment::Center).padding([14, 0]),
        container(Space::with_height(1)).width(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(COR_BORDA_SUAVE())),
                ..Default::default()
            }),
    ].spacing(0).width(Length::Fill).into()
}

pub fn menu<'a, M: Clone + 'a>(
    titulo: &str,
    itens: impl IntoIterator<Item = (String, bool, M)>,
) -> Element<'a, M> {
    let mut col = column![
        text(titulo.to_owned()).size(16).color(COR_TEXTO()),
        Space::with_height(10),
    ].spacing(4);
    for (nome, ativo, msg) in itens {
        let simbolo = match nome.as_str() {
            "Aparência" => "palette",
            "Editor" => "file-code",
            "Atalhos" => "keyboard",
            "Sobre" => "info-circle",
            _ => "settings",
        };
        col = col.push(
            button(row![icones::icone(simbolo, 16.0), text(nome).size(13)]
                .spacing(10).align_y(Alignment::Center))
                .width(Length::Fill)
                .padding([9, 11])
                .style(estilo_botao_pequeno(ativo))
                .on_press(msg),
        );
    }
    container(col.padding([20, 12])).width(190).height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(COR_FUNDO_JANELA())),
            border: Border { color: COR_BORDA_SUAVE(), width: 1.0, ..Default::default() },
            ..Default::default()
        }).into()
}

pub fn tela<'a, M: Clone + 'a + 'static>(
    menu: Element<'a, M>,
    titulo: &str,
    corpo: Element<'a, M>,
    fechar: Option<(&str, M)>,
) -> Element<'a, M> {
    let mut cabecalho = row![
        text(titulo.to_owned()).size(20).color(COR_TEXTO()),
        Space::with_width(Length::Fill),
    ].align_y(Alignment::Center);
    if let Some((_nome, msg)) = fechar {
        cabecalho = cabecalho.push(
            button(icones::icone("x", 15.0)).padding([5, 7])
                .style(estilo_botao_pequeno(false)).on_press(msg)
        );
    }
    row![
        menu,
        rolagem(container(
            column![cabecalho, corpo].spacing(12).max_width(820)
        ).padding([20, 26]).width(Length::Fill))
            .height(Length::Fill).width(Length::Fill),
    ].height(Length::Fill).into()
}
