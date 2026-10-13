//! Controles visuais de efeitos de imagem compartilhados nos aplicativos Iced.
//! Nenhuma dependência de motores: devolve apenas a chave escolhida ao consumidor.
use iced::{Element, widget::{button, column, row, text}};
use crate::{formatacao, tema};

/// As chaves estáveis são interpretadas por `featheroffice-image::EfeitoRapido`
/// nos aplicativos; esta biblioteca define somente controles de interface.
pub const OPCOES: [(&str, &str); 4] = [
    ("original", "Original"),
    ("cinza", "Cinza"),
    ("desfoque", "Desfocar"),
    ("nitidez", "Nitidez"),
];

pub fn painel<'a, M: Clone + 'a>(
    selecionado: &str,
    escolher: impl Fn(&'static str) -> M + 'a,
    fechar: M,
) -> Element<'a, M> {
    let mut opcoes = column![].spacing(5);
    for (chave, rotulo) in OPCOES {
        opcoes = opcoes.push(
            button(text(rotulo).size(12))
                .padding([5, 9])
                .style(tema::estilo_botao_pequeno(selecionado == chave))
                .on_press(escolher(chave)),
        );
    }
    let corpo = column![
        opcoes,
        row![
            button(text("Fechar").size(12))
                .on_press(fechar)
                .style(tema::estilo_botao_pequeno(false))
        ],
    ].spacing(8);
    formatacao::popover("Efeitos de imagem", corpo.into())
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn controles_e_chaves_sao_independentes_do_motor() {
        let chaves: Vec<&str> = OPCOES.iter().map(|x| x.0).collect();
        assert_eq!(chaves, ["original", "cinza", "desfoque", "nitidez"]);
        let _: Element<'_, String> = painel("cinza", str::to_owned, "fechar".into());
    }
}
