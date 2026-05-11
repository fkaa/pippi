use femtovg::{Align, Baseline, Canvas, Color, ImageFlags, ImageId, Paint, Path, renderer::OpenGl};

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
    current_duration: f32,
    cover_image: Option<ImageId>,
}

impl UiWindow for LyricsApp {
    fn draw(&mut self, canvas: &mut Canvas<OpenGl>) {
        let w = canvas.width() as f32;
        let h = canvas.height() as f32;
        canvas.clear_rect(0, 0, canvas.width(), canvas.height(), Color::black());

        let Some(meta) = &self.disc_meta else {
            println!("early return draw");
            return;
        };

        if self.current_track == -1 {
            println!("early return track");
            return;
        }

        let track = &meta.tracks[self.current_track as usize];

        let paint = Paint::color(Color::white())
            .with_font(&[self.fonts.sans])
            .with_font_size(36.0)
            .with_anti_alias(true)
            .with_text_align(femtovg::Align::Left)
            .with_text_baseline(femtovg::Baseline::Top);

        let mut y = 50.0;
        let x = 50.0;

        let isize = 300.0;

        if let Some(cover) = self.cover_image {
            let paint = Paint::image(cover, x, y, isize, isize, 0.0, 1.0);
            let mut path = Path::new();
            path.rect(x, y, isize, isize);
            canvas.fill_path(&path, &paint);
        }

        y += isize + 20.0;
        let x = 50.0;

        let metrics = canvas
            .fill_text(x, y, format!("{}", meta.title), &paint)
            .unwrap();
        y += metrics.height();
        let metrics = canvas
            .fill_text(
                x,
                y,
                format!("{}", meta.artist),
                &paint.clone().with_color(Color::hex("#aaaaaa")),
            )
            .unwrap();
        y += metrics.height();

        for i in 0..meta.tracks.len() {
            let t = &meta.tracks[i];

            let unselected_paint = paint
                .clone()
                .with_font_size(18.0)
                .with_color(Color::hex("#aaaaaa"));
            let selected_paint = paint
                .clone()
                .with_font_size(18.0)
                .with_color(Color::hex("#c9f3aeff"));

            let paint = if i == self.current_track as usize {
                selected_paint
            } else {
                unselected_paint
            };

            let metrics = canvas
                .fill_text(x, y, format!("{}. {}", i + 1, t.title), &paint)
                .unwrap();
            y += metrics.height();
        }

        let x = (50.0 + isize + 50.0);

        let percent = self.current_time / self.current_duration;

        let time_start = x;
        let time_width: f32 = (w - 20.0) - x;
        let time_y = h - 100.0;

        let line_paint = Paint::color(Color::hex("#4b4b4b"));

        let mut path = Path::new();
        path.rect(time_start, time_y, time_width, 5.0);
        canvas.fill_path(&path, &line_paint);

        let line_paint = Paint::color(Color::hex("#788177"));
        let mut path = Path::new();
        path.rect(time_start, time_y, percent * time_width, 5.0);
        canvas.fill_path(&path, &line_paint);

        let Some(lyrics) = &track.lyrics else {
            println!("early return lyrics");

            return;
        };

        let mut y = h / 2.0;
        let x = (w - x) / 2.0 + x;

        let mut a = 1.0;
        let paint = paint
            .clone()
            .with_text_align(Align::Center)
            .with_text_baseline(Baseline::Middle);

        let closest_lines = get_closest_lines(self.current_time, lyrics);

        let middle = closest_lines.len() / 2;

        for (i, (is_current, line)) in closest_lines.iter().enumerate() {
            let a = if *is_current { 1.0 } else { 0.5 };
            let metrics = canvas
                .fill_text(
                    x,
                    y,
                    line,
                    &paint.clone().with_color(Color::hsla(1.0, 1.0, 1.0, a)),
                )
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
            current_duration: 0f32,
            cover_image: None,
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

            if let Some(meta) = meta {
                println!("got meta trracks {}", meta.tracks.len());
                if let Some(cover_bytes) = meta.cover.as_ref() {
                    self.cover_image = Some(
                        canvas
                            .load_image_mem(&cover_bytes, ImageFlags::empty())
                            .unwrap(),
                    );
                }
            }

            window.set_visible(true);
            window.request_redraw();
        }

        if let Message::PlayerState {
            current_track,
            position,
            duration,
        } = message
        {
            self.current_track = *current_track;
            self.current_time = position.as_secs_f32();
            self.current_duration = duration.as_secs_f32();

            window.request_redraw();
            println!(
                "{}, {}, {}",
                self.current_track, self.current_time, self.current_duration
            );
        }

        false
    }
}

fn get_closest_lines(current_time: f32, lyrics: &lrc::Lyrics) -> Vec<(bool, String)> {
    let mut lines = lyrics.get_timed_lines();

    let mut start_idx = 0;
    for i in 0..lines.len() {
        let i = lines.len() - i - 1;
        let (t, l) = &lines[i];
        let t = t.get_timestamp() as f32 / 1000.0;
        if current_time >= t {
            start_idx = i;
            break;
        }
    }
    let mut closest_lines = Vec::new();
    let mut found_current = false;
    
    for i in start_idx..lines.len() {
        let (t, l) = &lines[i];
        let t = t.get_timestamp() as f32 / 1000.0;
        let current = current_time >= t && !found_current;

        if current_time >= t {
            found_current = true;
        }
        closest_lines.push((current, l.clone()));

        if closest_lines.len() > 3 {
            break;
        }
    }

    closest_lines

    // let mut lines = lyrics
    //     .get_timed_lines()
    //     .into_iter()
    //     .map(|(t, l)| {
    //         (
    //             (current_time - (t.get_timestamp() as f32 / 1000.0f32)).abs(),
    //             t,
    //             l,
    //         )
    //     })
    //     .collect::<Vec<_>>();
    // lines.sort_by(|(a, _, _), (b, _, _)| a.partial_cmp(b).unwrap());
    // lines
    //     .into_iter()
    //     .map(|(_, _, l)| l.clone())
    //     .take(3)
    //     .collect()
}
