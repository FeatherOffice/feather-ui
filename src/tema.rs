//! Estilos visuais e cores que reproduzem exatamente o design da suíte FeatherOffice.

use iced::border::Radius;
use iced::widget::{button, container};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use iced::{Background, Border, Color, Shadow};

// Paleta da suíte, clara ou escura, com a cor de destaque escolhida nas Configurações.
// Cada cor é uma função (nome em maiúsculas, para manter o jeito de usar de antes) porque
// muda com o tema em tempo de execução.

static ESCURO: AtomicBool = AtomicBool::new(false);
static ACENTO: AtomicUsize = AtomicUsize::new(0);

/// As cores de destaque: (nome, claro, escuro).
pub const ACENTOS: [(&str, &str, &str); 6] = [
    ("Petróleo", "#2b7a78", "#4fb3ad"),
    ("Azul", "#0b57d0", "#8ab4f8"),
    ("Verde", "#188038", "#81c995"),
    ("Laranja", "#e8710a", "#fcad70"),
    ("Vermelho", "#d93025", "#f28b82"),
    ("Roxo", "#8e24aa", "#d7aefb"),
];

/// Resolve a preferência automática usando o tema nativo detectado pelo Iced.
/// Na versão 0.13, a detecção é feita ao iniciar o processo.
pub fn escuro_resolvido(automatico: bool, escuro_preferido: bool) -> bool {
    if automatico {
        matches!(iced::Theme::default(), iced::Theme::Dark)
    } else {
        escuro_preferido
    }
}

pub fn definir_tema(escuro: bool, acento: usize) {
    ESCURO.store(escuro, Ordering::Relaxed);
    ACENTO.store(acento.min(ACENTOS.len() - 1), Ordering::Relaxed);
}

pub fn escuro() -> bool {
    ESCURO.load(Ordering::Relaxed)
}

pub fn tema_iced() -> iced::Theme {
    if escuro() { iced::Theme::Dark } else { iced::Theme::Light }
}

