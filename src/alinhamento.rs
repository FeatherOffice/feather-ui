//! UI-011 — grupo de alinhamento horizontal compartilhado (Iced).
//!
//! Recebe somente o estado visual e uma função que produz a mensagem do
//! aplicativo. Não depende de cores, formatos nem modelos de Docs/Sheet/Slide.

use iced::widget::row;
use iced::{Alignment, Element};

use crate::{componentes::botao_icone, icones::icone};

/// Alinhamento apenas da interface. Cada aplicativo decide como aplicar
/// a escolha ao seu modelo e ao seu histórico de desfazer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Horizontal {
    Esquerda,
    Centro,
    Direita,
    Justificado,
}

impl Horizontal {
    pub const TODOS: [Self; 4] = [
        Self::Esquerda,
        Self::Centro,
        Self::Direita,
        Self::Justificado,
    ];

    pub const fn icone(self) -> &'static str {
        match self {
            Self::Esquerda => "align-left",
            Self::Centro => "align-center",
            Self::Direita => "align-right",
            Self::Justificado => "align-justified",
        }
    }

    pub const fn dica(self) -> &'static str {
        match self {
            Self::Esquerda => "Alinhar à esquerda",
            Self::Centro => "Centralizar",
            Self::Direita => "Alinhar à direita",
            Self::Justificado => "Justificar",
        }
    }

    /// Admite identificadores legados dos três apps sem importar seus modelos.
    /// Retorna None em vez de selecionar silenciosamente a primeira opção.
    pub fn de_identificador(id: &str) -> Option<Self> {
        match id.trim().to_ascii_lowercase().as_str() {
            "esquerda" | "left" => Some(Self::Esquerda),
            "centro" | "center" | "centre" => Some(Self::Centro),
            "direita" | "right" => Some(Self::Direita),
            "justificado" | "justify" | "justified" => Some(Self::Justificado),
            _ => None,
        }
    }
}

/// Renderiza 3 ou 4 botões com SVG, tooltip e destaque da seleção.
///
/// - `habilitado = false` conserva ícones visíveis, mas não emite mensagens;
/// - `justificar = false` oculta o quarto controle (Docs e Sheet);
/// - o consumidor mantém suas mensagens, seleção e histórico nativos.
pub fn seletor_horizontal<'a, M, F>(
    selecionado: Option<Horizontal>,
    habilitado: bool,
    justificar: bool,
    ao_escolher: F,
) -> Element<'a, M>
where
    M: Clone + 'a + 'static,
    F: Fn(Horizontal) -> M + 'a,
{
    let mut linha = row![].spacing(3).align_y(Alignment::Center);
    for opcao in Horizontal::TODOS {
        if !justificar && opcao == Horizontal::Justificado {
            continue;
        }
        let mensagem = habilitado.then(|| ao_escolher(opcao));
        linha = linha.push(botao_icone(
            icone(opcao.icone(), 18.0),
            opcao.dica(),
            mensagem,
            selecionado == Some(opcao),
        ));
    }
    linha.into()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn opcoes_e_identificadores_preservam_compatibilidade() {
        assert_eq!(Horizontal::TODOS.len(), 4);
        for (id, expected) in [
            ("Esquerda", Horizontal::Esquerda),
            ("left", Horizontal::Esquerda),
            (" CENTRO ", Horizontal::Centro),
            ("center", Horizontal::Centro),
            ("right", Horizontal::Direita),
            ("direita", Horizontal::Direita),
            ("justificado", Horizontal::Justificado),
            ("justify", Horizontal::Justificado),
        ] {
            assert_eq!(Horizontal::de_identificador(id), Some(expected));
        }
        assert_eq!(Horizontal::de_identificador(""), None);
        assert_eq!(Horizontal::de_identificador("invalido"), None);
        assert_ne!(Horizontal::Centro.icone(), Horizontal::Esquerda.icone());
    }

    #[test]
    fn constroi_seletor_tres_quatro_opcoes_e_desabilitado() {
        let _tres = seletor_horizontal(Some(Horizontal::Centro), true, false, |v| v);
        let _quatro = seletor_horizontal(Some(Horizontal::Justificado), true, true, |v| v);
        let _desabilitado = seletor_horizontal(None, false, true, |v| v);
    }
}
