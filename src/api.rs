use crate::{
    calculations::{calculate_genre_months, calculate_year, largest_value_hashmap},
    imageprocessing, lfm, spotify,
};
use axum::{extract::Path, Json};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{DynamicImage, ImageFormat, ImageReader};
use itertools::Itertools;
use serde_json::{json, Value};
use std::io::Cursor;

fn img_to_response(img: DynamicImage) -> Value {
    let mut buffer = Cursor::new(Vec::new());
    let _ = img.write_to(&mut buffer, ImageFormat::Png);
    let encoded_image = buffer.get_ref().clone();
    let b64 = STANDARD.encode(encoded_image);
    json!({ "image": b64 })
}

fn img_mins_to_response(img: DynamicImage, minutes: i64) -> Value {
    let mut buffer = Cursor::new(Vec::new());
    let _ = img.write_to(&mut buffer, ImageFormat::Png);
    let encoded_image = buffer.get_ref().clone();
    let b64 = STANDARD.encode(encoded_image);
    json!({ "image": b64, "minutes": minutes })
}

fn imgs_to_response(imgs: Vec<DynamicImage>) -> Value {
    let mut encoded = Vec::with_capacity(imgs.capacity());
    for i in imgs {
        let mut buffer = Cursor::new(Vec::new());
        let _ = i.write_to(&mut buffer, ImageFormat::Png);
        let encoded_image = buffer.get_ref().clone();
        encoded.push(STANDARD.encode(encoded_image));
    }
    json!({ "images": encoded })
}

pub async fn minutes_listened(Path(username): Path<String>) -> Json<Value> {
    println!("{}", username);
    let lfm_client = lfm::init_client(&username);
    let spotify_client = spotify::auth().await;

    let total = calculate_year(lfm_client, &spotify_client).await;
    let total_minutes = ((total.values().sum::<i64>()) / 1000) / 60;
    let busiest = largest_value_hashmap(&total);
    let busiest_time = (busiest[1] / 1000) / 60;

    let img = imageprocessing::minutes_listened(total_minutes, busiest[0], busiest_time).unwrap();
    Json(img_mins_to_response(img, total_minutes))
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

    let song_cover_info =
        spotify::find_song_cover(&spotify_client, top_track, &top_track_name).await;
    let song_cover_url = song_cover_info["url"]
        .as_str()
        .unwrap_or("")
        .trim_matches('\"');
    let song_cover = reqwest::get(song_cover_url)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let song_cover_reader = ImageReader::new(Cursor::new(&song_cover))
        .with_guessed_format()
        .unwrap();
    let song_cover_img = song_cover_reader.decode().unwrap();

    let img = imageprocessing::top_song(
        top_track.clone(),
        *top_tracks_sorted[0].1 as i64,
        song_cover_img,
    )
    .unwrap();
    Json(img_to_response(img))
}

pub async fn top_5_songs(Path(username): Path<String>) -> Json<Value> {
    let spotify_client = spotify::auth().await;

    let top_tracks = lfm::fetch_top_5_tracks(&username).await;
    let mut top_tracks_sorted = top_tracks.iter().collect::<Vec<_>>();
    top_tracks_sorted.sort_by_key(|k| k.1);
    top_tracks_sorted.reverse();
    let mut meow = Vec::with_capacity(5);
    for song in top_tracks_sorted {
        let top_track_name = song.0.split(" - ").collect::<Vec<_>>()[1].to_string();
        let song_cover_info =
            spotify::find_song_cover(&spotify_client, song.0, &top_track_name).await;
        let song_cover_url = song_cover_info["url"]
            .as_str()
            .unwrap_or("")
            .trim_matches('\"');
        let song_cover = reqwest::get(song_cover_url)
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();
        let song_cover_reader = ImageReader::new(Cursor::new(&song_cover))
            .with_guessed_format()
            .unwrap();
        let song_cover_img = song_cover_reader.decode().unwrap();

        meow.push((song.0, (song_cover_img, song.1)));
    }

    let img = imageprocessing::top_5_songs(meow).unwrap();
    Json(img_to_response(img))
}

pub async fn genre_evolution(Path(username): Path<String>) -> Json<Value> {
    let lfm_client = lfm::init_client(&username);
    let spotify_client = spotify::auth().await;

    let months = calculate_genre_months(lfm_client, spotify_client)
        .await
        .unwrap();
    //let meow = GenreMonths::new();
    let imgs = imageprocessing::genre_evolution(months).unwrap();
    Json(imgs_to_response(imgs))
}

pub async fn final_image(Path((username, minutes)): Path<(String, i64)>) -> Json<Value> {
    println!("{}", username);
    let spotify_client = spotify::auth().await;

    let top_tracks = lfm::fetch_top_5_tracks(&username).await;
    let mut top_tracks_sorted = top_tracks.iter().collect::<Vec<_>>();
    top_tracks_sorted.sort_by_key(|k| k.1);
    top_tracks_sorted.reverse();
    let top_track_names = top_tracks_sorted
        .iter()
        .map(|(x, _)| x.split(" - ").collect_vec()[1])
        .collect::<Vec<&str>>();

    let top_artists = lfm::fetch_top_5_artists(&username).await;
    let mut top_artists_sorted = top_artists.iter().collect::<Vec<_>>();
    top_artists_sorted.sort_by_key(|k| k.1);
    top_artists_sorted.reverse();
    let top_artist_names = top_artists_sorted
        .iter()
        .map(|x| x.0.as_str())
        .collect::<Vec<&str>>();

    let icon_info =
        spotify::find_artist_icon(&spotify_client, top_artist_names[0]).await;
    let icon_url = icon_info["url"].as_str().unwrap_or("").trim_matches('\"');
    let icon = reqwest::get(icon_url).await.unwrap().bytes().await.unwrap();
    let icon_reader = ImageReader::new(Cursor::new(&icon))
        .with_guessed_format()
        .unwrap();
    let icon_img = icon_reader.decode().unwrap();

    let img =
        imageprocessing::final_image(minutes, top_track_names, top_artist_names, icon_img)
            .unwrap();
    Json(img_to_response(img))
}