fn hex(s: &str) -> Color {
    let v = u32::from_str_radix(s.trim_start_matches('#'), 16).unwrap_or(0);
    Color::from_rgb8((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

fn par(claro: &str, escuro_: &str) -> Color {
    hex(if escuro() { escuro_ } else { claro })
}

fn mistura(a: Color, b: Color, t: f32) -> Color {
    Color { r: a.r + (b.r - a.r) * t, g: a.g + (b.g - a.g) * t, b: a.b + (b.b - a.b) * t, a: 1.0 }
}

/// A cor de destaque em texto `#rrggbb` (para os ícones SVG).
pub fn acento_hex() -> &'static str {
    let (_, c, e) = ACENTOS[ACENTO.load(Ordering::Relaxed)];
    if escuro() { e } else { c }
}

/// Traduz as cores fixas dos ícones (`#5f6368`…) para as do tema atual.
pub fn cor_do_icone(cor: &str) -> String {
    let c = match cor {
        "#1a73e8" | "#0b57d0" => acento_hex().to_string(),
        "#5f6368" => if escuro() { "#9aa0a6".into() } else { cor.into() },
        "#202124" => if escuro() { "#e8eaed".into() } else { cor.into() },
        "#bdc1c6" => if escuro() { "#5f6368".into() } else { cor.into() },
        outra => outra.to_string(),
    };
    c
}

#[allow(non_snake_case)]
pub fn COR_FUNDO_JANELA() -> Color { par("#f1f3f4", "#181a1d") }
#[allow(non_snake_case)]
pub fn COR_PAINEL() -> Color { par("#ffffff", "#2a2d32") }
#[allow(non_snake_case)]
pub fn COR_PAINEL_FAIXA() -> Color { par("#f8f9fa", "#2d3035") }
#[allow(non_snake_case)]
pub fn COR_BORDA() -> Color { par("#dadce0", "#3d4147") }
#[allow(non_snake_case)]
pub fn COR_BORDA_SUAVE() -> Color { par("#e8eaed", "#33363b") }
#[allow(non_snake_case)]
pub fn COR_TEXTO() -> Color { par("#202124", "#e8eaed") }
#[allow(non_snake_case)]
pub fn COR_TEXTO_MUTED() -> Color { par("#5f6368", "#9aa0a6") }
#[allow(non_snake_case)]
pub fn COR_AZUL_ACENTO() -> Color { hex(acento_hex()) }

/// Variação AA do acento apenas quando utilizado como texto na Ribbon.
/// O acento original continua disponível para fundos, SVG e preferências.
#[allow(non_snake_case)]
pub fn COR_ACENTO_TEXTO(fundo: Color) -> Color {
    crate::acessibilidade::texto_acento(COR_AZUL_ACENTO(), fundo)
}

/// Borda de foco em controles Iced, >= 3:1 sobre a superfície indicada.
#[allow(non_snake_case)]
pub fn COR_FOCO(fundo: Color) -> Color {
    crate::acessibilidade::borda_foco(COR_AZUL_ACENTO(), fundo)
}
#[allow(non_snake_case)]
pub fn COR_AZUL_FUNDO() -> Color { mistura(COR_PAINEL(), COR_AZUL_ACENTO(), if escuro() { 0.22 } else { 0.10 }) }
#[allow(non_snake_case)]
pub fn COR_HOVER() -> Color { par("#dfe3e8", "#3a3e44") }
#[allow(non_snake_case)]
/// A barra de abas da faixa.
pub fn COR_CROMO() -> Color { par("#f4f6f8", "#202327") }
#[allow(non_snake_case)]
/// O corpo da faixa (onde ficam os grupos de botões).
pub fn COR_CROMO_2() -> Color { par("#eaedf1", "#26292e") }
/// Vermelho de erro (texto, borda de campo inválido, ícone de alerta). AA sobre `COR_PAINEL()`.
#[allow(non_snake_case)]
pub fn COR_ERRO() -> Color { par("#d93025", "#f28b82") }
/// Véu atrás de diálogo modal: preto semitransparente, mais denso no tema escuro.
#[allow(non_snake_case)]
pub fn COR_VEU() -> Color { Color { a: if escuro() { 0.60 } else { 0.45 }, ..Color::BLACK } }
/// Cor da sombra de diálogo (usar com `offset (0, 6)` e `blur_radius 22`).
#[allow(non_snake_case)]
pub fn COR_SOMBRA() -> Color { Color { a: 0.28, ..Color::BLACK } }
#[allow(non_snake_case)]
pub fn COR_HOVER_AZUL() -> Color { mistura(COR_PAINEL(), COR_AZUL_ACENTO(), if escuro() { 0.32 } else { 0.20 }) }
#[allow(non_snake_case)]
pub fn COR_MESA() -> Color { par("#e8eaed", "#2b2e33") }

/// Botão Pílula "Página inicial" no topo (arredondado com borda fina)
pub fn estilo_pilha_inicio(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(COR_HOVER())),
        button::Status::Pressed => Some(Background::Color(COR_AZUL_FUNDO())),
        _ => Some(Background::Color(COR_PAINEL())),
    };
    button::Style {
        background: bg,
        text_color: COR_TEXTO(),
        border: Border {
            color: COR_BORDA(),
            width: 1.0,
            radius: Radius::from(16.0),
        },
        shadow: Shadow::default(),
    }
}

/// Botão Primário Azul ("Nova apresentação")
pub fn estilo_botao_primario(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color { r: 0.090, g: 0.396, b: 0.800, a: 1.0 },
        button::Status::Pressed => Color { r: 0.082, g: 0.345, b: 0.690, a: 1.0 },
        button::Status::Disabled => Color { r: 0.741, g: 0.757, b: 0.776, a: 1.0 },
        _ => COR_AZUL_ACENTO(),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: crate::acessibilidade::texto_sobre(bg),
        border: Border {
            radius: Radius::from(6.0),
            ..Border::default()
        },
        shadow: Shadow::default(),
    }
}

