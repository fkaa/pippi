use crate::Message;
use std::sync::mpsc;
use std::thread;
    use std::process::{Command, Stdio};
    use std::io::{BufRead, Write, Read, BufReader};
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

pub enum MediaCommand {
    TogglePlay,
    VolumeUp,
    VolumeDown,
    Seek { seconds: i32 },
    StartMedia { path: String },
}

#[derive(Default)]
struct VlcState {
    
}

pub fn start_controller(sender: mpsc::Sender<Message>) -> mpsc::Sender<MediaCommand> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        vlc_loop(rx);
    });

    tx
}

fn vlc_loop(rx: mpsc::Receiver<MediaCommand>) {
    let mut state = VlcState::default();

    let mut vlc = vlc();

    while let Ok(cmd) = rx.recv() {
        match cmd {
            MediaCommand::StartMedia { path } => {
                vlc.clear();
                vlc.enqueue(&path);
                let duration = vlc.get_duration();

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

fn vlc() -> VlcPipe {
    let mut proc = Command::new("cvlc")
        .args(&["-I", "rc", "--rc-unix", "test"])
        .spawn()
        .expect("Failed to launch cvlc");

    thread::sleep(std::time::Duration::from_secs(1));

    let mut socket = UnixStream::connect("test").unwrap();
    let mut reader = BufReader::new(socket.try_clone().unwrap());

    let mut pipe = VlcPipe::new(socket, reader);

    pipe
}

struct VlcPipe {
    writer: UnixStream,
    reader: BufReader<UnixStream>,
    line: String,
}

impl VlcPipe {
    pub fn new(writer: UnixStream, reader: BufReader<UnixStream>) -> VlcPipe {
        VlcPipe {
            writer,
            reader,
            line: String::default(),
        }
    }

    pub fn toggle_play(&mut self) {
        self.writer.write_all(b"pause").unwrap();
        self.read_line();
    }
    pub fn clear(&mut self) {
        self.writer.write_all(b"clear").unwrap();
        self.read_line();
    }
    pub fn play(&mut self) {
        self.writer.write_all(b"play").unwrap();
        self.read_line();
    }
    pub fn pause(&mut self) {
        self.writer.write_all(b"pause").unwrap();
        self.read_line();
    }
    pub fn seek(&mut self, seconds: i32) {
        write!(&mut self.writer, "seek {}\n", seconds).unwrap();
        self.read_line();
    }
    pub fn vol_down(&mut self, step: u32) {
        write!(&mut self.writer, "voldown {}\n", step).unwrap();
        self.read_line();
    }
    pub fn vol_up(&mut self, step: u32) {
        write!(&mut self.writer, "volup {}\n", step).unwrap();
        self.read_line();
    }
    pub fn add(&mut self, media: &str) {
        write!(&mut self.writer, "add {}\n", media).unwrap();
        self.read_line();
    }
    pub fn enqueue(&mut self, media: &str) {
        write!(&mut self.writer, "enqueue {}\n", media).unwrap();
        self.read_line();
        self.read_line();
    }

    pub fn is_playing(&mut self) -> bool {
        self.writer.write_all(b"is_playing").unwrap();
        self.read_line();
        self.line == "1"
    }
    pub fn get_duration(&mut self) -> Duration {
        self.writer.write_all(b"get_length").unwrap();
        self.read_line();
        let seconds = self.line.parse::<u32>().unwrap();

        Duration::from_secs(seconds as _)
    }
    pub fn get_time(&mut self) -> Duration {
        self.writer.write_all(b"get_time").unwrap();
        self.read_line();
        let seconds = self.line.parse::<u32>().unwrap();

        Duration::from_secs(seconds as _)
    }

    fn read_line(&mut self) {
        loop {
            self.line.clear();
            self.reader.read_line(&mut self.line).unwrap();
if !self.line.starts_with("status_change") {
    break;
}
            println!("skipping {:?}", self.line);
        }
    }
}
