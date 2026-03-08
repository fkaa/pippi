use std::{path::PathBuf, sync::mpsc::{Receiver, Sender, channel}, thread};

use lrc::Lyrics;
use musicbrainz_rs::{Fetch, FetchCoverart, MusicBrainzClient, entity::discid::Discid};
use winit::event_loop::EventLoopProxy;

use crate::Message;

#[derive(Debug, Clone)]
pub struct DiscMetadata {
    pub title: String,
    pub artist: String,
    pub cover: Option<Vec<u8>>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub title: String,
    pub lyrics: Option<Lyrics>,
}

pub enum CdInfo {
    Cd {
        album: String,
        artist: String,
        tracks: Vec<Track>,
    },
    Dvd {
        title: String,
    },
}

pub struct CdMetadataFetcher {
    cache_dir: PathBuf,
    client: MusicBrainzClient,
}

impl CdMetadataFetcher {
    pub fn with_cache_dir(cache_dir: PathBuf, proxy: EventLoopProxy<Message>) -> Sender<String> {
        let (tx, rx): (Sender<String>, Receiver<String>) = channel();
        
        thread::spawn(move || {
            let mut client = MusicBrainzClient::default();
            client.set_user_agent("pippi/0.0.1 ()").unwrap();
            client.coverart_archive_url = "http://coverartarchive.org".into();

            let fetcher = CdMetadataFetcher { cache_dir, client };

            while let Ok(disc_id) = rx.recv() {
                let metadata = fetcher.fetch_cd_metadata(&disc_id);
                proxy.send_event(Message::DiskMetadata(metadata)).unwrap();
            }
        });
        

        tx
    }

    pub fn fetch_cd_metadata(&self, disc_id: &str) -> Option<DiscMetadata> {
        let query = Discid::fetch().id(disc_id).execute_with_client(&self.client).ok()?;
        let release = query.releases.clone().unwrap()[0].clone();
        let _cover = release.get_coverart().execute_with_client(&self.client).ok()?;

        None
    }
}

