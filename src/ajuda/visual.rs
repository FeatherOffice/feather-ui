//! Ajuda FeatherOffice: landing de tópicos + navegação em três grupos.
//! O modelo, a busca e os eventos continuam compartilhados entre os apps;
//! todas as cores vêm dos tokens da preferência ativa (claro/escuro/acento).
use super::*;
use crate::{consulta, icones, tema};
use crate::consulta::estilo_cartao as estilo_card;
use iced::widget::{button, column, container, row, scrollable, svg, text, text_input, Space};
use iced::{Alignment, Background, Border, Element, Font, Length, Theme};

#[derive(Clone)]
struct Guia {
    titulo: String,
    descricao: String,
    icone: &'static str,
    destino: Evento,
}
fn texto_negrito(nome: impl Into<String>, tamanho: f32) -> Element<'static, Evento> {
    consulta::negrito(nome, tamanho)
}

fn barra() -> container::Style {
    container::Style {
        background: Some(Background::Color(tema::COR_PAINEL())),
        border: Border {
            color: tema::COR_BORDA_SUAVE(),
            width: 1.,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}
fn superficie_sidebar(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(tema::COR_PAINEL_FAIXA())),
        border: Border {
            color: tema::COR_BORDA_SUAVE(),
            width: 1.,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn icone_hero() -> Element<'static, Evento> {
    consulta::marca()
}
fn ilustra_planilha() -> Element<'static, Evento> {
    consulta::ilustracao_planilha()
}
fn resumo(t: &Topico) -> String {
    t.linhas.iter().find_map(|l| match l {
        Linha::Texto(txt) | Linha::Marcador(txt) => Some(txt.clone()),
        _ => None,
    }).map(|txt| {
        let mut s=txt.chars().take(84).collect::<String>();
        if txt.chars().count()>84 {s.push('…');}
        s
    }).unwrap_or_else(|| "Abra o guia e veja exemplos práticos.".into())
}
fn icone_topico(id: &str) -> &'static str {
    match id {
        "comecar" => "file-plus", "formulas" => "math-function",
        "preencher" => "edit", "abas" => "layout-sidebar",
        "formatar" => "bold", "tabelas" => "table",
        "graficos" => "chart-bar", "dados" => "filter",
        "macros" => "code", "arquivos" => "folder-open",
        "config" => "settings", "funcoes" => "math-function",
        "atalhos" => "keyboard", "slides"=>"layout-grid", "elementos"|"formas"=>"forms", "texto"=>"text-size", "apresentar"=>"player-play", "configuracoes"=>"settings", _ => "help",
    }
}
fn guia_do_topico(s: usize, i: usize, t: &Topico) -> Guia {
    Guia {
        titulo: t.titulo.clone(),
        descricao: resumo(t),
        icone: icone_topico(&t.id),
        destino: Evento::IrTopico(s,i),
    }
}
fn cartao(guia: &Guia) -> Element<'static, Evento> {
    consulta::cartao(guia.icone, guia.titulo.clone(), guia.descricao.clone(), Some(guia.destino.clone()))
}
fn grade_de_cards(guias: &[Guia], colunas: usize) -> Element<'static, Evento> {
    consulta::grade(guias.iter().map(cartao).collect(), colunas)
}
fn rotulo_nav(rotulo: &'static str) -> Element<'static,Evento> {
    consulta::rotulo_grupo(rotulo)
}
fn item_nav(rotulo: String, icone: &str, ativo: bool, destino: Evento) -> Element<'static,Evento> {
    consulta::item_nav(rotulo, icone, ativo, Some(destino))
}
fn navega(catalogo: &Catalogo, estado: &Estado) -> Element<'static,Evento> {
    let mut nav=column![].spacing(2);
    nav=nav.push(item_nav("Início".into(),"file-plus",
        estado.secao==0 && estado.topico.is_none() && estado.busca.trim().is_empty(),
        Evento::IrSecao(0)));
    let Some(primeiro)=catalogo.secoes.first() else {return nav.into();};
    let indices=|ids: &[&str]| -> Vec<usize> {
        primeiro.topicos.iter().enumerate()
            .filter_map(|(i,t)|ids.contains(&t.id.as_str()).then_some(i)).collect()
    };
    let basico=indices(&["comecar","abas","preencher","formulas","formatar","tabelas","graficos","slides","elementos","texto","formas"]);
    let avancado=indices(&["dados","macros","apresentar"]);
    let outros=indices(&["arquivos","config","configuracoes"]);
    let outros_indices=(0..primeiro.topicos.len()).filter(|i|
        !basico.contains(i)&&!avancado.contains(i)&&!outros.contains(i)
    ).collect::<Vec<_>>();
    let group=|nav: iced::widget::Column<'static,Evento>,
                   titulo: &'static str, itens: &[usize]| {
        if itens.is_empty(){return nav;}
        let mut nav=nav.push(rotulo_nav(titulo));
        for &i in itens {
            if let Some(t)=primeiro.topicos.get(i) {
                nav=nav.push(item_nav(t.titulo.clone(),icone_topico(&t.id),
                    estado.busca.trim().is_empty()&&estado.secao==0&&estado.topico==Some(i),
                    Evento::IrTopico(0,i)));
            }
        }
        nav
    };
    nav=group(nav,"BÁSICO",&basico);
    nav=group(nav,"AVANÇADO",&avancado);
    for (i,s) in catalogo.secoes.iter().enumerate().skip(1) {
        if s.id=="atalhos" {continue;}
        nav=nav.push(item_nav(s.titulo.clone(),icone_topico(&s.id),
            estado.busca.trim().is_empty() && estado.secao==i && estado.topico.is_none(),
            Evento::IrSecao(i)));
    }
    nav=group(nav,"OUTROS",&outros);
    for i in outros_indices {
        if let Some(t)=primeiro.topicos.get(i) {
            nav=nav.push(item_nav(t.titulo.clone(),icone_topico(&t.id),
                estado.busca.trim().is_empty()&&estado.secao==0&&estado.topico==Some(i),
                Evento::IrTopico(0,i)));
        }
    }
    for (i,s) in catalogo.secoes.iter().enumerate().skip(1) {
        if s.id!="atalhos" {continue;}
        nav=nav.push(item_nav(s.titulo.clone(),"keyboard",
            estado.busca.trim().is_empty()&&estado.secao==i&&estado.topico.is_none(),
            Evento::IrSecao(i)));
    }
    nav.into()
}
fn home(catalogo: &Catalogo) -> Element<'static, Evento> {
    let Some(secao)=catalogo.secoes.first() else {return text("Ajuda indisponível.").into()};
    let intro=secao.introducao.clone();
    let hero=row![
        icone_hero(),
        column![
            texto_negrito(secao.titulo.clone(), 24.),
            text(intro).size(13).line_height(1.45)
                .color(tema::COR_TEXTO_MUTED()),
        ].spacing(12).width(Length::Fill),
        if secao.topicos.iter().any(|t|t.id=="apresentar") {ilustra_apresentacao()}else{ilustra_planilha()},
    ].spacing(18).align_y(Alignment::Center);
    let ordem=["comecar","formatar","tabelas","graficos","dados","macros","funcoes","config","atalhos"];
    let mut itens=Vec::new();
    for id in ordem {
        if let Some((i,t))=secao.topicos.iter().enumerate().find(|(_,t)|t.id==id) {
            itens.push(guia_do_topico(0,i,t));
        } else if let Some((i,s))=catalogo.secoes.iter().enumerate().find(|(_,s)|s.id==id) {
            itens.push(Guia {
                titulo:s.titulo.clone(), descricao:s.introducao.chars().take(84).collect(),
                icone:icone_topico(id), destino:Evento::IrSecao(i)
            });
        }
    }
    // Inclui todos os assuntos dos outros editores, além da ordem histórica do Sheet.
    for (i,t) in secao.topicos.iter().enumerate().filter(|(_,t)|!ordem.contains(&t.id.as_str())) {
        itens.push(guia_do_topico(0,i,t));
    }
    // Guia rápido abre sua explicação correspondente; não cria arquivos sem confirmação.
    let apresentacao=secao.topicos.iter().any(|t|t.id=="apresentar");
    let atalhos=if apresentacao {vec![
        ("file-plus","Criar apresentação","comecar"),("folder-open","Abrir arquivo","arquivos"),
        ("device-floppy","Salvar","arquivos"),("forms","Inserir forma","elementos"),("player-play","Apresentar","apresentar"),
    ]} else {vec![
        ("file-plus","Criar planilha","comecar"),
        ("folder-open","Abrir arquivo","arquivos"),
        ("device-floppy","Salvar","arquivos"),
        ("printer","Imprimir","arquivos"),
        ("filter","Importar dados","dados"),
        ("math-function","Inserir função","formulas"),
        ("table","Criar tabela","tabelas"),
    ]};
    let mut quick=row![].spacing(7);
    for &(icone,titulo,topico) in &atalhos {
        if let Some((i,_))=secao.topicos.iter().enumerate().find(|(_,t)|t.id==topico) {
            quick=quick.push(button(row![
                icones::icone(icone,14.),
                text(titulo).size(11.5),
            ].spacing(6).align_y(Alignment::Center))
                .style(estilo_card).padding([8,9])
                .on_press(Evento::IrTopico(0,i)));
        }
    }
    column![
        hero,
        texto_negrito("Tópicos principais", 18.),
        grade_de_cards(&itens,3),
        container(Space::with_height(Length::Fixed(1.)))
            .width(Length::Fill).style(|_|container::Style {
                background:Some(Background::Color(tema::COR_BORDA_SUAVE())),
                ..Default::default()
            }),
        texto_negrito("Guias rápidos", 18.),
        quick,
    ].spacing(17).width(Length::Fill).into()
}
fn botao_copiar(texto: &str, copiado: Option<&str>) -> Element<'static, Evento> {
    let feito = copiado == Some(texto);
    crate::componentes::botao_icone(
        icones::icone(if feito { "check" } else { "copy" }, 16.),
        if feito { "Copiado" } else { "Copiar" },
        Some(Evento::Copiar(texto.to_owned())),
        feito,
    )
}
fn ver_topico(topico: &Topico, copiado: Option<&str>) -> Element<'static, Evento> {
    let mut body=column![texto_negrito(topico.titulo.clone(),23.)].spacing(13);
    for linha in &topico.linhas {
        let elemento:Element<'static,Evento>=match linha {
            Linha::Texto(s)=>text(s.clone()).size(14.).line_height(1.5).into(),
            Linha::Marcador(s)=>row![text("•"),text(s.clone()).size(14.).line_height(1.5)]
                .spacing(9).into(),
            Linha::Atalho{descricao,tecla}=>row![
                text(descricao.clone()).size(13.).width(Length::FillPortion(3)),
                text(tecla.clone()).size(13.).font(Font::MONOSPACE)
                    .width(Length::FillPortion(2)),
            ].spacing(9).into(),
            Linha::Exemplo(s)=>container(row![
                    text(s.clone()).size(13.).font(Font::MONOSPACE).width(Length::Fill),
                    botao_copiar(s, copiado),
                ].spacing(10).align_y(Alignment::Start))
                .padding(14).width(Length::Fill)
                .style(|_|container::Style {
                    background:Some(Background::Color(tema::COR_PAINEL_FAIXA())),
                    border:Border {radius:8.0.into(), ..Default::default()},
                    ..Default::default()
                }).into(),
        };
        body=body.push(elemento);
    }
    body.into()
}
fn conteudo(catalogo:&Catalogo,estado:&Estado)->Element<'static,Evento>{
    if !estado.busca.trim().is_empty() {
        let termo=normalizar(estado.busca.trim());
        let mut encontrados=Vec::new();
        for (s,secao) in catalogo.secoes.iter().enumerate() {
            for (i,topico) in secao.topicos.iter().enumerate() {
                if corresponde(&termo,topico) {
                    encontrados.push(guia_do_topico(s,i,topico));
                }
            }
        }
        if encontrados.is_empty() {
            return column![
                texto_negrito("Nenhum resultado encontrado", 21.),
                text(catalogo.nada_encontrado.clone()).size(13.)
                    .color(tema::COR_TEXTO_MUTED()),
            ].spacing(12).into();
        }
        return column![
            texto_negrito(format!("Resultados da busca ({})",encontrados.len()),22.),
            grade_de_cards(&encontrados,2),
        ].spacing(16).into();
    }
    let Some(secao)=catalogo.secoes.get(estado.secao) else {
        return text("Ajuda indisponível.").into();
    };
    if estado.secao==0&&estado.topico.is_none() {
        return home(catalogo);
    }
    if let Some(topico)=estado.topico.and_then(|t|secao.topicos.get(t)) {
        return column![
            text(secao.titulo.clone()).size(12.).color(tema::COR_TEXTO_MUTED()),
            ver_topico(topico, estado.copiado.as_deref()),
        ].spacing(13).into();
    }
    let guias=secao.topicos.iter().enumerate()
        .map(|(i,t)|guia_do_topico(estado.secao,i,t)).collect::<Vec<_>>();
    column![
        texto_negrito(secao.titulo.clone(),24.),
        text(secao.introducao.clone()).size(13.)
            .color(tema::COR_TEXTO_MUTED()).line_height(1.4),
        grade_de_cards(&guias,2),
    ].spacing(17).into()
}
fn paginacao(catalogo: &Catalogo,estado: &Estado)->Element<'static,Evento> {
    let previous=estado.secao>0||estado.topico.is_some();
    let is_last=catalogo.secoes.last()
        .is_some_and(|last|estado.secao+1>=catalogo.secoes.len()
            && estado.topico==last.topicos.len().checked_sub(1));
    row![
        button(text(format!("‹ {}",catalogo.anterior)).size(12.))
            .padding([8,13]).style(estilo_card)
            .on_press_maybe(previous.then_some(Evento::Anterior)),
        Space::with_width(Length::Fill),
        button(text(format!("{} ›",catalogo.proximo)).size(12.))
            .padding([8,13]).style(crate::tema::estilo_botao_primario)
            .on_press_maybe((!is_last).then_some(Evento::Proximo)),
    ].spacing(10).align_y(Alignment::Center).into()
}
impl Catalogo {
    pub fn view<'a>(&'a self,estado:&'a Estado)->Element<'a,Evento> {
        let sidebar=container(scrollable(
            column![navega(self,estado)].spacing(4)
                .padding([14,12]).width(Length::Fill).height(Length::Shrink)
        ).height(Length::Fill))
            .width(Length::Fixed(255.))
            .height(Length::Fill).style(superficie_sidebar);
        let pesquisa=row![
            icones::icone("search",18.),
            text_input(&self.busca_rotulo,&estado.busca)
                .id(iced::widget::text_input::Id::new(ID_BUSCA))
                .on_input(Evento::Buscar)
                .padding(8).size(14.)
                .style(|theme,status|{
                    let mut estilo=text_input::default(theme,status);
                    estilo.background=Background::Color(tema::COR_PAINEL());
                    estilo.border=Border::default();
                    estilo
                }).width(Length::Fill),
            container(text("Ctrl + F").size(10.5).color(tema::COR_TEXTO_MUTED()))
                .padding([5,8]).style(|_|container::Style {
                    background:Some(Background::Color(tema::COR_PAINEL_FAIXA())),
                    border:Border {color:tema::COR_BORDA(),width:1.,radius:6.0.into()},
                    ..Default::default()
                }),
        ].spacing(10).align_y(Alignment::Center);
        let busca=container(pesquisa).padding([6,12]).width(Length::Fill)
            .style(|_|barra());
        // O viewport rola e preenche a janela; seu CONTEÚDO deve ter
        // altura Shrink. Iced 0.13 entra em panic se receber Fill aqui.
        let rolagem=scrollable(container(conteudo(self,estado))
            .padding([18, 8]).width(Length::Fill).height(Length::Shrink))
            .height(Length::Fill).width(Length::Fill);
        let mut leitor=column![busca,rolagem].spacing(10)
            .padding([16,20]).width(Length::Fill).height(Length::Fill);
        if estado.busca.trim().is_empty() {
            leitor=leitor.push(paginacao(self,estado));
        }
        row![
            sidebar,
            container(leitor).width(Length::Fill)
                .height(Length::Fill)
                .style(|_|container::Style {
                    background:Some(Background::Color(tema::COR_PAINEL())),
                    ..Default::default()
                }),
        ].height(Length::Fill).width(Length::Fill).into()
    }
}

