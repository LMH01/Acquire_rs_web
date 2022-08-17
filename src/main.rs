use std::sync::RwLock;

use game::GameManager;
use request_data::EventData;
use rocket::{
    fs::{relative, FileServer},
    launch, routes, tokio::sync::broadcast::channel,
};

use crate::requests::*;

/// The underlying game, contains logic and components that are required to run the game
mod game;
/// Different data types that are required to process requests
mod request_data;
/// Different data types that are required to authenticate users and requests
mod authentication;
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
        .mount("/", routes![events, lobby, lobby_join, create_game, create_game_without_ip, players_in_game, debug])
        .manage(RwLock::new(GameManager::new()))
        .manage(channel::<EventData>(1024).0)
}

// TODO
//
// Das private cookie jar als authentication testen
/*
 * Verhalten bei disconnect/ Seite neu laden:
 * 
 * Lädt man die Spielseite neu, wird man zurück zum lobby screen für die game id gebracht
 * Dort kann man seinen Nutzernamen noch einmal eingeben und wird wieder zum Spiel weitergeleitet, wenn die Ip-Adresse und der Nutzername stimmen.
 * Der Nutzer bekommt seine Nutzer-Id dann wieder zurück, um sich wieder beim Server authentifizieren zu können. 
 * 
 * Lobby screen (wenn man eine Spezielle Lobby für die game_id anfordert), folgende Möglichkeiten gibt es:
 * 1. Lobby existiert nicht
 * 2. Lobby existiert und Spiel hat noch nicht begonnen, dann kann man sich einen Nutzernamen raus suchen (keine Duplikate) und wird in die Lobby gepackt, 
 *      an dieser Stelle bekommt man die Userid und gilt als registriert am Server
 * 3. Lobby existiert und Spiel hat begonnen, in diesem Fall kann man einen Nutzernamen eingeben und der Server überprüft folgendes:
 *      1. Ist ein Nutzer mit diesem Nutzernamen registriert
 *      2. Stimmen die Ip-Adressen von dem registrierten Nutzer und dem neuen Nutzer überein?
 *      (3. Ist der SSE Stream von dem registrierten Nutzer abgebrochen?)
 *      4. Ist die Antwort auf all diese Fragen ja, dann bekommt der neue Nutzer die User id von dem alten Nutzer und wird auf die Spielseite weitergeleitet
 */
