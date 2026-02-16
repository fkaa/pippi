use mio::{Events, Interest, Poll, Token};

use std::io;
use std::sync::mpsc;
use std::thread;

use crate::Message;

#[derive(Debug)]
pub enum DiskReaderEvent {
    Inserted { is_audio: bool },
    Ejected,
}

pub fn monitor_disk_reader(tx: mpsc::Sender<Message>) {
    thread::spawn(move || {
        monitor_cd_drive_fn(tx);
    });
}

pub fn poll(
    mut socket: udev::MonitorSocket,
    sender: mpsc::Sender<Message>,
) -> io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);

    poll.registry().register(
        &mut socket,
        Token(0),
        Interest::READABLE | Interest::WRITABLE,
    )?;

    loop {
        poll.poll(&mut events, None)?;

        for event in &events {
            if event.token() == Token(0) && event.is_writable() {
                for e in socket.iter() {
                    if e.sysname().to_str() != Some("sr0") {
                        continue;
                    }

                    let eject = e.property_value("DISK_EJECT_REQUEST");
                    let change = e.property_value("DISK_MEDIA_CHANGE");

                    if eject.is_some() {
                        sender.send(Message::Disk(DiskReaderEvent::Ejected)).unwrap();
                    }
                    if change.is_some() {
                        sender.send(Message::Disk(DiskReaderEvent::Inserted { is_audio: false })).unwrap();
                    }
                }
            }
        }
    }
}

fn print_event(event: udev::Event) {
    println!(
        "{}: {} {} (subsystem={}, sysname={}, devtype={})",
        event.sequence_number(),
        event.event_type(),
        event.syspath().to_str().unwrap_or("---"),
        event
            .subsystem()
            .map_or("", |s| { s.to_str().unwrap_or("") }),
        event.sysname().to_str().unwrap_or(""),
        event.devtype().map_or("", |s| { s.to_str().unwrap_or("") })
    );
    for property in event.properties() {
        println!("PROP {:?} = {:?}", property.name(), property.value());
    }
}

fn monitor_cd_drive_fn(sender: mpsc::Sender<Message>) -> io::Result<()> {
    let socket = udev::MonitorBuilder::new_kernel()?
        // .match_subsystem_devtype("usb", "usb_device")?
        .match_subsystem("block")?
        .listen()?;

    poll(socket, sender)
}
