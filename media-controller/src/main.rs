use std::default;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;
use std::{collections::HashMap, thread};

mod cd;
mod dvd_monitor;
mod hdmi_cec;
mod ir_remote_monitor;
mod ui;
mod vlc;

use dvd_monitor::DiskReaderEvent;
use glutin::prelude::PossiblyCurrentGlContext;
use glutin::surface::GlSurface;
use vlc::MediaCommand;
use winit::dpi::PhysicalPosition;
use winit::event::WindowEvent;
use winit::event_loop::{self, ActiveEventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use winit::{
    application::ApplicationHandler,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
};
        use enigo::{
    Button, Coordinate,
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Mouse, Settings,
};

use crate::ui::debug_console::DebugConsoleWindow;
use crate::ui::prompt_window::PromptWindow;
use crate::ui::welcome_window::WelcomeWindow;
use crate::ui::{UiWindow, WindowState};

#[derive(Debug, Default, Clone)]
pub enum Message {
    #[default]
    None,
    Disk(dvd_monitor::DiskReaderEvent),
    Ir(ir_remote_monitor::RemoteButton),
    SetPrompt {
        prompt: String,
    },
    PromptChosen(bool),
    SetListPrompt {
        title: String,
        choices: Vec<String>,
        current_choice: i32,
    },
    ListPromptChosen(i32),
}

enum WindowPos {
    Center,
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

fn set_window_pos(window: &Window, pos: WindowPos) {
    let monitor = window.current_monitor().unwrap();
    let monitor_size = monitor.size();
    let window_size = window.outer_size();

    let (x, y) = match pos {
        WindowPos::Center => (
            monitor_size.width / 2 - window_size.width / 2,
            monitor_size.height / 2 - window_size.height / 2,
        ),
        WindowPos::TopLeft => (0, 0),
        WindowPos::Top => (monitor_size.width / 2, 0),
        WindowPos::TopRight => (monitor_size.width - window_size.width, 0),
        WindowPos::Right => (
            monitor_size.width - window_size.width,
            monitor_size.height / 2,
        ),
        WindowPos::BottomRight => (
            monitor_size.width - window_size.width,
            monitor_size.height - window_size.height,
        ),
        WindowPos::Bottom => (
            monitor_size.width / 2 - window_size.width / 2,
            monitor_size.height - window_size.height,
        ),
        WindowPos::BottomLeft => (0, monitor_size.height - window_size.height),
        WindowPos::Left => (0, monitor_size.height / 2),
    };

    window.set_outer_position(PhysicalPosition::new(x, y));
}

struct MediaControlApp {
    windows: Vec<WindowState>,
    debug_window: WindowId,
    welcome_window: WindowId,
    prompt_window: WindowId,
    vlc_tx: Sender<MediaCommand>,
    proxy: EventLoopProxy<Message>,
    enigo: Enigo,
}

impl MediaControlApp {
    fn new(vlc_tx: Sender<MediaCommand>, proxy: EventLoopProxy<Message>) -> Self {
        let mut enigo = Enigo::new(&Settings::default()).unwrap();

        Self {
            windows: Default::default(),
            debug_window: WindowId::dummy(),
            welcome_window: WindowId::dummy(),
            prompt_window: WindowId::dummy(),
            vlc_tx,
            proxy,
            enigo,
        }
    }

    fn add_window<T: UiWindow + 'static>(
        &mut self,
        event_loop: &ActiveEventLoop,
        size: (u32, u32),
        pos: WindowPos,
    ) -> WindowId {
        let window: WindowState = ui::create_gl_window::<T>(event_loop, size);

        set_window_pos(&window.window, pos);

        let key = window.window.id();
        self.windows.push(window);
        key
    }

    fn show_window(&mut self, window_id: &WindowId, visible: bool) {
        let Some(mut window) = self.windows.iter_mut().find(|w| w.window.id() == *window_id) else {
            return;
        };

        window.window.set_visible(visible);
    }
}

impl ApplicationHandler<Message> for MediaControlApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        /*self.debug_window =
            self.add_window::<DebugConsoleWindow>(event_loop, (300, 400), WindowPos::TopLeft);
        self.welcome_window =
            self.add_window::<WelcomeWindow>(event_loop, (500, 120), WindowPos::Center);
        self.prompt_window =
            self.add_window::<PromptWindow>(event_loop, (400, 160), WindowPos::Bottom);*/
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(mut window) = self.windows.iter_mut().find(|w| w.window.id() == window_id) else {
            println!("unknown window");
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                window.context.make_current(&window.surface).unwrap();
                window.app.draw(&mut window.canvas);
                window.canvas.flush_to_surface(&());
                window.surface.swap_buffers(&window.context).unwrap();
            }
            _ => {}
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Message) {
        println!("{:?}", event);

        let mut event_handled = false;
        for window in &mut self.windows {
            if window.app.on_message(&event, &*window.window, &self.proxy) {
                event_handled = true;
                break;
            }
        }

        if event_handled {
            return;
        }

        use ir_remote_monitor::RemoteButton;
        match event {
            Message::Disk(DiskReaderEvent::Inserted { .. }) => {
                // let cd_info = cd::scan();
                self.vlc_tx
                    .send(MediaCommand::StartMedia {
                        path: "dvdsimple:///dev/sr0".into(),
                    })
                    .unwrap();
            }
            Message::Ir(RemoteButton::Star) => hdmi_cec::turn_tv_on(),
            Message::Ir(RemoteButton::Hash) => hdmi_cec::turn_tv_off(),
            Message::Ir(RemoteButton::Ok) => self.vlc_tx.send(MediaCommand::TogglePlay).unwrap(),
            Message::Ir(RemoteButton::Up) => self.vlc_tx.send(MediaCommand::VolumeUp).unwrap(),
            Message::Ir(RemoteButton::Down) => self.vlc_tx.send(MediaCommand::VolumeDown).unwrap(),
            Message::Ir(RemoteButton::Left) => self
                .vlc_tx
                .send(MediaCommand::Seek { seconds: -15 })
                .unwrap(),
            Message::Ir(RemoteButton::Right) => self
                .vlc_tx
                .send(MediaCommand::Seek { seconds: 15 })
                .unwrap(),
            _ => {}
        }
    }
}

fn main() {
    let mut vlc = vlc::vlc();

    dbg!(vlc.is_playing());
    dbg!(vlc.enqueue("file:///home/tmtu/dev/clown-escape/clown-escape/assets/sfx/rats.wav"));
    dbg!(vlc.toggle_play());
    dbg!(vlc.is_playing());

    loop{}
    /*
    let (tx, rx) = mpsc::channel();
    dvd_monitor::monitor_disk_reader(tx.clone());
    //ir_remote_monitor::monitor_remote(tx.clone());

    let vlc_tx: Sender<MediaCommand> = vlc::start_controller(tx.clone());

    tx.send(Message::Disk(DiskReaderEvent::Inserted { is_audio: false }));
    tx.send(Message::SetPrompt {
        prompt: "Vill du börja där du slutade?".into(),
    });
    tx.send(Message::Ir(ir_remote_monitor::RemoteButton::Left));
    tx.send(Message::Ir(ir_remote_monitor::RemoteButton::Ok));

    // event loop
    let event_loop: EventLoop<Message> = EventLoopBuilder::default().build().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let proxy = event_loop.create_proxy();

    thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            proxy.send_event(msg).unwrap();
        }
    });

    let proxy = event_loop.create_proxy();

    let mut app = MediaControlApp::new(vlc_tx, proxy);
    event_loop.run_app(&mut app).unwrap();*/
}
