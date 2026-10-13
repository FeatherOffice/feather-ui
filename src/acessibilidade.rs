//! Contraste WCAG 2.x para componentes Iced da suíte FeatherOffice.
//!
//! Funções puras: não consultam o tema global, não dependem de documento e
//! podem ser testadas em paralelo, independentemente de janelas/monitor.
//! Os valores devem ser cores opacas (RGB sRGB, alpha=1.0).

use iced::Color;

/// WCAG AA para texto normal (incluindo labels pequenos e abas da Ribbon).
pub const CONTRASTE_TEXTO_AA: f32 = 4.5;
/// WCAG 1.4.11 para componentes visuais e indicadores de foco.
pub const CONTRASTE_CONTROLE_AA: f32 = 3.0;

fn linear(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// Luminância relativa sRGB [0,1] sem arredondamentos de 8 bits.
pub fn luminancia(c: Color) -> f32 {
    0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b)
}

/// Razão WCAG de contraste entre duas cores RGB **opacas** (1.0–21.0).
pub fn contraste(a: Color, b: Color) -> f32 {
    let la = luminancia(a);
    let lb = luminancia(b);
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Escolhe preto ou branco, garantindo no mínimo 4.5:1 para fundo RGB opaco.
pub fn texto_sobre(fundo: Color) -> Color {
    let preto = Color::BLACK;
    let branco = Color::WHITE;
    if contraste(preto, fundo) >= contraste(branco, fundo) {
        preto
    } else {
        branco
    }
}

fn mesclar(origem: Color, destino: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: origem.r + (destino.r - origem.r) * t,
        g: origem.g + (destino.g - origem.g) * t,
        b: origem.b + (destino.b - origem.b) * t,
        a: 1.0,
    }
}

fn adaptar(acent: Color, fundo: Color, alvo: f32, destino: Color) -> (Color, f32) {
    // Não participar da escolha quando nem a cor extrema alcança o contraste
    // pedido. Antes, um destino inalcançável ganhava o empate em t=1,
    // podendo devolver branco sobre branco para um pedido de 21:1.
    if contraste(destino, fundo) < alvo {
        return (destino, f32::INFINITY);
    }
    // Busca binária da menor alteração de intensidade que alcance o alvo;
    // preserva o tom do acento quando já tem contraste suficiente.
    let mut minimo = 0.0_f32;
    let mut maximo = 1.0_f32;
    for _ in 0..24 {
        let meio = (minimo + maximo) * 0.5;
        if contraste(mesclar(acent, destino, meio), fundo) >= alvo {
            maximo = meio;
        } else {
            minimo = meio;
        }
    }
    (mesclar(acent, destino, maximo), maximo)
}

/// Ajusta o acento *somente quando utilizado como texto*, preservando ao máximo
/// sua tonalidade original. Não modifica a paleta armazenada do usuário.
pub fn acento_legivel(acento: Color, fundo: Color, razao_minima: f32) -> Color {
    let alvo = if razao_minima.is_finite() {
        razao_minima.clamp(1.0, 21.0)
    } else {
        CONTRASTE_TEXTO_AA
    };
    if contraste(acento, fundo) >= alvo {
        return acento;
    }
    let (escuro, passo_escuro) = adaptar(acento, fundo, alvo, Color::BLACK);
    let (claro, passo_claro) = adaptar(acento, fundo, alvo, Color::WHITE);
    // Ambos inalcançáveis: o contraste pedido foi limitado a 21:1,
    // mas fundos intermediários não permitem esse valor com nenhum texto.
    // Escolher o extremo mais legível em vez de retornar uma cor invisível.
    if passo_escuro.is_infinite() && passo_claro.is_infinite() {
        return texto_sobre(fundo);
    }
    // Ajuste mínimo de RGB: nunca preferir um destino inalcançável.
    if passo_escuro < passo_claro { escuro } else { claro }
}

/// Texto de acento em fundo conhecido: 4.5:1 WCAG AA.
pub fn texto_acento(acento: Color, fundo: Color) -> Color {
    acento_legivel(acento, fundo, CONTRASTE_TEXTO_AA)
}

