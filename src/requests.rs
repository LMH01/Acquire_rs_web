use std::{path::Path, sync::RwLock, net::IpAddr, collections::HashMap};

use rocket::{
    fs::NamedFile,
    get,
    http::{ContentType, CookieJar, Status},
    log::private::info,
    State, Response, response::{Redirect, stream::{EventStream, Event}}, serde::json::Json, post, Shutdown, tokio::sync::broadcast::Sender,
    tokio::{sync::broadcast::error::RecvError, select},
};

use crate::{game::{GameCode, GameManager}, request_data::{UserRegistration, Username, EventData}};

#[get("/lobby")]
pub async fn lobby(game_manager: &State<RwLock<GameManager>>) -> Option<NamedFile> {
    NamedFile::open(Path::new("web/protected/lobby.html"))
        .await
        .ok()
}

#[get("/lobby/<game_code>")]
pub async fn lobby_join(game_manager: &State<RwLock<GameManager>>, game_code: &str) -> Result<Option<NamedFile>, Redirect> {
    info!("Game code: {}", game_code);
    let game_code = match GameCode::from_string(game_code) {
        Some(code) => code,
        None => return Err(Redirect::to("/lobby")),
    };
    if game_manager.write().unwrap().does_game_exist(&game_code) {
        Ok(NamedFile::open(Path::new("web/protected/lobby.html"))
            .await
            .ok())
    } else {
        Err(Redirect::to("/lobby"))
    }
}

/// 
/// # Requires
/// The user needs to send a username formatted in a json string in the post request body.
#[post("/api/create_game", data = "<username>", rank = 1)]
pub fn create_game(game_manager: &State<RwLock<GameManager>>, username: Json<Username<'_>>, ip_addr: IpAddr) -> Option<Json<UserRegistration>> {
    let mut game_manager = game_manager.write().unwrap();
    match game_manager.create_game(String::from(username.username), Some(ip_addr)) {
        Some(registration) => Some(Json(registration)),
        None => None,
    }
}

/// 
/// # Requires
/// The user needs to send a username formatted in a json string in the post request body.
#[post("/api/create_game", data = "<username>", rank = 2)]
pub fn create_game_without_ip(game_manager: &State<RwLock<GameManager>>, username: Json<Username<'_>>) -> Option<Json<UserRegistration>> {
    let mut game_manager = game_manager.write().unwrap();
    match game_manager.create_game(String::from(username.username), None) {
        Some(registration) => Some(Json(registration)),
        None => None,
    }
}

/// Return the games players as json string.
/// 
/// # Requires
/// - `game_code` header with valid [GameCode](../game/struct.GameCode.html)
#[get("/api/players_in_game")]
pub fn players_in_game(game_manager: &State<RwLock<GameManager>>, game_code: GameCode) -> Json<Vec<String>> {
    let game_manager = game_manager.write().unwrap();
    info!("{}", game_code.to_string());
    Json(game_manager.players_in_game(game_code).unwrap())
}

/// Server send events
/// 
/// For each game a separate sse stream exists, these streams are accessed by submitting a get request to `/sse/<game_code>`.
/// 
/// This makes it possible to have multiple games run in parallel without interferences in the sse streams.
/// 
/// Only sse events that match the `game_code` will be transmitted back.
#[get("/sse/<game_code>")]
pub fn events(event: &State<Sender<EventData>>, mut end: Shutdown, game_code: String) -> Option<EventStream![]> {
    let mut rx = event.subscribe();
    match GameCode::from_string(&game_code) {
        Some(code) => {
            Some(EventStream! {
                loop {
                    let msg = select! {
                        msg = rx.recv() => match msg {
                            Ok(msg) => msg,
                            Err(RecvError::Closed) => break,
                            Err(RecvError::Lagged(_)) => continue,
                        },
                        _ = &mut end => break,
                    };
                    let msg_game_code = msg.game_code();
                    if msg_game_code == code.to_string() {
                        yield Event::json(&msg);
                    }
                }
            })
        },
        None => None,
    }
}

#[get("/api/debug/<game_code>")]
pub fn debug(game_manager: &State<RwLock<GameManager>>, ip_addr: IpAddr, event: &State<Sender<EventData>>, game_code: &str) -> String {
    let mut game_manager = game_manager.write().unwrap();
    let mut map = HashMap::new();
    map.insert(String::from("Hallo"), String::from("Welt"));
    let _e = event.send(EventData::new(0, GameCode::from_string(game_code).unwrap(), map));
    String::from(game_manager.debug().to_string())
}

/// Some utility functions
mod utils {
    use std::sync::RwLockWriteGuard;

    use crate::{
        game::{game_instance::GameInstance, GameManager},
        authentication::UserAuth,
    };

    /// Returns the game a player is assigned to by using the `player_auth`
    pub fn game_by_player_auth<'a>(
        game_manager: &'a mut RwLockWriteGuard<GameManager>,
        player_auth: UserAuth,
    ) -> Option<&'a mut GameInstance> {
        match game_manager.game_by_user_id(player_auth.user_id) {
            Some(game) => Some(game),
            None => None,
        }
    }
}
