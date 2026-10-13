//! Marca vetorial única. A cor da fábrica pertence ao aplicativo;
//! aparência Claro/Escuro/Automático não altera a logo sem opção explícita.
use iced::widget::svg;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use crate::tema;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Produto { FeatherSheet=0, FeatherDocs=1, FeatherSlide=2, FeatherPDF=3 }
static PRODUTO: AtomicU8=AtomicU8::new(0);
static COR_PERSONALIZADA: AtomicBool=AtomicBool::new(false);

// Arquivos independentes editáveis em Inkscape, Figma e editores de texto.
pub const PENA_ORIGINAL:&str=include_str!("../assets/logos/pena.svg");
pub const SHEET_SVG:&str=include_str!("../assets/logos/feathersheet.svg");
pub const DOCS_SVG:&str=include_str!("../assets/logos/featherdocs.svg");
pub const SLIDE_SVG:&str=include_str!("../assets/logos/featherslide.svg");

pub fn definir_marca(produto:Produto, personalizada:bool) {
    PRODUTO.store(produto as u8,Ordering::Relaxed);
    COR_PERSONALIZADA.store(personalizada,Ordering::Relaxed);
}
pub fn produto_atual()->Produto {
    match PRODUTO.load(Ordering::Relaxed) {
        1=>Produto::FeatherDocs,2=>Produto::FeatherSlide,3=>Produto::FeatherPDF,_=>Produto::FeatherSheet
    }
}
fn clarear(cor:&str,valor:f32)->String {
    let bits=u32::from_str_radix(cor.trim_start_matches('#'),16).unwrap_or(0x287846);
    let f=valor.clamp(0.,1.);
    let canal=|shift| {
        let cor=((bits >> shift) & 0xff_u32) as f32;
        (cor+(255.-cor)*f).round().clamp(0.,255.) as u32
    };
    format!("#{:02x}{:02x}{:02x}",canal(16),canal(8),canal(0))
}
/// A cor de destaque só recolore a marca quando foi selecionada pelo usuário.
pub fn svg_para(produto:Produto,personalizada:bool)->String {
    if !personalizada {
        return match produto {
            Produto::FeatherSheet => SHEET_SVG.to_owned(),
            Produto::FeatherDocs => DOCS_SVG.to_owned(),
            Produto::FeatherSlide => SLIDE_SVG.to_owned(),
            Produto::FeatherPDF => PENA_ORIGINAL
                .replace("#287846", "#b84558")
                .replace("#72aa86", "#d28f9a"),
        };
    }
    let cor=tema::acento_hex();
    PENA_ORIGINAL
        .replace("#287846",cor)
        .replace("#72aa86",&clarear(cor,0.40))
        .replace("#f4faf5",&clarear(cor,0.95))
        .replace("#e8f4eb",&clarear(cor,0.88))
        .replace("#e4f0e7",&clarear(cor,0.85))
}
/// Para uso pelo Iced, garantimos um canvas **opaco** na cor real da
/// superfície. Os cantos do retângulo arredondado continuam visualmente
/// transparentes, mas não há pixels com alfa parcial para o backend de SVG
/// compor contra preto (franja escura nos quatro cantos, especialmente em
/// escala fracionária). Os arquivos SVG distribuídos permanecem transparentes.
fn svg_para_iced(produto:Produto, personalizada:bool)->String {
    let svg=svg_para(produto,personalizada);
    let bg=tema::COR_PAINEL();
    let canal=|v:f32| -> u8 {(v.clamp(0.,1.)*255.).round() as u8};
    let cor=format!("#{:02x}{:02x}{:02x}",canal(bg.r),canal(bg.g),canal(bg.b));
    let fundo=format!(
        r#"<rect id="fundo-opaco-iced" x="0" y="0" width="128" height="128" fill="{cor}"/>"#
    );
    // A inserção fica antes do fundo com cantos arredondados: a geometria
    // original, o degradê e as duas penas não são modificados.
    svg.replacen("<rect id=\"fundo-arredondado\"", 
        &format!("{fundo}\n  <rect id=\"fundo-arredondado\""), 1)
}
pub fn icone_para(produto:Produto,personalizada:bool,tamanho:f32)
    ->svg::Svg<'static,iced::Theme>
{
    svg::Svg::new(svg::Handle::from_memory(
        svg_para_iced(produto,personalizada).into_bytes()
    )).width(tamanho).height(tamanho)
}
pub fn icone_atual(tamanho:f32)->svg::Svg<'static,iced::Theme> {
    icone_para(produto_atual(),COR_PERSONALIZADA.load(Ordering::Relaxed),tamanho)
}
#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn quatro_produtos_preservam_svg_vetorial_e_cores_de_fabrica() {
        for (produto,cor) in [
            (Produto::FeatherSheet,"#287846"),
            (Produto::FeatherDocs,"#236ac7"),
            (Produto::FeatherSlide,"#d1a219"),
            (Produto::FeatherPDF,"#b84558"),
        ] {
            let svg=svg_para(produto,false);
            assert!(svg.contains(cor));
            // SVG editado pelo Inkscape tem atributos id/fill antes de d.
            // Verificar dois caminhos independentes sem depender dessa ordem.
            assert_eq!(svg.matches("<path ").count(), 2, "duas penas vetoriais");
            assert!(svg.contains("id=\"pena-principal\""));
            assert!(svg.contains("id=\"pena-secundaria\""));
            assert!(svg.matches(" d=\"").count() >= 2);
            assert!(!svg.contains("<image"));
            assert!(!svg.contains("base64"));
        }
    }
    #[test]
    fn acento_personalizado_recolore_sem_sobrescrever_marca_padrao() {
        let svg=svg_para(Produto::FeatherSlide,true);
        assert!(svg.contains(tema::acento_hex()));
        assert_ne!(svg,svg_para(Produto::FeatherSlide,false));
        assert!(svg_para(Produto::FeatherSheet,false).contains("#287846"));
    }
    #[test]
    fn renderizacao_iced_cobre_cantos_transparentes_sem_borda_preta() {
        for produto in [Produto::FeatherSheet,Produto::FeatherDocs,Produto::FeatherSlide,Produto::FeatherPDF] {
            let original=svg_para(produto,false);
            let rendered=svg_para_iced(produto,false);
            assert!(!original.contains("fundo-opaco-iced"),
                "Os SVGs editáveis devem manter a transparência original.");
            let fundo=rendered.find("id=\"fundo-opaco-iced\"")
                .expect("O rasterizador precisa receber um fundo opaco.");
            let arredondado=rendered.find("id=\"fundo-arredondado\"")
                .expect("Os cantos originais permanecem arredondados.");
            assert!(fundo<arredondado);
            assert!(rendered.contains("width=\"128\" height=\"128\" fill=\"#"));
            assert!(rendered.contains("id=\"pena-principal\""));
            assert!(rendered.contains("id=\"pena-secundaria\""));
        }
    }
    #[test]
    fn svg_instanciavel_com_iced_sem_png_ou_arquivo_de_rede() {
        let _:svg::Svg<'static,iced::Theme>=icone_para(Produto::FeatherSheet,false,24.);
        let _:svg::Svg<'static,iced::Theme>=icone_para(Produto::FeatherDocs,false,72.);
    }
}