/// Indicador de foco com contraste mínimo 3:1 sobre a superfície do controle.
pub fn borda_foco(acento: Color, fundo: Color) -> Color {
    acento_legivel(acento, fundo, CONTRASTE_CONTROLE_AA)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn cor(hex: &str) -> Color {
        let v = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap();
        Color::from_rgb8((v >> 16) as u8, (v >> 8) as u8, v as u8)
    }

    #[test]
    fn preto_e_branco_w3c_21_para_1() {
        assert!((contraste(Color::BLACK, Color::WHITE) - 21.0).abs() < 0.001);
        assert!((contraste(Color::WHITE, Color::BLACK) - 21.0).abs() < 0.001);
        assert!((contraste(Color::BLACK, Color::BLACK) - 1.0).abs() < 0.001);
        assert!(contraste(cor("#777777"), cor("#ffffff")) < CONTRASTE_TEXTO_AA);
    }

    #[test]
    fn texto_contrasta_com_qualquer_fundo_rgb_amostrado() {
        for r in (0..=255).step_by(17) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(17) {
                    let bg = Color::from_rgb8(r, g, b);
                    assert!(contraste(texto_sobre(bg), bg) >= CONTRASTE_TEXTO_AA);
                }
            }
        }
    }

    #[test]
    fn seis_acentos_em_tema_claro_e_escuro_passam_aa() {
        let cores = [
            ("#2b7a78", "#4fb3ad"), ("#0b57d0", "#8ab4f8"),
            ("#188038", "#81c995"), ("#e8710a", "#fcad70"),
            ("#d93025", "#f28b82"), ("#8e24aa", "#d7aefb"),
        ];
        for (claro, escuro) in cores {
            for (acento, fundo) in [(claro, "#ffffff"), (escuro, "#2a2d32")] {
                let bg = cor(fundo);
                let c = texto_acento(cor(acento), bg);
                assert!(contraste(c, bg) + 0.0001 >= CONTRASTE_TEXTO_AA,
                    "acento {acento} fundo {fundo}: {}", contraste(c, bg));
                let foco = borda_foco(cor(acento), bg);
                assert!(contraste(foco, bg) + 0.0001 >= CONTRASTE_CONTROLE_AA);
            }
        }
    }

    #[test]
    fn acento_ja_legivel_nao_e_modificado() {
        let azul = cor("#0b57d0");
        let fundo = Color::WHITE;
        assert_eq!(acento_legivel(azul, fundo, CONTRASTE_TEXTO_AA), azul);
    }

    #[test]
    fn alternancia_de_acento_nao_contamina_cor_de_fundo() {
        let fundo = cor("#ffffff");
        let laranja = texto_acento(cor("#e8710a"), fundo);
        let roxo = texto_acento(cor("#8e24aa"), fundo);
        assert!(contraste(laranja, fundo) >= CONTRASTE_TEXTO_AA);
        assert!(contraste(roxo, fundo) >= CONTRASTE_TEXTO_AA);
        assert_eq!(fundo, cor("#ffffff"));
    }

    #[test]
    fn ratios_invalidos_sao_limitados_ao_intervalo_wcag() {
        let bg = Color::WHITE;
        let a = acento_legivel(cor("#bbbbbb"), bg, f32::NAN);
        assert!(contraste(a, bg) >= CONTRASTE_TEXTO_AA);
        let b = acento_legivel(cor("#bbbbbb"), bg, f32::INFINITY);
        assert!(contraste(b, bg) >= CONTRASTE_TEXTO_AA);
        assert_eq!(acento_legivel(Color::BLACK, bg, 99.0), Color::BLACK);
        assert_eq!(acento_legivel(Color::WHITE, bg, 99.0), Color::BLACK);
        assert_eq!(acento_legivel(Color::BLACK, Color::BLACK, 99.0), Color::WHITE);
        // Para cinza, 21:1 é matematicamente inalcançável.
        // A função deve escolher a cor de maior contraste, não a menos legível.
        let cinza = cor("#888888");
        let impossivel = acento_legivel(cor("#888888"), cinza, 21.0);
        assert_eq!(impossivel, texto_sobre(cinza));
    }
}
