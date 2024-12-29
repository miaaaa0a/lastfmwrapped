use crate::{lfm, spotify};
use chrono::{Datelike, Local, Months, TimeZone, Utc};
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use lastfm::Client;
use rspotify::ClientCredsSpotify;
use serde_json::{json, Value};
use std::{collections::HashMap, error::Error, fs};
use tqdm::tqdm;

enum CacheType {
    Duration,
    Genre,
}

#[derive(Clone)]
pub struct GenreMonths {
    pub january: HashMap<String, Vec<Value>>,
    pub april: HashMap<String, Vec<Value>>,
    pub august: HashMap<String, Vec<Value>>,
}

impl GenreMonths {
    pub fn new() -> Self {
        Self {
            january: HashMap::new(),
            april: HashMap::new(),
            august: HashMap::new(),
        }
    }
    pub fn set(&mut self, i: i8, v: HashMap<String, Vec<Value>>) {
        match i {
            0 => self.january = v,
            1 => self.april = v,
            2 => self.august = v,
            default => panic!(
                "outta bounds!!!!!!!!! index {} is HIGHER than 2!! >:c",
                default
            ),
        }
    }
    pub fn get(self, i: usize) -> HashMap<String, Vec<Value>> {
        match i {
            0 => self.january,
            1 => self.april,
            2 => self.august,
            default => panic!(
                "outta bounds!!!!!!!!! index {} is HIGHER than 2!! >:c",
                default
            ),
        }
    }
    pub fn get_month_string(self, i: usize) -> String {
        vec!["January", "April", "August"][i].to_string()
    }
}

impl Default for GenreMonths {
    fn default() -> Self {
        Self::new()
    }
}

fn load_cache(ctype: CacheType) -> Value {
    let cache_text = match ctype {
        CacheType::Duration => fs::read_to_string("duration.json").unwrap_or("{}".to_string()),
        CacheType::Genre => fs::read_to_string("genre.json").unwrap_or("{}".to_string()),
    };
    let cache: Value = serde_json::from_str(&cache_text).unwrap();
    cache
}

fn save_cache(cache: Value, ctype: CacheType) {
    let cache_text = serde_json::to_string_pretty(&cache).unwrap();
    match ctype {
        CacheType::Duration => {
            let _ = fs::write("duration.json", cache_text);
        }
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
    vec![largest_key, largest]
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

    vec![largest_key, largest.to_string()]
}

pub fn sort_value(v: Value) -> Vec<(String, String)> {
    let mut vvec = Vec::new();
    for (i, j) in v.as_object().unwrap() {
        vvec.push((i.clone(), j.to_string()));
    }

    vvec.sort_by(|a, b| {
        b.1.parse::<i32>()
            .unwrap_or(0)
            .cmp(&(a.1.parse::<i32>().unwrap_or(0)))
    });

    vvec
}

async fn calculate_scrobble_time(
    lfm_client: Client<String, &str>,
    spotify_client: ClientCredsSpotify,
    from: i64,
    to: i64,
) -> Result<i32, Box<dyn Error>> {
    let track_stream = lfm_client
        .recent_tracks(Some(from), Some(to))
        .await?
        .into_stream();
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
                        dur = spotify::find_song_duration(&spotify_client, &track_name, &t.name)
                            .await
                            .unwrap_or(0) as i32;
                    }
                    cached_durs[&track_name] = json!(dur);
                } else {
                    dur = cached_durs[&track_name].as_i64().unwrap() as i32;
                }
                total_seconds += dur;
            }
            Err(_e) => {
                //println!("error: {:?}", e);
            }
        }
    }
    //let total_minutes: i32 = (total_seconds / 1000) / 60;
    save_cache(cached_durs, CacheType::Duration);
    Ok(total_seconds)
}

async fn calculate_top_genres(
    lfm_client: Client<String, &str>,
    spotify_client: ClientCredsSpotify,
    from: i64,
    to: i64,
) -> Result<HashMap<String, Vec<Value>>, Box<dyn Error>> {
    let track_stream = lfm_client
        .recent_tracks(Some(from), Some(to))
        .await?
        .into_stream();
    let mut cached_genres = load_cache(CacheType::Genre);
    let mut artist_scrobbles: Value = Default::default();
    // add item for errored out tracks
    cached_genres[""] = json!(0);
    pin_mut!(track_stream);
    while let Some(i) = track_stream.next().await {
        match i {
            Ok(t) => {
                if cached_genres[&t.artist.name] == Value::Null {
                    let artist = &t.artist.name.split(&[';', ',']).collect::<Vec<&str>>()[0];
                    let genres = spotify::find_artist_genres(&spotify_client, artist).await;
                    cached_genres[&t.artist.name] = genres;
                }
                artist_scrobbles[&t.artist.name] =
                    json!(artist_scrobbles[&t.artist.name].as_i64().unwrap_or(0) + 1);
            }
            Err(_e) => {
                //println!("error: {:?}", e);
            }
        }
    }

    let mut top_by_scrobble = sort_value(artist_scrobbles);
    let mut top_genres: HashMap<String, Vec<Value>> = HashMap::with_capacity(5);
    top_by_scrobble.drain(3..top_by_scrobble.len());
    for (i, _) in top_by_scrobble {
        top_genres.insert(i.clone(), cached_genres[i].as_array().unwrap().clone());
    }

    save_cache(cached_genres, CacheType::Genre);
    Ok(top_genres)
}

pub async fn calculate_year(
    lfm_client: Client<String, &str>,
    spotify_client: &ClientCredsSpotify,
) -> HashMap<i64, i64> {
    let now = Local::now();
    //let yearago = now.checked_sub_months(Months::new(12)).unwrap();
    let yearago = Utc
        .with_ymd_and_hms(now.year(), 1, 1, 0, 0, 0)
        .unwrap()
        .timestamp();

    // subtracting one since it would count 365 to 366 (or december 31st to january 1st) otherwise
    let days_in_year = if now.year() % 4 == 0 { 365 } else { 364 };
    let seconds_in_day = 24 * 60 * 60;
    let mut days: HashMap<i64, i64> = HashMap::with_capacity(days_in_year as usize);
    for i in 1..days_in_year {
        let from_ts = yearago + (i * seconds_in_day);
        let to_ts = yearago + ((i + 1) * seconds_in_day);
        let time =
            calculate_scrobble_time(lfm_client.clone(), spotify_client.clone(), from_ts, to_ts)
                .await
                .unwrap_or(0) as i64;
        days.insert(from_ts, if time > 24 * 60 * 60 * 1000 { 0 } else { time });
        //println!("{}", i);
    }
    days
}

// Vec<Vec<(String, Vec<Value>)>>
pub async fn calculate_genre_months(
    lfm_client: Client<String, &str>,
    spotify_client: ClientCredsSpotify,
) -> Result<GenreMonths, Box<dyn Error>> {
    let now = Local::now().naive_utc();
    let yearago = now.checked_sub_months(Months::new(12)).unwrap();
    // january, april, august
    let mut months = GenreMonths::new();

    for i in 0..3 {
        let from_ts = yearago
            .checked_add_months(Months::new(i * 3))
            .unwrap()
            .and_utc()
            .timestamp();
        let to_ts = yearago
            .checked_add_months(Months::new((i * 3) + 1))
            .unwrap()
            .and_utc()
            .timestamp();
        months.set(
            i as i8,
            calculate_top_genres(lfm_client.clone(), spotify_client.clone(), from_ts, to_ts)
                .await?,
        );
    }

    Ok(months)
}
