use std::thread;
use std::time::Duration;
use std::sync::mpsc;

mod dvd_monitor;
mod ir_remote_monitor;
mod hdmi_cec;
mod vlc;
mod cd;

use dvd_monitor::DiskReaderEvent;
use vlc::MediaCommand;

#[derive(Debug)]
pub enum Message {
    Disk(dvd_monitor::DiskReaderEvent),
    Ir(ir_remote_monitor::RemoteButton),
}

fn main() {
    let (tx, rx) = mpsc::channel();
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
            _ => {}
        }
    }
}
