use femtovg::{Canvas, Color, FontId, Paint, renderer::OpenGl};

use crate::ui::{FONT_ROBOTO_BOLD, FONT_ROBOTO_LIGHT, FONT_ROBOTO_REGULAR, Fonts, UiWindow};

pub struct WelcomeWindow {
    fonts: Fonts,
}



impl UiWindow for WelcomeWindow {
    fn draw(&mut self, canvas: &mut Canvas<OpenGl>) {
        canvas.clear_rect(0, 0, canvas.width(), canvas.height(), Color::black());
        let paint = Paint::color(Color::white())
        .with_font(&[self.fonts.sans])
        .with_font_size(28.0);
        canvas.fill_text(20.0, 20.0, "hello world!", &paint).unwrap();
    }
    
    fn create(canvas: &mut Canvas<OpenGl>) -> Self where Self: Sized {
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
}