/// Botão Secundário Branco com Borda ("Abrir…")
pub fn estilo_botao_secundario(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(COR_HOVER())),
        button::Status::Pressed => Some(Background::Color(COR_AZUL_FUNDO())),
        _ => Some(Background::Color(COR_PAINEL())),
    };
    button::Style {
        background: bg,
        text_color: COR_TEXTO(),
        border: Border {
            color: COR_BORDA(),
            width: 1.0,
            radius: Radius::from(6.0),
        },
        shadow: Shadow::default(),
    }
}

/// Botão Redondo para ações no topo (Configurações, Ajuda)
pub fn estilo_botao_circular(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(COR_HOVER())),
        button::Status::Pressed => Some(Background::Color(COR_AZUL_FUNDO())),
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: COR_TEXTO_MUTED(),
        border: Border {
            radius: Radius::from(17.0),
            ..Border::default()
        },
        shadow: Shadow::default(),
    }
}

/// Aba da Faixa de Opções (Ribbon Tab)
pub fn estilo_aba_faixa(ativa: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (cor_texto, cor_borda, bg) = if ativa {
            (COR_ACENTO_TEXTO(COR_CROMO()), COR_AZUL_ACENTO(), None)
        } else {
            let cor = match status {
                button::Status::Hovered => COR_TEXTO(),
                _ => COR_TEXTO_MUTED(),
            };
            let bg = match status {
                button::Status::Hovered => Some(Background::Color(COR_HOVER())),
                _ => None,
            };
            (cor, Color::TRANSPARENT, bg)
        };

        button::Style {
            background: bg,
            text_color: cor_texto,
            border: Border {
                color: cor_borda,
                width: if ativa { 2.0 } else { 0.0 },
                radius: Radius::from(3.0),
            },
            shadow: Shadow::default(),
        }
    }
}

/// Dimensões do menu TS (.menu-panel): painel compacto, sem largura de sidebar.
pub const LARGURA_MENU: f32 = 272.0;
pub const LARGURA_SUBMENU: f32 = 236.0;

/// Painel suspenso nativo equivalente a .menu-panel/.menu-sub do TypeScript.
pub fn estilo_popup_menu(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COR_PAINEL())),
        border: Border { color: COR_BORDA_SUAVE(), width: 1.0, radius: Radius::from(8.0) },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, if escuro() { 0.35 } else { 0.18 }),
            offset: iced::Vector::new(0.0, 6.0),
            blur_radius: 18.0,
        },
        ..Default::default()
    }
}

/// Título da barra de menus: altura 22px, radius 4px, destaque discreto.
pub fn estilo_titulo_menu(ativo: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, estado| button::Style {
        background: if ativo || matches!(estado, button::Status::Hovered | button::Status::Pressed) {
            Some(Background::Color(if ativo { COR_AZUL_FUNDO() } else { COR_HOVER() }))
        } else { None },
        text_color: if ativo { COR_TEXTO() } else { COR_TEXTO_MUTED() },
        border: Border { radius: Radius::from(4.0), ..Default::default() },
        ..Default::default()
    }
}

/// Item de menu: radius 5px e realce apenas em hover/seleção.
pub fn estilo_item_menu(
    selecionado: bool,
    disponivel: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, estado| button::Style {
        background: if disponivel && (selecionado
            || matches!(estado, button::Status::Hovered | button::Status::Pressed)) {
            Some(Background::Color(COR_HOVER()))
        } else { None },
        text_color: if disponivel { COR_TEXTO() } else { COR_TEXTO_MUTED() },
        border: Border { radius: Radius::from(5.0), ..Default::default() },
        ..Default::default()
    }
}

