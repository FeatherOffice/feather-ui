//! Fundação da interface nativa da suíte FeatherOffice (Iced 0.13).
//! Sem dependência dos motores: pode ser usado pelos três aplicativos, quando
//! tiverem interfaces em Iced. Tokens, estado de tema e componentes vivem aqui.

pub mod atalhos;
/// Ajuda pesquisável e navegável nativa, compartilhada entre Docs/Sheet/Slide.
pub mod ajuda;
pub mod cromo;
/// Diálogo interno padrão (painel sobreposto com véu, título, fechar e rodapé).
pub mod dialogo;
pub mod componentes;
pub mod alinhamento;
pub mod ribbon;
pub mod dropdown;
pub mod configuracoes;
pub mod creditos;
pub mod icones;
pub mod marca;
pub mod monitor;
pub mod janelas;
pub mod tema;
/// Aba Desenhar: camada de tinta e controles das canetas (sem motor).
pub mod tinta;
/// Contorno das letras pela fonte do Iced (texto artístico / WordArt).
pub mod texto_contorno;
/// Pressão da caneta (XInput2 no Linux, WM_POINTER no Windows) e diagnóstico.
pub mod caneta;
pub mod acessibilidade;

pub mod seletor_cor;
pub mod campo_numero;

pub mod formatacao;

/// Painel de presets de imagem sem regras de negocio.
pub mod efeitos_imagem;
