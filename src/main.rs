use axum::{routing::get, Router};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
pub mod api;
pub mod calculations;
pub mod defaults;
pub mod imageprocessing;
pub mod lfm;
pub mod spotify;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cors = CorsLayer::new().allow_origin(Any);
    let router = Router::new()
        .route("/api/minuteslistened/:username", get(api::minutes_listened))
        .route("/api/topsong/:username", get(api::top_song))
        .route("/api/top5songs/:username", get(api::top_5_songs))
        .route("/api/genreevolution/:username", get(api::genre_evolution))
        .route("/api/finalimage/:username/:minutes", get(api::final_image))
        .layer(cors);
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let tcp = TcpListener::bind(&addr).await?;
    axum::serve(tcp, router).await?;
    Ok(())
}