/// Botão Grande na Faixa de Opções (com ícone em cima e rótulo embaixo)
pub fn estilo_botao_grande(ativo: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let bg = if ativo {
            Some(Background::Color(COR_AZUL_FUNDO()))
        } else {
            match status {
                button::Status::Hovered => Some(Background::Color(COR_HOVER())),
                button::Status::Pressed => Some(Background::Color(COR_AZUL_FUNDO())),
                _ => None,
            }
        };

        button::Style {
            background: bg,
            text_color: if status == button::Status::Disabled {
                Color { a: 0.4, ..COR_TEXTO() }
            } else if ativo {
                COR_ACENTO_TEXTO(COR_AZUL_FUNDO())
            } else {
                COR_TEXTO()
            },
            border: Border {
                radius: Radius::from(6.0),
                ..Border::default()
            },
            shadow: Shadow::default(),
        }
    }
}

/// Botão Pequeno Compacto na Faixa (formatação de texto, alinhamento, etc.)
pub fn estilo_botao_pequeno(ativo: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let bg = if ativo {
            Some(Background::Color(COR_AZUL_FUNDO()))
        } else {
            match status {
                button::Status::Hovered => Some(Background::Color(COR_HOVER())),
                button::Status::Pressed => Some(Background::Color(COR_HOVER_AZUL())),
                _ => None,
            }
        };

        button::Style {
            background: bg,
            text_color: if status == button::Status::Disabled {
                Color { a: 0.4, ..COR_TEXTO() }
            } else if ativo {
                COR_ACENTO_TEXTO(COR_AZUL_FUNDO())
            } else {
                COR_TEXTO()
            },
            border: Border {
                radius: Radius::from(4.0),
                ..Border::default()
            },
            shadow: Shadow::default(),
        }
    }
}

/// Barra Superior (Topo)
pub fn estilo_container_topo(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COR_PAINEL())),
        ..container::Style::default()
    }
}

/// Painel (cartões e blocos claros, com borda fina)
pub fn estilo_painel_faixa(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COR_PAINEL_FAIXA())),
        border: Border { color: COR_BORDA(), width: 1.0, ..Border::default() },
        ..container::Style::default()
    }
}

/// O corpo da faixa: fundo cinza-azulado, sem borda (a linha vem de `linha_vertical`/`linha_horizontal`).
pub fn estilo_corpo_faixa(_theme: &iced::Theme) -> container::Style {
    container::Style { background: Some(Background::Color(COR_CROMO_2())), ..container::Style::default() }
}

/// A barra de abas da faixa.
pub fn estilo_barra_abas(_theme: &iced::Theme) -> container::Style {
    container::Style { background: Some(Background::Color(COR_CROMO())), ..container::Style::default() }
}

/// Uma linha fina (divisória) na cor da borda; o tamanho vem de quem a usa.
pub fn estilo_linha(_theme: &iced::Theme) -> container::Style {
    container::Style { background: Some(Background::Color(COR_BORDA())), ..container::Style::default() }
}

/// O sublinhado da aba ativa.
pub fn estilo_sublinhado_aba(ativa: bool) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(if ativa { COR_AZUL_ACENTO() } else { Color::TRANSPARENT })),
        ..container::Style::default()
    }
}

