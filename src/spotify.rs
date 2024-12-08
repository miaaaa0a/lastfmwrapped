use rspotify::{model::SearchType, prelude::*, ClientCredsSpotify, Credentials};
use serde_json::{self, json, Value};
use dotenvy;

pub async fn auth() -> ClientCredsSpotify {
    let _ = dotenvy::dotenv();
    let creds = Credentials::from_env().unwrap();
    let spotify = ClientCredsSpotify::new(creds);
    spotify.request_token().await.unwrap();
    return spotify;
}

pub async fn find_song_duration(c: &ClientCredsSpotify, q: &String, name: &String) -> Option<i64> {
    let search_result = serde_json::to_value(c.search(&q, SearchType::Track, None, None, Some(1), None).await.unwrap()).unwrap();
    //println!("{} - {}", search_result["tracks"]["items"][0]["artists"][0]["name"], search_result["tracks"]["items"][0]["name"]);
    return match search_result["tracks"]["items"][0]["name"].as_str().unwrap().to_lowercase() == name.to_lowercase() {
        true => {
            //println!("matches");
            search_result["tracks"]["items"][0]["duration_ms"].as_i64()
        },
        false => {
            Some(0 as i64)
        }
    }
    //println!("{:?}", search_result["tracks"]["items"][0]["duration_ms"]);
    //return search_result["tracks"]["items"][0]["duration_ms"].as_i64();
}

pub async fn find_artist_genres(c: &ClientCredsSpotify, q: &String) -> Value {
    let search_result = serde_json::to_value(c.search(&q, SearchType::Artist, None, None, Some(1), None).await.unwrap()).unwrap();
    return match search_result["artists"]["items"][0]["name"].as_str().unwrap_or("").to_lowercase() == q.to_lowercase() {
        true => {
            //println!("matches");
            search_result["artists"]["items"][0]["genres"].clone()
        },
        false => {
            json!(vec![""])
        }
    }
}