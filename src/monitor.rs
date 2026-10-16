//! Monitor F12 da suíte: Memória, Console e Métricas.
//!
//! Tudo do monitor mora aqui — leitura de memória e CPU do processo, gráfico,
//! console com busca e filtro, abas e rodapé. O aplicativo só entrega o que é
//! dele: as mensagens do console ([`Monitor::registrar`]) e as métricas do
//! documento ([`Metricas`]). Nada é gravado em disco nem sai da máquina.
use crate::tema::*;
use iced::widget::canvas::{self, Frame, Geometry, Text};
use iced::widget::{button, column, container, row, scrollable, stack, svg, text, text_input, Space};
use iced::{alignment, mouse, Alignment, Background, Border, Color, Element, Font, Length, Point,
    Rectangle, Renderer, Size, Task, Theme};
use std::{collections::VecDeque, time::Instant};

const MAX_REGISTROS: usize = 1_000;
const MAX_BYTES: usize = 512 * 1024;
const MAX_MENSAGEM: usize = 4_096;
const MAX_AMOSTRAS: usize = 80;
const ID_CONSOLE: &str = "feather-ui-monitor-console";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nivel {
    Info,
    Aviso,
    Erro,
}

impl Nivel {
    fn rotulo(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Aviso => "AVISO",
            Self::Erro => "ERRO",
        }
    }

    fn cor(self) -> Color {
        match self {
            Self::Info => COR_AZUL_ACENTO(),
            Self::Aviso => {
                if escuro() { Color::from_rgb8(245, 193, 97) } else { Color::from_rgb8(135, 85, 12) }
            }
            Self::Erro => {
                if escuro() { Color::from_rgb8(255, 145, 145) } else { Color::from_rgb8(184, 45, 55) }
            }
        }
    }

    /// O nível de uma mensagem de estado, pelas palavras dela.
    pub fn da_mensagem(msg: &str) -> Self {
        let msg = msg.to_lowercase();
        if msg.starts_with("erro")
            || msg.starts_with("error")
            || msg.starts_with("failed")
            || msg.starts_with("could not")
            || msg.contains("invalid")
            || msg.starts_with("falha")
            || msg.starts_with("não foi possível")
            || msg.contains("inválid")
            || msg.contains("interrompida")
        {
            Self::Erro
        } else if msg.contains("cancelad")
            || msg.contains("cancel")
            || msg.starts_with("select ")
            || msg.starts_with("open ")
            || msg.starts_with("enter ")
            || msg.starts_with("selecione")
            || msg.starts_with("abra ")
            || msg.starts_with("informe ")
        {
            Self::Aviso
        } else {
            Self::Info
        }
    }
}

/// O que acontece dentro da janela do monitor. O aplicativo embrulha numa
/// mensagem própria e devolve em [`Monitor::atualizar`].
#[derive(Debug, Clone)]
pub enum Evento {
    Aba(usize),
    Busca(String),
    Filtro(Option<Nivel>),
    Limpar,
    Copiar,
    Acompanhar,
    AlternarAmostras,
    Amostras(usize),
}

pub struct Registro {
    pub tempo: String,
    pub nivel: Nivel,
    pub origem: &'static str,
    pub mensagem: String,
}

/// O console local da sessão, com teto de memória.
pub struct Console {
    inicio: Instant,
    registros: VecDeque<Registro>,
    bytes: usize,
    descartados: usize,
    revisao: u64,
    busca: String,
    filtro: Option<Nivel>,
    acompanhar: bool,
}

impl Console {
    pub fn novo(aplicativo: &str) -> Self {
        let mut console = Self {
            inicio: Instant::now(),
            registros: VecDeque::new(),
            bytes: 0,
            descartados: 0,
            revisao: 0,
            busca: String::new(),
            filtro: None,
            acompanhar: true,
        };
        console.registrar(Nivel::Info, "Sistema",
            &crate::idioma::tf("{} iniciado. Console local da sessão pronto.", &[&aplicativo]));
        console
    }

    pub fn registrar(&mut self, nivel: Nivel, origem: &'static str, mensagem: &str) {
        if mensagem.is_empty() {
            return;
        }
        let mut fim = mensagem.len().min(MAX_MENSAGEM);
        while !mensagem.is_char_boundary(fim) {
            fim -= 1;
        }
        let truncada = fim < mensagem.len();
        let mut mensagem = mensagem[..fim].to_owned();
        // O truncamento usa a fronteira UTF-8 para preservar acentos.
        if truncada {
            mensagem.push_str(crate::idioma::t(" … [registro limitado]"));
        }
        let segundos = self.inicio.elapsed().as_secs();
        self.revisao = self.revisao.wrapping_add(1);
        self.bytes += mensagem.len();
        self.registros.push_back(Registro {
            tempo: format!("{:02}:{:02}:{:02}", segundos / 3600, segundos / 60 % 60, segundos % 60),
            nivel,
            origem,
            mensagem,
        });
        while self.registros.len() > MAX_REGISTROS || self.bytes > MAX_BYTES {
            if let Some(antigo) = self.registros.pop_front() {
                self.bytes -= antigo.mensagem.len();
                self.descartados += 1;
            }
        }
    }