#[cfg(test)]
mod testes_visual {
    use super::*;

    #[test]
    fn icone_do_cabecalho_nao_expande_altura_no_scroll_vertical() {
        // center_y recebe a dimensão final, não apenas o alinhamento.
        // Fill neste elemento provocava overflow em cosmic-text ao pressionar F1.
        let cabecalho = icone_hero();
        let dimensao = cabecalho.as_widget().size();
        assert_eq!(dimensao.width, Length::Fixed(96.));
        assert_eq!(dimensao.height, Length::Fixed(96.));
    }
    #[test]
    fn scroll_ajuda_aceita_home_topico_busca_sem_fill_vertical() {
        // Reproduz a construção da view que ocorre imediatamente após F1.
        let catalogo=Catalogo{
            titulo:"Ajuda".into(), busca_rotulo:"Buscar".into(),
            nada_encontrado:"Nada".into(), anterior:"Anterior".into(),
            proximo:"Próximo".into(), nesta_secao:"Tópicos".into(),
            secoes:vec![Secao{
                id:"uso".into(), titulo:"Como usar".into(),
                introducao:"Manual de uso".into(),
                topicos:vec![Topico{
                    id:"comecar".into(), titulo:"Primeiros passos".into(),
                    linhas:vec![Linha::Texto("Leia a documentação".into())],
                }],
            }],
        };
        let mut estado=Estado::default();
        let _: Element<'_, Evento> = catalogo.view(&estado);
        estado.topico=Some(0);
        let _: Element<'_, Evento> = catalogo.view(&estado);
        estado.busca="primeiros".into();
        let _: Element<'_, Evento> = catalogo.view(&estado);
    }

