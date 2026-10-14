//! Ajuda nativa Iced compartilhada pela suíte (UI-013).
//! O conteúdo é fornecido pelos aplicativos: nenhum JS/DOM, motor ou idioma
//! concreto entra no componente. A busca cruza seções e ignora acentos.
mod visual;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Linha {
    Texto(String),
    Marcador(String),
    Atalho { descricao: String, tecla: String },
    Exemplo(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Topico {
    pub id: String,
    pub titulo: String,
    pub linhas: Vec<Linha>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Secao {
    pub id: String,
    pub titulo: String,
    pub introducao: String,
    pub topicos: Vec<Topico>,
}
#[derive(Clone, Debug)]
pub struct Catalogo {
    pub titulo: String,
    pub busca_rotulo: String,
    pub nada_encontrado: String,
    pub anterior: String,
    pub proximo: String,
    pub nesta_secao: String,
    pub secoes: Vec<Secao>,
}
#[derive(Clone, Debug, Default)]
pub struct Estado {
    pub secao: usize,
    /// None é a página de introdução; Some(index) é um tópico.
    pub topico: Option<usize>,
    pub busca: String,
    /// Último texto copiado pelo botão Copiar; o botão mostra "Copiado" enquanto
    /// o texto continuar igual (some ao navegar ou em `Evento::LimparCopiado`).
    pub copiado: Option<String>,
}
#[derive(Clone, Debug)]
pub enum Evento {
    IrSecao(usize),
    IrTopico(usize, usize),
    Buscar(String),
    Anterior,
    Proximo,
    /// Pede ao aplicativo que ponha este texto na área de transferência
    /// (o componente não conhece a janela). `Estado::atualizar` só marca o
    /// botão como "Copiado"; quem trata o evento faz a cópia de verdade.
    Copiar(String),
    /// Tira o retorno "Copiado" (o aplicativo agenda depois de um instante).
    LimparCopiado,
}

/// Campo de busca da janela F1 (Ctrl+F focaliza este campo).
pub const ID_BUSCA: &str = "featheroffice-ajuda-busca";
pub fn focar_busca<M: Clone + 'static>() -> iced::Task<M> {
    iced::widget::text_input::focus(iced::widget::text_input::Id::new(ID_BUSCA))
}

/// Unicode pt-BR com acentos canônicos/mais comuns; não depende da locale do SO.
pub fn normalizar(s: &str) -> String {
    s.chars()
        .flat_map(char::to_lowercase)
        .filter_map(|c| match c {
            '\u{300}'..='\u{36f}' => None,
            'á' | 'à' | 'â' | 'ã' | 'ä' => Some('a'),
            'é' | 'è' | 'ê' | 'ë' => Some('e'),
            'í' | 'ì' | 'î' | 'ï' => Some('i'),
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => Some('o'),
            'ú' | 'ù' | 'û' | 'ü' => Some('u'),
            'ç' => Some('c'),
            'ñ' => Some('n'),
            _ => Some(c),
        })
        .collect()
}
fn corresponde(termo: &str, topico: &Topico) -> bool {
    normalizar(&topico.titulo).contains(termo)
        || topico.linhas.iter().any(|l| match l {
            Linha::Texto(s) | Linha::Marcador(s) | Linha::Exemplo(s) => {
                normalizar(s).contains(termo)
            }
            Linha::Atalho { descricao, tecla } => {
                normalizar(descricao).contains(termo) || normalizar(tecla).contains(termo)
            }
        })
}
impl Catalogo {
    pub fn pagina(&self, id: &str) -> Option<(usize, Option<usize>)> {
        self.secoes.iter().enumerate().find_map(|(i, s)| {
            if s.id == id {
                Some((i, None))
            } else {
                s.topicos
                    .iter()
                    .position(|t| t.id == id)
                    .map(|j| (i, Some(j)))
            }
        })
    }
    pub fn quantidade_encontrada(&self, busca: &str) -> usize {
        let termo = normalizar(busca.trim());
        if termo.is_empty() {
            return 0;
        }
        self.secoes
            .iter()
            .map(|s| s.topicos.iter().filter(|t| corresponde(&termo, t)).count())
            .sum()
    }
}

impl Estado {
    pub fn atualizar(&mut self, evento: Evento, catalogo: &Catalogo) {
        if !matches!(evento, Evento::Copiar(_) | Evento::LimparCopiado) {
            self.copiado = None;
        }
        match evento {
            Evento::Copiar(texto) => self.copiado = Some(texto),
            Evento::LimparCopiado => self.copiado = None,
            Evento::IrSecao(s) if s < catalogo.secoes.len() => {
                self.secao = s;
                self.topico = None;
                self.busca.clear();
            }
            Evento::IrTopico(s, i)
                if catalogo.secoes.get(s).is_some_and(|c| i < c.topicos.len()) =>
            {
                self.secao = s;
                self.topico = Some(i);
                self.busca.clear();
            }
            Evento::Buscar(s) => self.busca = s,
            Evento::Anterior | Evento::Proximo => {
                let paginas: Vec<_> = catalogo
                    .secoes
                    .iter()
                    .enumerate()
                    .flat_map(|(s, c)| {
                        std::iter::once((s, None))
                            .chain((0..c.topicos.len()).map(move |i| (s, Some(i))))
                    })
                    .collect();
                if let Some(pos) = paginas.iter().position(|p| *p == (self.secao, self.topico)) {
                    let destino = if matches!(evento, Evento::Anterior) {
                        pos.saturating_sub(1)
                    } else {
                        (pos + 1).min(paginas.len().saturating_sub(1))
                    };
                    if let Some(&(s, i)) = paginas.get(destino) {
                        self.secao = s;
                        self.topico = i;
                        self.busca.clear();
                    }
                }
            }
            _ => {}
        }
    }
    pub fn abrir(&mut self, catalogo: &Catalogo, secao: &str, topico: Option<&str>) {
        if let Some(s) = catalogo.secoes.iter().position(|c| c.id == secao) {
            self.secao = s;
            self.topico =
                topico.and_then(|id| catalogo.secoes[s].topicos.iter().position(|t| t.id == id));
            self.busca.clear();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn catalogo() -> Catalogo {
        Catalogo {
            titulo: "Ajuda".into(),
            busca_rotulo: "Buscar".into(),
            nada_encontrado: "Nada".into(),
            anterior: "Anterior".into(),
            proximo: "Próximo".into(),
            nesta_secao: "Nesta seção".into(),
            secoes: vec![
                Secao {
                    id: "uso".into(),
                    titulo: "Como usar".into(),
                    introducao: "".into(),
                    topicos: vec![Topico {
                        id: "grafico".into(),
                        titulo: "Gráficos".into(),
                        linhas: vec![Linha::Texto("Gráfico de média".into())],
                    }],
                },
                Secao {
                    id: "atalhos".into(),
                    titulo: "Atalhos".into(),
                    introducao: "".into(),
                    topicos: vec![Topico {
                        id: "f1".into(),
                        titulo: "Ajuda".into(),
                        linhas: vec![Linha::Atalho {
                            descricao: "Abrir".into(),
                            tecla: "F1".into(),
                        }],
                    }],
                },
            ],
        }
    }
    #[test]
    fn pesquisa_sem_acentos_e_cruza_secoes() {
        let c = catalogo();
        assert_eq!(normalizar("MÉDIA"), "media");
        assert_eq!(c.quantidade_encontrada("media"), 1);
        assert_eq!(c.quantidade_encontrada("f1"), 1);
        assert_eq!(c.quantidade_encontrada("inexistente"), 0);
    }
    #[test]
    fn navegacao_inclui_introducoes_e_nao_ultrapassa_limites() {
        let c = catalogo();
        let mut e = Estado::default();
        e.atualizar(Evento::Anterior, &c);
        assert_eq!((e.secao, e.topico), (0, None));
        e.atualizar(Evento::Proximo, &c);
        assert_eq!((e.secao, e.topico), (0, Some(0)));
        e.atualizar(Evento::Proximo, &c);
        assert_eq!((e.secao, e.topico), (1, None));
        e.atualizar(Evento::Proximo, &c);
        assert_eq!((e.secao, e.topico), (1, Some(0)));
        e.atualizar(Evento::Proximo, &c);
        assert_eq!((e.secao, e.topico), (1, Some(0)));
        e.abrir(&c, "uso", Some("grafico"));
        assert_eq!((e.secao, e.topico), (0, Some(0)));
        e.abrir(&c, "uso", Some("inexistente"));
        assert_eq!(e.topico, None);
    }
}