    /// Uma saída de várias linhas (a de uma macro, por exemplo), cada linha
    /// com o nível que as palavras dela indicam.
    pub fn registrar_saida(&mut self, origem: &'static str, saida: &str) {
        // Mantém no máximo o trecho final de uma saída extensa, sem duplicá-la inteira.
        let mut inicio = saida.len().saturating_sub(MAX_BYTES / 2);
        while !saida.is_char_boundary(inicio) {
            inicio += 1;
        }
        if inicio > 0 {
            self.registrar(Nivel::Aviso, origem, crate::idioma::t("Saída extensa: exibindo o trecho final."));
        }
        for linha in saida[inicio..].lines().filter(|s| !s.is_empty()) {
            self.registrar(Nivel::da_mensagem(linha), origem, linha);
        }
    }

    /// Sobe a cada registro: quem chama compara para saber se algo entrou.
    pub fn revisao(&self) -> u64 {
        self.revisao
    }

    pub fn registros(&self) -> impl Iterator<Item = &Registro> {
        self.registros.iter()
    }

    pub fn erros(&self) -> usize {
        self.registros.iter().filter(|r| r.nivel == Nivel::Erro).count()
    }

    fn visiveis(&self) -> impl Iterator<Item = &Registro> {
        let busca = self.busca.trim().to_lowercase();
        self.registros.iter().filter(move |r| {
            self.filtro.is_none_or(|n| n == r.nivel)
                && (busca.is_empty()
                    || r.mensagem.to_lowercase().contains(&busca)
                    || r.origem.to_lowercase().contains(&busca))
        })
    }

    fn texto_visivel(&self) -> String {
        self.visiveis()
            .map(|r| format!("[{}] [{}] [{}] {}", r.tempo, r.nivel.rotulo(), r.origem, r.mensagem))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn rolar_ao_fim<M: Send + 'static>(&self) -> Task<M> {
        if self.acompanhar {
            scrollable::snap_to(scrollable::Id::new(ID_CONSOLE), scrollable::RelativeOffset::END)
        } else {
            Task::none()
        }
    }
}

/// O que o aplicativo mostra na aba Métricas. "Erros no console" e "Tempo
/// de sessão" o monitor acrescenta sozinho.
#[derive(Default)]
pub struct Metricas {
    /// Cartões do alto: (rótulo, valor, detalhe).
    pub cartoes: Vec<(String, String, String)>,
    /// Linhas de detalhe: (rótulo, valor).
    pub linhas: Vec<(String, String)>,
    /// Texto do rodapé quando há trabalho pesado em curso ("Importando…").
    pub ocupado: Option<String>,
}

/// O monitor inteiro: amostras do processo, console e estado da janela.
pub struct Monitor {
    aba: usize,
    rss_kb: u64,
    pico_kb: u64,
    cpu: f32,
    cpu_anterior: Option<(u64, u64)>,
    historico: Vec<u64>,
    amostras: usize,
    menu_amostras: bool,
    pub console: Console,
}

impl Monitor {
    pub fn novo(aplicativo: &str) -> Self {
        Self {
            aba: 0,
            rss_kb: 0,
            pico_kb: 0,
            cpu: 0.0,
            cpu_anterior: None,
            historico: Vec::new(),
            amostras: MAX_AMOSTRAS,
            menu_amostras: false,
            console: Console::novo(aplicativo),
        }
    }

    pub fn registrar(&mut self, nivel: Nivel, origem: &'static str, mensagem: &str) {
        self.console.registrar(nivel, origem, mensagem);
    }

    /// O console da sessão (para conferir o que foi registrado).
    pub fn console(&self) -> &Console {
        &self.console
    }

    /// A aba aberta: 0 Memória, 1 Console, 2 Métricas.
    pub fn aba(&self) -> usize {
        self.aba
    }

    /// RSS da última amostra, em KiB (0 se o sistema não informa).
    pub fn rss_kb(&self) -> u64 {
        self.rss_kb
    }

    pub fn amostras(&self) -> usize {
        self.amostras
    }

    pub fn menu_amostras_aberto(&self) -> bool {
        self.menu_amostras
    }

    /// Lê memória e CPU do processo. Chamar uma vez por segundo.
    pub fn amostrar(&mut self) {
        if let Some(kb) = rss_atual_kb() {
            self.rss_kb = kb;
            self.pico_kb = self.pico_kb.max(kb);
            self.historico.push(kb);
            if self.historico.len() > MAX_AMOSTRAS {
                self.historico.remove(0);
            }
        }
        if let Some((proc, total)) = cpu_atual() {
            if let Some((proc_antes, total_antes)) = self.cpu_anterior {
                self.cpu = (proc.saturating_sub(proc_antes) as f32
                    / total.saturating_sub(total_antes).max(1) as f32 * 100.0).clamp(0.0, 100.0);
            }
            self.cpu_anterior = Some((proc, total));
        }
    }

