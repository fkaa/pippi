
pub enum CdInfo {
    Cd {
        album: String,
        artist: String,
        tracks: Vec<Track>,
    },
    Dvd {
        title: String,
    }
}

pub struct Track {
    pub title: String,
}

pub fn scan() -> Option<CdInfo> {

    todo!()
}