/// Barra Lateral de Miniaturas dos Slides
pub fn estilo_painel_miniaturas(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COR_FUNDO_JANELA())),
        border: Border {
            color: COR_BORDA(),
            width: 1.0,
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// Painel de Anotações do Apresentador
pub fn estilo_painel_notas(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COR_PAINEL())),
        border: Border {
            color: COR_BORDA(),
            width: 1.0,
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// Barra de Status no Rodapé
pub fn estilo_rodape(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COR_PAINEL())),
        border: Border {
            color: COR_BORDA(),
            width: 1.0,
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// Switch compartilhado: segue o acento configurado (e não o azul padrão
/// de iced::Theme), inclusive após troca de tema sem reiniciar a janela.
pub fn estilo_toggle(
    tema: &iced::Theme,
    status: iced::widget::toggler::Status,
) -> iced::widget::toggler::Style {
    use iced::widget::toggler::Status;
    let (ligado, sobre, habilitado) = match status {
        Status::Active { is_toggled } => (is_toggled, false, true),
        Status::Hovered { is_toggled } => (is_toggled, true, true),
        Status::Disabled => (false, false, false),
    };
    // Iced 0.13: background/foreground são Color (não Background).
    // Mantém a geometria do controle nativo e substitui apenas a paleta.
    let mut estilo = iced::widget::toggler::default(tema, status);
    let acento = COR_AZUL_ACENTO();
    let trilho = if ligado {
        acento
    } else if sobre {
        COR_HOVER()
    } else {
        COR_CROMO_2()
    };
    estilo.background = Color {
        a: if habilitado { 1.0 } else { 0.45 },
        ..trilho
    };
    estilo.foreground = Color {
        a: if habilitado { 1.0 } else { 0.5 },
        ..if ligado {
            crate::acessibilidade::texto_sobre(acento)
        } else {
            COR_TEXTO_MUTED()
        }
    };
    estilo.background_border_width = if ligado { 0.0 } else { 1.0 };
    estilo.background_border_color = if ligado { acento } else { COR_BORDA() };
    estilo.foreground_border_width = 0.0;
    estilo.foreground_border_color = Color::TRANSPARENT;
    estilo
}

/// Dropdowns compartilhados usam um campo legível e realce de foco,
/// inclusive em telas escuras. Popups usam a mesma paleta da suíte.
pub fn estilo_menu_da_faixa(_theme: &iced::Theme, status: iced::widget::pick_list::Status) -> iced::widget::pick_list::Style {
    use iced::widget::pick_list::Status;
    let sobre = matches!(status, Status::Hovered | Status::Opened);
    iced::widget::pick_list::Style {
        text_color: COR_TEXTO(),
        placeholder_color: COR_TEXTO_MUTED(),
        handle_color: COR_AZUL_ACENTO(),
        background: Background::Color(if sobre { COR_AZUL_FUNDO() } else { COR_PAINEL() }),
        border: Border {
            color: if sobre { COR_FOCO(COR_AZUL_FUNDO()) } else { COR_BORDA() },
            width: 1.0,
            radius: Radius::from(5.0),
        },
    }
}

/// Paleta e bordas do painel de opções, usada por todos os pick_list e
/// combo_box. O destaque continua visível no tema claro e no escuro.
pub fn estilo_menu_popup(_theme: &iced::Theme) -> iced::widget::overlay::menu::Style {
    iced::widget::overlay::menu::Style {
        background: Background::Color(COR_PAINEL()),
        border: Border {
            color: COR_BORDA(),
            width: 1.0,
            radius: Radius::from(6.0),
        },
        text_color: COR_TEXTO(),
        selected_text_color: COR_TEXTO(),
        selected_background: Background::Color(COR_AZUL_FUNDO()),
    }
}

/// Entrada de texto do dropdown pesquisável (Iced ComboBox).
pub fn estilo_entrada_suspensa(
    _theme: &iced::Theme,
    status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    use iced::widget::text_input::Status;
    let focado = matches!(status, Status::Focused { .. });
    let sobre = focado || matches!(status, Status::Hovered);
    iced::widget::text_input::Style {
        background: Background::Color(if sobre { COR_AZUL_FUNDO() } else { COR_PAINEL() }),
        border: Border {
            color: if focado { COR_FOCO(COR_AZUL_FUNDO()) } else { COR_BORDA() },
            width: if focado { 1.5 } else { 1.0 },
            radius: Radius::from(5.0),
        },
        placeholder: COR_TEXTO_MUTED(),
        value: COR_TEXTO(),
        selection: COR_HOVER_AZUL(),
        icon: COR_TEXTO_MUTED(),
    }
}

/// O balão de dica (tooltip): painel pequeno com borda fina.
pub fn estilo_dica(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COR_PAINEL())),
        text_color: Some(COR_TEXTO()),
        border: Border { color: COR_BORDA(), width: 1.0, radius: Radius::from(4.0) },
        ..container::Style::default()
    }
}

