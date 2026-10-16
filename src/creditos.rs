//! Quem fez a suíte — o bloco "Colaboradores" do "Sobre" de cada aplicativo.
//! Mora aqui para os apps mostrarem a mesma lista (regra 3 do CLAUDE.md).
use iced::widget::{column, row, text};
use iced::{Element, Length};

use crate::tema::{COR_TEXTO, COR_TEXTO_MUTED};

/// (nome, papel)
pub const COLABORADORES: &[(&str, &str)] = &[
    ("maragon", "Criação, direção e testes"),
    ("Claude (Anthropic)", "Desenvolvimento"),
    ("GPT / Codex (OpenAI)", "Desenvolvimento"),
    ("Gemini (Google)", "Desenvolvimento"),
];

pub fn colaboradores<'a, M: 'a>() -> Element<'a, M> {
    let mut lista = column![text(crate::idioma::t("Colaboradores")).size(15).color(COR_TEXTO())].spacing(6);
    for (nome, papel) in COLABORADORES {
        lista = lista.push(row![
            text(*nome).size(13.5).color(COR_TEXTO()).width(Length::Fixed(190.0)),
            text(*papel).size(13).color(COR_TEXTO_MUTED()),
        ].spacing(10));
    }
    lista.into()
}
