//! Componentes genéricos reutilizáveis da faixa do FeatherOffice.
//! A mensagem e os eventos continuam sendo propriedade do aplicativo:
//! o crate de UI nunca importa nenhum motor de documentos, planilhas ou slides.

use iced::widget::{button, column, container, pick_list, row, svg, text, Space};
use iced::{Alignment, Element, Length};

use crate::tema::{estilo_botao_grande, estilo_botao_pequeno, estilo_linha, COR_TEXTO_MUTED};

/// Altura-alvo de menus para uma migração futura: Iced 0.13 não expõe
/// `menu_height` em PickList/ComboBox. O popup atual usa o scroll nativo
/// e os limites disponíveis da janela, sem teto fixo de 256 px.
pub const ALTURA_MAX_DROPDOWN: f32 = 256.0;

/// Botão de ferramenta com ícone acima do rótulo (faixa/Ribbon).
pub fn botao_grande<'a, M: Clone + 'a>(
    icone: svg::Svg<'static, iced::Theme>,
    rotulo: &str,
    mensagem: Option<M>,
    ativo: bool,
) -> Element<'a, M> {
    button(
        column![
            icone,
            text(rotulo.to_owned()).size(11).wrapping(text::Wrapping::None),
        ]
        .align_x(Alignment::Center)
        .spacing(3),
    )
    .padding([4, 8])
    .style(estilo_botao_grande(ativo))
    .on_press_maybe(mensagem)
    .into()
}

/// Botão compacto de ferramenta, com ícone à esquerda do rótulo.
pub fn botao_pequeno<'a, M: Clone + 'a>(
    icone: svg::Svg<'static, iced::Theme>,
    rotulo: &str,
    mensagem: Option<M>,
    ativo: bool,
) -> Element<'a, M> {
    button(
        row![
            icone,
            text(rotulo.to_owned()).size(11).wrapping(text::Wrapping::None),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
    )
    .padding([2, 6])
    .style(estilo_botao_pequeno(ativo))
    .on_press_maybe(mensagem)
    .into()
}

/// Botão só com o ícone: o nome da ação fica na dica, sem texto duplicado
/// dentro da faixa. Reutilizável no Sheet, Slide e Docs.
pub fn botao_icone<'a, M: Clone + 'a + 'static>(
    icone: svg::Svg<'static, iced::Theme>,
    dica: &str,
    mensagem: Option<M>,
    ativo: bool,
) -> Element<'a, M> {
    let controle = button(icone)
        .padding([4, 6])
        .style(estilo_botao_pequeno(ativo))
        .on_press_maybe(mensagem);
    iced::widget::tooltip(
        controle,
        container(text(dica.to_owned()).size(12))
            .padding([4, 6])
            .style(crate::tema::estilo_dica),
        iced::widget::tooltip::Position::Bottom,
    )
    .into()
}

/// Dropdown de aparência compartilhada: fonte, tamanho, formato, estilos,
/// escolha e foco continuam nativos do Iced 0.13. Mensagem pertence ao aplicativo.
pub fn lista_suspensa<'a, M: Clone + 'a>(
    rotulo: &'static str,
    opcoes: &'static [&'static str],
    selecionado: Option<&'static str>,
    escolher: impl Fn(&'static str) -> M + 'a,
    largura: f32,
) -> Element<'a, M> {
    pick_list(opcoes, selecionado, escolher)
        .placeholder(rotulo)
        .text_size(11)
        .text_shaping(text::Shaping::Advanced)
        .padding([4, 8])
        .width(Length::Fixed(largura))
        .style(crate::tema::estilo_menu_da_faixa)
        .menu_style(crate::tema::estilo_menu_popup)
        .into()
}

/// Lista tipada para apps que usam valores numéricos, enums ou &str dinâmico.
/// Identidade visual e scroll seguem os widgets nativos Iced 0.13.
/// Navegação por setas, foco, Enter e Esc seguem o widget nativo Iced 0.13.
pub fn lista_tipada<'a, M, T>(
    rotulo: &'static str,
    opcoes: &'a [T],
    selecionado: Option<T>,
    escolher: impl Fn(T) -> M + 'a,
    largura: f32,
) -> Element<'a, M>
where
    M: Clone + 'a,
    T: Clone + PartialEq + ToString + 'a,
{
    pick_list(opcoes, selecionado, escolher)
        .placeholder(rotulo)
        .text_size(11)
        .text_shaping(text::Shaping::Advanced)
        .padding([4, 8])
        .width(Length::Fixed(largura))
        .style(crate::tema::estilo_menu_da_faixa)
        .menu_style(crate::tema::estilo_menu_popup)
        .into()
}

