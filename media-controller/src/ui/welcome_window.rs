use femtovg::{Canvas, Color, Paint, renderer::OpenGl};

use crate::{
    Message,
    ui::{FONT_ROBOTO_BOLD, FONT_ROBOTO_LIGHT, FONT_ROBOTO_REGULAR, Fonts, UiWindow},
};

pub struct WelcomeWindow {
    fonts: Fonts,
}

impl UiWindow for WelcomeWindow {
    fn draw(&mut self, canvas: &mut Canvas<OpenGl>) {
        let w = canvas.width() as f32;
        let h = canvas.height() as f32;
        canvas.clear_rect(0, 0, canvas.width(), canvas.height(), Color::black());
        let paint = Paint::color(Color::white())
            .with_font(&[self.fonts.sans])
            .with_font_size(36.0)
            .with_anti_alias(true)
            .with_text_align(femtovg::Align::Center)
            .with_text_baseline(femtovg::Baseline::Middle);
        canvas
            .fill_text(w / 2.0, h / 2.0, "Sätt in en CD eller DVD", &paint)
            .unwrap();
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

        WelcomeWindow { fonts }
    }

    fn on_message(
        &mut self,
        message: &crate::Message,
        window: &winit::window::Window,
        _canvas: &mut Canvas<OpenGl>,
        _proxy: &winit::event_loop::EventLoopProxy<crate::Message>,
    ) -> bool {
        let Message::Disk(disk) = message else {
            return false;
        };

        match disk {
            crate::dvd_monitor::DiskReaderEvent::Inserted(..) => {
                window.set_visible(false);
            }
            crate::dvd_monitor::DiskReaderEvent::Ejected => {
                window.set_visible(true);
            }
        }

        false
    }
}
