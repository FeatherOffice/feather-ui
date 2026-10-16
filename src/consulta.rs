//! Peças da janela de consulta da suíte (o padrão da Ajuda F1), genéricas na
//! mensagem: lateral com grupos, cabeçalho com a marca, cartões em grade,
//! divisor e o corpo de leitura com rodapé fixo. A Ajuda F1, o "Primeiros
//! passos" e o "O que há de novo" montam a tela com elas, para as três
//! ficarem iguais. Cores só pelos tokens do tema (GUIA-JANELAS.md).
use crate::{icones, tema};
use iced::widget::button::Status;
use iced::widget::{button, column, container, row, scrollable, svg, text, Space};
use iced::{Alignment, Background, Border, Element, Font, Length, Theme};

/// Altura única dos cartões: todos iguais, a grade não fica desalinhada.
pub const ALTURA_CARTAO: f32 = 118.;
/// Largura fixa da lateral de navegação.
pub const LARGURA_LATERAL: f32 = 255.;

/// Texto semibold (títulos dos quatro degraus da hierarquia).
pub fn negrito<'a, M: 'static>(conteudo: impl Into<String>, tamanho: f32) -> Element<'a, M> {
    text(conteudo.into()).size(tamanho).font(Font {
        weight: iced::font::Weight::Semibold, ..Font::DEFAULT
    }).into()
}

/// Texto de apoio, apagado.
pub fn apagado<'a, M: 'static>(conteudo: impl Into<String>, tamanho: f32) -> Element<'a, M> {
    text(conteudo.into()).size(tamanho).line_height(1.4)
        .color(tema::COR_TEXTO_MUTED()).into()
}

/// Cartão e botões claros da F1: painel com borda; ao passar o mouse, pastel.
pub fn estilo_cartao(_: &Theme, status: Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(
            if matches!(status, Status::Hovered | Status::Pressed) {
                tema::COR_AZUL_FUNDO()
            } else {
                tema::COR_PAINEL()
            }
        )),
        text_color: tema::COR_TEXTO(),
        border: Border { color: tema::COR_BORDA(), width: 1., radius: 11.0.into() },
        ..Default::default()
    }
}

/// Item da lateral: ativo em pastel, mouse por cima em `COR_HOVER`.
pub fn estilo_nav(_: &Theme, status: Status, ativo: bool) -> button::Style {
    let fundo = if ativo {
        Some(Background::Color(tema::COR_AZUL_FUNDO()))
    } else if matches!(status, Status::Hovered | Status::Pressed) {
        Some(Background::Color(tema::COR_HOVER()))
    } else {
        None
    };
    button::Style {
        background: fundo,
        text_color: tema::COR_TEXTO(),
        border: Border { radius: 9.0.into(), ..Default::default() },
        ..Default::default()
    }
}

/// Rótulo de grupo da lateral, em caixa alta (`BÁSICO`, `VERSÕES`…).
pub fn rotulo_grupo<'a, M: 'static>(rotulo: &str) -> Element<'a, M> {
    container(text(rotulo.to_uppercase()).size(10.5).color(tema::COR_TEXTO_MUTED()))
        .padding(iced::Padding { left: 15., right: 5., top: 13., bottom: 4. }).into()
}

/// Item da lateral: ícone 18 + texto 12,5. Sem destino fica desabilitado.
pub fn item_nav<'a, M: Clone + 'static>(
    rotulo: impl Into<String>, icone: &str, ativo: bool, destino: Option<M>,
) -> Element<'a, M> {
    let conteudo = row![
        icones::icone_traco(icone, 18.),
        text(rotulo.into()).size(12.5).width(Length::Fill),
    ].spacing(12).align_y(Alignment::Center);
    button(conteudo).width(Length::Fill)
        .padding([10, 13]).style(move |t, s| estilo_nav(t, s, ativo))
        .on_press_maybe(destino).into()
}

