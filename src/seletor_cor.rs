//! Seletor HSV + alfa compartilhado pelo FeatherOffice.
//! Nasceu no FeatherSlide; não importa mensagens nem modelos de nenhum editor.
use iced::mouse;
use iced::widget::canvas::{self, event, gradient, Canvas, Fill, Frame, Geometry, Gradient, Path, Program, Stroke};
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Point, Rectangle, Renderer, Shadow, Size, Theme, Vector};
use crate::tema::*;

#[derive(Debug, Clone)]
pub enum Evento {
    SaturacaoBrilho(f32, f32), Matiz(f32), Alfa(f32),
    TextoHex(String), Canal(usize, String), Escolhida(String), Fechar,
}

/// A cor `#rrggbb` ou `#rrggbbaa` como [r, g, b, a]; vazia ou inválida vira preto opaco.
pub fn rgba_de(hex: &str) -> [u8; 4] {
    let s = hex.trim().trim_start_matches('#');
    let par = |i: usize| s.get(i..i + 2).and_then(|p| u8::from_str_radix(p, 16).ok());
    match s.len() {
        6 | 8 => [par(0).unwrap_or(0), par(2).unwrap_or(0), par(4).unwrap_or(0), if s.len() == 8 { par(6).unwrap_or(255) } else { 255 }],
        _ => [0, 0, 0, 255],
    }
}

/// `#rrggbb` se opaca, `#rrggbbaa` se tem transparência.
pub fn hex_de(c: [u8; 4]) -> String {
    if c[3] == 255 {
        format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", c[0], c[1], c[2], c[3])
    }
}

/// Só aceita o que `rgba_de` lê sem chute: 6 ou 8 dígitos hexadecimais.
pub fn hex_valido(s: &str) -> bool {
    let s = s.trim().trim_start_matches('#');
    (s.len() == 6 || s.len() == 8) && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// h em graus (0..360), s e v em 0..1.
pub fn hsv_para_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let h = h.rem_euclid(360.0) / 60.0;
    let (c, i) = (v * s, h.floor() as i32);
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match i {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    [((r + m) * 255.0).round() as u8, ((g + m) * 255.0).round() as u8, ((b + m) * 255.0).round() as u8]
}

/// [h graus, s, v, a 0..1]; `matiz_antes` mantém o matiz quando a cor é cinza (sem matiz próprio).
pub fn rgba_para_hsva(c: [u8; 4], matiz_antes: f32) -> [f32; 4] {
    let (r, g, b) = (c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0);
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let d = mx - mn;
    let h = if d < 1e-6 {
        matiz_antes
    } else if mx == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if mx == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    [h.rem_euclid(360.0), if mx <= 0.0 { 0.0 } else { d / mx }, mx, c[3] as f32 / 255.0]
}

/// Traduz o arrasto do mouse na área em (x, y) de 0 a 1; `None` se o evento não conta.
fn arrasto(arrastando: &mut bool, ev: &canvas::Event, b: Rectangle, c: mouse::Cursor) -> Option<(f32, f32)> {
    let norm = |p: Point| (((p.x - b.x) / b.width).clamp(0.0, 1.0), ((p.y - b.y) / b.height).clamp(0.0, 1.0));
    match ev {
        canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if c.position_in(b).is_some() => {
            *arrastando = true;
            c.position().map(norm)
        }
        canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if *arrastando => c.position().map(norm),
        canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
            *arrastando = false;
            None
        }
        _ => None,
    }
}

fn degrade_horizontal(w: f32, paradas: &[(f32, Color)]) -> Fill {
    let mut l = gradient::Linear::new(Point::new(0.0, 0.0), Point::new(w, 0.0));
    for (o, c) in paradas {
        l = l.add_stop(*o, *c);
    }
    Fill::from(Gradient::Linear(l))
}

fn degrade_vertical(h: f32, paradas: &[(f32, Color)]) -> Fill {
    let mut l = gradient::Linear::new(Point::new(0.0, 0.0), Point::new(0.0, h));
    for (o, c) in paradas {
        l = l.add_stop(*o, *c);
    }
    Fill::from(Gradient::Linear(l))
}

