use femtovg::{Canvas, Color, FontId, Paint, renderer::OpenGl};
use winit::event_loop::EventLoopProxy;

use crate::{
    Message,
    ui::{FONT_ROBOTO_BOLD, FONT_ROBOTO_LIGHT, FONT_ROBOTO_REGULAR, Fonts, UiWindow},
};

pub struct DebugConsoleWindow {
    fonts: Fonts,
    messages: Vec<Message>,
}

impl UiWindow for DebugConsoleWindow {
    fn draw(&mut self, canvas: &mut Canvas<OpenGl>) {
        let w = canvas.width() as f32;
        let h = canvas.height() as f32;
        canvas.clear_rect(0, 0, canvas.width(), canvas.height(), Color::black());
        let paint = Paint::color(Color::white())
            .with_font(&[self.fonts.sans])
            .with_font_size(12.0)
            .with_anti_alias(true)
            .with_text_align(femtovg::Align::Left)
            .with_text_baseline(femtovg::Baseline::Bottom);

        let x = 0.0;
        let mut y = h;

        for m in self.messages.iter().rev() {
            let metrics = canvas.fill_text(x, y, format!("{:?}", m), &paint).unwrap();
            y -= metrics.height();
            if y < -30.0 {
                break;
            }
        }
    }

    fn on_message(&mut self, message: &Message, proxy: &EventLoopProxy<Message>) -> bool {
        self.messages.push(message.clone());
        false
    }

    fn create(canvas: &mut Canvas<OpenGl>) -> Self
    where
        Self: Sized,
    {
        let fonts = Fonts {
            sans: canvas
                .add_font_mem(FONT_ROBOTO_REGULAR)
                .expect("Cannot add font"),
            bold: canvas
                .add_font_mem(FONT_ROBOTO_BOLD)
                .expect("Cannot add font"),
            light: canvas
                .add_font_mem(FONT_ROBOTO_LIGHT)
                .expect("Cannot add font"),
        };

        DebugConsoleWindow {
            fonts,
            messages: vec![],
        }
    }
}
