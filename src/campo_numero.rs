//! Campo de número com setas à direita e roda do mouse, como o "spin box"
//! do Excel/Word: seta para cima soma um passo, para baixo tira; rolar a
//! roda com o mouse em cima do campo faz o mesmo. O texto continua editável
//! e é sempre o que a aplicação recebe (vírgula decimal, pt-BR).
use iced::widget::{button, column, container, mouse_area, row, text_input};
use iced::{mouse, Alignment, Element, Length};

use crate::tema::{estilo_botao_pequeno, COR_BORDA};

/// Lê "1,5" ou "1.5"; vazio ou inválido = `None`.
pub fn ler(texto: &str) -> Option<f64> {
    let t = texto.trim().replace('.', "").replace(',', ".");
    // "1.5" digitado com ponto decimal (sem vírgula) também vale.
    let t = if !texto.contains(',') && texto.matches('.').count() == 1 { texto.trim().to_owned() } else { t };
    t.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Escreve com vírgula, sem zeros sobrando (até 4 casas).
pub fn escrever(v: f64) -> String {
    let s = format!("{v:.4}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    let s = if s == "-0" { "0" } else { s };
    s.replace('.', ",")
}

/// `atual` mais `passos` vezes `passo`, preso em `[min, max]`. Sem valor,
/// parte de `min` (ou 0).
pub fn passar(texto: &str, passos: f64, passo: f64, min: Option<f64>, max: Option<f64>) -> String {
    let base = ler(texto).unwrap_or(min.unwrap_or(0.0).max(0.0));
    // Arredonda a 6 casas para não acumular 0,1 + 0,2 = 0,30000000004
    // (sem prender ao passo: 1,5 + 1 continua 2,5).
    let mut v = ((base + passos * passo) * 1e6).round() / 1e6;
    if let Some(m) = min { v = v.max(m); }
    if let Some(m) = max { v = v.min(m); }
    escrever(v)
}

/// Faixa e passo do campo.
#[derive(Clone, Copy, Debug)]
pub struct Limites {
    pub passo: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl Default for Limites {
    fn default() -> Self { Self { passo: 1.0, min: None, max: None } }
}

/// O campo: `texto` atual, `dica` quando vazio, `largura` total e
/// `ao_mudar` recebendo o texto novo (digitado ou pelo passo).
pub fn campo_numero<'a, M: Clone + 'static>(
    texto: &str,
    dica: &str,
    largura: f32,
    limites: Limites,
    invalido: bool,
    ao_mudar: impl Fn(String) -> M + Clone + 'a,
) -> Element<'a, M> {
    let Limites { passo, min, max } = limites;
    let mais = ao_mudar(passar(texto, 1.0, passo, min, max));
    let menos = ao_mudar(passar(texto, -1.0, passo, min, max));
    let seta = |nome: &'static str, msg: M| {
        button(container(crate::icones::icone(nome, 10.0)).center(Length::Fill))
            .padding(0)
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(11.0))
            .style(estilo_botao_pequeno(false))
            .on_press(msg)
    };
    let setas = column![seta("chevron-up", mais.clone()), seta("chevron-down", menos.clone())].spacing(1);
    let entrada = text_input(dica, texto)
        .on_input(ao_mudar)
        .size(12)
        .padding([4, 6])
        .width(Length::Fill)
        .style(move |t: &iced::Theme, s| {
            let mut st = iced::widget::text_input::default(t, s);
            if invalido {
                st.border.color = crate::tema::COR_ERRO();
            } else if matches!(s, iced::widget::text_input::Status::Active) {
                st.border.color = COR_BORDA();
            }
            st
        });
    let corpo = row![entrada, setas].spacing(2).align_y(Alignment::Center).width(Length::Fixed(largura));
    mouse_area(corpo)
        .on_scroll(move |d| {
            let y = match d {
                mouse::ScrollDelta::Lines { y, .. } => y,
                mouse::ScrollDelta::Pixels { y, .. } => y,
            };
            if y >= 0.0 { mais.clone() } else { menos.clone() }
        })
        .into()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn passo_com_virgula_e_limites() {
        assert_eq!(passar("1,5", 1.0, 0.5, None, None), "2");
        assert_eq!(passar("0,1", 1.0, 0.1, None, None), "0,2");
        assert_eq!(passar("0", -1.0, 1.0, Some(0.0), None), "0");
        assert_eq!(passar("99", 1.0, 1.0, None, Some(100.0)), "100");
        assert_eq!(passar("", 1.0, 1.0, None, None), "1");
        assert_eq!(passar("1.5", 1.0, 1.0, None, None), "2,5");
        assert_eq!(ler("1.234,5"), Some(1234.5));
        assert_eq!(ler("abc"), None);
    }
}
