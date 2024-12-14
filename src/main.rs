use std::{error::Error, io::Cursor};
use image::ImageReader;
pub mod lfm;
pub mod calculations;
pub mod spotify;
pub mod imageprocessing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let lfm_client = lfm::init_client("redoverflow");
    let spotify_client = spotify::auth().await;
    let top_tracks = lfm::fetch_top_5_tracks(&"redoverflow".to_string()).await;
    println!("{:?}", top_tracks);
    let mut top_tracks_sorted = top_tracks.iter().collect::<Vec<_>>();
    top_tracks_sorted.sort_by_key(|k| k.1);
    top_tracks_sorted.reverse();
    let top_track = top_tracks_sorted[0].0;
    let top_track_name = top_track.split(" - ").collect::<Vec<_>>()[1].to_string();
    let song_cover_info = spotify::find_song_cover(&spotify_client, top_track, &top_track_name).await;
    let song_cover_url = song_cover_info["url"].as_str().unwrap_or("").trim_matches('\"');
    println!("{}", song_cover_url);
    let song_cover = reqwest::get(song_cover_url).await?.bytes().await?;
    println!("{:?}", song_cover.len());
    let song_cover_reader = ImageReader::new(Cursor::new(&song_cover)).with_guessed_format()?;
    let song_cover_img = song_cover_reader.decode()?;
    
    let topsong = imageprocessing::top_song(top_track.clone(), *top_tracks_sorted[0].1 as i64, song_cover_img)?;
    topsong.save("meow.png")?;
    Ok(())
}
