use crate::defaults::Defaults;
use dotenvy;
use rspotify::{model::SearchType, prelude::*, ClientCredsSpotify, Credentials};
use serde_json::{self, json, Value};
use std::str::FromStr;

pub async fn auth() -> ClientCredsSpotify {
    let _ = dotenvy::dotenv();
    let creds = Credentials::from_env().unwrap();
    let spotify = ClientCredsSpotify::new(creds);
    spotify.request_token().await.unwrap();
    spotify
}

pub async fn find_song_duration(c: &ClientCredsSpotify, q: &str, name: &str) -> Option<i64> {
    let value = c
        .search(q, SearchType::Track, None, None, Some(1), None)
        .await;
    // incredible error handling
    let search_result = serde_json::to_value(match value.as_ref().err() {
        Some(_) => return Some(0_i64),
        None => value.unwrap(),
    })
    .unwrap();
    //println!("{} - {}", search_result["tracks"]["items"][0]["artists"][0]["name"], search_result["tracks"]["items"][0]["name"]);
    return match search_result["tracks"]["items"][0]["name"]
        .as_str()
        .unwrap()
        .to_lowercase()
        == name.to_lowercase()
    {
        true => {
            //println!("matches");
            search_result["tracks"]["items"][0]["duration_ms"].as_i64()
        }
        false => Some(0_i64),
    };
    //println!("{:?}", search_result["tracks"]["items"][0]["duration_ms"]);
    //return search_result["tracks"]["items"][0]["duration_ms"].as_i64();
}

pub async fn find_artist_genres(c: &ClientCredsSpotify, q: &str) -> Value {
    let search_result = serde_json::to_value(loop {
        if let Ok(result) = c
            .search(q, SearchType::Artist, None, None, Some(1), None)
            .await
        {
            break result;
        }
    })
    .unwrap();
    return match search_result["artists"]["items"][0]["name"]
        .as_str()
        .unwrap_or("")
        .to_lowercase()
        == q.to_lowercase()
    {
        true => {
            //println!("matches");
            search_result["artists"]["items"][0]["genres"].clone()
        }
        false => {
            json!(vec![""])
        }
    };
}

pub async fn find_song_cover(c: &ClientCredsSpotify, q: &str, name: &str) -> Value {
    let search_result = serde_json::to_value(loop {
        if let Ok(result) = c
            .search(q, SearchType::Track, None, None, Some(1), None)
            .await
        {
            break result;
        }
    })
    .unwrap();
    //println!("{} - {}", search_result["tracks"]["items"][0]["artists"][0]["name"], search_result["tracks"]["items"][0]["name"]);
    return match search_result["tracks"]["items"][0]["name"]
        .as_str()
        .unwrap()
        .to_lowercase()
        == name.to_lowercase()
    {
        true => {
            //println!("matches");
            search_result["tracks"]["items"][0]["album"]["images"][0].clone()
        }
        false => Value::from_str(Defaults::BLACK_IMAGE).unwrap(),
    };
    //println!("{:?}", search_result["tracks"]["items"][0]["duration_ms"]);
    //return search_result["tracks"]["items"][0]["duration_ms"].as_i64();
}

pub async fn find_artist_icon(c: &ClientCredsSpotify, q: &str) -> Value {
    let search_result = serde_json::to_value(loop {
        if let Ok(result) = c
            .search(q, SearchType::Artist, None, None, Some(1), None)
            .await
        {
            break result;
        }
    })
    .unwrap();
    if search_result["artists"]["items"][0]["name"]
        .as_str()
        .unwrap()
        .to_lowercase()
        == q.to_lowercase()
    {
        search_result["artists"]["items"][0]["images"][0].clone()
    } else {
        Value::from_str(Defaults::BLACK_IMAGE).unwrap()
    }
}