/// Botão da barra de controles da apresentação: quase invisível até passar o mouse.
pub fn estilo_controle_apresentacao(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let alfa = if matches!(status, button::Status::Hovered | button::Status::Pressed) { 0.55 } else { 0.0 };
    button::Style {
        background: Some(Background::Color(Color::from_rgba8(255, 255, 255, alfa))),
        text_color: Color::WHITE,
        border: Border { radius: Radius::from(14.0), ..Border::default() },
        shadow: Shadow::default(),
    }
}

/// Rolagem vertical com a barra num canal próprio ao lado do conteúdo (sem ficar por cima dele).
///
/// Iced 0.13 exige que o FILHO tenha altura Shrink, embora o próprio
/// scrollable possa ocupar Fill. Alguns painéis enviam um Element dinâmico
/// com height Fill: adaptar só o viewport não evita o panic.
pub fn rolagem<'a, M: 'a>(conteudo: impl Into<iced::Element<'a, M>>) -> iced::widget::Scrollable<'a, M> {
    let conteudo: iced::Element<'a, M> = conteudo.into();
    let corpo = iced::widget::container(conteudo)
        .width(iced::Length::Fill)
        .height(iced::Length::Shrink);
    iced::widget::scrollable(corpo).direction(iced::widget::scrollable::Direction::Vertical(
        iced::widget::scrollable::Scrollbar::new().width(6).scroller_width(6).spacing(4),
    ))
}

#[cfg(test)]
mod testes_rolagem {
    #[test]
    fn aceita_elemento_fill_sem_panic_no_eixo_vertical() {
        let conteudo = iced::widget::column![
            iced::widget::text(crate::idioma::t("Planilhas recentes"))
        ].height(iced::Length::Fill);
        // Antes: rolagem(conteudo) causava panic em iced_widget::scrollable.
        let _: iced::widget::Scrollable<'_, ()> = super::rolagem(conteudo);
    }
}

#[cfg(test)]
mod testes_toggle_suite {
    use super::*;

    #[test]
    fn toggle_ativo_usa_o_acento_da_suite_e_desligado_e_neutro() {
        use iced::widget::toggler::Status;
        let tema = iced::Theme::Light;
        let ligado = estilo_toggle(&tema, Status::Active { is_toggled: true });
        let desligado = estilo_toggle(&tema, Status::Active { is_toggled: false });
        assert_eq!(ligado.background, COR_AZUL_ACENTO());
        assert_ne!(desligado.background, COR_AZUL_ACENTO());
        assert_eq!(desligado.background_border_color, COR_BORDA());
        assert_eq!(ligado.foreground, crate::acessibilidade::texto_sobre(COR_AZUL_ACENTO()));
    }
}

/// Botão "fechar" de diálogo: ícone `x` (15 px), estilo pequeno, dica "Fechar".
/// Fica no canto direito do cabeçalho; `msg` é a mensagem de fechamento.
/// Já existia o mesmo padrão embutido em `configuracoes::tela`; este é o
/// helper avulso para os diálogos dos aplicativos.
pub fn botao_fechar<'a, M: Clone + 'a + 'static>(msg: M) -> iced::Element<'a, M> {
    use iced::widget::tooltip;
    let botao = button(crate::icones::icone("x", 15.0))
        .padding([5, 7])
        .style(estilo_botao_pequeno(false))
        .on_press(msg);
    tooltip(botao, iced::widget::text(crate::idioma::t("Fechar")).size(12), tooltip::Position::Bottom)
        .style(estilo_dica)
        .into()
}
