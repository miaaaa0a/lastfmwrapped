use std::{collections::HashMap, error::Error, io::Cursor};
use axum::{routing::get, Router};
use calculations::{calculate_genre_months, GenreMonths};
use futures_util::SinkExt;
use image::ImageReader;
use itertools::Itertools;
use lfm::fetch_top_5_artists;
use tower_http::cors::{Any, CorsLayer};
use std::net::SocketAddr;
use tokio::net::TcpListener;
pub mod lfm;
pub mod calculations;
pub mod spotify;
pub mod imageprocessing;
pub mod api;
pub mod defaults;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cors = CorsLayer::new()
        .allow_origin(Any);
    let router = Router::new()
        .route("/api/minuteslistened/:username", get(api::minutes_listened))
        .route("/api/topsong/:username", get(api::top_song))
        .route("/api/top5songs/:username", get(api::top_5_songs))
        .route("/api/genreevolution/:username", get(api::genre_evolution))
        .route("/api/finalimage/:username", get(api::final_image))
        .layer(cors);
    let addr = SocketAddr::from(([127,0,0,1], 8000));
    let tcp = TcpListener::bind(&addr).await?;
    axum::serve(tcp, router).await?;
    Ok(())
}
