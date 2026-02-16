use std::thread;
use std::time::Duration;
use std::sync::mpsc;

mod dvd_monitor;
mod ir_remote_monitor;
mod hdmi_cec;

#[derive(Debug)]
pub enum Message {
    Disk(dvd_monitor::DiskReaderEvent),
    Ir(ir_remote_monitor::RemoteButton),
}

fn main() {
    let (tx, rx) = mpsc::channel();
    dvd_monitor::monitor_disk_reader(tx.clone());
    ir_remote_monitor::monitor_remote(tx.clone());

    while let Ok(msg) = rx.recv() {
        println!("{:?}", msg);

        use ir_remote_monitor::RemoteButton;
        match msg {
            Message::Ir(RemoteButton::Star) => hdmi_cec::turn_tv_on(),
            Message::Ir(RemoteButton::Hash) => hdmi_cec::turn_tv_off(),
            _ => {}
        }
    }
}