    /// Rola o console até o fim, se ele estiver aberto e acompanhando.
    pub fn acompanhar<M: Send + 'static>(&self) -> Task<M> {
        if self.aba == 1 { self.console.rolar_ao_fim() } else { Task::none() }
    }

    pub fn atualizar<M: Send + 'static>(&mut self, evento: Evento) -> Task<M> {
        match evento {
            Evento::Aba(aba) => {
                self.aba = aba.min(2);
                self.menu_amostras = false;
                return self.acompanhar();
            }
            Evento::Busca(v) => self.console.busca = v,
            Evento::Filtro(v) => self.console.filtro = v,
            Evento::Limpar => {
                self.console.registros.clear();
                self.console.bytes = 0;
                self.console.descartados = 0;
            }
            Evento::Copiar => return iced::clipboard::write(self.console.texto_visivel()),
            Evento::Acompanhar => {
                self.console.acompanhar = !self.console.acompanhar;
                return self.console.rolar_ao_fim();
            }
            Evento::AlternarAmostras => self.menu_amostras = !self.menu_amostras,
            Evento::Amostras(n) => {
                if [20, 40, 80].contains(&n) {
                    self.amostras = n;
                }
                self.menu_amostras = false;
            }
        }
        Task::none()
    }

    /// A janela inteira. `msg` embrulha os eventos na mensagem do aplicativo.
    pub fn view<'a, M: Clone + 'static>(
        &'a self,
        metricas: Metricas,
        msg: impl Fn(Evento) -> M + Clone + 'a,
    ) -> Element<'a, M> {
        let mut abas = row![].spacing(14).align_y(Alignment::Center);
        for (i, (titulo, icone)) in [
            (crate::idioma::t("Memória"), "device-desktop"),
            (crate::idioma::t("Console"), "terminal"),
            (crate::idioma::t("Métricas"), "chart-bar"),
        ].into_iter().enumerate() {
            let selecionado = self.aba == i;
            let cor = if selecionado { verde() } else { COR_TEXTO_MUTED() };
            let conteudo = row![
                crate::icones::icone(icone, 21.0).style(move |_, _| svg::Style { color: Some(cor) }),
                text(titulo).size(14),
            ].spacing(10).align_y(Alignment::Center);
            let aba = button(conteudo)
                .padding([11, 18])
                .style(move |tema, estado| {
                    let mut estilo = estilo_botao_secundario(tema, estado);
                    estilo.border.width = 0.0;
                    estilo.border.radius = 9.0.into();
                    estilo.background = selecionado.then(|| Background::Color(COR_AZUL_FUNDO()));
                    estilo.text_color = if selecionado { verde() } else { COR_TEXTO_MUTED() };
                    estilo
                })
                .on_press(msg(Evento::Aba(i)));
            let detalhe = container(Space::with_height(3))
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: selecionado.then(|| Background::Color(verde())),
                    border: Border { radius: 3.0.into(), ..Default::default() },
                    ..Default::default()
                });
            abas = abas.push(column![aba, detalhe].spacing(0));
        }
        let ocupado = metricas.ocupado.clone();
        let corpo = match self.aba {
            1 => self.view_console(msg.clone()),
            2 => self.view_metricas(metricas),
            _ => self.view_memoria(msg.clone()),
        };
        let rodape = row![
            text(ocupado.unwrap_or_else(|| crate::idioma::t("Sessão ativa").into())).size(11).color(COR_TEXTO_MUTED()),
            Space::with_width(Length::Fill),
            container(Space::new(Length::Fixed(9.0), Length::Fixed(9.0)))
                .style(|_| container::Style {
                    background: Some(Background::Color(verde())),
                    border: Border { radius: 5.0.into(), ..Default::default() },
                    ..Default::default()
                }),
            text(crate::idioma::t("Registros locais · atualização a cada segundo")).size(11).color(COR_TEXTO_MUTED()),
        ].spacing(9).align_y(Alignment::Center);
        container(column![abas, divisor(), corpo, divisor(), rodape].spacing(13))
            .padding([18, 22])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_console<'a, M: Clone + 'static>(&'a self, msg: impl Fn(Evento) -> M + Clone + 'a) -> Element<'a, M> {
        let console = &self.console;
        let mut filtros = row![].spacing(4);
        for (nome, nivel) in [
            (crate::idioma::t("Todos"), None),
            (crate::idioma::t("Informações"), Some(Nivel::Info)),
            (crate::idioma::t("Avisos"), Some(Nivel::Aviso)),
            (crate::idioma::t("Erros"), Some(Nivel::Erro)),
        ] {
            filtros = filtros.push(
                button(text(nome).size(12))
                    .padding([6, 10])
                    .style(estilo_item_menu(console.filtro == nivel, true))
                    .on_press(msg(Evento::Filtro(nivel))),
            );
        }
        let ao_buscar = msg.clone();
        let busca = text_input(crate::idioma::t("Pesquisar mensagem ou origem…"), &console.busca)
            .size(13)
            .padding([9, 12])
            .style(estilo_entrada_suspensa)
            .on_input(move |v| ao_buscar(Evento::Busca(v)));
        let visiveis: Vec<_> = console.visiveis().collect();
        let total_visivel = visiveis.len();
        let mut linhas = column![].spacing(0);
        for r in visiveis {
            linhas = linhas
                .push(
                    container(
                        row![
                            text(&r.tempo).font(Font::MONOSPACE).size(11).color(COR_TEXTO_MUTED()).width(76),
                            text(r.nivel.rotulo()).size(10).color(r.nivel.cor()).width(46),
                            text(r.origem).size(11).color(COR_TEXTO_MUTED()).width(72),
                            text(&r.mensagem).font(Font::MONOSPACE).size(12).width(Length::Fill),
                        ].spacing(10),
                    )
                    .padding([9, 12])
                    .width(Length::Fill),
                )
                .push(divisor());
        }
        if total_visivel == 0 {
            linhas = linhas.push(
                container(
                    column![
                        text(crate::idioma::t("Nenhum registro para exibir")).size(17),
                        text(if console.registros.is_empty() {
                            crate::idioma::t("Novas operações, erros e saídas de macros aparecerão aqui.")
                        } else {
                            crate::idioma::t("Altere a busca ou o filtro para ver outros registros.")
                        })
                        .size(13)
                        .color(COR_TEXTO_MUTED()),
                    ].spacing(8),
                )
                .padding(28),
            );
        }
        let lista = container(scrollable(linhas).id(scrollable::Id::new(ID_CONSOLE)).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(painel);
        let resumo = format!(
            "{}{}",
            crate::idioma::tf("{} de {} registros", &[&total_visivel, &console.registros.len()]),
            if console.descartados > 0 {
                crate::idioma::tf(" · {} antigos descartados", &[&console.descartados])
            } else {
                String::new()
            }
        );
        column![
            busca,
            row![
                filtros,
                Space::with_width(Length::Fill),
                acao(crate::idioma::t("Copiar"), msg(Evento::Copiar)),
                acao(crate::idioma::t("Limpar"), msg(Evento::Limpar)),
            ].spacing(6).align_y(Alignment::Center),
            lista,
            row![
                text(resumo).size(11).color(COR_TEXTO_MUTED()),
                Space::with_width(Length::Fill),
                acao(
                    if console.acompanhar { crate::idioma::t("Acompanhar: ligado") } else { crate::idioma::t("Acompanhar: desligado") },
                    msg(Evento::Acompanhar),
                ),
            ].spacing(8).align_y(Alignment::Center),
        ]
        .spacing(10)
        .height(Length::Fill)
        .into()
    }

    fn view_memoria<'a, M: Clone + 'static>(&'a self, msg: impl Fn(Evento) -> M + Clone + 'a) -> Element<'a, M> {
        let mib = |kb: u64| if kb > 0 { format!("{:.1} MiB", kb as f64 / 1024.) } else { "—".into() };
        let porcentagem_rss = percentual_rss(self.rss_kb, ram_total_kb());
        let porcentagem_cpu = self.cpu_anterior.map(|_| self.cpu);
        let cartoes = row![
            cartao_destaque(crate::idioma::t("Memória em uso"), mib(self.rss_kb), crate::idioma::t("RAM residente do processo"),
                "device-desktop", verde(), Some((porcentagem_rss, "#2ab66f"))),
            cartao_destaque(crate::idioma::t("Pico observado"), mib(self.pico_kb), crate::idioma::t("Maior amostra desta sessão"),
                "chart-line", azul(), None),
            cartao_destaque("CPU",
                if self.cpu_anterior.is_some() { format!("{:.1}%", self.cpu) } else { "—".into() },
                crate::idioma::t("Fração da capacidade total"), "settings", roxo(), Some((porcentagem_cpu, "#8051d9"))),
        ].spacing(14);

        let titulo = row![
            crate::icones::icone("chart-bar", 20.0).style(|_, _| svg::Style { color: Some(verde()) }),
            text(crate::idioma::t("Histórico de memória")).size(16),
            Space::with_width(Length::Fill),
            button(row![
                text(crate::idioma::tf("Últimas {} amostras", &[&self.amostras])).size(11),
                crate::icones::icone("chevron-down", 14.0)
                    .style(|_, _| svg::Style { color: Some(COR_TEXTO_MUTED()) }),
            ].spacing(10).align_y(Alignment::Center))
                .padding([8, 11])
                .style(estilo_botao_secundario)
                .on_press(msg(Evento::AlternarAmostras)),
        ].spacing(11).align_y(Alignment::Center);
        let grafico = canvas::Canvas::new(GraficoMemoria { amostras: &self.historico, janela: self.amostras })
            .width(Length::Fill)
            .height(Length::Fill);
        let grafico_base = container(column![
            titulo,
            grafico,
            text(if self.historico.is_empty() {
                crate::idioma::t("Aguardando uma amostra disponível neste sistema…")
            } else {
                crate::idioma::t("Cada barra representa uma amostra de memória do processo (RSS).")
            }).size(11).color(COR_TEXTO_MUTED()),
        ].spacing(12))
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(painel);
        let grafico: Element<'a, M> = if self.menu_amostras {
            let mut opcoes = column![].spacing(2).padding(5);
            for n in [20, 40, 80] {
                opcoes = opcoes.push(
                    button(row![
                        text(crate::idioma::tf("Últimas {} amostras", &[&n])).size(12),
                        Space::with_width(Length::Fill),
                        if self.amostras == n {
                            crate::icones::icone("circle-check", 15.0)
                                .style(|_, _| svg::Style { color: Some(verde()) })
                        } else {
                            crate::icones::icone("chevron-down", 15.0)
                                .style(|_, _| svg::Style { color: Some(COR_TEXTO_MUTED()) })
                        },
                    ].spacing(8).align_y(Alignment::Center))
                        .width(Length::Fill)
                        .padding([8, 10])
                        .style(estilo_botao_secundario)
                        .on_press(msg(Evento::Amostras(n))),
                );
            }
            let menu = container(opcoes).width(Length::Fixed(211.0)).style(painel);
            stack![
                grafico_base,
                column![
                    Space::with_height(Length::Fixed(57.0)),
                    row![Space::with_width(Length::Fill), menu],
                ]
            ].width(Length::Fill).height(Length::Fill).into()
        } else {
            grafico_base.into()
        };
        // Sem rolagem: o histórico ocupa o que sobra da janela abaixo dos
        // cartões, e encolhe com ela.
        column![cartoes, grafico, banner_informativo()].spacing(12)
            .height(Length::Fill)
            .into()
    }

    fn view_metricas<'a, M: 'static>(&'a self, metricas: Metricas) -> Element<'a, M> {
        let mut cartoes = row![].spacing(12);
        for (rotulo, valor, detalhe) in metricas.cartoes {
            cartoes = cartoes.push(cartao(rotulo, valor, detalhe));
        }
        cartoes = cartoes.push(cartao(crate::idioma::t("Erros no console").into(), self.console.erros().to_string(),
            crate::idioma::t("No histórico retido").into()));
        let mut corpo = column![cartoes].spacing(12);
        let sessao = self.console.inicio.elapsed().as_secs();
        let linhas = metricas.linhas.into_iter()
            .chain([(crate::idioma::t("Tempo de sessão").into(), crate::idioma::tf("{} min {} s", &[&(sessao / 60), &(sessao % 60)]))]);
        for (rotulo, valor) in linhas {
            corpo = corpo.push(
                container(
                    row![
                        text(rotulo).size(13).color(COR_TEXTO_MUTED()).width(160),
                        text(valor).size(13).width(Length::Fill),
                    ].spacing(12),
                )
                .padding(16)
                .width(Length::Fill)
                .style(painel),
            );
        }
        scrollable(corpo).height(Length::Fill).into()
    }
}

