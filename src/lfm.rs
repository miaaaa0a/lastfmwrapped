use lastfm::Client;
use dotenvy;
use std::{collections::HashMap, env};
use serde_json::{json, Value};

fn get_api_key() -> String {
    let _ = dotenvy::dotenv();
    let key = env::var("API_KEY").expect("Expected an API key in the environment");
    return key;
}
pub fn init_client(username: &str) -> Client<String, &str> {
    let key = get_api_key();
    let client = Client::builder().api_key(key).username(username).build();
    return client;
}

pub async fn get_scrobble_count(client: Client<String, &str>) -> u64 {
    let scrobbles = client.all_tracks().await.unwrap().total_tracks;
    return scrobbles;
}

// returns in miliseconds
pub async fn get_track_info(artist: &String, title: &String) -> Value {
    let key = get_api_key();
    let request_url = format!("http://ws.audioscrobbler.com/2.0/?method=track.getinfo&api_key={}&artist={}&track={}&format=json", key, artist, title);
    let resp_text = reqwest::get(request_url).await.unwrap().text().await.unwrap();
    let resp: Value = serde_json::from_str(&resp_text).unwrap_or(json!({ "artist": { "name": "" }, "name": "" }));
    //println!("{}", resp);
    return resp;
}

pub fn get_track_duration(response: &Value) -> i32 {
    let duration = response["track"]["duration"].as_str().unwrap_or("0").parse::<i32>().unwrap_or(0);
    return duration;
}

// tracks over 1 year
pub async fn fetch_top_5_tracks(username: &String) -> HashMap<String, i32> {
    let key = get_api_key();
    let request_url = format!("http://ws.audioscrobbler.com/2.0/?method=user.gettoptracks&user={}&api_key={}&period=12month&limit=5&format=json", username, key);
    let resp_text = reqwest::get(request_url).await.unwrap().text().await.unwrap();
    let resp: Value = serde_json::from_str(&resp_text).unwrap();
    let mut tracks = HashMap::with_capacity(5);
    for t in resp["toptracks"]["track"].as_array().unwrap() {
        let track_name = format!("{} - {}", t["artist"]["name"], t["name"]).replace("\"", "");
        let playcount = t["playcount"].to_string().trim_matches('\"').parse::<i32>().unwrap_or(0);
        tracks.insert(track_name, playcount);
    }
    return tracks;
}

// artists over 1 year
pub async fn fetch_top_5_artists(username: &String) -> HashMap<String, i32> {
    let key = get_api_key();
    let request_url = format!("http://ws.audioscrobbler.com/2.0/?method=user.gettopartists&user={}&api_key={}&period=12month&limit=5&format=json", username, key);
    let resp_text = reqwest::get(request_url).await.unwrap().text().await.unwrap();
    let resp: Value = serde_json::from_str(&resp_text).unwrap();
    let mut artists = HashMap::with_capacity(5);
    for t in resp["topartists"]["artist"].as_array().unwrap() {
        let artist = t["name"].as_str().unwrap().to_string();
        let playcount = t["playcount"].to_string().trim_matches('\"').parse::<i32>().unwrap_or(0);
        artists.insert(artist, playcount);
    }
    return artists;
}