fn marcador(frame: &mut Frame, p: Point) {
    frame.stroke(&Path::circle(p, 6.0), Stroke::default().with_color(Color::BLACK).with_width(3.0));
    frame.stroke(&Path::circle(p, 6.0), Stroke::default().with_color(Color::WHITE).with_width(1.5));
}

/// O quadrado: saturação para a direita, brilho para cima.
struct Quadrado {
    hsva: [f32; 4],
}

impl Program<Evento> for Quadrado {
    type State = bool;

    fn update(&self, estado: &mut bool, ev: canvas::Event, b: Rectangle, c: mouse::Cursor) -> (event::Status, Option<Evento>) {
        match arrasto(estado, &ev, b, c) {
            Some((x, y)) => (event::Status::Captured, Some(Evento::SaturacaoBrilho(x, 1.0 - y))),
            None => (event::Status::Ignored, None),
        }
    }

    fn draw(&self, _: &bool, renderer: &Renderer, _: &Theme, b: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let mut f = Frame::new(renderer, b.size());
        let [r, g, bl] = hsv_para_rgb(self.hsva[0], 1.0, 1.0);
        let pura = Color::from_rgb8(r, g, bl);
        f.fill_rectangle(Point::ORIGIN, b.size(), degrade_horizontal(b.width, &[(0.0, Color::WHITE), (1.0, pura)]));
        f.fill_rectangle(Point::ORIGIN, b.size(), degrade_vertical(b.height, &[(0.0, Color::TRANSPARENT), (1.0, Color::BLACK)]));
        marcador(&mut f, Point::new(self.hsva[1] * b.width, (1.0 - self.hsva[2]) * b.height));
        vec![f.into_geometry()]
    }
}

/// A barra vertical do matiz (as cores do arco-íris).
struct BarraMatiz {
    hsva: [f32; 4],
}

impl Program<Evento> for BarraMatiz {
    type State = bool;

    fn update(&self, estado: &mut bool, ev: canvas::Event, b: Rectangle, c: mouse::Cursor) -> (event::Status, Option<Evento>) {
        match arrasto(estado, &ev, b, c) {
            Some((_, y)) => (event::Status::Captured, Some(Evento::Matiz((y * 360.0).min(359.99)))),
            None => (event::Status::Ignored, None),
        }
    }

    fn draw(&self, _: &bool, renderer: &Renderer, _: &Theme, b: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let mut f = Frame::new(renderer, b.size());
        let paradas: Vec<(f32, Color)> = (0..=6)
            .map(|i| {
                let [r, g, bl] = hsv_para_rgb(i as f32 * 60.0, 1.0, 1.0);
                (i as f32 / 6.0, Color::from_rgb8(r, g, bl))
            })
            .collect();
        f.fill_rectangle(Point::ORIGIN, b.size(), degrade_vertical(b.height, &paradas));
        let y = self.hsva[0] / 360.0 * b.height;
        f.fill_rectangle(Point::new(0.0, (y - 2.0).max(0.0)), Size::new(b.width, 4.0), Color::WHITE);
        f.stroke(&Path::rectangle(Point::new(0.5, (y - 2.0).max(0.0)), Size::new(b.width - 1.0, 4.0)), Stroke::default().with_color(Color::BLACK).with_width(1.0));
        vec![f.into_geometry()]
    }
}

/// A barra horizontal de transparência, sobre um xadrez.
struct BarraAlfa {
    hsva: [f32; 4],
}

impl Program<Evento> for BarraAlfa {
    type State = bool;

    fn update(&self, estado: &mut bool, ev: canvas::Event, b: Rectangle, c: mouse::Cursor) -> (event::Status, Option<Evento>) {
        match arrasto(estado, &ev, b, c) {
            Some((x, _)) => (event::Status::Captured, Some(Evento::Alfa(x))),
            None => (event::Status::Ignored, None),
        }
    }