fn painel(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COR_PAINEL())),
        border: Border { color: COR_BORDA_SUAVE(), width: 1., radius: 14.0.into() },
        text_color: Some(COR_TEXTO()),
        ..Default::default()
    }
}

fn divisor<'a, M: 'static>() -> Element<'a, M> {
    container(Space::with_height(1)).width(Length::Fill).style(estilo_linha).into()
}

fn cartao<'a, M: 'static>(rotulo: String, valor: String, detalhe: String) -> Element<'a, M> {
    container(
        column![
            text(rotulo).size(12).color(COR_TEXTO_MUTED()),
            text(valor).size(25),
            text(detalhe).size(11).color(COR_TEXTO_MUTED()),
        ].spacing(7),
    )
    .padding(18)
    .width(Length::Fill)
    .style(painel)
    .into()
}

fn verde() -> Color {
    if escuro() { Color::from_rgb8(92, 214, 145) } else { Color::from_rgb8(21, 137, 67) }
}

fn azul() -> Color {
    if escuro() { Color::from_rgb8(123, 194, 246) } else { Color::from_rgb8(19, 135, 124) }
}

fn roxo() -> Color {
    if escuro() { Color::from_rgb8(189, 155, 255) } else { Color::from_rgb8(127, 66, 210) }
}

