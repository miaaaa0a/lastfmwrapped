use std::{error::Error, fs};
use serde_json::json;
use chrono::{TimeZone, Utc};
pub mod lfm;
pub mod calculations;
pub mod spotify;
pub mod imageprocessing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let lfm_client = lfm::init_client("redoverflow");
    let spotify_client = spotify::auth().await;
    let total = calculations::calculate_year(lfm_client, spotify_client).await;
    //println!("each month: {:?}\n total: {}", total, total.iter().sum::<i64>());
    let largest = calculations::largest_value_hashmap(&total);
    println!("busiest day: {} (seconds: {} ({}m))", Utc.timestamp_opt(largest[0], 0).unwrap().to_rfc2822(), largest[1] / 1000, (largest[1] / 1000) / 60);
    let mut total_time: i64 = 0;
    for i in total.values() {
        total_time += i;
    }
    println!("total: {}s ({}m)", total_time / 1000, (total_time / 1000) / 60);
    //let total_value = json!(total);
    //let total_str = serde_json::to_string_pretty(&total_value).unwrap_or("".to_string());
    //let _ = fs::write("total.json", total_str);
    let tm_img = imageprocessing::minutes_listened(((total_time / 1000) / 60).to_string())?;
    tm_img.save("meow.png")?;
    Ok(())
}