    #[test]
    fn tela_sem_titulo_ajuda_duplicado_e_navegacao_reutilizavel() {
        let c=Catalogo{
            titulo:"Ajuda do FeatherSheet".into(), busca_rotulo:"Buscar".into(),
            nada_encontrado:"Nada encontrado".into(), anterior:"Anterior".into(),
            proximo:"Próximo".into(), nesta_secao:"Nesta seção".into(),
            secoes:vec![Secao{
                id:"uso".into(), titulo:"Como usar o FeatherSheet".into(),
                introducao:"Guia rápido da planilha".into(),
                topicos:vec![Topico{
                    id:"comecar".into(),titulo:"Primeiros passos".into(),
                    linhas:vec![Linha::Texto("Abra uma planilha".into())],
                }],
            }],
        };
        let e=Estado::default();
        let _:Element<'_,Evento>=c.view(&e);
        assert_eq!(c.pagina("comecar"),Some((0,Some(0))));
        assert!(icones::existe("file-plus"));
        assert!(icones::existe("search"));
    }
}

fn ilustra_apresentacao()->Element<'static,Evento>{
    let svg=format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 170 115\"><rect x=\"4\" y=\"8\" width=\"162\" height=\"96\" rx=\"10\" fill=\"{}\" stroke=\"#dce4e9\"/><rect x=\"12\" y=\"18\" width=\"30\" height=\"22\" rx=\"3\" fill=\"#dce4e9\"/><rect x=\"12\" y=\"46\" width=\"30\" height=\"22\" rx=\"3\" fill=\"#dce4e9\"/><rect x=\"50\" y=\"18\" width=\"106\" height=\"76\" rx=\"5\" fill=\"#fff\"/><path d=\"M65 40h60M65 53h45M65 66h55\" stroke=\"{}\" stroke-width=\"5\" stroke-linecap=\"round\"/></svg>",if tema::escuro(){"#33363b"}else{"#f1f3f4"},tema::acento_hex());
    svg::Svg::new(svg::Handle::from_memory(svg.into_bytes())).width(170).height(115).into()
}
