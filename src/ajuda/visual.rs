//! Ajuda FeatherOffice: landing de tópicos + navegação em três grupos.
//! O modelo, a busca e os eventos continuam compartilhados entre os apps;
//! todas as cores vêm dos tokens da preferência ativa (claro/escuro/acento).
use super::*;
use crate::{icones, tema};
use iced::widget::{button, column, container, row, scrollable, svg, text, text_input, Space};
use iced::{Alignment, Background, Border, Color, Element, Font, Length, Theme};
use iced::widget::button::Status;

#[derive(Clone)]
struct Guia {
    titulo: String,
    descricao: String,
    icone: &'static str,
    destino: Evento,
}
fn texto_negrito(nome: impl Into<String>, tamanho: f32) -> Element<'static, Evento> {
    text(nome.into()).size(tamanho).font(Font {
        weight: iced::font::Weight::Semibold, ..Font::DEFAULT
    }).into()
}
fn fundo_pastel() -> Color {
    tema::COR_AZUL_FUNDO()
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
fn estilo_card(_: &Theme, status: Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(
            if matches!(status, Status::Hovered | Status::Pressed) {
                tema::COR_AZUL_FUNDO()
            } else {
                tema::COR_PAINEL()
            }
        )),
        text_color: tema::COR_TEXTO(),
        border: Border {
            color: tema::COR_BORDA(),
            width: 1.,
            radius: 11.0.into()
        },
        ..Default::default()
    }
}
fn estilo_nav(_: &Theme, status: Status, selecionado: bool) -> button::Style {
    let fundo = if selecionado {
        Some(Background::Color(tema::COR_AZUL_FUNDO()))
    } else if matches!(status, Status::Hovered | Status::Pressed) {
        Some(Background::Color(tema::COR_HOVER()))
    } else {
        None
    };
    button::Style {
        background: fundo,
        text_color: tema::COR_TEXTO(),
        border: Border { radius: 9.0.into(), ..Default::default() },
        ..Default::default()
    }
}
fn icone_hero() -> Element<'static, Evento> {
    container(crate::marca::icone_atual(86.))
        // Nunca usar center_y(Fill) dentro de scrollable vertical:
        // center_y define a altura do Container e sobrescreve height(Fixed).
        // No F1 isso gerava altura praticamente ilimitada e overflow
        // ao converter coordenadas de glifos no cosmic-text.
        .center_x(Length::Fixed(96.))
        .center_y(Length::Fixed(96.)).into()
}
fn ilustra_planilha() -> Element<'static, Evento> {
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="164" height="132" viewBox="0 0 164 132">
          <rect x="19" y="13" width="127" height="101" rx="12" fill="{fundo}" stroke="{borda}" stroke-width="1.5"/>
          <path d="M19 24Q19 13 31 13H134Q146 13 146 24V39H19Z" fill="{cor}" opacity=".85"/>
          <circle cx="33" cy="26" r="3" fill="white"/><circle cx="45" cy="26" r="3" fill="white"/>
          <circle cx="57" cy="26" r="3" fill="white"/>
          <path d="M29 51H136M29 70H136M29 89H136M53 44V103M80 44V103M107 44V103" stroke="{grade}" stroke-width="1.1"/>
          <rect x="54" y="52" width="26" height="18" rx="2" fill="{cor}" opacity=".34"/>
          <rect x="80" y="70" width="27" height="19" rx="2" fill="{cor}" opacity=".2"/>
          <rect x="111" y="92" width="46" height="29" rx="8" fill="{fundo}" stroke="{borda}"/>
          <text x="134" y="111" font-size="17" text-anchor="middle" fill="{cor}" font-weight="bold" font-family="sans-serif">fx</text>
        </svg>"##,
        cor=tema::acento_hex(),
        fundo=if tema::escuro() {"#292d32"} else {"#ffffff"},
        borda=if tema::escuro() {"#485055"} else {"#dce4e9"},
        grade=if tema::escuro() {"#52565e"} else {"#e0e5eb"}
    );
    container(svg::Svg::new(svg::Handle::from_memory(svg.into_bytes()))
        .width(Length::Fixed(164.)).height(Length::Fixed(132.)))
        .center_x(Length::Fixed(184.))
        .style(|_|container::Style {
            background: Some(Background::Color(fundo_pastel())),
            border: Border {radius: 20.0.into(), ..Default::default()},
            ..Default::default()
        }).into()
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
        "atalhos" => "keyboard", _ => "help",
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
    let icon = container(icones::icone(guia.icone,25.))
        .padding(13).style(|_|container::Style {
            background:Some(Background::Color(fundo_pastel())),
            border:Border {radius:12.0.into(),..Default::default()},
            ..Default::default()
        });
    let texto = column![
        texto_negrito(guia.titulo.clone(), 13.5),
        text(guia.descricao.clone()).size(11.5)
            .color(tema::COR_TEXTO_MUTED()).line_height(1.25),
    ].spacing(5).width(Length::Fill);
    button(row![icon,texto,text("›").size(21.)]
        .spacing(10).align_y(Alignment::Center))
        .padding(13)
        .style(estilo_card)
        .height(Length::Fixed(118.))
        .width(Length::FillPortion(1))
        .on_press(guia.destino.clone()).into()
}
fn grade_de_cards(guias: &[Guia], colunas: usize) -> Element<'static, Evento> {
    let mut grid = column![].spacing(12);
    for faixa in guias.chunks(colunas) {
        let mut linha=row![].spacing(12);
        for item in faixa {
            linha = linha.push(cartao(item));
        }
        for _ in faixa.len()..colunas {
            linha=linha.push(Space::with_width(Length::FillPortion(1)));
        }
        grid=grid.push(linha);
    }
    grid.into()
}
fn rotulo_nav(rotulo: &'static str) -> Element<'static,Evento> {
    container(text(rotulo).size(10.5).color(tema::COR_TEXTO_MUTED()))
        .padding(iced::Padding{left:15.,right:5.,top:13.,bottom:4.}).into()
}
fn item_nav(rotulo: String, icone: &str, ativo: bool, destino: Evento) -> Element<'static,Evento> {
    let conteudo=row![
        icones::icone(icone,18.),
        text(rotulo).size(12.5).width(Length::Fill),
    ].spacing(12).align_y(Alignment::Center);
    button(conteudo).width(Length::Fill)
        .padding([10,13]).style(move|tema,status|estilo_nav(tema,status,ativo))
        .on_press(destino).into()
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
    let basico=indices(&["comecar","abas","preencher","formulas","formatar","tabelas","graficos"]);
    let avancado=indices(&["dados","macros"]);
    let outros=indices(&["arquivos","config"]);
    let outros_indices=(0..primeiro.topicos.len()).filter(|i|
        !basico.contains(i)&&!avancado.contains(i)&&!outros.contains(i)
    ).collect::<Vec<_>>();
    let group=|nav: iced::widget::Column<'static,Evento>,
                   titulo: &'static str, itens: &[usize]| {
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
        ilustra_planilha(),
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
    // Guia rápido abre sua explicação correspondente; não cria arquivos sem confirmação.
    let atalhos=[
        ("file-plus","Criar planilha","comecar"),
        ("folder-open","Abrir arquivo","arquivos"),
        ("device-floppy","Salvar","arquivos"),
        ("printer","Imprimir","arquivos"),
        ("filter","Importar dados","dados"),
        ("math-function","Inserir função","formulas"),
        ("table","Criar tabela","tabelas"),
    ];
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
fn ver_topico(topico: &Topico) -> Element<'static, Evento> {
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
            Linha::Exemplo(s)=>container(text(s.clone()).size(13.).font(Font::MONOSPACE))
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
            ver_topico(topico),
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