    fn draw(&self, _: &bool, renderer: &Renderer, _: &Theme, b: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let mut f = Frame::new(renderer, b.size());
        xadrez(&mut f, b.size());
        let [r, g, bl] = hsv_para_rgb(self.hsva[0], self.hsva[1], self.hsva[2]);
        let (c0, c1) = (Color::from_rgba8(r, g, bl, 0.0), Color::from_rgb8(r, g, bl));
        f.fill_rectangle(Point::ORIGIN, b.size(), degrade_horizontal(b.width, &[(0.0, c0), (1.0, c1)]));
        let x = self.hsva[3] * b.width;
        f.fill_rectangle(Point::new((x - 2.0).max(0.0), 0.0), Size::new(4.0, b.height), Color::WHITE);
        f.stroke(&Path::rectangle(Point::new((x - 2.0).max(0.0), 0.5), Size::new(4.0, b.height - 1.0)), Stroke::default().with_color(Color::BLACK).with_width(1.0));
        vec![f.into_geometry()]
    }
}

/// O xadrez cinza e branco que mostra o que é transparente.
fn xadrez(f: &mut Frame, tamanho: Size) {
    let lado = 6.0;
    f.fill_rectangle(Point::ORIGIN, tamanho, Color::WHITE);
    let (colunas, linhas) = ((tamanho.width / lado).ceil() as i32, (tamanho.height / lado).ceil() as i32);
    for l in 0..linhas {
        for c in 0..colunas {
            if (l + c) % 2 == 1 {
                let w = lado.min(tamanho.width - c as f32 * lado);
                let h = lado.min(tamanho.height - l as f32 * lado);
                f.fill_rectangle(Point::new(c as f32 * lado, l as f32 * lado), Size::new(w, h), Color::from_rgb8(0xcc, 0xcc, 0xcc));
            }
        }
    }
}

/// A prévia da cor atual (com xadrez por baixo se tem transparência).
struct Previa {
    cor: Color,
}

impl<M> Program<M> for Previa {
    type State = ();

    fn draw(&self, _: &(), renderer: &Renderer, _: &Theme, b: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let mut f = Frame::new(renderer, b.size());
        xadrez(&mut f, b.size());
        f.fill_rectangle(Point::ORIGIN, b.size(), self.cor);
        vec![f.into_geometry()]
    }
}


fn cor(hex: &str) -> Color {
    let [r,g,b,a] = rgba_de(hex);
    Color::from_rgba8(r,g,b,a as f32/255.0)
}
const PRONTAS: [&str; 12] = [
    "#000000", "#666666", "#cccccc", "#ffffff", "#d93025", "#f29900",
    "#fbbc04", "#34a853", "#1a73e8", "#0b2545", "#9334e6", "#e52592",
];

/// Elemento reutilizável, sem posicionamento e sem estado próprio.
/// O aplicativo posiciona sobre sua Ribbon e mapeia Evento para a mensagem local.
pub fn painel<'a>(titulo: &'static str, atual: &str, hsva: [f32;4], texto_hex: &str) -> Element<'a, Evento> {
    painel_base(titulo, atual, hsva, texto_hex, false)
}

/// Versão integrada a diálogos de formatação: mesmos controles HSV/RGBA,
/// sem botão de fechar um popover que não existe nessa apresentação.
pub fn painel_embutido<'a>(titulo: &'static str, atual: &str, hsva: [f32;4], texto_hex: &str) -> Element<'a, Evento> {
    painel_base(titulo, atual, hsva, texto_hex, true)
}

