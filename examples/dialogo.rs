//! Exemplo do diálogo interno padrão (`feather_ui::dialogo`).
//!
//! `cargo run --example dialogo`: um botão abre o diálogo; o x, o Cancelar,
//! o clique no véu e o Esc fecham.
use feather_ui::dialogo::{self as dlg, Dialogo};
use iced::widget::{button, column, container, text};
use iced::{keyboard, Element, Length, Subscription};

#[derive(Default)]
struct Exemplo {
    aberto: bool,
    aplicado: u32,
}

#[derive(Debug, Clone)]
enum Msg {
    Abrir,
    Fechar,
    Aplicar,
}

impl Exemplo {
    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Abrir => self.aberto = true,
            Msg::Fechar => self.aberto = false,
            Msg::Aplicar => {
                self.aplicado += 1;
                self.aberto = false;
            }
        }
    }

    fn view(&self) -> Element<'_, Msg> {
        let base = container(
            column![
                text(format!("Aplicado {} vez(es)", self.aplicado)),
                button(text("Abrir diálogo")).on_press(Msg::Abrir)
                    .style(feather_ui::tema::estilo_botao_primario),
            ]
            .spacing(12),
        )
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(feather_ui::tema::COR_FUNDO_JANELA())),
            ..Default::default()
        });
        if !self.aberto {
            return base.into();
        }
        let corpo = column![
            dlg::secao("Contagens"),
            dlg::grade_rotulo_valor(
                vec![
                    ("Células preenchidas".into(), "5".into()),
                    ("Vazias".into(), "1".into()),
                    ("Números".into(), "4".into()),
                    ("Valores distintos".into(), "4".into()),
                ],
                2,
            ),
            dlg::secao("Números"),
            dlg::linha_rotulo_valor("Média", "—"),
            dlg::aviso_vazio("Sem números na seleção: média, mínimo e máximo não se aplicam."),
        ]
        .spacing(8);
        Dialogo::new("Estatísticas da seleção", corpo)
            .icone("chart-bar")
            .subtitulo("Intervalo A1:B3 · somente leitura")
            .ao_fechar(Msg::Fechar)
            .topo(80.0)
            .secundario("Cancelar", Some(Msg::Fechar))
            .primario("Aplicar", Some(Msg::Aplicar))
            .sobre(base)
    }

    fn subscription(&self) -> Subscription<Msg> {
        keyboard::on_key_press(|tecla, _| match tecla {
            keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Msg::Fechar),
            _ => None,
        })
    }
}

fn main() -> iced::Result {
    iced::application("Diálogo interno", Exemplo::update, Exemplo::view)
        .subscription(Exemplo::subscription)
        .theme(|_| feather_ui::tema::tema_iced())
        .run()
}
