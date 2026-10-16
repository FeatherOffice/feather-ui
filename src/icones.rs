//! Ícones coloridos do Ribbon e catálogo de ações Tabler 3.49.0 (MIT).
//!
//! UI-012: catálogo incorporado com `include_str!`; as GUIs Iced de Linux e
//! Windows usam o MESMO SVG, sem ler arquivos gráficos em tempo de execução.
use iced::widget::svg;

#[path = "icones_ribbon.rs"]
mod ribbon;

const CATALOGO_EMBUTIDO: &str = include_str!("../dados/tabler-acoes.tsv");

/// SVG puro, construído só com o catálogo embutido. Separado do widget para
/// que os testes de paridade não dependam de janela, GPU, DPI ou SO.
fn svg_ribbon(nome: &str) -> Option<&'static str> {
    ribbon::FALTANTES
        .iter()
        .chain(ribbon::CATALOGO.iter())
        .chain(ribbon::INSERIR.iter())
        .chain(ribbon::SISTEMA.iter())
        .find(|(chave, alias, _)| nome == *chave || nome == *alias)
        .map(|(_, _, desenho)| *desenho)
}

/// Resolve as variáveis CSS do pacote: o rasterizador SVG não interpreta var().
fn resolver_cores(desenho: &str, cor: &str) -> String {
    resolver_cores_com_tema(desenho, cor, false)
}

fn resolver_cores_com_tema(desenho: &str, cor: &str, escuro: bool) -> String {
    let mut resultado = desenho.replace("currentColor", cor);
    while let Some(inicio) = resultado.find("var(") {
        let fim = resultado[inicio..]
            .find(')')
            .expect("variável CSS sem fechamento")
            + inicio;
        let (variavel, padrao) = resultado[inicio + 4..fim]
            .split_once(',')
            .expect("variável CSS sem cor padrão");
        let padrao = if escuro {
            match variavel.trim() {
                "--ic-paper" => "#1e293b",
                "--ic-blue" => "#60a5fa",
                "--ic-blue-bg" => "#253c5a",
                "--ic-emerald" => "#34d399",
                "--ic-amber" => "#fbbf24",
                "--ic-amber-bg" => "#544132",
                "--ic-orange" => "#fb923c",
                "--ic-purple" => "#a78bfa",
                "--ic-purple-bg" => "#393557",
                "--ic-red" => "#f87171",
                "--ic-teal" => "#2dd4bf",
                "--ic-pink" => "#fb7185",
                _ => padrao.trim(),
            }
        } else {
            padrao.trim()
        }
        .to_owned();
        resultado.replace_range(inicio..=fim, &padrao);
    }
    resultado
}

fn svg_embutido(nome: &str) -> String {
    if let Some(desenho) = svg_ribbon(nome) {
        return resolver_cores(desenho, "#808080");
    }
    svg_tabler(nome)
}

fn svg_tabler(nome: &str) -> String {
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
    // Variante `-filled` do Tabler: preenchida e sem contorno.
    let (preenchimento, contorno) = if nome.ends_with("-filled") {
        ("#808080", "none")
    } else {
        ("none", "#808080")
    };
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 {view_box} {view_box}\" fill=\"{preenchimento}\" stroke=\"{contorno}\" stroke-width=\"{traco}\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{paths}</svg>"
    )
}

/// Verifica se o ícone existe no catálogo embutido. O editor VBE usa isto
/// em testes para impedir que uma chave incorreta desenhe o SVG de fallback.
pub fn existe(nome: &str) -> bool {
    svg_ribbon(nome).is_some()
        || CATALOGO_EMBUTIDO
            .lines()
            .any(|linha| linha.split('\t').next() == Some(nome))
}

/// Widget de ícone. O vetor é entregue em memória: sem fonte SVG, URL nem
/// caminho de arquivo que possa faltar no pacote instalado em outro sistema.
pub fn icone(nome: &str, tamanho: f32) -> svg::Svg<'static, iced::Theme> {
    let colorido = svg_ribbon(nome).is_some();
    let desenho = if let Some(desenho) = svg_ribbon(nome) {
        resolver_cores_com_tema(
            desenho,
            &crate::tema::cor_do_icone("#202124"),
            crate::tema::escuro(),
        )
    } else {
        svg_embutido(nome)
    };
    svg::Svg::new(svg::Handle::from_memory(desenho.into_bytes()))
        .width(tamanho)
        .height(tamanho)
        .style(move |_, _| svg::Style {
            color: if colorido {
                None
            } else {
                Some(crate::tema::COR_TEXTO())
            },
        })
}

