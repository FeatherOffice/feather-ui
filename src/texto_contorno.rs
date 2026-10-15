//! Contorno das letras pela fonte do Iced: o texto vira polígonos (um por subcaminho), com os
//! miolos preservados (O, B, a, e…). Serve ao texto artístico (WordArt) de qualquer aplicativo:
//! quem chama deforma os pontos (por exemplo com `feather_shapes::wordart::transformar`) e
//! desenha os polígonos com preenchimento par/ímpar ou não nulo.
//!
//! Sem dependência de motor: a deformação entra como função, para o `feather-ui` não puxar o
//! `feather-shapes`.
use iced::widget::canvas::path::lyon_path::{iterator::PathIterator, Event};
use iced::widget::canvas::Text;

/// Tolerância do achatamento das curvas da fonte, em pixels.
const TOLERANCIA: f32 = 0.12;

/// Resolve a fonte do `texto` e devolve todos os subcaminhos fechados como polígonos
/// (`[x, y]` na mesma coordenada de `texto.position`). Subcaminho com menos de 3 pontos sai.
pub fn contornos(texto: &Text) -> Vec<Vec<[f32; 2]>> {
    let mut fora = Vec::new();
    texto.draw_with(|caminho, _| {
        let mut atual = Vec::new();
        for evento in caminho.raw().iter().flattened(TOLERANCIA) {
            match evento {
                Event::Begin { at } => atual = vec![[at.x, at.y]],
                Event::Line { to, .. } => atual.push([to.x, to.y]),
                Event::End { .. } => {
                    if atual.len() > 2 {
                        fora.push(std::mem::take(&mut atual));
                    }
                }
                _ => {}
            }
        }
    });
    fora
}

/// [`contornos`] com cada ponto passado por `deformar` (a transformação do WordArt).
pub fn contornos_deformados(texto: &Text, deformar: impl Fn([f32; 2]) -> [f32; 2]) -> Vec<Vec<[f32; 2]>> {
    contornos(texto)
        .into_iter()
        .map(|p| p.into_iter().map(&deformar).collect())
        .collect()
}

/// Caixa `[x, y, largura, altura]` que envolve os polígonos; `None` se não houver pontos.
pub fn caixa(poligonos: &[Vec<[f32; 2]>]) -> Option<[f32; 4]> {
    let mut it = poligonos.iter().flatten();
    let &[x0, y0] = it.next()?;
    let (mut a, mut b, mut c, mut d) = (x0, y0, x0, y0);
    for &[x, y] in it {
        a = a.min(x);
        b = b.min(y);
        c = c.max(x);
        d = d.max(y);
    }
    Some([a, b, c - a, d - b])
}

#[cfg(test)]
mod testes {
    use super::*;
    use iced::{Color, Point};

    fn texto(conteudo: &str) -> Text {
        Text {
            content: conteudo.into(),
            position: Point::new(10.0, 10.0),
            color: Color::BLACK,
            size: 48.0.into(),
            shaping: iced::widget::text::Shaping::Advanced,
            ..Default::default()
        }
    }

    #[test]
    fn letra_o_tem_miolo() {
        let c = contornos(&texto("O"));
        assert!(c.len() >= 2, "O precisa de contorno externo e interno, veio {}", c.len());
        let [x, y, w, h] = caixa(&c).unwrap();
        assert!(x >= 9.0 && y >= 9.0 && w > 10.0 && h > 10.0);
    }

    #[test]
    fn deformar_desloca_todos_os_pontos() {
        let t = texto("A");
        let base = contornos(&t);
        let movidos = contornos_deformados(&t, |[x, y]| [x + 5.0, y]);
        assert_eq!(base.len(), movidos.len());
        assert_eq!(movidos[0][0], [base[0][0][0] + 5.0, base[0][0][1]]);
        assert!(contornos(&texto("")).is_empty());
        assert_eq!(caixa(&[]), None);
    }
}