/// Dropdown com pesquisa por digitação e navegação por setas/Enter.
/// O estado das opções pertence ao App; visual e acessibilidade, ao Iced.
/// Reutilizável em qualquer editor da suíte, sem importar o motor de planilhas.
pub fn lista_pesquisavel<'a, M: Clone + 'a>(
    estado: &'a iced::widget::combo_box::State<String>,
    dica: &'static str,
    escolher: impl Fn(String) -> M + 'static,
    largura: f32,
) -> Element<'a, M> {
    iced::widget::combo_box(estado, dica, None::<&String>, escolher)
        .size(11.0)
        .padding([4, 8])
        .width(Length::Fixed(largura))
        .input_style(crate::tema::estilo_entrada_suspensa)
        .menu_style(crate::tema::estilo_menu_popup)
        .into()
}

/// Grupo vertical da faixa: conteúdo, legenda e linha separadora.
pub fn grupo_faixa<'a, M: 'a>(
    itens: Element<'a, M>,
    rotulo: &'static str,
) -> Element<'a, M> {
    row![
        container(
            column![
                container(itens).height(Length::Fill).align_y(Alignment::Center),
                text(rotulo)
                    .size(11)
                    .wrapping(text::Wrapping::None)
                    .color(COR_TEXTO_MUTED()),
            ]
            .align_x(Alignment::Center)
            .spacing(2),
        )
        .padding([4, 12]),
        container(Space::with_width(1))
            .height(Length::Fill)
            .style(estilo_linha),
    ]
    .into()
}

/// Alturas explícitas da Ribbon compacta. A legenda não disputa espaço com
/// os controles (evita que fique escondida pela barra de rolagem horizontal).
pub const ALTURA_CONTEUDO_GRUPO_COMPACTO: f32 = 68.0;
pub const ALTURA_GRUPO_COMPACTO: f32 = 96.0;

/// Variação para faixas de duas linhas, como a de Fonte no FeatherSheet.
/// Deixa a legenda sempre na parte inferior do grupo, dentro da Ribbon,
/// sem alterar o estilo dos demais aplicativos que usam `grupo_faixa`.
pub fn grupo_faixa_compacto<'a, M: 'a>(
    itens: Element<'a, M>,
    rotulo: &'static str,
) -> Element<'a, M> {
    row![
        container(
            column![
                container(itens)
                    .height(Length::Fixed(ALTURA_CONTEUDO_GRUPO_COMPACTO))
                    .align_y(Alignment::Center),
                text(rotulo)
                    .size(11)
                    .wrapping(text::Wrapping::None)
                    .color(COR_TEXTO_MUTED()),
            ]
            .align_x(Alignment::Center)
            .spacing(3),
        )
        .padding([3, 10])
        .height(Length::Fixed(ALTURA_GRUPO_COMPACTO)),
        container(Space::with_width(1))
            .height(Length::Fixed(ALTURA_GRUPO_COMPACTO))
            .style(estilo_linha),
    ]
    .into()
}

#[cfg(test)]
mod testes {
    use super::{grupo_faixa_compacto, ALTURA_CONTEUDO_GRUPO_COMPACTO, ALTURA_GRUPO_COMPACTO};
    use crate::tema::{acento_hex, definir_tema, ACENTOS};
    use iced::widget::text;

    #[test]
    fn grupo_compacto_reserva_legenda_antes_da_rolagem() {
        // 68 de corpo + 3 de intervalo + 14 de legenda + 6 de padding
        // cabem em 96; a Ribbon reserva 100 para seus grupos.
        assert!(ALTURA_CONTEUDO_GRUPO_COMPACTO + 3.0 + 14.0 + 6.0 <= ALTURA_GRUPO_COMPACTO);
        assert!(ALTURA_GRUPO_COMPACTO <= 100.0);
        let _grupo: iced::Element<'_, ()> =
            grupo_faixa_compacto(text("controles").into(), "Fonte");
    }

    #[test]
    fn paletas_e_acento_fora_do_intervalo() {
        // O tema usa estado global para refletir as Configurações em toda a
        // interface. Testar no mesmo caso evita duas threads concorrentes
        // alterando o acento enquanto a outra lê o resultado.
        for (indice, &(_, claro, escuro)) in ACENTOS.iter().enumerate() {
            definir_tema(false, indice);
            assert_eq!(acento_hex(), claro);
            definir_tema(true, indice);
            assert_eq!(acento_hex(), escuro);
        }
        definir_tema(false, usize::MAX);
        assert_eq!(acento_hex(), ACENTOS[ACENTOS.len() - 1].1);
        definir_tema(false, 0);
    }
}
