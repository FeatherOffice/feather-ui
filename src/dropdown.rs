//! Contrato de interação para listas suspensas, independente dos documentos.
//! A renderização, foco real, eventos de mouse e scroll são do Iced.
//! O estado guarda SOMENTE índices/consulta, nunca IDs ou comandos de negócio.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventoDropdown {
    Focar,
    Desfocar,
    Alternar,
    Abrir,
    Fechar,
    Cima,
    Baixo,
    Inicio,
    Fim,
    Confirmar,
    Escape,
    Rolar(i32),
    Digitar(char),
    Apagar,
    LimparFiltro,
    Escolher(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RespostaDropdown {
    Nenhuma,
    Aberto,
    Fechado,
    Escolhido(usize),
}

/// Estado visual local; o valor selecionado continua no modelo de cada app.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EstadoDropdown {
    pub aberto: bool,
    pub focado: bool,
    pub filtro: String,
    /// Índice ORIGINAL da opção, mesmo quando há pesquisa/filtro.
    pub destaque: Option<usize>,
    /// Deslocamento em relação à lista filtrada, não à lista original.
    pub primeiro_visivel: usize,
}

impl EstadoDropdown {
    pub fn novo() -> Self { Self::default() }

    /// Pesquisa sem diferenciação de caixa, inclusive letras Unicode; não remove
    /// acentos, pois estes distinguem nomes reais de fontes, fórmulas e arquivos.
    pub fn indices_filtrados<T: AsRef<str>>(&self, opcoes: &[T]) -> Vec<usize> {
        if self.filtro.is_empty() {
            return (0..opcoes.len()).collect();
        }
        let consulta = self.filtro.to_lowercase();
        opcoes.iter().enumerate()
            .filter(|(_, nome)| nome.as_ref().to_lowercase().contains(&consulta))
            .map(|(indice, _)| indice)
            .collect()
    }

    fn fechar(&mut self) -> RespostaDropdown {
        self.aberto = false;
        self.destaque = None;
        self.primeiro_visivel = 0;
        self.filtro.clear();
        RespostaDropdown::Fechado
    }

    fn manter_visivel(&mut self, filtrados: &[usize], capacidade: usize) {
        if filtrados.is_empty() {
            self.destaque = None;
            self.primeiro_visivel = 0;
            return;
        }
        let capacidade = capacidade.max(1);
        let max_inicio = filtrados.len().saturating_sub(capacidade);
        self.primeiro_visivel = self.primeiro_visivel.min(max_inicio);
        if let Some(indice) = self.destaque {
            if let Some(pos) = filtrados.iter().position(|&i| i == indice) {
                if pos < self.primeiro_visivel { self.primeiro_visivel = pos; }
                if pos >= self.primeiro_visivel + capacidade {
                    self.primeiro_visivel = pos + 1 - capacidade;
                }
            } else {
                self.destaque = filtrados.first().copied();
                self.primeiro_visivel = 0;
            }
        }
    }