/// A lateral inteira (255 px, rolável) com um rodapé opcional fixo embaixo.
pub fn lateral<'a, M: 'static>(
    itens: impl Into<Element<'a, M>>, rodape: Option<Element<'a, M>>,
) -> Element<'a, M> {
    let mut corpo = column![
        scrollable(container(itens).padding([14, 12]).width(Length::Fill))
            .height(Length::Fill),
    ];
    if let Some(r) = rodape {
        corpo = corpo.push(container(r).padding([12, 14]).width(Length::Fill));
    }
    container(corpo.height(Length::Fill))
        .width(Length::Fixed(LARGURA_LATERAL))
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(tema::COR_PAINEL_FAIXA())),
            border: Border { color: tema::COR_BORDA_SUAVE(), width: 1., ..Default::default() },
            ..Default::default()
        }).into()
}

/// Ícone num quadrado pastel (o "selo" dos cartões).
pub fn icone_pastel<'a, M: 'static>(icone: &str, tamanho: f32) -> Element<'a, M> {
    container(icones::icone_traco(icone, tamanho))
        .padding(13).style(|_| container::Style {
            background: Some(Background::Color(tema::COR_AZUL_FUNDO())),
            border: Border { radius: 12.0.into(), ..Default::default() },
            ..Default::default()
        }).into()
}

/// Corta o resumo de um cartão em ~84 caracteres, com reticências.
pub fn resumir(texto: &str) -> String {
    let mut s: String = texto.chars().take(84).collect();
    if texto.chars().count() > 84 { s.push('…'); }
    s
}

/// Cartão da F1: selo + título 13,5 + resumo 11,5. Com destino vira botão
/// (com a seta à direita); sem destino é só leitura, com a mesma aparência.
pub fn cartao<'a, M: Clone + 'static>(
    icone: &str, titulo: impl Into<String>, descricao: impl Into<String>, destino: Option<M>,
) -> Element<'a, M> {
    let texto = column![
        negrito(titulo, 13.5),
        text(descricao.into()).size(11.5)
            .color(tema::COR_TEXTO_MUTED()).line_height(1.25),
    ].spacing(5).width(Length::Fill);
    let mut linha = row![icone_pastel(icone, 25.), texto].spacing(10).align_y(Alignment::Center);
    if destino.is_some() {
        linha = linha.push(icones::icone("chevron-right", 18.));
    }
    match destino {
        Some(msg) => button(linha).padding(13).style(estilo_cartao)
            .height(Length::Fixed(ALTURA_CARTAO)).width(Length::FillPortion(1))
            .on_press(msg).into(),
        None => container(linha).padding(13)
            .height(Length::Fixed(ALTURA_CARTAO)).width(Length::FillPortion(1))
            .align_y(Alignment::Center)
            .style(|t| {
                let b = estilo_cartao(t, Status::Active);
                container::Style {
                    background: b.background, border: b.border,
                    text_color: Some(b.text_color), ..Default::default()
                }
            }).into(),
    }
}

/// Grade de cartões; a última linha é completada com espaços para os
/// cartões não esticarem.
pub fn grade<'a, M: 'static>(cartoes: Vec<Element<'a, M>>, colunas: usize) -> Element<'a, M> {
    let colunas = colunas.max(1);
    let mut grade = column![].spacing(12);
    let mut linha = row![].spacing(12);
    let mut na_linha = 0;
    for c in cartoes {
        linha = linha.push(c);
        na_linha += 1;
        if na_linha == colunas {
            grade = grade.push(linha);
            linha = row![].spacing(12);
            na_linha = 0;
        }
    }
    if na_linha > 0 {
        for _ in na_linha..colunas {
            linha = linha.push(Space::with_width(Length::FillPortion(1)));
        }
        grade = grade.push(linha);
    }
    grade.into()
}

/// Linha fina entre blocos.
pub fn divisor<'a, M: 'static>() -> Element<'a, M> {
    container(Space::with_height(Length::Fixed(1.)))
        .width(Length::Fill).style(|_| container::Style {
            background: Some(Background::Color(tema::COR_BORDA_SUAVE())),
            ..Default::default()
        }).into()
}

