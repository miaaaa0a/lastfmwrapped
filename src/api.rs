use std::io::Cursor;
use axum::{extract::Path, Json};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{ImageFormat, ImageReader};
use serde_json::{json, Value};
use crate::{calculations::{calculate_year, largest_value_hashmap}, imageprocessing, lfm, spotify};

pub async fn minutes_listened(Path(username): Path<String>) -> Json<Value> {
    println!("{}", username);
    let lfm_client = lfm::init_client(&username);
    let spotify_client = spotify::auth().await;

    let total = calculate_year(lfm_client, spotify_client).await;
    let total_minutes = ((total.values().into_iter().sum::<i64>()) / 1000) / 60;
    let busiest = largest_value_hashmap(&total);
    let busiest_time = (busiest[1] / 1000) / 60;

    let img = imageprocessing::minutes_listened(total_minutes, busiest[0], busiest_time).unwrap();
    let mut buffer = Cursor::new(Vec::new());
    let _ = img.write_to(&mut buffer, ImageFormat::Png);
    let encoded_image = buffer.get_ref().clone();
    let b64 = STANDARD.encode(encoded_image);
    let response = json!({ "image": b64 });
    Json(response)
}

pub async fn top_song(Path(username): Path<String>) -> Json<Value> {
    println!("{}", username);
    let spotify_client = spotify::auth().await;

    let top_tracks = lfm::fetch_top_5_tracks(&username).await;
    let mut top_tracks_sorted = top_tracks.iter().collect::<Vec<_>>();
    top_tracks_sorted.sort_by_key(|k| k.1);
    top_tracks_sorted.reverse();

    let top_track = top_tracks_sorted[0].0;
    let top_track_name = top_track.split(" - ").collect::<Vec<_>>()[1].to_string();

    let song_cover_info = spotify::find_song_cover(&spotify_client, top_track, &top_track_name).await;
    let song_cover_url = song_cover_info["url"].as_str().unwrap_or("").trim_matches('\"');
    let song_cover = reqwest::get(song_cover_url).await.unwrap().bytes().await.unwrap();
    let song_cover_reader = ImageReader::new(Cursor::new(&song_cover)).with_guessed_format().unwrap();
    let song_cover_img = song_cover_reader.decode().unwrap();
    
    let img = imageprocessing::top_song(top_track.clone(), *top_tracks_sorted[0].1 as i64, song_cover_img).unwrap();
    let mut buffer = Cursor::new(Vec::new());
    let _ = img.write_to(&mut buffer, ImageFormat::Png);
    let encoded_image = buffer.get_ref().clone();
    let b64 = STANDARD.encode(encoded_image);
    let response = json!({ "image": b64 });
    Json(response)
}