/// RSS do processo em KiB (Linux: `/proc/self/status`).
fn rss_atual_kb() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status").ok()?
        .lines().find(|l| l.starts_with("VmRSS:"))?
        .split_whitespace().nth(1)?.parse().ok()
}

/// (tempo de CPU do processo, tempo total do sistema), em ticks.
fn cpu_atual() -> Option<(u64, u64)> {
    let proc = std::fs::read_to_string("/proc/self/stat").ok()?;
    let campos: Vec<&str> = proc.rsplit_once(')')?.1.split_whitespace().collect();
    let processo = campos.get(11)?.parse::<u64>().ok()? + campos.get(12)?.parse::<u64>().ok()?;
    let total = std::fs::read_to_string("/proc/stat").ok()?.lines().next()?.split_whitespace()
        .skip(1).filter_map(|v| v.parse::<u64>().ok()).sum();
    Some((processo, total))
}

/// Total físico Linux lido uma vez, para NÃO inventar percentual RSS.
/// Sem fonte nativa disponível (Windows), o medidor exibe '—'.
fn ram_total_kb() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        static TOTAL: std::sync::OnceLock<Option<u64>> = std::sync::OnceLock::new();
        *TOTAL.get_or_init(|| {
            std::fs::read_to_string("/proc/meminfo").ok()?
                .lines().find_map(|line| {
                    line.strip_prefix("MemTotal:")?.split_whitespace().next()?.parse::<u64>().ok()
                }).filter(|&n| n > 0)
        })
    }
    #[cfg(not(target_os = "linux"))]
    { None }
}

