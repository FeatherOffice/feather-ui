//! Ícones de ações Tabler 3.49.0 (MIT), compartilhados sem depender de um motor.
//!
//! UI-012: catálogo incorporado com `include_str!`; as GUIs Iced de Linux e
//! Windows usam o MESMO SVG, sem ler arquivos gráficos em tempo de execução.
use iced::widget::svg;

const CATALOGO_EMBUTIDO: &str = include_str!("../dados/tabler-acoes.tsv");

/// SVG puro, construído só com o catálogo embutido. Separado do widget para
/// que os testes de paridade não dependam de janela, GPU, DPI ou SO.
fn svg_embutido(nome: &str) -> String {
    let (caminhos, original_sheet) = CATALOGO_EMBUTIDO
        .lines()
        .find_map(|l| {
            let mut partes = l.split('\t');
            let nome_linha = partes.next()?;
            let familia = partes.next()?;
            let _tags = partes.next()?;
            let caminhos = partes.next()?;
            (nome_linha == nome).then_some((caminhos, familia == "FeatherSheet"))
        })
        .unwrap_or(("M12 5v14M5 12h14", false));
    let paths = caminhos
        .split('|')
        .map(|d| format!("<path d=\"{d}\"/>"))
        .collect::<String>();
    // Os ícones originais do FeatherSheet usam viewBox 16 × 16 e traço 1.4.
    // Os Tabler da suíte continuam em 24 × 24, sem alterar Slide/Docs.
    let (view_box, traco) = if original_sheet { (16, 1.4) } else { (24, 2.0) };
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 {view_box} {view_box}\" fill=\"none\" stroke=\"#808080\" stroke-width=\"{traco}\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{paths}</svg>"
    )
}

/// Verifica se o ícone existe no catálogo embutido. O editor VBE usa isto
/// em testes para impedir que uma chave incorreta desenhe o SVG de fallback.
pub fn existe(nome: &str) -> bool {
    CATALOGO_EMBUTIDO
        .lines()
        .any(|linha| linha.split('\t').next() == Some(nome))
}

/// Widget de ícone. O vetor é entregue em memória: sem fonte SVG, URL nem
/// caminho de arquivo que possa faltar no pacote instalado em outro sistema.
pub fn icone(nome: &str, tamanho: f32) -> svg::Svg<'static, iced::Theme> {
    svg::Svg::new(svg::Handle::from_memory(
        svg_embutido(nome).into_bytes(),
    ))
    .width(tamanho)
    .height(tamanho)
    .style(|_, _| svg::Style {
        color: Some(crate::tema::COR_TEXTO()),
    })
}

/// Variante cromática do SVG Tabler para ações de destaque na Ribbon.
/// Os paths e viewBox continuam no catálogo original (sem arquivos ou fontes extras).
/// A cor é restrita às usadas na identidade visual da suíte.
pub fn icone_colorido(nome: &str, tamanho: f32, cor: &str) -> svg::Svg<'static, iced::Theme> {
    let cor = match cor {
        "#21865c" | "#2179a7" | "#d68126" | "#7d5cc6" | "#ca5555" => cor,
        _ => "#21865c",
    };
    let original = svg_embutido(nome);
    let desenho = original.replacen(
        "stroke=\"#808080\"", &format!("stroke=\"{cor}\""), 1,
    );
    svg::Svg::new(svg::Handle::from_memory(desenho.into_bytes()))
        .width(tamanho)
        .height(tamanho)
        .style(|_, _| svg::Style { color: None })
}

#[cfg(test)]
mod testes_ui012 {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalogo_de_icones_e_embutido_e_sem_recursos_em_rede() {
        let mut nomes = HashSet::new();
        let mut quantidade = 0;
        for linha in CATALOGO_EMBUTIDO.lines() {
            let colunas: Vec<_> = linha.split('\t').collect();
            assert_eq!(colunas.len(), 4, "linha inválida do catálogo: {linha}");
            assert!(nomes.insert(colunas[0]), "ícone duplicado: {}", colunas[0]);
            let svg = svg_embutido(colunas[0]);
            assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
            assert!(svg.ends_with("</svg>"));
            assert!(svg.contains("<path d=\""));
            assert!(!svg.contains("<image"));
            assert!(!svg.contains("<script"));
            assert!(!svg.contains("href="));
            assert!(!svg.contains("url("));
            assert!(!svg.contains("file://"));
            assert!(!svg.contains("<foreignObject"));
            assert_eq!(svg.matches("<path d=\"").count(), colunas[3].split('|').count());
            quantidade += 1;
        }
        assert!(quantidade >= 60, "catálogo embutido incompleto: {quantidade}");
    }

    #[test]
    fn familia_feathersheet_preserva_viewbox_16_sem_mudar_tabler_24() {
        let sheet = svg_embutido("clock");
        assert!(sheet.contains("viewBox=\"0 0 16 16\""));
        assert!(sheet.contains("stroke-width=\"1.4\""));
        let tabler = svg_embutido("align-center");
        assert!(tabler.contains("viewBox=\"0 0 24 24\""));
        assert!(tabler.contains("stroke-width=\"2\""));
    }

    #[test]
    fn fallback_de_icone_inexistente_e_local_deterministico() {
        let ausente = svg_embutido("nao-existe-UI012");
        assert!(ausente.contains("<path d=\"M12 5v14M5 12h14\"/>"));
        assert_eq!(ausente, svg_embutido("outra-chave-inexistente"));
        assert!(!ausente.contains("href="));
    }

    #[test]
    fn representacao_svg_tem_hash_estavel_em_linux_e_windows() {
        // FNV-1a de 64 bits sobre os SVGs emitidos na ordem do catálogo.
        // O baseline inclui traço, viewBox, família, paths e fallback, sem
        // depender do rasterizador, SO ou escala física do monitor.
        let mut hash = 0xcbf29ce484222325_u64;
        for linha in CATALOGO_EMBUTIDO.lines() {
            let nome = linha.split('\t').next().expect("ícone sem nome");
            for byte in svg_embutido(nome).bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        assert_eq!(
            hash, 0x97fda324d08665e8,
            "o desenho SVG embutido mudou; revisar visual Linux/Windows antes de atualizar o baseline",
        );
    }

    #[test]
    fn iconografia_nao_depende_de_diretorio_corrente() {
        let svg1 = svg_embutido("copy");
        let svg2 = svg_embutido("copy");
        assert_eq!(svg1.as_bytes(), svg2.as_bytes());
        assert!(svg1.contains("viewBox=\"0 0 24 24\""));
        // A mesma string é entregue ao Handle::from_memory nos dois sistemas.
    }
}
