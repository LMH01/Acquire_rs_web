use std::sync::RwLock;

use game::GameManager;
use rocket::{
    fs::{relative, FileServer},
    launch, routes,
};

use crate::requests::*;

/// The underlying game, contains logic and components that are required to run the game
mod game;
/// Different data types that are required to process requests
mod request_data;
/// All requests that the server can handle
///
/// All requests that interact with games require a player authentication that is set when the player registers for a game.
/// This authentication is done by setting a cookie that is checked each time the player interacts with the server endpoints.
/// When the cookie is invalid or not set the connection is refused.
mod requests;

#[launch]
/// Start the web server
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from(relative!("web/public")))
        .mount("/", routes![lobby, debug])
        .manage(RwLock::new(GameManager::new()))
}

// TODO
//
// Das private cookie jar als authentication testen
