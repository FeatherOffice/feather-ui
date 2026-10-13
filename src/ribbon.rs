//! Modelos de Ribbon e menus reutilizáveis pela suíte Iced.
//! Este crate conhece mensagens visuais, jamais os comandos/documentos de cada app.
//!
//! `AbaRibbon`, `GrupoRibbon` e `ItemMenu` são descritores; as aplicações
//! podem renderizar seus controles existentes sem trocar o estado de domínio.

use iced::widget::{button, column, row, text, Space};
use iced::{Element, Length};

/// Aba com identidade do consumidor (enum próprio do Slide/Docs/Sheet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbaRibbon<I> {
    pub id: I,
    pub rotulo: String,
    pub visivel: bool,
    pub habilitada: bool,
}

impl<I> AbaRibbon<I> {
    pub fn nova(id: I, rotulo: impl Into<String>) -> Self {
        Self { id, rotulo: rotulo.into(), visivel: true, habilitada: true }
    }
    pub fn visivel(mut self, valor: bool) -> Self {
        self.visivel = valor;
        self
    }
    pub fn habilitada(mut self, valor: bool) -> Self {
        self.habilitada = valor;
        self
    }
}

/// Ação tipada, com callback/mensagem pertencente ao aplicativo.
#[derive(Debug, Clone, PartialEq)]
pub struct AcaoRibbon<M> {
    pub id: String,
    pub rotulo: String,
    pub mensagem: Option<M>,
    pub ativa: bool,
}
impl<M> AcaoRibbon<M> {
    pub fn nova(id: impl Into<String>, rotulo: impl Into<String>, mensagem: Option<M>) -> Self {
        Self { id: id.into(), rotulo: rotulo.into(), mensagem, ativa: false }
    }
    pub fn ativa(mut self, valor: bool) -> Self { self.ativa = valor; self }
}

/// Grupo lógico de comandos para ferramentas, botões e menus de uma aba.
#[derive(Debug, Clone, PartialEq)]
pub struct GrupoRibbon<M> {
    pub id: String,
    pub rotulo: String,
    pub acoes: Vec<AcaoRibbon<M>>,
}
impl<M> GrupoRibbon<M> {
    pub fn novo(id: impl Into<String>, rotulo: impl Into<String>, acoes: Vec<AcaoRibbon<M>>) -> Self {
        Self { id: id.into(), rotulo: rotulo.into(), acoes }
    }
}

/// Menu/ submenu hierárquico. `mensagem = None` desabilita a folha.
/// Um item com filhos é exclusivamente um submenu, não aciona dois eventos.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemMenu<M> {
    pub id: String,
    pub rotulo: String,
    pub habilitado: bool,
    pub mensagem: Option<M>,
    pub filhos: Vec<ItemMenu<M>>,
}
impl<M> ItemMenu<M> {
    pub fn acao(id: impl Into<String>, rotulo: impl Into<String>, mensagem: M) -> Self {
        Self { id: id.into(), rotulo: rotulo.into(), habilitado: true, mensagem: Some(mensagem), filhos: vec![] }
    }
    /// Folha informativa/separador sem callback, como opções legadas indisponíveis.
    pub fn inerte(id: impl Into<String>, rotulo: impl Into<String>) -> Self {
        Self {
            id: id.into(), rotulo: rotulo.into(), habilitado: false,
            mensagem: None, filhos: vec![],
        }
    }
    pub fn submenu(id: impl Into<String>, rotulo: impl Into<String>, filhos: Vec<Self>) -> Self {
        Self { id: id.into(), rotulo: rotulo.into(), habilitado: true, mensagem: None, filhos }
    }
    pub fn desabilitado(mut self) -> Self {
        self.habilitado = false;
        self
    }
    pub fn item_no_caminho<'a>(itens: &'a [Self], caminho: &[usize]) -> Option<&'a Self> {
        let (&primeiro, resto) = caminho.split_first()?;
        let item = itens.get(primeiro)?;
        if resto.is_empty() { Some(item) }
        else { Self::item_no_caminho(&item.filhos, resto) }
    }
    /// Nunca despacha pai de submenu, folha desabilitada ou caminho inválido.
    pub fn acionar(itens: &[Self], caminho: &[usize]) -> Option<M>
    where M: Clone {
        let mut grupo = itens;
        let mut ultimo = None;
        for &indice in caminho {
            let item = grupo.get(indice)?;
            if !item.habilitado { return None; }
            ultimo = Some(item);
            grupo = &item.filhos;
        }
        let folha = ultimo?;
        if !folha.filhos.is_empty() { return None; }
        folha.mensagem.clone()
    }
}

/// Caminho aberto do submenu, por índices, **não** por IDs de documentos.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EstadoMenuRibbon {
    pub aberto: bool,
    pub caminho: Vec<usize>,
}
impl EstadoMenuRibbon {
    pub fn fechar(&mut self) { self.aberto = false; self.caminho.clear(); }
    pub fn abrir<M>(&mut self, itens: &[ItemMenu<M>], caminho: &[usize]) -> bool {
        let mut grupo = itens;
        for &indice in caminho {
            let Some(item) = grupo.get(indice) else { return false; };
            if !item.habilitado || item.filhos.is_empty() { return false; }
            grupo = &item.filhos;
        }
        self.aberto = true;
        self.caminho = caminho.to_vec();
        true
    }
}

