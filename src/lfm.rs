use dotenvy;
use lastfm::Client;
use serde_json::{json, Value};
use std::{collections::HashMap, env, fmt};

#[derive(Debug)]
pub enum UnprocessableErrors {
    UserNotFound,
    NotEnoughScrobbles,
}

impl fmt::Display for UnprocessableErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn get_api_key() -> String {
    let _ = dotenvy::dotenv();
    env::var("API_KEY").expect("Expected an API key in the environment")
}

pub fn init_client(username: &str) -> Client<String, &str> {
    let key = get_api_key();
    Client::builder().api_key(key).username(username).build()
}

pub async fn get_scrobble_count(client: Client<String, &str>) -> u64 {
    client.all_tracks().await.unwrap().total_tracks
}

// returns in miliseconds
pub async fn get_track_info(artist: &String, title: &String) -> Value {
    let key = get_api_key();
    let request_url = format!("http://ws.audioscrobbler.com/2.0/?method=track.getinfo&api_key={}&artist={}&track={}&format=json", key, artist, title);
    let resp_text = reqwest::get(request_url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let resp: Value =
        serde_json::from_str(&resp_text).unwrap_or(json!({ "artist": { "name": "" }, "name": "" }));
    //println!("{}", resp);
    resp
}

pub fn get_track_duration(response: &Value) -> i32 {
    let duration = response["track"]["duration"]
        .as_str()
        .unwrap_or("0")
        .parse::<i32>()
        .unwrap_or(0);
    duration
}

// tracks over 1 year
pub async fn fetch_top_5_tracks(username: &String) -> HashMap<String, i32> {
    let key = get_api_key();
    let request_url = format!("http://ws.audioscrobbler.com/2.0/?method=user.gettoptracks&user={}&api_key={}&period=12month&limit=5&format=json", username, key);
    let resp_text = reqwest::get(request_url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let resp: Value = serde_json::from_str(&resp_text).unwrap();
    let mut tracks = HashMap::with_capacity(5);
    for t in resp["toptracks"]["track"].as_array().unwrap() {
        let track_name = format!("{} - {}", t["artist"]["name"], t["name"]).replace("\"", "");
        let playcount = t["playcount"]
            .to_string()
            .trim_matches('\"')
            .parse::<i32>()
            .unwrap_or(0);
        tracks.insert(track_name, playcount);
    }
    tracks
}

// artists over 1 year
pub async fn fetch_top_5_artists(username: &String) -> HashMap<String, i32> {
    let key = get_api_key();
    let request_url = format!("http://ws.audioscrobbler.com/2.0/?method=user.gettopartists&user={}&api_key={}&period=12month&limit=5&format=json", username, key);
    let resp_text = reqwest::get(request_url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let resp: Value = serde_json::from_str(&resp_text).unwrap();
    let mut artists = HashMap::with_capacity(5);
    for t in resp["topartists"]["artist"].as_array().unwrap() {
        let artist = t["name"].as_str().unwrap().to_string();
        let playcount = t["playcount"]
            .to_string()
            .trim_matches('\"')
            .parse::<i32>()
            .unwrap_or(0);
        artists.insert(artist, playcount);
    }
    artists
}

pub async fn user_processable(username: &String) -> Result<(), UnprocessableErrors> {
    let key = get_api_key();
    let request_url = format!(
        "http://ws.audioscrobbler.com/2.0/?method=user.getinfo&user={}&api_key={}&format=json",
        username, key
    );
    let resp_text = reqwest::get(request_url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let resp: Value = serde_json::from_str(&resp_text).unwrap();
    if resp["error"] != Value::Null {
        if resp["error"].as_i64().unwrap_or(0) == 6 {
            return Err(UnprocessableErrors::UserNotFound);
        }
    } else if resp["playcount"].to_string().parse::<i32>().unwrap_or(0) < 365 {
        return Err(UnprocessableErrors::NotEnoughScrobbles);
    }

    Ok(())
}