fn painel_base<'a>(titulo: &'static str, atual: &str, hsva: [f32;4], texto_hex: &str, embutido: bool) -> Element<'a, Evento> {
    const LARGURA: f32 = 264.0;
    let rgba = rgba_de(atual);
    let quadrado = Canvas::new(Quadrado { hsva })
        .width(Length::Fill).height(Length::Fixed(150.0));
    let matiz = Canvas::new(BarraMatiz { hsva })
        .width(Length::Fixed(16.0)).height(Length::Fixed(150.0));
    let alfa = Canvas::new(BarraAlfa { hsva })
        .width(Length::Fill).height(Length::Fixed(14.0));
    let previa = Canvas::new(Previa {
        cor: if atual.is_empty() { Color::TRANSPARENT } else { cor(atual) }
    }).width(Length::Fixed(34.0)).height(Length::Fixed(34.0));

    let campo = |rotulo: &'static str, i: usize| -> Element<'a, Evento> {
        column![
            text_input("", &rgba[i].to_string())
                .on_input(move |t| Evento::Canal(i, t))
                .size(12).padding([3, 4]).width(Length::Fill),
            text(rotulo).size(10).color(COR_TEXTO_MUTED()),
        ].spacing(2).align_x(Alignment::Center).width(Length::Fill).into()
    };

    let prontas = row(PRONTAS.iter().map(|hex| {
        let c = cor(hex);
        let escolhida = atual.eq_ignore_ascii_case(hex);
        Element::from(
            button(Space::new(Length::Fixed(16.0), Length::Fixed(16.0)))
                .padding(0)
                .style(move |_, _| button::Style {
                    background: Some(Background::Color(c)),
                    border: Border {
                        color: if escolhida { COR_AZUL_ACENTO() } else { COR_BORDA() },
                        width: if escolhida { 2.0 } else { 1.0 },
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
                .on_press(Evento::Escolhida(hex.to_string()))
        )
    })).spacing(4);

    let mut rodape = row![
        button(text(crate::idioma::t("Sem cor")).size(12)).padding([5, 9])
            .style(estilo_botao_pequeno(atual.is_empty()))
            .on_press(Evento::Escolhida(String::new())),
    ].align_y(Alignment::Center).spacing(8);
    if !embutido {
        rodape = rodape.push(Space::with_width(Length::Fill))
            .push(button(text(crate::idioma::t("Fechar")).size(12)).padding([5, 9])
                .style(estilo_botao_pequeno(false)).on_press(Evento::Fechar));
    }
    let conteudo = column![
        text(titulo).size(12).color(COR_TEXTO_MUTED()),
        row![quadrado, matiz].spacing(8).height(Length::Fixed(150.0)),
        alfa,
        row![previa, text(crate::idioma::t("Prévia")).size(11).color(COR_TEXTO_MUTED())]
            .spacing(10).align_y(Alignment::Center),
        row![
            column![
                text_input("#rrggbb", texto_hex).on_input(Evento::TextoHex)
                    .size(12).padding([3, 4]).width(Length::Fill),
                text("Hex").size(10).color(COR_TEXTO_MUTED()),
            ].spacing(2).align_x(Alignment::Center).width(Length::FillPortion(3)),
            campo("R", 0), campo("G", 1), campo("B", 2), campo("A", 3),
        ].spacing(6),
        prontas,
        rodape,
    ].spacing(10);

    container(conteudo).padding(12)
        .width(Length::Fixed(LARGURA))
        .style(|_| container::Style {
            background: Some(Background::Color(COR_PAINEL())),
            text_color: Some(COR_TEXTO()),
            border: Border { color: COR_BORDA(), width: 1.0, radius: 10.0.into() },
            shadow: Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.3),
                offset: Vector::new(0.0, 6.0),
                blur_radius: 18.0,
            },
        }).into()
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn seletor_embutido_mantem_eventos_hsv() {
        let _: Element<'_, Evento> = painel_embutido("Cor", "#00ff00", [120.,1.,1.,1.], "#00ff00");
    }
    #[test]
    fn cores_hsv_transparencia_e_validacao() {
        assert_eq!(hsv_para_rgb(0.,1.,1.), [255,0,0]);
        assert_eq!(hex_de([255,0,0,255]), "#ff0000");
        assert_eq!(hex_de([255,0,0,128]), "#ff000080");
        assert!(hex_valido("#ff000080"));
        assert!(!hex_valido("#ff00gg"));
        let hsv=rgba_para_hsva([255,0,0,128],0.);
        assert_eq!(hsv[0],0.);
        assert!((hsv[3]-0.502).abs()<0.002);
    }
}