/// Barra lógica. O ID e a mensagem são do consumidor, sem strings de comando
/// embutidas no framework compartilhado.
#[derive(Debug, Clone, PartialEq)]
pub struct BarraRibbon<I, M> {
    pub abas: Vec<AbaRibbon<I>>,
    pub grupos: Vec<GrupoRibbon<M>>,
    pub menus: Vec<ItemMenu<M>>,
}
impl<I, M> BarraRibbon<I, M> {
    pub fn nova(abas: Vec<AbaRibbon<I>>, grupos: Vec<GrupoRibbon<M>>, menus: Vec<ItemMenu<M>>) -> Self {
        Self { abas, grupos, menus }
    }
}

/// Uma aba visual comum. Útil para Slide/Sheet com barra e layout históricos.
pub fn aba<'a, I, M>(
    item: &AbaRibbon<I>,
    selecionada: &I,
    escolher: impl Fn(I) -> M,
) -> Element<'a, M>
where I: Clone + PartialEq, M: Clone + 'a {
    aba_com_padding(item, selecionada, escolher, 14.0)
}

/// Variação para as larguras históricas de cada Ribbon; evita regressão no
/// FeatherSheet (12px laterais), mantendo os 14px de Slide/Docs.
pub fn aba_com_padding<'a, I, M>(
    item: &AbaRibbon<I>,
    selecionada: &I,
    escolher: impl Fn(I) -> M,
    padding_lateral: f32,
) -> Element<'a, M>
where I: Clone + PartialEq, M: Clone + 'a {
    if !item.visivel { return Space::with_width(0).into(); }
    let ativa = &item.id == selecionada;
    let titulo = text(item.rotulo.clone()).size(13)
        .wrapping(iced::widget::text::Wrapping::None)
        .font(iced::Font {
            weight: if ativa { iced::font::Weight::Semibold } else { iced::font::Weight::Normal },
            ..iced::Font::DEFAULT
        });
    button(titulo)
        .padding([7.0, padding_lateral])
        .style(crate::tema::estilo_aba_faixa(ativa))
        .on_press_maybe(item.habilitada.then(|| escolher(item.id.clone())))
        .into()
}

/// Renderiza abas visíveis na mesma sequência do modelo, sem ID em URL e sem
/// guardar comandos no crate. Cada botão emite a mensagem do aplicativo.
pub fn abas<'a, I, M>(
    itens: &[AbaRibbon<I>],
    selecionada: &I,
    escolher: impl Fn(I) -> M,
) -> Element<'a, M>
where I: Clone + PartialEq, M: Clone + 'a {
    let mut barra = row![].spacing(3);
    for item in itens.iter().filter(|aba| aba.visivel) {
        barra = barra.push(aba(item, selecionada, &escolher));
    }
    barra.into()
}

/// Barra de ferramentas simples: mantém callbacks, grupos e estado de seleção.
pub fn ferramentas<'a, M: Clone + 'a>(grupos: &[GrupoRibbon<M>]) -> Element<'a, M> {
    let mut barra = row![].spacing(3);
    for grupo in grupos {
        let mut botoes = row![].spacing(3);
        for acao in &grupo.acoes {
            botoes = botoes.push(
                button(text(acao.rotulo.clone()).size(11))
                    .style(crate::tema::estilo_botao_pequeno(acao.ativa))
                    .on_press_maybe(acao.mensagem.clone())
            );
        }
        barra = barra.push(
            row![
                iced::widget::container(
                    column![
                        botoes,
                        text(grupo.rotulo.clone())
                            .size(11)
                            .color(crate::tema::COR_TEXTO_MUTED()),
                    ]
                    .spacing(2)
                    .align_x(iced::Alignment::Center),
                )
                .padding([4, 12]),
                iced::widget::container(Space::with_width(1))
                    .height(Length::Fill)
                    .style(crate::tema::estilo_linha),
            ],
        );
    }
    barra.into()
}

/// Evento do menu para o aplicativo: abrir submenu ou despachar folha.
#[derive(Debug, Clone, PartialEq)]
pub enum EventoMenu<M> {
    Abrir(Vec<usize>),
    Executar(M),
    Fechar,
}

