use std::default;
use std::{collections::HashMap, thread};
use std::time::Duration;
use std::sync::mpsc;

mod dvd_monitor;
mod ir_remote_monitor;
mod hdmi_cec;
mod vlc;
mod cd;
mod ui;

use dvd_monitor::DiskReaderEvent;
use glutin::surface::GlSurface;
use vlc::MediaCommand;
use winit::event::WindowEvent;
use winit::window::WindowId;
use winit::{application::ApplicationHandler, event_loop::{ControlFlow, EventLoop, EventLoopBuilder}};

use crate::ui::{UiWindow, WindowState};
use crate::ui::welcome_window::WelcomeWindow;

#[derive(Debug, Default)]
pub enum Message {
    #[default]
    None,
    Disk(dvd_monitor::DiskReaderEvent),
    Ir(ir_remote_monitor::RemoteButton),
}

#[derive(Default)]
struct MediaControlApp {
    windows: HashMap<WindowId, WindowState>,
}

impl ApplicationHandler<Message> for MediaControlApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window: WindowState = ui::create_gl_window::<WelcomeWindow>(event_loop, (200, 200));

        self.windows.insert(window.window.id(), window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(mut window) = self.windows.get_mut(&window_id) else {
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                window.app.draw(&mut window.canvas);
                window.canvas.flush_to_surface(&());
                window.surface.swap_buffers(&window.context).unwrap();
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop: EventLoop<Message> = EventLoopBuilder::default().build().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = MediaControlApp::default();
    event_loop.run_app(&mut app).unwrap();
    /*let (tx, rx) = mpsc::channel();
    dvd_monitor::monitor_disk_reader(tx.clone());
    let vlc_tx = vlc::start_controller(tx.clone());
    //ir_remote_monitor::monitor_remote(tx.clone());

    while let Ok(msg) = rx.recv() {
        println!("{:?}", msg);

        use ir_remote_monitor::RemoteButton;
        match msg {
            Message::Disk(DiskReaderEvent::Inserted { .. }) => {
                // let cd_info = cd::scan();
                vlc_tx.send(MediaCommand::StartMedia { path: "dvdsimple:///dev/sr0".into() }).unwrap();
            }
            Message::Ir(RemoteButton::Star) => hdmi_cec::turn_tv_on(),
            Message::Ir(RemoteButton::Hash) => hdmi_cec::turn_tv_off(),
            Message::Ir(RemoteButton::Ok) => vlc_tx.send(MediaCommand::TogglePlay).unwrap(),
            Message::Ir(RemoteButton::Up) => vlc_tx.send(MediaCommand::VolumeUp).unwrap(),
            Message::Ir(RemoteButton::Down) => vlc_tx.send(MediaCommand::VolumeDown).unwrap(),
            Message::Ir(RemoteButton::Left) => vlc_tx.send(MediaCommand::Seek{ seconds: -15 }).unwrap(),
            Message::Ir(RemoteButton::Right) => vlc_tx.send(MediaCommand::Seek{ seconds: 15 }).unwrap(),
            _ => {}
        }
    }*/
}
