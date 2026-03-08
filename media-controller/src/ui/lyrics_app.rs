use femtovg::{Canvas, Color, ImageFlags, ImageId, Paint, Path, renderer::OpenGl};

use crate::{
    Message,
    cd::DiscMetadata,
    ui::{FONT_ROBOTO_BOLD, FONT_ROBOTO_LIGHT, FONT_ROBOTO_REGULAR, Fonts, UiWindow},
};

pub struct LyricsApp {
    fonts: Fonts,
    disc_meta: Option<DiscMetadata>,
    current_track: i32,
    current_time: f32,
    cover_image: Option<ImageId>,
}

impl UiWindow for LyricsApp {
    fn draw(&mut self, canvas: &mut Canvas<OpenGl>) {
        let w = canvas.width() as f32;
        let h = canvas.height() as f32;
        canvas.clear_rect(0, 0, canvas.width(), canvas.height(), Color::black());

        let Some(meta) = &self.disc_meta else {
            return;
        };

        if self.current_track == -1 {
            return;
        }

        let track = &meta.tracks[self.current_track as usize];

        let Some(lyrics) = &track.lyrics else {
            return;
        };

        let paint = Paint::color(Color::white())
            .with_font(&[self.fonts.sans])
            .with_font_size(36.0)
            .with_anti_alias(true)
            .with_text_align(femtovg::Align::Left)
            .with_text_baseline(femtovg::Baseline::Top);

        let mut y = 50.0;
        let x = 50.0;

        if let Some(cover) = self.cover_image {
            let isize = 200.0;
            let paint = Paint::image(cover, x, y, isize, isize, 0.0, 1.0);
            let mut path = Path::new();
            path.rect(x, y, isize, isize);
            canvas.fill_path(&path, &paint);
        }

        let x = 50.0 + 200.0 + 20.0;

        let metrics = canvas
                .fill_text(x, y, format!("{} - {}", meta.title, meta.artist), &paint)
                .unwrap();
            y += metrics.height();
        let metrics = canvas
            .fill_text(x, y, format!("{}", track.title), &paint)
            .unwrap();
            y += metrics.height();

        let mut y = 100.0+ 200.0 + 20.0;
        let x = 50.0+ 200.0+20.0;

        for (time, line) in lyrics.get_timed_lines() {
            let metrics = canvas
                .fill_text(x, y, format!("{} {}", time, line), &paint)
                .unwrap();

            y += metrics.height();
        }
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

        LyricsApp {
            fonts,
            disc_meta: None,
            current_track: 4,
            current_time: 0f32,
            cover_image: None
        }
    }

    fn on_message(
        &mut self,
        message: &crate::Message,
        window: &winit::window::Window,
        canvas: &mut Canvas<OpenGl>,
        _proxy: &winit::event_loop::EventLoopProxy<crate::Message>,
    ) -> bool {
        if let Message::DiskMetadata(meta) = message {
            self.disc_meta = meta.clone();

            if let Some(cover_bytes) = meta.as_ref().and_then(|d|d.cover.as_ref()) {
                self.cover_image = Some(canvas
                    .load_image_mem(&cover_bytes, ImageFlags::empty())
                    .unwrap());
            }


            window.set_visible(true);
            window.request_redraw();
        }

        false
    }
}