fn percentual_rss(rss_kb: u64, total_kb: Option<u64>) -> Option<f32> {
    let total = total_kb.filter(|&n| n > 0)?;
    (rss_kb > 0).then_some((rss_kb as f64 / total as f64 * 100.0).clamp(0.0, 100.0) as f32)
}

/// SVG circular com um único arco contínuo e círculo de fundo.
/// O atributo pathLength não é aplicado de modo uniforme a dasharrays em
/// círculo pelo renderizador Iced/resvg, criando pontos verdes/roxos soltos.
/// Evitamos tracejados e calculamos um arco geométrico com o ângulo real.
fn rosca_svg(valor: Option<f32>, hex: &str) -> String {
    const CX: f32 = 50.0;
    const CY: f32 = 50.0;
    const RAIO: f32 = 38.0;
    let fundo = if escuro() { "#454a52" } else { "#e6ebef" };
    let base = format!(
        r##"<circle cx="{CX}" cy="{CY}" r="{RAIO}" fill="none" stroke="{fundo}" stroke-width="9"/>"##
    );
    let progresso = match valor.filter(|v| v.is_finite()).map(|v| v.clamp(0.0, 100.0)) {
        None => String::new(),
        Some(v) if v <= 0.0 => String::new(),
        Some(v) if v >= 100.0 => format!(
            r##"<circle cx="{CX}" cy="{CY}" r="{RAIO}" fill="none" stroke="{hex}" stroke-width="9"/>"##
        ),
        Some(v) => {
            // Inicia em 12 horas e cresce no sentido horário.
            let angulo = (-90.0 + 360.0 * v / 100.0).to_radians();
            let fim_x = CX + RAIO * angulo.cos();
            let fim_y = CY + RAIO * angulo.sin();
            let arco_grande = if v > 50.0 { 1 } else { 0 };
            // Em percentuais pequenos, tampas arredondadas ampliam a fatia.
            let ponta = if v < 4.0 { "butt" } else { "round" };
            format!(
                r##"<path d="M 50 12 A 38 38 0 {arco_grande} 1 {fim_x:.3} {fim_y:.3}" fill="none" stroke="{hex}" stroke-width="9" stroke-linecap="{ponta}"/>"##
            )
        }
    };
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="100" height="100">{base}{progresso}</svg>"##
    )
}

