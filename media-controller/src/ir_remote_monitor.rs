use gpio_cdev::{Chip, EventRequestFlags, EventType, LineRequestFlags};
use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;

use crate::Message;

const HEADER_GAP_MS: f64 = 9.0 + 4.5;
const BIT0_GAP_MS: f64 = 0.5625 + 0.5625;
const BIT1_GAP_MS: f64 = 0.5625 + 0.5625 * 3.0;

#[derive(Copy, Clone, Debug)]
pub enum RemoteButton {
    Number(u32),
    Left,
    Up,
    Right,
    Down,
    Hash,
    Star,
    Ok,
}

pub fn monitor_remote(tx: mpsc::Sender<Message>) {
    thread::spawn(move || {
        monitor_remote_fn(tx);
    });
}

fn monitor_remote_fn(sender: mpsc::Sender<Message>) {
    let mapping = get_button_mapping();

    // Read the state of GPIO4 on a raspberry pi.  /dev/gpiochip0
    // maps to the driver for the SoC (builtin) GPIO controller.
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();
    let handle = chip.get_line(4).unwrap();
    let mut prev_ty = EventType::RisingEdge;
    let mut prev_ts = 0.0;
    let mut prev_dur = 0.0;
    let mut edges_read = 0;
    let mut bits_read = 0;
    let mut ir_value = 0u32;
    for event in handle
        .events(
            LineRequestFlags::INPUT,
            EventRequestFlags::BOTH_EDGES,
            "read-input",
        )
        .unwrap()
    {
        let evt = event.unwrap();

        edges_read += 1;

        let time_ms = evt.timestamp() as f64 / 1e6;
        let ty = evt.event_type();
        let duration = time_ms - prev_ts;

        if edges_read >= 2 {
            if prev_dur + duration <= BIT0_GAP_MS * 1.1 {
                ir_value = ir_value << 1;
                bits_read += 1;
            } else if prev_dur + duration <= BIT1_GAP_MS * 1.1 {
                ir_value = (ir_value << 1) + 1;
                bits_read += 1;
            } else if prev_dur + duration <= HEADER_GAP_MS * 1.1 {
                //println!("\nstart");
                bits_read = 0;
            }
            edges_read = 0;
            if bits_read == 32 {
                if let Some(mapped) = mapping.get(&ir_value) {
                    sender.send(Message::Ir(*mapped)).unwrap();
                }
                //println!("> 0x{:08x}", ir_value);
            }
        }

        prev_dur = duration;
        prev_ts = time_ms;
        prev_ty = ty;
    }
}

fn get_button_mapping() -> HashMap<u32, RemoteButton> {
    let mut map = HashMap::new();

    map.insert(0xff9867, RemoteButton::Number(0));
    map.insert(0xffa25d, RemoteButton::Number(1));
    map.insert(0xff629d, RemoteButton::Number(2));
    map.insert(0xffe21d, RemoteButton::Number(3));
    map.insert(0xff22dd, RemoteButton::Number(4));
    map.insert(0xff02fd, RemoteButton::Number(5));
    map.insert(0xffc23d, RemoteButton::Number(6));
    map.insert(0xffe01f, RemoteButton::Number(7));
    map.insert(0xffa857, RemoteButton::Number(8));
    map.insert(0xff906f, RemoteButton::Number(9));
    map.insert(0xff6897, RemoteButton::Star);
    map.insert(0xffb04f, RemoteButton::Hash);
    map.insert(0xff38c7, RemoteButton::Ok);
    map.insert(0xff10ef, RemoteButton::Left);
    map.insert(0xff4ab5, RemoteButton::Down);
    map.insert(0xff5aa5, RemoteButton::Right);
    map.insert(0xff18e7, RemoteButton::Up);

    map
}
