use std::{
    fmt, fs, path::PathBuf, sync::mpsc::{Receiver, Sender, channel}, thread
};

use lrc::Lyrics;
use musicbrainz_rs::{
    Fetch, FetchCoverart, MusicBrainzClient,
    entity::{CoverartResponse, discid::Discid, release::Release},
};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use winit::event_loop::EventLoopProxy;

use crate::Message;

#[derive(Clone, Serialize, Deserialize)]
pub struct DiscMetadata {
    pub title: String,
    pub artist: String,
    pub cover: Option<Vec<u8>>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub title: String,
    pub lyrics: Option<Lyrics>,
}

impl fmt::Debug for DiscMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiscMetadata")
        .field("title", &self.title)
        .field("artist", &self.artist)
        //.field("cover", &self.cover)
        //.field("tracks", &self.tracks)
        .finish()
    }
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
    lyrics_db_path: PathBuf,
    client: MusicBrainzClient,
}

impl CdMetadataFetcher {
    pub fn with_cache_dir(
        cache_dir: PathBuf,
        lyrics_db_path: PathBuf,
        proxy: EventLoopProxy<Message>,
    ) -> Sender<String> {
        let (tx, rx): (Sender<String>, Receiver<String>) = channel();

        thread::spawn(move || {
            let mut client = MusicBrainzClient::default();
            client.set_user_agent("pippi/0.0.1 ()").unwrap();
            client.coverart_archive_url = "http://coverartarchive.org".into();

            let fetcher = CdMetadataFetcher {
                cache_dir,
                lyrics_db_path,
                client,
            };

            while let Ok(disc_id) = rx.recv() {
                let metadata = fetcher.fetch_cd_metadata(&disc_id);
                proxy.send_event(Message::DiskMetadata(metadata)).unwrap();
            }
        });

        tx
    }

    pub fn fetch_cd_metadata(&self, disc_id: &str) -> Option<DiscMetadata> {
        let mut cache_file = self.cache_dir.clone();
        cache_file.push(format!("{}.json", disc_id));

        if fs::exists(&cache_file).unwrap() {
            println!("Found cached file for {disc_id}");
            let meta = serde_json::from_str(&fs::read_to_string(&cache_file).unwrap()).unwrap();
            return Some(meta);
        }

        let query = match Discid::fetch()
            .id(disc_id)
            .with_artist_credits()
            .with_recordings()
            .execute_with_client(&self.client)
        {
            Ok(query) => query,
            Err(e) => {
                eprintln!("Failed to fetch disc ID: {e:?}");
                return None;
            }
        };
        let release = query.releases.clone().unwrap()[0].clone();
        // dbg!(&release);
        let title = release.title.clone();
        let artist = release.artist_credit.as_ref().unwrap()[0].name.clone();

        let media = release.media.as_ref().unwrap()[0].clone();

        let tracks = media
            .tracks
            .unwrap()
            .into_iter()
            .filter_map(|t| {
                t.recording.map(|r| {
                    (
                        r.title,
                        r.artist_credit
                            .unwrap()
                            .into_iter()
                            .next()
                            .map(|c| c.name)
                            .unwrap_or_default(),
                        r.length.unwrap_or(0),
                    )
                })
            })
            .collect::<Vec<_>>();

        let tracks = tracks
            .into_iter()
            .map(|(title, artist, duration)| {
                let lyrics = self.fetch_lyrics(
                    &title.to_lowercase(),
                    &release.title.to_lowercase(),
                    &artist.to_lowercase(),
                    duration,
                );

                Track {
                    title: title,
                    lyrics,
                }
            })
            .collect::<Vec<_>>();

        let cover = match release.get_coverart().execute_with_client(&self.client) {
            Ok(cover) => cover,
            Err(e) => {
                eprintln!("Failed to fetch disc cover: {e:?}");
                return None;
            }
        };

        let front = if let CoverartResponse::Json(json) = cover {
            json.images
                .iter()
                .filter_map(|i| {
                    if i.front == true {
                        Some(i.thumbnails.clone())
                    } else {
                        None
                    }
                })
                .next()
        } else {
            None
        };
        dbg!(&front);

        let cover = front.and_then(|t| t.large).and_then(|url| {
            ureq::get(url)
                .call()
                .ok()
                .and_then(|mut r| r.body_mut().read_to_vec().ok())
        });

        let mut cache_file = self.cache_dir.clone();
        let _ = fs::create_dir(&cache_file);

        cache_file.push(format!("{}.json", disc_id));

        let meta = DiscMetadata {
            title,
            artist,
            cover,
            tracks,
        };

        let json = serde_json::to_string(&meta).unwrap();
        fs::write(cache_file, &json).unwrap();

        Some(meta)
    }

    pub fn fetch_lyrics(
        &self,
        track: &str,
        album: &str,
        artist: &str,
        duration: u32,
    ) -> Option<Lyrics> {
        let mut db = Connection::open(&self.lyrics_db_path).unwrap();
        let mut stmt = db
            .prepare("SELECT t.id, t.duration, l.synced_lyrics FROM tracks t INNER JOIN lyrics l ON l.track_id = t.id WHERE t.name_lower = ?1 AND t.album_name_lower = ?2 AND t.artist_name_lower = ?3")
            .unwrap();

        let data = stmt
            .query_map(params![track, album, artist], |row| {
                Ok((
                    row.get::<_, i64>(0).unwrap(),
                    row.get::<_, f64>(1).unwrap(),
                    row.get::<_, String>(2).unwrap(),
                ))
            })
            .unwrap()
            .collect::<Result<Vec<(i64, f64, String)>, _>>()
            .unwrap();

        let first = data.into_iter().next();

        first.map(|(_, _, lyrics)| Lyrics::from_str(lyrics).unwrap())
    }
}
