use crate::{lfm, spotify};
use lastfm::Client;
use rspotify::ClientCredsSpotify;
use std::{collections::HashMap, error::Error, fs};
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use serde_json::{json, Value};
use chrono::{Datelike, Local, Months, TimeZone, Utc};
use tqdm::tqdm;

enum CacheType {
    Duration,
    Genre
}

fn load_cache(ctype: CacheType) -> Value {
    let cache_text;
    match ctype {
        CacheType::Duration => {
            cache_text = fs::read_to_string("duration.json").unwrap_or("{}".to_string());
        },
        CacheType::Genre => {
            cache_text = fs::read_to_string("genre.json").unwrap_or("{}".to_string());
        }
    }
    let cache: Value = serde_json::from_str(&cache_text).unwrap();
    return cache;
}

fn save_cache(cache: Value, ctype: CacheType) {
    let cache_text = serde_json::to_string_pretty(&cache).unwrap();
    match ctype {
        CacheType::Duration => {
            let _ = fs::write("duration.json", cache_text);
        },
        CacheType::Genre => {
            let _ = fs::write("genre.json", cache_text);
        }
    }
}

pub fn largest_value_hashmap(hm: &HashMap<i64, i64>) -> Vec<i64> {
    let mut largest = 0;
    let mut largest_key = 0;
    for (i, j) in hm {
        if *j > largest {
            largest = *j;
            largest_key = *i;
        }
    }
    return vec![largest_key, largest];
}

pub fn largest_value(v: Value) -> Vec<String> {
    let mut largest = 0;
    let mut largest_key = String::new();
    for (i, j) in v.as_object().unwrap() {
        if j.as_i64().unwrap_or(0) > largest {
            largest_key = i.clone();
            largest = j.as_i64().unwrap_or(0);
        }
    }

    return vec![largest_key, largest.to_string()];
}

pub fn sort_value(v: Value) -> Vec<(String, String)> {
    let mut vvec = Vec::with_capacity(v.as_object().unwrap().len());
    for (i, j) in v.as_object().unwrap() {
        vvec.push((i.clone(), j.to_string()));
    }

    vvec.sort_by(|a, b| 
        b.1
        .parse::<i32>()
        .unwrap_or(0)
        .cmp(
            &(a.1.parse::<i32>()
            .unwrap_or(0) as i32)
        )
    );

    vvec
}

async fn calculate_scrobble_time(lfm_client: Client<String, &str>, spotify_client: ClientCredsSpotify, from: i64, to: i64) -> Result<i32, Box<dyn Error>> {
    let track_stream = lfm_client.recent_tracks(Some(from), Some(to)).await?.into_stream();
    let mut total_seconds: i32 = 0;
    let mut cached_durs = load_cache(CacheType::Duration);
    // add item for errored out tracks
    cached_durs[""] = json!(0);
    pin_mut!(track_stream);
    while let Some(i) = track_stream.next().await {
        match i {
            Ok(t) => {
                let track_name = format!("{} - {}", t.artist.name, t.name);
                let mut dur;
                if cached_durs[&track_name] == Value::Null {
                    let info = lfm::get_track_info(&t.artist.name, &t.name).await;
                    dur = lfm::get_track_duration(&info);
                    if dur == 0 {
                        dur = spotify::find_song_duration(&spotify_client, &track_name, &t.name).await.unwrap_or(0) as i32;
                    }
                    cached_durs[&track_name] = json!(dur);
                } else {
                    dur = cached_durs[&track_name].as_i64().unwrap() as i32;
                }
                total_seconds += dur;
            },
            Err(_e) => {
                //println!("error: {:?}", e);
            }
        }
    }
    //let total_minutes: i32 = (total_seconds / 1000) / 60;
    save_cache(cached_durs, CacheType::Duration);
    Ok(total_seconds)
}

async fn calculate_top_genres(lfm_client: Client<String, &str>, spotify_client: ClientCredsSpotify, from: i64, to: i64) -> Result<Vec<(String, Vec<Value>)>, Box<dyn Error>> {
    let track_stream = lfm_client.recent_tracks(Some(from), Some(to)).await?.into_stream();
    let mut cached_genres = load_cache(CacheType::Duration);
    let mut artist_scrobbles: Value = Default::default();
    // add item for errored out tracks
    cached_genres[""] = json!(0);
    pin_mut!(track_stream);
    while let Some(i) = track_stream.next().await {
        match i {
            Ok(t) => {
                if cached_genres[&t.artist.name] == Value::Null {
                    let genres = spotify::find_artist_genres(&spotify_client, &t.artist.name).await;
                    cached_genres[&t.artist.name] = genres;
                }
                artist_scrobbles[&t.artist.name] = json!(artist_scrobbles[&t.artist.name].as_i64().unwrap_or(0) + 1);
            },
            Err(e) => {
                println!("error: {:?}", e);
            }
        }
    }

    let mut top_by_scrobble = sort_value(artist_scrobbles);
    let mut top_genres: Vec<(String, Vec<Value>)> = Vec::with_capacity(5);
    top_by_scrobble.drain(3..top_by_scrobble.len());
    for (i, _) in top_by_scrobble {
        top_genres.push(
            (i.clone(), cached_genres[i].as_array().unwrap().clone())
        );
    }

    save_cache(cached_genres, CacheType::Genre);
    Ok(top_genres)
}


pub async fn calculate_year(lfm_client: Client<String, &str>, spotify_client: ClientCredsSpotify) -> HashMap<i64, i64> {
    let now = Local::now();
    //let yearago = now.checked_sub_months(Months::new(12)).unwrap();
    let yearago = Utc.with_ymd_and_hms(now.year(), 1, 1, 0, 0, 0).unwrap().timestamp();

    // subtracting one since it would count 365 to 366 (or december 31st to january 1st) otherwise
    let days_in_year = if now.year() % 4 == 0 { 365 } else { 364 };
    let seconds_in_day = 24 * 60 * 60;
    let mut days: HashMap<i64, i64> = HashMap::with_capacity(days_in_year as usize);
    for i in tqdm(1..days_in_year) {
        let from_ts = yearago + (i * seconds_in_day);
        let to_ts = yearago + ((i+1) * seconds_in_day);
        let time = calculate_scrobble_time(lfm_client.clone(), spotify_client.clone(), from_ts, to_ts).await.unwrap_or(0) as i64;
        days.insert(
            from_ts, 
            if time > 24 * 60 * 60 * 1000 { 0 } else { time }
        );
        //println!("{}", i);
    }
    return days;
}


// Vec<Vec<(String, Vec<Value>)>>
pub async fn calculate_genre_months(lfm_client: Client<String, &str>, spotify_client: ClientCredsSpotify) -> Result<Vec<Vec<(String, Vec<Value>)>>, Box<dyn Error>> {
    let now = Local::now().naive_utc();
    let yearago = now.checked_sub_months(Months::new(12)).unwrap();
    // january, march, june, september
    let mut months = Vec::with_capacity(4);

    for i in tqdm(0..4) {
        let from_ts = yearago.checked_add_months(Months::new(i * 3)).unwrap().and_utc().timestamp();
        let to_ts = yearago.checked_add_months(Months::new((i * 3) + 1)).unwrap().and_utc().timestamp();
        months.push(calculate_top_genres(lfm_client.clone(), spotify_client.clone(), from_ts, to_ts).await?);
    }

    Ok(months)
}