/// Sempre o traço Tabler monocromático (cor do texto), mesmo quando a faixa
/// tem um desenho colorido com esse nome: janelas de consulta (F1,
/// Primeiros passos, Novidades) não misturam ícones coloridos e de traço.
pub fn icone_traco(nome: &str, tamanho: f32) -> svg::Svg<'static, iced::Theme> {
    svg::Svg::new(svg::Handle::from_memory(svg_tabler(nome).into_bytes()))
        .width(tamanho)
        .height(tamanho)
        .style(|_, _| svg::Style { color: Some(crate::tema::COR_TEXTO()) })
}

/// Variante cromática para ações de destaque. Os SVGs do Ribbon conservam
/// suas cores próprias; a cor de destaque aplica-se apenas aos ícones Tabler.
/// A cor é restrita às usadas na identidade visual da suíte.
pub fn icone_colorido(nome: &str, tamanho: f32, cor: &str) -> svg::Svg<'static, iced::Theme> {
    let cor = match cor {
        "#21865c" | "#2179a7" | "#d68126" | "#7d5cc6" | "#ca5555" => cor,
        _ => "#21865c",
    };
    if svg_ribbon(nome).is_some() {
        return icone(nome, tamanho);
    }
    let original = svg_embutido(nome);
    let desenho = original.replacen("stroke=\"#808080\"", &format!("stroke=\"{cor}\""), 1);
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
            let svg = svg_tabler(colunas[0]);
            assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
            assert!(svg.ends_with("</svg>"));
            assert!(svg.contains("<path d=\""));
            assert!(!svg.contains("<image"));
            assert!(!svg.contains("<script"));
            assert!(!svg.contains("href="));
            assert!(!svg.contains("url("));
            assert!(!svg.contains("file://"));
            assert!(!svg.contains("<foreignObject"));
            assert_eq!(
                svg.matches("<path d=\"").count(),
                colunas[3].split('|').count()
            );
            quantidade += 1;
        }
        assert!(
            quantidade >= 60,
            "catálogo embutido incompleto: {quantidade}"
        );
    }

    #[test]
    fn familia_feathersheet_preserva_viewbox_16_sem_mudar_tabler_24() {
        let sheet = svg_embutido("clock");
        assert!(sheet.contains("viewBox=\"0 0 16 16\""));
        assert!(sheet.contains("stroke-width=\"1.4\""));
        let tabler = svg_tabler("align-center");
        assert!(tabler.contains("viewBox=\"0 0 24 24\""));
        assert!(tabler.contains("stroke-width=\"2\""));
    }

    #[test]
    fn ribbon_preserva_desenhos_aliases_e_cores_sem_css() {
        assert_eq!(ribbon::CATALOGO.len(), 43);
        assert_eq!(ribbon::INSERIR.len(), 35);
        let mut chaves = HashSet::new();
        for &(nome, alias, original) in ribbon::CATALOGO.iter().chain(ribbon::INSERIR) {
            assert!(chaves.insert(nome), "chave repetida: {nome}");
            if alias != nome {
                assert!(chaves.insert(alias), "alias repetido: {alias}");
            }
            assert!(existe(nome) && existe(alias));
            assert_eq!(svg_ribbon(nome), Some(original));
            // Um desenho dedicado em FALTANTES vence o alias; a chave em
            // português continua levando ao desenho original.
            let dedicado = ribbon::FALTANTES.iter().find(|(_, a, _)| *a == alias);
            assert_eq!(svg_ribbon(alias), Some(dedicado.map_or(original, |d| d.2)));
            let desenho = svg_embutido(alias);
            if dedicado.is_none() {
                assert_eq!(desenho, svg_embutido(nome));
            }
            assert!(desenho.contains("viewBox=\"0 0 24 24\""));
            assert!(!desenho.contains("var("));
            assert!(!desenho.contains("currentColor"));
            if original.contains("currentColor") {
                assert!(resolver_cores(original, "#e8eaed").contains("#e8eaed"));
            }
        }
        let colar = svg_embutido("clipboard");
        assert!(colar.contains("#d97706") && colar.contains("#3b82f6"));
        let _ = icone("clipboard", 24.0);
        let _ = icone_colorido("clipboard", 24.0, "#21865c");
    }

    #[test]
    fn chamadas_antigas_do_ribbon_recebem_os_novos_desenhos() {
        for (antigo, arquivo) in [
            ("clipboard", "colar"),
            ("copy", "copiar"),
            ("cut", "recortar"),
            ("bold", "negrito"),
            ("plus", "inserir_celula"),
            ("edit", "formatar_celula"),
            ("trash", "excluir_celula"),
            ("sum", "autosoma"),
        ] {
            assert_eq!(svg_embutido(antigo), svg_embutido(arquivo));
            assert_ne!(svg_embutido(antigo), svg_tabler(antigo));
        }
        for linha in CATALOGO_EMBUTIDO.lines() {
            assert!(existe(linha.split('\t').next().unwrap()));
        }
    }

    #[test]
    fn inserir_substitui_chaves_antigas_e_resolve_paleta_escura() {
        for (chave, arquivo) in [
            ("pivot-table", "tabela_dinamica"),
            ("table-suggest", "dinamicas_recomendadas"),
            ("refresh", "atualizar_dinamica"),
            ("table", "tabela"),
            ("photo", "imagens"),
            ("shapes", "formas"),
            ("sitemap", "smartart"),
            ("chart-bar", "grafico_geral"),
            ("chart-line", "grafico_linhas"),
            ("link", "inserir_link"),
            ("text-box", "caixa_texto"),
        ] {
            assert_eq!(svg_embutido(chave), svg_embutido(arquivo));
            assert_ne!(svg_embutido(chave), svg_tabler(chave));
        }
        let original = svg_ribbon("header-footer").unwrap();
        let claro = resolver_cores_com_tema(original, "#202124", false);
        let escuro = resolver_cores_com_tema(original, "#e8eaed", true);
        assert!(claro.contains("#ffffff"));
        assert!(escuro.contains("#1e293b") && escuro.contains("#34d399"));
        assert!(!escuro.contains("var(") && !escuro.contains("currentColor"));
    }

    #[test]
    fn icones_novos_da_faixa_resolvem_para_o_colorido() {
        // Nomes genericos de dialogos/menus: so a chave em portugues leva ao colorido.
        // list, window, keyboard e layout-grid também aparecem fora da faixa.
        const GENERICOS: [&str; 12] = [
            "informacao", "inspecao", "relogio", "menos", "guia_funcoes", "fechar",
            "seta_direita", "seta_esquerda", "recentes", "componentes", "atalhos_teclado",
            "desmesclar_celulas",
        ];
        let novos = &ribbon::FALTANTES[16..];
        assert_eq!(novos.len(), 24);
        for &(chave, alias, original) in novos {
            assert_eq!(svg_ribbon(chave), Some(original), "chave pt: {chave}");
            assert!(original.contains("viewBox=\"0 0 24 24\""));
            let desenho = svg_embutido(chave);
            assert!(!desenho.contains("var(") && !desenho.contains("currentColor"));
            if GENERICOS.contains(&chave) {
                assert_eq!(alias, chave, "apelido generico indevido: {chave}");
            } else {
                assert_eq!(svg_ribbon(alias), Some(original), "alias: {alias}");
                assert_eq!(svg_embutido(alias), desenho);
                assert_ne!(svg_embutido(alias), svg_tabler(alias));
            }
        }
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
        // FNV-1a de 64 bits sobre o catálogo legado, antes dos overrides do Ribbon.
        // O baseline inclui traço, viewBox, família, paths e fallback, sem
        // depender do rasterizador, SO ou escala física do monitor.
        let mut hash = 0xcbf29ce484222325_u64;
        for linha in CATALOGO_EMBUTIDO.lines() {
            let nome = linha.split('\t').next().expect("ícone sem nome");
            for byte in svg_tabler(nome).bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        assert_eq!(
            hash, 0x979c9bf809c867af,
            "o desenho SVG embutido mudou; revisar visual Linux/Windows antes de atualizar o baseline",
        );
    }

    #[test]
    fn icones_de_dialogo_existem_e_sao_monocromaticos() {
        for nome in [
            "check", "circle", "circle-filled", "arrow-left", "arrows-vertical",
            "arrow-down-right", "arrow-up-right", "alert-triangle", "x",
        ] {
            assert!(existe(nome), "{nome}");
            assert_eq!(svg_embutido(nome), svg_tabler(nome), "{nome} deve ser o Tabler puro");
        }
        assert!(svg_tabler("circle-filled").contains("fill=\"#808080\""));
        assert!(svg_tabler("circle").contains("fill=\"none\""));
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