/// Painel de menu/submenu visível. Não usa IDs do modelo de documento.
/// O estado de expansão pertence ao aplicativo; o menu só retorna eventos.
pub fn menu<'a, M: Clone + 'a>(
    itens: &[ItemMenu<M>],
    estado: &EstadoMenuRibbon,
) -> Element<'a, EventoMenu<M>> {
    if !estado.aberto { return Space::with_height(0).into(); }
    let mut lista = column![].spacing(2);
    let mut nivel = itens;
    let mut pai: Vec<usize> = Vec::new();
    for profundidade in 0..=estado.caminho.len() {
        let mut coluna = column![].spacing(2);
        for (indice, item) in nivel.iter().enumerate() {
            let mut caminho = pai.clone();
            caminho.push(indice);
            let submenu = !item.filhos.is_empty();
            let msg = if !item.habilitado { None }
                else if submenu { Some(EventoMenu::Abrir(caminho)) }
                else { item.mensagem.clone().map(EventoMenu::Executar) };
            let rotulo = if submenu {
                format!("{}  ▸", item.rotulo)
            } else { item.rotulo.clone() };
            coluna = coluna.push(
                button(text(rotulo).size(12))
                    .padding([5, 9])
                    .style(crate::tema::estilo_botao_pequeno(false))
                    .on_press_maybe(msg)
            );
        }
        lista = lista.push(coluna);
        let Some(&indice) = estado.caminho.get(profundidade) else { break };
        let Some(selecionado) = nivel.get(indice) else { break };
        if !selecionado.habilitado || selecionado.filhos.is_empty() { break; }
        nivel = &selecionado.filhos;
        pai.push(indice);
    }
    lista.push(
        button(text("Fechar").size(11))
            .on_press(EventoMenu::Fechar)
    ).into()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn abas_visiveis_habilitadas_e_callbacks_tipados() {
        #[derive(Clone, PartialEq, Debug)]
        enum Secao { Inicio, Inserir, Exibir }
        let itens = [
            AbaRibbon::nova(Secao::Inicio, "Início"),
            AbaRibbon::nova(Secao::Inserir, "Inserir").habilitada(false),
            AbaRibbon::nova(Secao::Exibir, "Exibir").visivel(false),
        ];
        assert_eq!(itens.iter().filter(|a| a.visivel).count(), 2);
        assert!(!itens[1].habilitada);
        let _ = abas(&itens, &Secao::Inicio, |id| id);
        let _ = aba(&itens[0], &Secao::Inicio, |id| id);
    }

    fn catalogo() -> Vec<ItemMenu<&'static str>> {
        vec![ItemMenu::submenu("arquivo", "Arquivo", vec![
            ItemMenu::acao("salvar", "Salvar", "salvar"),
            ItemMenu::submenu("exportar", "Exportar", vec![
                ItemMenu::acao("pdf", "PDF", "pdf"),
                ItemMenu::acao("csv", "CSV", "csv").desabilitado(),
            ]),
        ]), ItemMenu::acao("copiar", "Copiar", "copiar").desabilitado()]
    }

    #[test]
    fn submenu_retornara_mensagem_somente_na_folha_habilitada() {
        let c = catalogo();
        assert_eq!(ItemMenu::acionar(&c, &[0, 0]), Some("salvar"));
        assert_eq!(ItemMenu::acionar(&c, &[0, 1, 0]), Some("pdf"));
        assert_eq!(ItemMenu::acionar(&c, &[0, 1, 1]), None);
        assert_eq!(ItemMenu::acionar(&c, &[0, 1]), None);
        assert_eq!(ItemMenu::acionar(&c, &[1]), None);
        assert_eq!(ItemMenu::acionar(&c, &[9]), None);
        assert_eq!(ItemMenu::acionar(&c, &[0, 9]), None);
        assert_eq!(ItemMenu::acionar(&c, &[]), None);
        assert_eq!(ItemMenu::item_no_caminho(&c, &[0, 1, 0]).unwrap().id, "pdf");
    }

    #[test]
    fn estado_submenu_nao_deixa_abrir_folha_ou_indice_invalido() {
        let c = catalogo();
        let mut e = EstadoMenuRibbon::default();
        assert!(e.abrir(&c, &[]));
        assert!(e.abrir(&c, &[0, 1]));
        assert_eq!(e.caminho, vec![0, 1]);
        assert!(!e.abrir(&c, &[0, 0]));
        assert!(!e.abrir(&c, &[0, 1, 1]));
        assert!(!e.abrir(&c, &[0, 8]));
        assert_eq!(e.caminho, vec![0, 1]);
        e.fechar();
        assert!(!e.aberto && e.caminho.is_empty());
        let _ = menu(&c, &e);
        e.abrir(&c, &[0]);
        let _ = menu(&c, &e);
    }

    #[test]
    fn barra_de_ferramentas_preserva_mensagens_do_consumidor() {
        let b = BarraRibbon::nova(vec![AbaRibbon::nova(0u8, "Início")], vec![
            GrupoRibbon::novo("edit", "Editar", vec![
                AcaoRibbon::nova("copy", "Copiar", Some(1u8)).ativa(true),
                AcaoRibbon::nova("paste", "Colar", None),
            ])
        ], vec![ItemMenu::acao("save", "Salvar", 3u8)]);
        assert_eq!(b.grupos[0].acoes[0].mensagem, Some(1));
        assert_eq!(b.grupos[0].acoes[1].mensagem, None);
        assert_eq!(ItemMenu::acionar(&b.menus, &[0]), Some(3));
        let _ = ferramentas(&b.grupos);
    }
}
