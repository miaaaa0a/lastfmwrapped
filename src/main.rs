pub mod lfm;
pub mod calculations;
pub mod spotify;
use chrono::{Month, TimeZone, Utc};
use log::info;
use env_logger;
//use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() {
    env_logger::init();
    let client = lfm::init_client("redoverflow");
    info!("last.fm client init");
    let spotify = spotify::auth().await;
    info!("spotify client init");
    //let scrobbles = lfm::get_scrobble_count(client.clone()).await;
    //println!("scrobbles: {}", scrobbles);
    //let to = i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()).unwrap();
    //let from = i64::try_from(to - 365 * 24 * 60 * 60).unwrap();
    //let from = i64::try_from(to - 24 * 60 * 60).unwrap();
    //let total_minutes = calculations::calculate_total_minutes(client.clone(), spotify, from, to).await;
    //println!("{}", total_minutes);
    //spotify::find_song(spotify, "car seat headrest - sober to death".to_string()).await;
    //let total = calculations::calculate_year(client, spotify).await;
    //println!("each month: {:?}\n total: {}", total, total.iter().sum::<i32>());
    //let largest = calculations::largest_value_hashmap(total);
    //println!("busiest day: {} (seconds: {} ({}m))", Utc.timestamp_opt(largest[0], 0).unwrap().to_rfc2822(), largest[1] / 1000, (largest[1] / 1000) / 60);
    //let tracks = lfm::fetch_top_5_tracks(&"redoverflow".to_string()).await;
    //println!("{:?}", tracks);
    let genres = calculations::calculate_genre_months(client, spotify).await.unwrap();
    //println!("{:?}", genres);
    for i in 0..4 {
        println!("{}: {:?}", Month::try_from(i*3).unwrap_or(Month::January).name(), genres.get(i as usize));
    }
}