fn rosca<'a, M: 'static>(valor: Option<f32>, hex: &'static str) -> Element<'a, M> {
    let rotulo = valor.map(|v| format!("{v:.0}%")).unwrap_or_else(|| "—".into());
    let desenho = svg::Svg::new(svg::Handle::from_memory(rosca_svg(valor, hex).into_bytes()))
        .width(Length::Fixed(90.0))
        .height(Length::Fixed(90.0));
    stack![
        desenho,
        container(text(rotulo).size(15).color(COR_TEXTO()))
            .width(90.0)
            .height(90.0)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    ].width(Length::Fixed(90.0)).height(Length::Fixed(90.0)).into()
}

fn cartao_destaque<'a, M: 'static>(
    titulo: &'static str,
    valor: String,
    detalhe: &'static str,
    icone: &'static str,
    cor: Color,
    medidor: Option<(Option<f32>, &'static str)>,
) -> Element<'a, M> {
    let fundo_icone = if escuro() { Color::from_rgb8(50, 60, 64) } else { Color::from_rgb8(239, 248, 245) };
    let icone = crate::icones::icone(icone, 25.0).style(move |_, _| svg::Style { color: Some(cor) });
    let selo = container(icone).padding(14).style(move |_| container::Style {
        background: Some(Background::Color(fundo_icone)),
        border: Border { radius: 34.0.into(), ..Default::default() },
        ..Default::default()
    });
    let conteudo = column![
        text(titulo).size(13).color(COR_TEXTO_MUTED()),
        text(valor).size(27).color(COR_TEXTO()),
        text(detalhe).size(11).color(COR_TEXTO_MUTED()),
    ].spacing(8).width(Length::Fill);
    let mut linha = row![selo, conteudo].spacing(16).align_y(Alignment::Center);
    if let Some((pct, cor)) = medidor {
        linha = linha.push(rosca(pct, cor));
    }
    container(linha)
        .padding([14, 16])
        .width(Length::Fill)
        .height(Length::Fixed(118.0))
        .style(painel)
        .into()
}

fn banner_informativo<'a, M: 'static>() -> Element<'a, M> {
    let icone = crate::icones::icone("info-circle", 21.0).style(|_, _| svg::Style { color: Some(verde()) });
    container(row![
        icone,
        text(crate::idioma::t("O monitor acompanha a memória do aplicativo inteiro, incluindo a interface e os documentos abertos."))
            .size(12)
            .color(COR_TEXTO_MUTED()),
    ].spacing(13).align_y(Alignment::Center))
        .padding(16)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(COR_PAINEL_FAIXA())),
            border: Border { color: COR_BORDA_SUAVE(), width: 1.0, radius: 12.0.into() },
            ..Default::default()
        })
        .into()
}

fn acao<'a, M: Clone + 'static>(rotulo: &'static str, mensagem: M) -> Element<'a, M> {
    button(text(rotulo).size(12))
        .padding([7, 12])
        .style(estilo_botao_secundario)
        .on_press(mensagem)
        .into()
}

/// Calcula teto em MiB com quatro intervalos redondos (1/2/5 × 10^n).
fn escala_mib(maior_kb: u64) -> f64 {
    let alvo = ((maior_kb as f64 / 1024.0) * 1.18).max(1.0);
    let minimo = alvo / 4.0;
    let magnitude = 10_f64.powf(minimo.log10().floor());
    let passo = [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|n| n * magnitude)
        .find(|v| *v >= minimo)
        .unwrap_or(10.0 * magnitude);
    passo * 4.0
}

/// Gráfico do histórico RSS (MiB) desenhado no Canvas, sem widgets por amostra.
struct GraficoMemoria<'a> {
    amostras: &'a [u64],
    janela: usize,
}

