//! Gestão de janelas auxiliares usada por todos os programas FeatherOffice.
//! O documento e os comandos continuam sendo responsabilidade do aplicativo.
use iced::{window, Size, Task};
use std::time::Instant;

/// Uma única janela por função (Configurações, Monitor, Ajuda).
#[derive(Debug)]
pub struct Janelas<T: Copy + Eq> {
    pub principal: Option<window::Id>,
    auxiliares: Vec<(window::Id, T)>,
}
impl<T: Copy + Eq> Default for Janelas<T> {
    fn default() -> Self {
        Self { principal: None, auxiliares: Vec::new() }
    }
}
impl<T: Copy + Eq> Janelas<T> {
    pub fn tipo(&self, id: window::Id) -> Option<T> {
        self.auxiliares.iter().find(|(i, _)| *i == id).map(|(_, tipo)| *tipo)
    }
    pub fn id(&self, tipo: T) -> Option<window::Id> {
        self.auxiliares.iter().find(|(_, t)| *t == tipo).map(|(i, _)| *i)
    }
    pub fn aberta(&self, tipo: T) -> bool {
        self.id(tipo).is_some()
    }
    pub fn registrar(&mut self, id: window::Id, tipo: T) {
        self.auxiliares.retain(|(i, t)| *i != id && *t != tipo);
        self.auxiliares.push((id, tipo));
    }
    pub fn retirar(&mut self, id: window::Id) -> Option<T> {
        let pos = self.auxiliares.iter().position(|(i, _)| *i == id)?;
        Some(self.auxiliares.remove(pos).1)
    }
    pub fn fechar_tipo<M: 'static>(&mut self, tipo: T) -> Task<M> {
        match self.id(tipo) {
            Some(id) => {
                self.retirar(id);
                window::close(id)
            }
            None => Task::none(),
        }
    }
    /// Replica o comportamento do Slide: clicar no botão alterna a janela.
    /// A Task devolve o Id após a abertura, para o app mapear um evento.
    pub fn alternar(&mut self, tipo: T, tamanho: Size, barra_propria: bool) -> Task<window::Id> {
        if let Some(id) = self.id(tipo) {
            self.retirar(id);
            return window::close(id);
        }
        let (id, abrir) = window::open(crate::marca::preparar_janela(window::Settings {
            size: tamanho,
            decorations: !barra_propria,
            exit_on_close_request: false,
            ..Default::default()
        }));
        self.registrar(id, tipo);
        abrir
    }
}

/// Arrastar pelo título; dois cliques rápidos maximizam/restauram.
pub fn arrastar_ou_maximizar<M: 'static>(id: window::Id, ultimo: &mut Option<Instant>) -> Task<M> {
    let duplo = ultimo.is_some_and(|t| t.elapsed().as_millis() < 400);
    *ultimo = if duplo { None } else { Some(Instant::now()) };
    if duplo { window::toggle_maximize(id) } else { window::drag(id) }
}
#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn identifica_e_fecha_auxiliares_sem_confundir_a_principal() {
        let mut j = Janelas::<u8>::default();
        let principal = window::Id::unique();
        let config = window::Id::unique();
        j.principal = Some(principal);
        j.registrar(config, 1);
        assert_eq!(j.tipo(principal), None);
        assert_eq!(j.tipo(config), Some(1));
        assert!(j.aberta(1));
        assert_eq!(j.retirar(config), Some(1));
        assert!(!j.aberta(1));
        assert_eq!(j.principal, Some(principal));
    }
}