    /// Alimenta um evento proveniente de qualquer widget Iced. `selecionado`
    /// refere-se ao índice original e serve apenas para abrir no item atual.
    ///
    /// `linhas_visiveis` define o tamanho da janela de navegação/scroll;
    /// zero é tratado como uma linha para nunca haver faixa inválida.
    pub fn atualizar<T: AsRef<str>>(
        &mut self,
        evento: EventoDropdown,
        opcoes: &[T],
        selecionado: Option<usize>,
        pesquisavel: bool,
        linhas_visiveis: usize,
    ) -> RespostaDropdown {
        use EventoDropdown as E;
        let janela = linhas_visiveis.max(1);
        match evento {
            E::Focar => { self.focado = true; return RespostaDropdown::Nenhuma; }
            E::Desfocar => {
                self.focado = false;
                if self.aberto { return self.fechar(); }
                return RespostaDropdown::Nenhuma;
            }
            E::Fechar | E::Escape => {
                if self.aberto { return self.fechar(); }
                return RespostaDropdown::Nenhuma;
            }
            E::Alternar if self.aberto => return self.fechar(),
            E::Confirmar if self.aberto => {
                if let Some(indice) = self.destaque.filter(|&i| i < opcoes.len()) {
                    self.fechar();
                    return RespostaDropdown::Escolhido(indice);
                }
                return RespostaDropdown::Nenhuma;
            }
            E::Escolher(indice) if self.aberto && indice < opcoes.len() => {
                if self.indices_filtrados(opcoes).contains(&indice) {
                    self.fechar();
                    return RespostaDropdown::Escolhido(indice);
                }
                return RespostaDropdown::Nenhuma;
            }
            E::Escolher(_) => return RespostaDropdown::Nenhuma,
            E::Digitar(c) if pesquisavel && !c.is_control() => {
                self.focado = true;
                self.aberto = true;
                // Proteção simples contra filtros enormes no loop da UI.
                if self.filtro.chars().count() < 256 { self.filtro.push(c); }
                self.primeiro_visivel = 0;
                self.destaque = self.indices_filtrados(opcoes).first().copied();
                return RespostaDropdown::Aberto;
            }
            E::Apagar if pesquisavel => {
                self.filtro.pop();
                self.primeiro_visivel = 0;
                self.destaque = self.indices_filtrados(opcoes).first().copied();
                return RespostaDropdown::Nenhuma;
            }
            E::LimparFiltro => {
                self.filtro.clear();
                self.primeiro_visivel = 0;
                self.destaque = self.indices_filtrados(opcoes).first().copied();
                return RespostaDropdown::Nenhuma;
            }
            E::Abrir | E::Alternar | E::Confirmar if !self.aberto => {
                self.aberto = true;
                self.focado = true;
                let filtrados = self.indices_filtrados(opcoes);
                self.destaque = selecionado.filter(|s| filtrados.contains(s))
                    .or_else(|| filtrados.first().copied());
                self.manter_visivel(&filtrados, janela);
                return RespostaDropdown::Aberto;
            }
            E::Inicio | E::Fim | E::Cima | E::Baixo if !self.aberto => {
                self.aberto = true;
                self.focado = true;
                let filtrados = self.indices_filtrados(opcoes);
                self.destaque = selecionado.filter(|s| filtrados.contains(s))
                    .or_else(|| filtrados.first().copied());
                self.manter_visivel(&filtrados, janela);
                return RespostaDropdown::Aberto;
            }
            _ => {}
        }
        if !self.aberto { return RespostaDropdown::Nenhuma; }
        let filtrados = self.indices_filtrados(opcoes);
        let atual = self.destaque.and_then(|idx| filtrados.iter().position(|i| *i == idx));
        match evento {
            E::Inicio => self.destaque = filtrados.first().copied(),
            E::Fim => self.destaque = filtrados.last().copied(),
            E::Cima => self.destaque = atual
                .and_then(|i| i.checked_sub(1)).and_then(|i| filtrados.get(i))
                .or_else(|| filtrados.first()).copied(),
            E::Baixo => self.destaque = atual
                .and_then(|i| filtrados.get(i + 1))
                .or_else(|| filtrados.first()).copied(),
            E::Rolar(delta) => {
                let max_inicio = filtrados.len().saturating_sub(janela);
                self.primeiro_visivel = if delta < 0 {
                    self.primeiro_visivel.saturating_sub(delta.unsigned_abs() as usize)
                } else {
                    self.primeiro_visivel.saturating_add(delta as usize).min(max_inicio)
                };
                self.destaque = filtrados.get(self.primeiro_visivel).copied();
                return RespostaDropdown::Nenhuma;
            }
            _ => {}
        }
        self.manter_visivel(&filtrados, janela);
        RespostaDropdown::Nenhuma
    }
}

