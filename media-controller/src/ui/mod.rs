use std::num::NonZeroU32;
use std::sync::Arc;

use femtovg::{Canvas, FontId, renderer::OpenGl};
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::*,
    surface::{Surface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::Window;

use crate::Message;

pub mod debug_console;
pub mod prompt_window;
pub mod welcome_window;

pub static FONT_ROBOTO_REGULAR: &[u8] = &*include_bytes!("../../assets/Roboto-Regular.ttf");
pub static FONT_ROBOTO_BOLD: &[u8] = &*include_bytes!("../../assets/Roboto-Bold.ttf");
pub static FONT_ROBOTO_LIGHT: &[u8] = &*include_bytes!("../../assets/Roboto-Light.ttf");

pub struct Fonts {
    sans: FontId,
    bold: FontId,
    light: FontId,
}
pub trait UiWindow {
    fn create(canvas: &mut Canvas<OpenGl>) -> Self
    where
        Self: Sized;
    fn on_message(&mut self, message: &Message, window: &Window, proxy: &EventLoopProxy<Message>) -> bool {
        false
    }
    fn draw(&mut self, canvas: &mut Canvas<OpenGl>);
}

pub struct WindowState {
    pub window: Arc<Window>,
    pub context: PossiblyCurrentContext,
    pub surface: Surface<WindowSurface>,
    pub canvas: Canvas<OpenGl>,
    pub app: Box<dyn UiWindow>,
}

pub fn create_gl_window<T: UiWindow + 'static>(
    event_loop: &ActiveEventLoop,
    size: (u32, u32),
) -> WindowState {
    let window_attrs = Window::default_attributes()
        .with_inner_size(winit::dpi::PhysicalSize::new(size.0, size.1))
        .with_decorations(false)
        .with_resizable(false);

    let template = ConfigTemplateBuilder::new().with_alpha_size(8);

    let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attrs));

    let (window, gl_config) = display_builder
        .build(event_loop, template, |configs| {
            configs
                .reduce(|accum, config| {
                    let transparency_check = config.supports_transparency().unwrap_or(false)
                        & !accum.supports_transparency().unwrap_or(false);

                    if transparency_check || config.num_samples() < accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .unwrap();

    let window = window.unwrap();

    let raw_window_handle = window.window_handle().unwrap().as_raw();

    let gl_display = gl_config.display();

    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(Some(raw_window_handle));
    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(&gl_config, &fallback_context_attributes)
                    .expect("failed to create context")
            })
    });

    let (width, height): (u32, u32) = window.inner_size().into();
    let raw_window_handle = window.window_handle().unwrap().as_raw();
    let attrs = SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    );

    let surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };

    let gl_context = not_current_gl_context
        .take()
        .unwrap()
        .make_current(&surface)
        .unwrap();

    let renderer =
        unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s).cast()) }
            .expect("Cannot create renderer");

    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    canvas.set_size(width, height, window.scale_factor() as f32);

    let window = Arc::new(window);

    let app = Box::new(T::create(&mut canvas));

    WindowState {
        window,
        context: gl_context,
        surface,
        canvas,
        app,
    }
}
