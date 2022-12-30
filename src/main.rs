use std::sync::RwLock;

use game::GameManager;
use request_data::EventData;
use rocket::{
    fs::{relative, FileServer},
    launch, routes, tokio::sync::broadcast::channel,
};

use crate::paths::*;

/// The underlying game, contains logic and components that are required to run the game.
mod game;
/// Different data types that are required to process requests.
mod request_data;
/// Different data types that are required to authenticate users and requests.
mod authentication;
/// All paths for which a request handler is registered.
///
/// All requests that interact with games requires the request guard [UserAuth](../authentication/struct.UserAuth.html) to succeed.
/// 
/// # Request Guards
/// The following [Request Guards](../../rocket/request/trait.FromRequest.html#request-guards) are used to ensure that incoming requests are valid.
/// 
/// - [UserAuth](../authentication/struct.UserAuth.html)
/// - [GameCode](../game/game_instance/struct.GameCode.html)
/// 
/// When a [Request Guard](../../rocket/request/trait.FromRequest.html#request-guards) is provided in a function as parameter it is expected that all fields contained within are valid and can be used without further checks.
///
/// For information on what it means for a specific [Request Guard](../../rocket/request/trait.FromRequest.html#request-guards) to pass see the designated doc page.
/// 
/// For more information on authentication see [here](../authentication/index.html).
mod paths;

#[launch]
/// Start the web server
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from(relative!("web/public")))
        .mount("/", routes![events, lobby, lobby_join, game_page, create_game, create_game_without_ip, join_game, leave_game, join_game_without_ip, players_in_game, debug, debug_busy, debug_game])
        .manage(RwLock::new(GameManager::new()))
        .manage(channel::<EventData>(1024).0)
}

/* TODO Als nächstes:
    - Nutzerliste bei verlassen von Spieler überall aktualisieren und Spieler, die gerade als nicht verbunden markiert sind sollen nicht angezeigt werden.
    - Schauen, dass der Leave game Knopf im Browser richtig funktioniert (Request scheint aktuell nicht gesendet zu werden)
    - Generell bei der Lobby-Seite weiter machen
*/

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

 /*
    More todos

    - replace regaining of user session through ip address with placed cookie, that is used to regain the session when connection is lost.
    - Make all links in the documentation work.
 */