/// A marca do produto em 86 px, numa caixa fixa de 96.
pub fn marca<'a, M: 'static>() -> Element<'a, M> {
    container(crate::marca::icone_atual(86.))
        // Nunca usar center_y(Fill) dentro de scrollable vertical:
        // center_y define a altura do Container e sobrescreve height(Fixed).
        // Na F1 isso gerava altura praticamente ilimitada e overflow
        // ao converter coordenadas de glifos no cosmic-text.
        .center_x(Length::Fixed(96.))
        .center_y(Length::Fixed(96.)).into()
}

/// Cabeçalho (hero): marca, título 24 e introdução; ilustração à direita.
pub fn cabecalho<'a, M: 'static>(
    titulo: impl Into<String>, introducao: impl Into<String>, ilustracao: Option<Element<'a, M>>,
) -> Element<'a, M> {
    let mut linha = row![
        marca(),
        column![
            negrito(titulo, 24.),
            text(introducao.into()).size(13).line_height(1.45)
                .color(tema::COR_TEXTO_MUTED()),
        ].spacing(12).width(Length::Fill),
    ].spacing(18).align_y(Alignment::Center);
    if let Some(i) = ilustracao { linha = linha.push(i); }
    linha.into()
}

/// As ilustrações do cabeçalho (164 × 132, cor de destaque do tema). Uma
/// por assunto, para cada página não repetir a mesma figura.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ilustracao {
    /// Planilha com células marcadas e o selo de função (a da F1).
    Planilha,
    /// Cartão de pessoa com comentário (autor, "Seus dados").
    Pessoa,
    /// Documento com o relógio da recuperação (salvamento).
    Salvamento,
    /// Janela clara e escura com as cores de destaque (aparência).
    Aparencia,
    /// Lista de tarefas concluída (tudo pronto).
    Pronto,
    /// Gráfico de colunas com linha e dica do mouse (novidades de gráficos).
    Grafico,
    /// Janela com as setas de atualizar.
    Atualizacao,
}

