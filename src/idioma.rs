//! Textos do `feather-ui` no idioma do aplicativo.
//!
//! A biblioteca não tem dicionário próprio: o aplicativo instala um
//! tradutor ([`definir_tradutor`]) que procura o texto em português no
//! arquivo de idioma dele (o `interface.en-US.json` do FeatherSheet, por
//! exemplo). Sem tradutor, ou sem a chave, fica o português.
use std::sync::RwLock;

/// Recebe o texto em português e devolve a tradução, se houver.
pub type Tradutor = fn(&str) -> Option<&'static str>;

static TRADUTOR: RwLock<Option<Tradutor>> = RwLock::new(None);

/// Instala o tradutor do aplicativo (uma vez, na abertura).
pub fn definir_tradutor(tradutor: Tradutor) {
    if let Ok(mut t) = TRADUTOR.write() {
        *t = Some(tradutor);
    }
}

/// O texto no idioma do aplicativo.
pub fn t(pt: &str) -> &str {
    let tradutor = TRADUTOR.read().ok().and_then(|t| *t);
    tradutor.and_then(|f| f(pt)).unwrap_or(pt)
}

/// Como [`t`], com os `{}` do modelo trocados pelos valores, na ordem.
pub fn tf(modelo: &str, valores: &[&dyn std::fmt::Display]) -> String {
    let mut saida = String::new();
    let mut resto = t(modelo);
    for v in valores {
        let Some(i) = resto.find("{}") else { break };
        saida.push_str(&resto[..i]);
        saida.push_str(&v.to_string());
        resto = &resto[i + 2..];
    }
    saida.push_str(resto);
    saida
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn sem_tradutor_fica_o_portugues_e_o_modelo_preenche_na_ordem() {
        assert_eq!(t("Memória"), "Memória");
        assert_eq!(tf("{} de {} registros", &[&3, &10]), "3 de 10 registros");
    }
}
