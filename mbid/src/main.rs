use discid::{DiscId, Features};
use musicbrainz_rs::entity::release::Release;
use musicbrainz_rs::entity::discid::Discid;
use musicbrainz_rs::prelude::*;
use musicbrainz_rs::MusicBrainzClient;

fn main() {
    env_logger::init();
  let disc = DiscId::read_features(None, Features::ISRC).expect("Reading disc failed");
  println!("Disc ID: {}", disc.id());

  for track in disc.tracks() {
    println!("Track #{} ISRC: {}", track.number, track.isrc);
  }

  let mut client = MusicBrainzClient::default();
  client.set_user_agent("pippi/0.0.1 ()");
  client.coverart_archive_url = "http://coverartarchive.org".into();

  let query = Discid::fetch().id(&disc.id()).execute_with_client(&client).unwrap();
  let release = query.releases.clone().unwrap()[0].clone();
  let cover = release.get_coverart().execute_with_client(&client).unwrap();

  dbg!(cover);
}
