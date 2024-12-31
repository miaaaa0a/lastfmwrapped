pub mod api;
pub mod calculations;
pub mod defaults;
pub mod imageprocessing;
pub mod lfm;
pub mod spotify;
#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![
            api::minutes_listened,
            api::top_song,
            api::top_5_songs,
            api::genre_evolution,
            api::final_image,
            api::user_processable
        ],
    )
}
