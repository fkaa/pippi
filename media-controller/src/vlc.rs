use crate::Message;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum MediaCommand {
    TogglePlay,
    VolumeUp,
    VolumeDown,
    Seek { seconds: i32 },
    StartMedia { path: String },
    Poll,
}

#[derive(Default)]
struct VlcState {}

pub fn start_controller(sender: mpsc::Sender<Message>) -> mpsc::Sender<MediaCommand> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        vlc_loop(rx, sender);
    });

    tx
}

fn vlc_loop(rx: mpsc::Receiver<MediaCommand>, sender: mpsc::Sender<Message>) {
    let _state = VlcState::default();

    let mut vlc = vlc();

    while let Ok(cmd) = rx.recv() {
        println!("Got VLC command: {:?}", cmd);
        match cmd {
            MediaCommand::Poll => {
                println!("getting track meta");
                let track = vlc.get_track_metadata();
                let position = vlc.get_time();
                let duration = vlc.get_duration();

                sender
                    .send(Message::PlayerState {
                        current_track: track.track_number,
                        position,
                        duration,
                    })
                    .unwrap();
            }
            MediaCommand::StartMedia { path } => {
                vlc.clear();
                vlc.enqueue(&path);
                let _duration = vlc.get_duration();

                vlc.play()
            }
            MediaCommand::TogglePlay => {
                vlc.toggle_play();
            }
            MediaCommand::VolumeUp => {
                vlc.vol_up(10);
            }
            MediaCommand::VolumeDown => {
                vlc.vol_down(10);
            }
            MediaCommand::Seek { seconds } => {
                let time = vlc.get_time().as_secs() as i32;
                vlc.seek(time + seconds);
            }
        }
    }
}

pub fn vlc() -> VlcPipe {
    let _proc = Command::new("vlc")
        .args(&["-I", "cli", "--lua-config", "cli={host='localhost:4212'}"])
        .spawn()
        .expect("Failed to launch cvlc");

    thread::sleep(std::time::Duration::from_secs(1));

    let socket = TcpStream::connect("localhost:4212").unwrap();
    let reader = BufReader::new(socket.try_clone().unwrap());

    let pipe = VlcPipe::new(socket, reader);

    pipe
}

#[derive(Default)]
pub struct TrackMetadata {
    track_number: i32,
}

pub struct VlcPipe {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
    line: String,
}

impl VlcPipe {
    pub fn new(writer: TcpStream, reader: BufReader<TcpStream>) -> VlcPipe {
        let mut pipe = VlcPipe {
            writer,
            reader,
            line: String::default(),
        };

        pipe.read_line();
        pipe.read_line();

        println!("read prompt");

        pipe
    }

    pub fn toggle_play(&mut self) {
        self.writer.write_all(b"pause\n").unwrap();
        //self.read_line();
    }
    pub fn clear(&mut self) {
        println!("sending clear");
        self.read_prompt();
        self.writer.write_all(b"clear\n").unwrap();
        //self.read_line();
        println!("sent clear");
    }
    pub fn play(&mut self) {
        println!("sending play");
        self.read_prompt();
        self.writer.write_all(b"play\n").unwrap();
        //self.read_line();
        println!("sent play");
    }
    pub fn pause(&mut self) {
        self.read_prompt();
        self.writer.write_all(b"pause\n").unwrap();
        self.read_line();
    }
    pub fn seek(&mut self, seconds: i32) {
        self.read_prompt();
        write!(&mut self.writer, "seek {}\n", seconds).unwrap();
        self.read_line();
    }
    pub fn vol_down(&mut self, step: u32) {
        self.read_prompt();

        write!(&mut self.writer, "voldown {}\n", step).unwrap();
        self.read_line();
    }
    pub fn vol_up(&mut self, step: u32) {
        self.read_prompt();

        write!(&mut self.writer, "volup {}\n", step).unwrap();
        self.read_line();
    }
    pub fn add(&mut self, media: &str) {
        self.read_prompt();

        write!(&mut self.writer, "add {}\n", media).unwrap();
        self.read_line();
    }
    pub fn enqueue(&mut self, media: &str) {
        println!("sending enqueue");
        self.read_prompt();

        write!(&mut self.writer, "enqueue {}\n", media).unwrap();
        //self.read_line();

        println!("sent enqueue");
    }

    pub fn is_playing(&mut self) -> bool {
        self.read_prompt();

        self.writer.write_all(b"is_playing\n").unwrap();
        self.read_line();
        self.line == "1"
    }
    pub fn get_duration(&mut self) -> Duration {
        self.read_prompt();

        self.writer.write_all(b"get_length\n").unwrap();
        self.read_line();

        let seconds = self.line.trim_end().parse::<u32>();
        let seconds = match seconds {
            Ok(seconds) => seconds,
            Err(_) => 0,
        };

        Duration::from_secs(seconds as _)
    }
    pub fn get_track_metadata(&mut self) -> TrackMetadata {
        let mut metadata: TrackMetadata = Default::default();

        if !self.is_playing() {
            return metadata;
        }

        println!("sending info");       
        self.writer.write_all(b"info\n").unwrap();

        loop {
            self.read_line();
            println!("MD: {:?}", self.line);

            if let Some((_, n)) = self.line.split_once("| track_number: ") {
                metadata.track_number = n.trim_end().parse::<i32>().unwrap();
            }

            if self.line == "+----[ end of stream info ]" {
                break;
            }
        }
        println!("done getting info");

        metadata
    }
    pub fn get_time(&mut self) -> Duration {
        self.read_prompt();

        self.writer.write_all(b"get_time\n").unwrap();
        self.read_line();
        let seconds = self.line.trim_end().parse::<u32>();
        let seconds = match seconds {
            Ok(seconds) => seconds,
            Err(_) => 0,
        };
        Duration::from_secs(seconds as _)
    }

    fn read_prompt(&mut self) {
        let mut prompt = [0u8; 2];
        self.reader.read_exact(&mut prompt).unwrap();
        assert_eq!(&prompt, b"> ");
    }
    fn read_line(&mut self) {
        loop {
            self.line.clear();
            self.reader.read_line(&mut self.line).unwrap();
            println!("$ReadVlc: {:?}", self.line);
            if !self.line.starts_with("status_change") {
                break;
            }
            println!("skipping {:?}", self.line);
        }
    }
}
