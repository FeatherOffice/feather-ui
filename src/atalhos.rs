//! Normalização de atalhos Iced (sem conhecer os comandos dos aplicativos).
use iced::keyboard::{key::Named, Key, Modifiers};

/// Ex.: Ctrl+Shift+S, F1, F12. Modificadores isolados não geram ações.
pub fn combo_de(tecla: &Key, m: Modifiers) -> Option<String> {
    let nome = match tecla {
        Key::Character(c) => c.to_uppercase(),
        Key::Named(Named::Control | Named::Shift | Named::Alt | Named::Super | Named::AltGraph | Named::CapsLock) => return None,
        Key::Named(n) => format!("{n:?}"),
        _ => return None,
    };
    let mut combo = String::new();
    if m.control() { combo.push_str("Ctrl+"); }
    if m.alt() { combo.push_str("Alt+"); }
    if m.shift() { combo.push_str("Shift+"); }
    combo.push_str(&nome);
    Some(combo)
}

/// Apenas intenções de interface globais, nunca comandos do documento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcaoGlobal {
    Ajuda,
    Monitor,
    AlternarFaixa,
}

/// Apenas o papel da janela. Cada aplicativo mantém o seu próprio enum,
/// preferências, ciclo de vida e comandos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PainelGlobal {
    Ajuda,
    Monitor,
    Configuracoes,
}

/// Atalhos reservados à navegação da suíte. Requer correspondência exata
/// dos modificadores: Ctrl+F1 não pode abrir a Ajuda e Alt+F12 não abre Monitor.
pub fn acao_global(tecla: &Key, modificadores: Modifiers) -> Option<AcaoGlobal> {
    match tecla {
        Key::Named(Named::F1) if modificadores == Modifiers::default() => Some(AcaoGlobal::Ajuda),
        Key::Named(Named::F12) if modificadores == Modifiers::default() => Some(AcaoGlobal::Monitor),
        Key::Named(Named::F1) if modificadores == Modifiers::CTRL => Some(AcaoGlobal::AlternarFaixa),
        _ => None,
    }
}

/// Janela auxiliar: Esc fecha a janela atual; F1/F12 fecham apenas seus
/// painéis homônimos. Isso evita que a tecla seja interpretada pelo documento
/// que permanece aberto atrás da janela. Não fecha Configurações por F1/F12.
pub fn fechar_painel(painel: PainelGlobal, tecla: &Key, mods: Modifiers) -> bool {
    if matches!(tecla, Key::Named(Named::Escape)) {
        return true;
    }
    matches!(
        (painel, acao_global(tecla, mods)),
        (PainelGlobal::Ajuda, Some(AcaoGlobal::Ajuda))
            | (PainelGlobal::Monitor, Some(AcaoGlobal::Monitor))
    )
}

/// Nomes para ajuda e tooltips compartilhados; não traduz os comandos do app.
pub fn descricao_global(acao: AcaoGlobal) -> &'static str {
    match acao {
        AcaoGlobal::Ajuda => "Ajuda (F1)",
        AcaoGlobal::Monitor => "Monitor (F12)",
        AcaoGlobal::AlternarFaixa => "Recolher/expandir faixa (Ctrl+F1)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comandos_globais_precisos_sem_interferir_nas_funcoes() {
        let none = Modifiers::default();
        assert_eq!(acao_global(&Key::Named(Named::F1), none), Some(AcaoGlobal::Ajuda));
        assert_eq!(acao_global(&Key::Named(Named::F12), none), Some(AcaoGlobal::Monitor));
        assert_eq!(acao_global(&Key::Named(Named::F1), Modifiers::CTRL), Some(AcaoGlobal::AlternarFaixa));
        assert_eq!(acao_global(&Key::Named(Named::F1), Modifiers::SHIFT), None);
        assert_eq!(acao_global(&Key::Named(Named::F1), Modifiers::CTRL | Modifiers::SHIFT), None);
        assert_eq!(acao_global(&Key::Named(Named::F12), Modifiers::ALT), None);
        assert_eq!(acao_global(&Key::Character("s".into()), Modifiers::CTRL), None);
    }

    #[test]
    fn fechamento_contextual_da_janela_auxiliar() {
        let none = Modifiers::default();
        assert!(fechar_painel(PainelGlobal::Ajuda, &Key::Named(Named::F1), none));
        assert!(!fechar_painel(PainelGlobal::Ajuda, &Key::Named(Named::F12), none));
        assert!(fechar_painel(PainelGlobal::Monitor, &Key::Named(Named::F12), none));
        assert!(!fechar_painel(PainelGlobal::Configuracoes, &Key::Named(Named::F1), none));
        assert!(fechar_painel(PainelGlobal::Configuracoes, &Key::Named(Named::Escape), none));
        assert!(fechar_painel(PainelGlobal::Monitor, &Key::Named(Named::Escape), Modifiers::SHIFT));
        assert!(!fechar_painel(PainelGlobal::Monitor, &Key::Named(Named::F12), Modifiers::CTRL));
    }

    #[test]
    fn rotulos_consistentes_com_atalhos() {
        assert!(descricao_global(AcaoGlobal::Ajuda).contains("F1"));
        assert!(descricao_global(AcaoGlobal::Monitor).contains("F12"));
        assert!(descricao_global(AcaoGlobal::AlternarFaixa).contains("Ctrl+F1"));
    }

    #[test]
    fn atalhos_de_teclado() {
        assert_eq!(combo_de(&Key::Named(Named::F1), Modifiers::default()).as_deref(), Some("F1"));
        assert_eq!(combo_de(&Key::Named(Named::F12), Modifiers::default()).as_deref(), Some("F12"));
        assert_eq!(combo_de(&Key::Character("s".into()), Modifiers::CTRL | Modifiers::SHIFT).as_deref(), Some("Ctrl+Shift+S"));
        assert_eq!(combo_de(&Key::Named(Named::Shift), Modifiers::SHIFT), None);
    }
}