impl<M> canvas::Program<M> for GraficoMemoria<'_> {
    type State = ();

    fn draw(&self, _state: &(), renderer: &Renderer, _theme: &Theme, bounds: Rectangle,
        _cursor: mouse::Cursor) -> Vec<Geometry> {
        let mut f = Frame::new(renderer, bounds.size());
        if bounds.width < 120.0 || bounds.height < 90.0 {
            return vec![f.into_geometry()];
        }
        let esquerda = 75.0;
        let direita = bounds.width - 8.0;
        let topo = 9.0;
        let base = bounds.height - 21.0;
        let largura = (direita - esquerda).max(1.0);
        let altura = (base - topo).max(1.0);
        let mut serie = self.amostras.iter().rev().take(self.janela.max(1)).copied().collect::<Vec<_>>();
        serie.reverse();
        let maior_kb = serie.iter().copied().max().unwrap_or(0);
        let teto = escala_mib(maior_kb);
        let grade = if escuro() { Color::from_rgb8(79, 84, 94) } else { Color::from_rgb8(227, 233, 239) };
        let rotulo = COR_TEXTO_MUTED();
        for marca in 0..=4 {
            let mib = teto * marca as f64 / 4.0;
            let y = base - altura * marca as f32 / 4.0;
            f.fill_rectangle(Point::new(esquerda, y), Size::new(largura, 1.0), grade);
            f.fill_text(Text {
                content: if mib.fract() == 0.0 { format!("{mib:.0} MiB") } else { format!("{mib:.1} MiB") },
                position: Point::new(esquerda - 11.0, y),
                size: iced::Pixels(11.0),
                color: rotulo,
                horizontal_alignment: alignment::Horizontal::Right,
                vertical_alignment: alignment::Vertical::Center,
                ..Text::default()
            });
        }
        let janelas = self.janela.max(1) as f32;
        let passo = largura / janelas;
        let separacao = if passo > 4.0 { 2.0 } else { 0.5 };
        let barra_largura = (passo - separacao).max(0.5);
        let cor = if escuro() { Color::from_rgb8(72, 193, 118) } else { Color::from_rgb8(38, 161, 87) };
        for (i, &kb) in serie.iter().enumerate() {
            // Amostras ainda não coletadas ficam sem barra, à esquerda.
            let pos = self.janela.max(serie.len()) - serie.len() + i;
            let x = esquerda + pos as f32 * passo + separacao * 0.5;
            let h = ((kb as f64 / 1024.0 / teto) as f32 * altura).clamp(0.0, altura);
            if h > 0.0 {
                f.fill_rectangle(Point::new(x, base - h), Size::new(barra_largura, h), cor);
            }
        }
        vec![f.into_geometry()]
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    enum Teste {
        Monitor(Evento),
    }

    #[test]
    fn as_tres_abas_desenham() {
        let mut m = Monitor::novo("Teste");
        m.amostrar();
        for aba in 0..3 {
            let _: Task<Teste> = m.atualizar(Evento::Aba(aba));
            let _ = m.view(Metricas::default(), Teste::Monitor);
        }
    }

    #[test]
    fn medidores_usam_percentuais_reais_e_toleram_medida_indisponivel() {
        assert_eq!(percentual_rss(2_048, Some(102_400)), Some(2.0));
        assert_eq!(percentual_rss(2_048, Some(0)), None);
        assert_eq!(percentual_rss(0, Some(102_400)), None);
        let svg = rosca_svg(Some(25.0), "#18a64f");
        assert!(svg.contains("M 50 12 A 38 38 0 0 1 88.000 50.000"));
        assert!(!svg.contains("stroke-dasharray"));
        assert!(!svg.contains("pathLength"));
        let sem_medida = rosca_svg(None, "#18a64f");
        assert_eq!(sem_medida.matches("<circle").count(), 1);
        assert!(!sem_medida.contains("<path"));
        assert_eq!(rosca_svg(Some(0.0), "#18a64f").matches("<circle").count(), 1);
        let cheia = rosca_svg(Some(100.0), "#18a64f");
        assert_eq!(cheia.matches("<circle").count(), 2);
        assert!(!cheia.contains("<path"));
    }

    #[test]
    fn rosca_cpu_baixa_nao_repete_pontos_ao_redor_do_anel() {
        let um_porcento = rosca_svg(Some(1.2), "#8051d9");
        assert_eq!(um_porcento.matches("<circle").count(), 1);
        assert_eq!(um_porcento.matches("<path").count(), 1);
        assert!(um_porcento.contains("stroke-linecap=\"butt\""));
        let setenta_e_cinco = rosca_svg(Some(75.0), "#8051d9");
        assert!(setenta_e_cinco.contains("M 50 12 A 38 38 0 1 1 12.000 50.000"));
        for valor in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let svg = rosca_svg(Some(valor), "#8051d9");
            assert_eq!(svg.matches("<circle").count(), 1);
            assert!(!svg.contains("NaN"));
        }
        assert_eq!(rosca_svg(Some(-9.0), "#8051d9").matches("<circle").count(), 1);
        assert_eq!(rosca_svg(Some(109.0), "#8051d9").matches("<circle").count(), 2);
    }

    #[test]
    fn periodo_20_40_80_e_escala_em_mib() {
        let mut m = Monitor::novo("Teste");
        for n in [20, 40, 80] {
            let _: Task<Teste> = m.atualizar(Evento::AlternarAmostras);
            assert!(m.menu_amostras_aberto());
            let _: Task<Teste> = m.atualizar(Evento::Amostras(n));
            assert_eq!(m.amostras(), n);
            assert!(!m.menu_amostras_aberto());
        }
        let _: Task<Teste> = m.atualizar(Evento::Amostras(1000));
        assert_eq!(m.amostras(), 80);
        assert_eq!(escala_mib(121 * 1024), 200.0);
        assert_eq!(escala_mib(0), 2.0);
    }

    #[test]
    fn usa_so_icones_do_catalogo() {
        for nome in ["device-desktop", "terminal", "chart-bar", "chart-line",
            "settings", "info-circle", "chevron-down", "circle-check"] {
            assert!(crate::icones::existe(nome), "Sem SVG: {nome}");
        }
    }

    #[test]
    fn console_limita_memoria_preserva_utf8_e_filtra_copia() {
        let mut c = Console::novo("Teste");
        for _ in 0..2_000 {
            c.registrar(Nivel::Info, "Teste", &"á".repeat(4_000));
        }
        assert!(c.bytes <= MAX_BYTES);
        assert!(c.registros.len() <= MAX_REGISTROS);
        assert!(c.descartados > 0);
        c.registrar(Nivel::Erro, "Arquivo", "Falha ao abrir orçamento.xlsx");
        c.filtro = Some(Nivel::Erro);
        c.busca = "ORÇAMENTO".into();
        assert_eq!(c.visiveis().count(), 1);
        assert!(c.texto_visivel().contains("[ERRO] [Arquivo]"));
        assert!(!c.texto_visivel().contains("[Teste]"));
        c.registrar_saida("Macros", "primeira linha\nErro: execução falhou");
        assert_eq!(c.registros.back().unwrap().nivel, Nivel::Erro);
    }
}