/// Esc, Enter e direção no Iced 0.13 sem obrigar cada app a repetir o mapeamento.
pub fn evento_teclado(tecla: &iced::keyboard::Key) -> Option<EventoDropdown> {
    use iced::keyboard::{key::Named, Key};
    match tecla {
        Key::Named(Named::ArrowUp) => Some(EventoDropdown::Cima),
        Key::Named(Named::ArrowDown) => Some(EventoDropdown::Baixo),
        Key::Named(Named::Home) => Some(EventoDropdown::Inicio),
        Key::Named(Named::End) => Some(EventoDropdown::Fim),
        Key::Named(Named::Enter) => Some(EventoDropdown::Confirmar),
        Key::Named(Named::Escape) => Some(EventoDropdown::Escape),
        Key::Named(Named::Backspace) => Some(EventoDropdown::Apagar),
        _ => None,
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    const OPCOES: &[&str] = &["Árvore", "Banana", "Café", "uva", "Uva verde"];

    fn ev(e: &mut EstadoDropdown, v: EventoDropdown) -> RespostaDropdown {
        e.atualizar(v, OPCOES, None, true, 2)
    }

    #[test]
    fn abre_foca_e_fecha_esc_sem_alterar_valor() {
        let mut e = EstadoDropdown::novo();
        assert_eq!(ev(&mut e, EventoDropdown::Focar), RespostaDropdown::Nenhuma);
        assert!(e.focado && !e.aberto);
        assert_eq!(ev(&mut e, EventoDropdown::Abrir), RespostaDropdown::Aberto);
        assert_eq!(ev(&mut e, EventoDropdown::Escape), RespostaDropdown::Fechado);
        assert!(e.focado && !e.aberto && e.filtro.is_empty());
        assert_eq!(ev(&mut e, EventoDropdown::Escape), RespostaDropdown::Nenhuma);
    }

    #[test]
    fn pesquisa_unicode_preserva_indice_original() {
        let mut e = EstadoDropdown::novo();
        assert_eq!(ev(&mut e, EventoDropdown::Digitar('U')), RespostaDropdown::Aberto);
        assert_eq!(e.indices_filtrados(OPCOES), vec![3, 4]);
        assert_eq!(ev(&mut e, EventoDropdown::Baixo), RespostaDropdown::Nenhuma);
        assert_eq!(e.destaque, Some(4));
        assert_eq!(ev(&mut e, EventoDropdown::Confirmar), RespostaDropdown::Escolhido(4));
    }

    #[test]
    fn acento_nao_e_perdido_na_pesquisa() {
        let mut e = EstadoDropdown::novo();
        ev(&mut e, EventoDropdown::Digitar('é'));
        assert_eq!(e.indices_filtrados(OPCOES), vec![2]);
        ev(&mut e, EventoDropdown::Apagar);
        assert_eq!(e.indices_filtrados(OPCOES).len(), OPCOES.len());
    }

    #[test]
    fn opcao_escolhida_precisa_estar_no_filtro() {
        let mut e = EstadoDropdown::novo();
        ev(&mut e, EventoDropdown::Digitar('U'));
        assert_eq!(ev(&mut e, EventoDropdown::Escolher(0)), RespostaDropdown::Nenhuma);
        assert_eq!(ev(&mut e, EventoDropdown::Escolher(3)), RespostaDropdown::Escolhido(3));
        assert!(!e.aberto);
    }

    #[test]
    fn scroll_limitado_e_destaque_permanece_visivel() {
        let mut e = EstadoDropdown::novo();
        ev(&mut e, EventoDropdown::Abrir);
        ev(&mut e, EventoDropdown::Rolar(100));
        assert_eq!(e.primeiro_visivel, 3);
        assert_eq!(e.destaque, Some(3));
        ev(&mut e, EventoDropdown::Rolar(-100));
        assert_eq!(e.primeiro_visivel, 0);
        ev(&mut e, EventoDropdown::Fim);
        assert_eq!(e.destaque, Some(4));
        assert_eq!(e.primeiro_visivel, 3);
    }

    #[test]
    fn sem_opcoes_nunca_produz_selecao() {
        let mut e = EstadoDropdown::novo();
        assert_eq!(e.atualizar(EventoDropdown::Abrir, &[] as &[&str], None, true, 0), RespostaDropdown::Aberto);
        assert_eq!(e.atualizar(EventoDropdown::Confirmar, &[] as &[&str], None, true, 0), RespostaDropdown::Nenhuma);
        assert!(e.destaque.is_none());
    }

    #[test]
    fn modo_sem_pesquisa_ignora_digitacao() {
        let mut e = EstadoDropdown::novo();
        assert_eq!(e.atualizar(EventoDropdown::Digitar('a'), OPCOES, None, false, 5), RespostaDropdown::Nenhuma);
        assert!(e.filtro.is_empty());
    }

    #[test]
    fn selecao_existente_ao_abrir_e_navegacao() {
        let mut e = EstadoDropdown::novo();
        assert_eq!(e.atualizar(EventoDropdown::Abrir, OPCOES, Some(4), true, 2), RespostaDropdown::Aberto);
        assert_eq!(e.destaque, Some(4));
        assert_eq!(e.primeiro_visivel, 3);
        ev(&mut e, EventoDropdown::Cima);
        assert_eq!(e.destaque, Some(3));
        ev(&mut e, EventoDropdown::Inicio);
        assert_eq!(e.destaque, Some(0));
    }

    #[test]
    fn perder_foco_fecha_e_limpa_consulta() {
        let mut e = EstadoDropdown::novo();
        ev(&mut e, EventoDropdown::Digitar('a'));
        assert_eq!(ev(&mut e, EventoDropdown::Desfocar), RespostaDropdown::Fechado);
        assert!(!e.aberto && !e.focado && e.filtro.is_empty());
    }

    #[test]
    fn teclas_iced_normalizadas() {
        use iced::keyboard::{key::Named, Key};
        assert_eq!(evento_teclado(&Key::Named(Named::Escape)), Some(EventoDropdown::Escape));
        assert_eq!(evento_teclado(&Key::Named(Named::ArrowDown)), Some(EventoDropdown::Baixo));
        assert_eq!(evento_teclado(&Key::Character("x".into())), None);
    }
}
