use femtovg::{Canvas, Color, FontId, Paint, Path, renderer::OpenGl};
use winit::{event_loop::EventLoopProxy, window::Window};

use crate::{
    Message,
    ir_remote_monitor::RemoteButton,
    ui::{FONT_ROBOTO_BOLD, FONT_ROBOTO_LIGHT, FONT_ROBOTO_REGULAR, Fonts, UiWindow},
};

pub struct PromptWindow {
    fonts: Fonts,
    choice: i32,
    prompt: String,
}

impl UiWindow for PromptWindow {
    fn draw(&mut self, canvas: &mut Canvas<OpenGl>) {
        let w = canvas.width() as f32;
        let h = canvas.height() as f32;
        canvas.clear_rect(0, 0, canvas.width(), canvas.height(), Color::hex("333333"));

        let mut path = Path::new();
        path.rect(5.0, 5.0, w - 10.0, h - 10.0);
        let path_paint = Paint::color(Color::rgba(150, 150, 150, 255)).with_line_width(10.0);

        canvas.stroke_path(&path, &path_paint);

        let paint = Paint::color(Color::white())
            .with_font(&[self.fonts.sans])
            .with_font_size(28.0)
            .with_anti_alias(true)
            .with_text_align(femtovg::Align::Center)
            .with_text_baseline(femtovg::Baseline::Top);

        let x = 0.0;
        let mut y = 20.0;

        let metrics = canvas.fill_text(w / 2.0, y, &self.prompt, &paint).unwrap();

        y += metrics.height() + 10.0;

        let selected_paint = Paint::color(Color::rgba(200, 255, 200, 225))
            .with_font(&[self.fonts.bold])
            .with_font_size(28.0)
            .with_anti_alias(true)
            .with_text_align(femtovg::Align::Center)
            .with_text_baseline(femtovg::Baseline::Middle)
            .with_line_width(10.0);
        let unselected_paint = Paint::color(Color::rgba(200, 200, 200, 225))
            .with_font(&[self.fonts.sans])
            .with_font_size(28.0)
            .with_anti_alias(true)
            .with_text_align(femtovg::Align::Center)
            .with_text_baseline(femtovg::Baseline::Middle)
            .with_line_width(10.0);

        let xoff = w / 4.0;
        let bw = 90.0;
        let bh = 45.0;

        let mut path = Path::new();
        path.rect(xoff - bw / 2.0, y, bw, bh);

        canvas.stroke_path(&path, &selected_paint);
        canvas
            .fill_text(xoff, y + bh / 2.0, "Ja", &selected_paint)
            .unwrap();

        let mut path = Path::new();
        path.rect(w / 2.0 + xoff - bw / 2.0, y, bw, bh);

        canvas.stroke_path(&path, &unselected_paint);
        canvas
            .fill_text(w / 2.0 + xoff, y + bh / 2.0, "Nej", &unselected_paint)
            .unwrap();
    }

    fn on_message(&mut self, message: &Message, window: &Window, proxy: &EventLoopProxy<Message>) -> bool {
        if let Message::SetPrompt { prompt } = message {
            self.prompt = prompt.to_string();
            window.set_visible(true);
            return true;
        }

        let Message::Ir(ir) = message else {
            return false;
        };

        match ir {
            RemoteButton::Left => {
                self.choice = 0;
                return true;
            }
            RemoteButton::Right => {
                self.choice = 1;
                return true;
            }
            RemoteButton::Ok => {
                proxy
                    .send_event(Message::PromptChosen(self.choice == 0))
                    .unwrap();
                window.set_visible(false);
                return true;
            }
            _ => {}
        }

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

        PromptWindow {
            fonts,
            prompt: "".into(),
            choice: -1,
        }
    }
}