fn svg_ilustracao(tipo: Ilustracao) -> String {
    let escuro = tema::escuro();
    let cor = tema::acento_hex();
    let fundo = if escuro { "#292d32" } else { "#ffffff" };
    let borda = if escuro { "#485055" } else { "#dce4e9" };
    let grade = if escuro { "#52565e" } else { "#e0e5eb" };
    // Selo de função (Tabler math-function) desenhado em traço: texto em
    // SVG depende de fonte e saía vazio.
    let selo_fx = format!(r##"<rect x="111" y="92" width="46" height="29" rx="8" fill="{fundo}" stroke="{borda}"/>
        <g transform="translate(122 94.5) scale(1)" fill="none" stroke="{cor}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M3 19a2 2 0 0 0 2 2c2 0 2 -4 3 -9s1 -9 3 -9a2 2 0 0 1 2 2"/><path d="M5 12h6"/><path d="M15 12l6 6"/><path d="M15 18l6 -6"/></g>"##);
    let moldura = format!(r##"<rect x="19" y="13" width="127" height="101" rx="12" fill="{fundo}" stroke="{borda}" stroke-width="1.5"/>"##);
    let barra_titulo = format!(r##"<path d="M19 24Q19 13 31 13H134Q146 13 146 24V35H19Z" fill="{cor}" opacity=".85"/>
        <circle cx="31" cy="24" r="2.6" fill="white"/><circle cx="41" cy="24" r="2.6" fill="white"/><circle cx="51" cy="24" r="2.6" fill="white"/>"##);
    let corpo = match tipo {
        Ilustracao::Planilha => format!(r##"{moldura}
          <path d="M19 24Q19 13 31 13H134Q146 13 146 24V39H19Z" fill="{cor}" opacity=".85"/>
          <circle cx="33" cy="26" r="3" fill="white"/><circle cx="45" cy="26" r="3" fill="white"/><circle cx="57" cy="26" r="3" fill="white"/>
          <path d="M29 51H136M29 70H136M29 89H136M53 44V103M80 44V103M107 44V103" stroke="{grade}" stroke-width="1.1"/>
          <rect x="54" y="52" width="26" height="18" rx="2" fill="{cor}" opacity=".34"/>
          <rect x="80" y="70" width="27" height="19" rx="2" fill="{cor}" opacity=".2"/>{selo_fx}"##),
        Ilustracao::Pessoa => format!(r##"{moldura}
          <circle cx="54" cy="56" r="20" fill="{cor}" opacity=".18"/>
          <circle cx="54" cy="50" r="7" fill="none" stroke="{cor}" stroke-width="2.4"/>
          <path d="M42 68a12 12 0 0 1 24 0" fill="none" stroke="{cor}" stroke-width="2.4" stroke-linecap="round"/>
          <path d="M84 46H128M84 58H116" stroke="{grade}" stroke-width="5" stroke-linecap="round"/>
          <path d="M84 70H122" stroke="{cor}" stroke-width="5" stroke-linecap="round" opacity=".45"/>
          <path d="M98 88h44a8 8 0 0 1 8 8v12a8 8 0 0 1 -8 8h-30l-8 7v-7h-6a8 8 0 0 1 -8 -8v-12a8 8 0 0 1 8 -8z" fill="{fundo}" stroke="{borda}"/>
          <circle cx="110" cy="102" r="7" fill="{cor}" opacity=".85"/>
          <path d="M122 99H140M122 106H134" stroke="{grade}" stroke-width="3" stroke-linecap="round"/>"##),
        Ilustracao::Salvamento => format!(r##"<path d="M40 16h50l22 22v70a8 8 0 0 1 -8 8h-64a8 8 0 0 1 -8 -8v-84a8 8 0 0 1 8 -8z" fill="{fundo}" stroke="{borda}" stroke-width="1.5"/>
          <path d="M90 16v14a8 8 0 0 0 8 8h14" fill="none" stroke="{borda}" stroke-width="1.5"/>
          <path d="M46 52H96M46 64H96M46 76H80" stroke="{grade}" stroke-width="5" stroke-linecap="round"/>
          <rect x="46" y="86" width="30" height="8" rx="4" fill="{cor}" opacity=".35"/>
          <circle cx="118" cy="92" r="24" fill="{fundo}" stroke="{cor}" stroke-width="2"/>
          <circle cx="118" cy="92" r="24" fill="{cor}" opacity=".12"/>
          <g transform="translate(103.6 77.6) scale(1.2)" fill="none" stroke="{cor}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 8l0 4l2 2"/><path d="M3.05 11a9 9 0 1 1 .5 4m-.5 5v-5h5"/></g>"##),
        Ilustracao::Aparencia => format!(r##"<rect x="14" y="16" width="92" height="66" rx="10" fill="#ffffff" stroke="#dce4e9" stroke-width="1.5"/>
          <path d="M14 26Q14 16 24 16H96Q106 16 106 26V30H14Z" fill="{cor}" opacity=".85"/>
          <path d="M24 44H90M24 56H76" stroke="#e0e5eb" stroke-width="5" stroke-linecap="round"/>
          <rect x="58" y="44" width="92" height="66" rx="10" fill="#2b2f36" stroke="#485055" stroke-width="1.5"/>
          <path d="M58 54Q58 44 68 44H140Q150 44 150 54V58H58Z" fill="{cor}"/>
          <path d="M68 72H134M68 84H118" stroke="#52565e" stroke-width="5" stroke-linecap="round"/>
          <circle cx="30" cy="112" r="7" fill="#1a7f7a"/><circle cx="48" cy="112" r="7" fill="#1a5fd6"/>
          <circle cx="66" cy="112" r="7" fill="#1e8a3e"/><circle cx="84" cy="112" r="7" fill="#e8710a"/>
          <circle cx="102" cy="112" r="7" fill="#d93025"/><circle cx="120" cy="112" r="7" fill="#8e24aa"/>"##),
        Ilustracao::Pronto => format!(r##"{moldura}{barra_titulo}
          <circle cx="40" cy="54" r="7" fill="{cor}"/><path d="M36.5 54l2.5 2.5l4.5 -4.5" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M54 54H122" stroke="{grade}" stroke-width="5" stroke-linecap="round"/>
          <circle cx="40" cy="74" r="7" fill="{cor}"/><path d="M36.5 74l2.5 2.5l4.5 -4.5" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M54 74H110" stroke="{grade}" stroke-width="5" stroke-linecap="round"/>
          <circle cx="40" cy="94" r="7" fill="{cor}"/><path d="M36.5 94l2.5 2.5l4.5 -4.5" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M54 94H116" stroke="{grade}" stroke-width="5" stroke-linecap="round"/>"##),
        Ilustracao::Grafico => format!(r##"{moldura}{barra_titulo}
          <path d="M32 100H134" stroke="{grade}" stroke-width="1.5"/>
          <rect x="38" y="72" width="14" height="28" rx="3" fill="{cor}" opacity=".35"/>
          <rect x="60" y="58" width="14" height="42" rx="3" fill="{cor}" opacity=".55"/>
          <rect x="82" y="66" width="14" height="34" rx="3" fill="{cor}" opacity=".35"/>
          <rect x="104" y="48" width="14" height="52" rx="3" fill="{cor}" opacity=".85"/>
          <path d="M45 66L67 52L89 60L111 42" fill="none" stroke="{cor}" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"/>
          <circle cx="111" cy="42" r="3.5" fill="{fundo}" stroke="{cor}" stroke-width="2"/>
          <rect x="112" y="84" width="46" height="30" rx="8" fill="{fundo}" stroke="{borda}"/>
          <circle cx="122" cy="94" r="3.5" fill="{cor}"/>
          <path d="M130 94H150M120 104H146" stroke="{grade}" stroke-width="3.5" stroke-linecap="round"/>"##),
        Ilustracao::Atualizacao => format!(r##"{moldura}{barra_titulo}
          <circle cx="82" cy="74" r="28" fill="{cor}" opacity=".12"/>
          <g transform="translate(58 50) scale(2)" fill="none" stroke="{cor}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <path d="M20 11a8.1 8.1 0 0 0 -15.5 -2m-.5 -4v4h4"/><path d="M4 13a8.1 8.1 0 0 0 15.5 2m.5 4v-4h-4"/></g>
          <rect x="114" y="92" width="38" height="22" rx="7" fill="{cor}"/>
          <path d="M124 103h18" stroke="white" stroke-width="2.4" stroke-linecap="round"/>"##),
    };
    format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="164" height="132" viewBox="0 0 164 132">{corpo}</svg>"##)
}

/// A ilustração `tipo` no quadro pastel arredondado do cabeçalho.
pub fn ilustracao<'a, M: 'static>(tipo: Ilustracao) -> Element<'a, M> {
    container(svg::Svg::new(svg::Handle::from_memory(svg_ilustracao(tipo).into_bytes()))
        .width(Length::Fixed(164.)).height(Length::Fixed(132.)))
        .center_x(Length::Fixed(184.))
        .style(|_| container::Style {
            background: Some(Background::Color(tema::COR_AZUL_FUNDO())),
            border: Border { radius: 20.0.into(), ..Default::default() },
            ..Default::default()
        }).into()
}

/// A ilustração da planilha da F1 (moldura, grade, células marcadas e o
/// selo de função), com a cor de destaque escolhida.
pub fn ilustracao_planilha<'a, M: 'static>() -> Element<'a, M> {
    ilustracao(Ilustracao::Planilha)
}

/// Rodapé de passos: Anterior (secundário) à esquerda e Próximo (primário)
/// à direita; sem mensagem o botão fica desabilitado.
pub fn paginacao<'a, M: Clone + 'static>(
    anterior: (&str, Option<M>), proximo: (&str, Option<M>),
) -> Element<'a, M> {
    row![
        button(row![icones::icone("chevron-left", 14.), text(anterior.0.to_owned()).size(12.)]
            .spacing(4).align_y(Alignment::Center))
            .padding([8, 13]).style(estilo_cartao)
            .on_press_maybe(anterior.1),
        Space::with_width(Length::Fill),
        button(row![text(proximo.0.to_owned()).size(12.), icones::icone("chevron-right", 14.)]
            .spacing(4).align_y(Alignment::Center))
            .padding([8, 13]).style(tema::estilo_botao_primario)
            .on_press_maybe(proximo.1),
    ].spacing(10).align_y(Alignment::Center).into()
}

/// A janela inteira: lateral + área de leitura (`COR_PAINEL`) que rola,
/// com o rodapé fixo embaixo, fora da rolagem.
pub fn janela<'a, M: 'static>(
    lateral: Element<'a, M>, conteudo: Element<'a, M>, rodape: Option<Element<'a, M>>,
) -> Element<'a, M> {
    // O viewport rola e preenche a janela; seu CONTEÚDO tem altura Shrink
    // (Iced 0.13 entra em pânico com Fill aqui).
    // Margem direita maior: a barra de rolagem não encosta nos cartões.
    let rolagem = scrollable(container(conteudo)
        .padding(iced::Padding { top: 18., bottom: 18., left: 8., right: 20. })
        .width(Length::Fill).height(Length::Shrink))
        .height(Length::Fill).width(Length::Fill);
    let mut leitor = column![rolagem].spacing(10)
        .padding([16, 20]).width(Length::Fill).height(Length::Fill);
    if let Some(r) = rodape { leitor = leitor.push(r); }
    row![
        lateral,
        container(leitor).width(Length::Fill).height(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(tema::COR_PAINEL())),
                ..Default::default()
            }),
    ].height(Length::Fill).width(Length::Fill).into()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[derive(Clone, Debug)]
    enum M { Ir }

    #[test]
    fn pecas_montam_janela_completa_sem_fill_no_conteudo_rolavel() {
        let nav = iced::widget::column![
            rotulo_grupo::<M>("Passos"),
            item_nav("Boas-vindas", "rocket", true, Some(M::Ir)),
            item_nav("Desligado", "user", false, None),
        ];
        let conteudo = iced::widget::column![
            cabecalho("Título", "Introdução", Some(ilustracao_planilha())),
            grade(vec![
                cartao("sparkles", "A", "resumo", Some(M::Ir)),
                cartao("history", "B", "só leitura", None),
            ], 3),
            divisor(),
        ];
        let tela: Element<'_, M> = janela(
            lateral(nav, Some(text("rodapé").into())),
            conteudo.into(),
            Some(paginacao(("Anterior", None), ("Próximo", Some(M::Ir)))),
        );
        drop(tela);
        for tipo in [Ilustracao::Planilha, Ilustracao::Pessoa, Ilustracao::Salvamento, Ilustracao::Aparencia,
            Ilustracao::Pronto, Ilustracao::Grafico, Ilustracao::Atualizacao] {
            let svg = svg_ilustracao(tipo);
            assert!(svg.starts_with("<svg") && !svg.contains("<text"), "{tipo:?}");
        }
        for nome in ["user", "world", "sparkles", "history", "rocket", "chevron-right", "chevron-left"] {
            assert!(icones::existe(nome), "{nome}");
        }
        assert_eq!(resumir(&"a".repeat(90)).chars().count(), 85);
    }
}
