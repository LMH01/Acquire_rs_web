use std::{path::Path, sync::RwLock, net::IpAddr, time::Duration, thread};

use rocket::{
    fs::NamedFile,
    get,
    log::private::info,
    State, response::{Redirect, stream::{EventStream, Event}}, serde::json::Json, post, Shutdown, tokio::sync::broadcast::Sender,
    tokio::{sync::broadcast::error::RecvError, select}, http::{CookieJar, Cookie},
};
use uuid::Uuid;

use crate::{game::{GameManager, disconnect_user, UserDisconnectedStatus, game_instance::GameCode, UserRegistrationError}, request_data::{UserRegistration, Username, EventData}, authentication::{UserAuth, UserRecovery}};

use self::utils::{get_gm_read_guard, get_gm_write_guard};

#[get("/lobby")]
pub async fn lobby() -> Option<NamedFile> {
    NamedFile::open(Path::new("web/protected/lobby.html"))
        .await
        .ok()
}

#[get("/lobby/<game_code>")]
pub async fn lobby_join(game_manager: &State<RwLock<GameManager>>, game_code: &str) -> Result<Option<NamedFile>, Redirect> {
    let game_code = match GameCode::from_string(game_code) {
        Some(code) => code,
        None => return Err(Redirect::to("/lobby")),
    };
    if get_gm_read_guard(game_manager, "lobby_join").does_game_exist(&game_code) {
        Ok(NamedFile::open(Path::new("web/protected/lobby.html"))
            .await
            .ok())
    } else {
        Err(Redirect::to("/lobby"))
    }
}

#[get("/lobby/<game_code>/game")]
pub async fn game_page(game_manager: &State<RwLock<GameManager>>, game_code: &str) -> Result<Option<NamedFile>, Redirect> {
    let game_code = match GameCode::from_string(game_code) {
        Some(code) => code,
        None => return Err(Redirect::to(String::from("/lobby/"))),
    };
    if get_gm_read_guard(game_manager, "game_page").does_game_exist(&game_code) {
        Ok(NamedFile::open(Path::new("web/protected/game.html"))
            .await
            .ok())
    } else {
        Err(Redirect::to(String::from("/lobby/")))
    }
}

/// 
/// # Requires
/// The user needs to send a username formatted in a json string in the post request body.
#[post("/api/create_game", data = "<username>", rank = 1)]
pub fn create_game(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>, username: Json<Username<'_>>) -> Option<Json<UserRegistration>> {
    let mut game_manager = get_gm_write_guard(game_manager, "create_game");
    match game_manager.create_game(String::from(username.username)) {
        Some(registration) => {
            // Set recovery cookie
            cookies.add(Cookie::new("urid", registration.urid.value().to_string()));
            Some(Json(registration))}
            ,
        None => None,
    }
}

/// 
/// # Requires
/// The user needs to send a username formatted in a json string in the post request body.
#[post("/api/join_game", data = "<username>", rank = 2)]
pub fn join_game(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>, event: &State<Sender<EventData>>, username: Json<Username<'_>>, game_code: GameCode) -> Result<Json<UserRegistration>, UserRegistrationError> {
    let mut game_manager = get_gm_write_guard(game_manager, "join_game");
    match game_manager.add_player_to_game(event, game_code, String::from(username.username), None) {
        Ok(registration) => {
            // Set recovery cookie
            cookies.add(Cookie::new("urid", registration.urid.value().to_string()));
            Ok(Json(registration))}
            ,
        Err(err) => Err(err),
    }
}

/// 
/// # Requires
/// The user needs to send a username formatted in a json string in the post request body.
#[post("/api/join_game", data = "<username>", rank = 1)]
pub fn join_game_recovery(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>, event: &State<Sender<EventData>>, username: Json<Username<'_>>, game_code: GameCode, ur: UserRecovery) -> Result<Json<UserRegistration>, UserRegistrationError> {
    let mut game_manager = get_gm_write_guard(game_manager, "join_game");
    let mut ur = ur.clone();
    ur.name = Some(String::from(username.username));
    match game_manager.add_player_to_game(event, game_code, String::from(username.username), Some(ur)) {
        Ok(registration) => {
            // Set recovery cookie
            cookies.add(Cookie::new("urid", registration.urid.value().to_string()));
            Ok(Json(registration))}
            ,
        Err(err) => Err(err),
    }
}

/// Makes the user leave the game where they are assigned to.
/// 
/// An event is then send to all other players in the game to notify them that the player left.
/// 
/// When the last player disconnects using this function, the game is deleted instantly, without waiting for a reconnect.
/// # Requires
/// Request guard [UserAuth]() to succeed.
#[post("/api/leave_game")]
pub fn leave_game(game_manager: &State<RwLock<GameManager>>, event: &State<Sender<EventData>>, user_auth: UserAuth) -> Json<String> {
    match disconnect_user(game_manager, user_auth, true) {
        UserDisconnectedStatus::GameAlive => {
            let _e = event.send(EventData::new(None, user_auth.game_code, (String::from("ReloadPlayerList"), None)));
            Json::from(String::from("User marked as disconnected"))
        },
        _ => Json::from(String::from("User marked as disconnected"))
    }
} 

/// Return the games players as json string.
/// 
/// # Requires
/// - `game_code` header with valid [GameCode](../game/struct.GameCode.html)
#[get("/api/players_in_game")]
pub fn players_in_game(game_manager: &State<RwLock<GameManager>>, game_code: GameCode) -> Json<Vec<String>> {
    let game_manager = get_gm_read_guard(game_manager, "players_in_game");
    info!("{}", game_code.to_string());
    Json(game_manager.players_in_game(game_code).unwrap())
}

/// Server send events
/// 
/// For each game and user a separate sse stream exists, these streams are accessed by submitting a get request to `/sse/<game_code>/<user_id>`.
/// 
/// This makes it possible to have multiple games run in parallel without interferences in the sse streams.
/// 
/// Only sse events that match the `game_code` and `user_id` will be transmitted back.
#[get("/sse/<game_code>/<user_id>")]
pub fn events<'a>(event: &'a State<Sender<EventData>>, game_manager: &'a State<RwLock<GameManager>>, mut end: Shutdown, game_code: String, user_id: Uuid) -> Option<EventStream![Event + 'a]> {
    let mut rx = event.subscribe();
    match UserAuth::from_uuid(get_gm_read_guard(game_manager, "user_auth for sse event"), user_id) {
        Some(user_auth) => {
            // Mark user as connected
            get_gm_write_guard(game_manager, "Set user connected").game_by_code_write(user_auth.game_code).unwrap().user_connected(user_id);
            Some(EventStream! {
                loop {
                    //TODO Find out how I can reliably call user_disconnected(game_manager.inner(), user_id); each time a user disconnects from the event stream
                    /*Workaround that could work: 
                        Create new route named /ping.
                        This function here sends a ping request every couple of seconds (maybe 30).
                        The client will receive that and send a new get request to /ping/<user_id>.
                        This route handler will then somehow determine if a request was missing 
                        (maybe this could be realized by using Receiver and Sender from the Crossbeam crate (https://docs.rs/crossbeam/latest/crossbeam/channel/index.html.
                            This tuple is then put into a request guard that is provided to the routes /sse/<game_code>/<user_id> and /ping/<user_id>.
                            This tuple is used to notify the ping request handler that a request should be arriving soon.
                            From there the absence of that could be counted and user_disconnect can then be invoked appropriately)
                        */
                    let msg = select! {
                        msg = rx.recv() => match msg {
                            Ok(msg) => msg,
                            Err(RecvError::Closed) => {
                                info!("User disconnected {}", user_id);
                                disconnect_user(game_manager.inner(), user_auth, false);
                                break
                            },
                            Err(RecvError::Lagged(_)) => continue,
                        },
                        _ = &mut end => {
                            info!("End: User disconnected {}", user_id);
                            break
                        },
                    };
                    let msg_game_code = msg.game_code();
                    let msg_user_id = msg.user_id();
                    if msg_game_code == user_auth.game_code.to_string() && ((msg_user_id == user_id.to_string()) || msg_user_id == "") {
                        yield Event::json(&msg);
                    }
                }
            })
        },
        None => None,
    }
}

#[get("/api/debug/<user_id>")]
pub fn debug(game_manager: &State<RwLock<GameManager>>, ip_addr: IpAddr, event: &State<Sender<EventData>>, user_id: Uuid) -> String {
    let auth = UserAuth::from_uuid(get_gm_read_guard(game_manager, ""), user_id).unwrap();
    let status = disconnect_user(game_manager, auth, false);
    String::from(format!("{:?}", status))
}

/// Acquires the game_manager lock and releases it again after 10 seconds.
/// 
/// After another `time` seconds the lock is reacquired and held for 5 more seconds.
/// 
/// This can be used to check behavior of other function when the `game_manager` lock could not be acquired.
#[get("/api/debug/keep_busy/<id>/<time>")]
pub fn debug_busy(game_manager: &State<RwLock<GameManager>>, id: i32, time: i32) -> String {
    info!("Starting debug {}", id);
    {
        let mut manager = match game_manager.try_write() {
            Ok(manager) => {
                manager
            },
            Err(_err) => {
                info!("Debug {}: Write lock for game manager could not be acquired, waiting...", id);
                game_manager.write().unwrap()
            }
        };
        info!("Debug {}: Acquired write lock for game manger", id);
        for i in (1..=10).rev() {
            info!("Debug {}: Releasing lock in: {} ", id, i);
            thread::sleep(Duration::from_secs(1));
        }
    }
    info!("Debug {}: Releasing lock for game manager", id);
    for i in (1..=time).rev() {
        info!("Debug {}: Seconds left of free game lock: {} ", id, i);
        thread::sleep(Duration::from_secs(1));
    }
    {
        let mut manager = match game_manager.try_write() {
            Ok(manager) => {
                manager
            },
            Err(_err) => {
                info!("Debug {}: Write lock for game manager could not be acquired, waiting...", id);
                game_manager.write().unwrap()
            }
        };
        info!("Debug {}: Acquired write lock for game manger", id);
        for i in (1..=5).rev() {
            info!("Debug {}: Releasing lock in: {} ", id, i);
            thread::sleep(Duration::from_secs(1));
        }
    }
    String::from("Success")
}

#[get("/api/debug/game")]
pub async fn debug_game() -> Option<NamedFile> {
    NamedFile::open(Path::new("web/protected/game.html"))
        .await
        .ok()
}

/// Some utility functions
pub mod utils {
    use std::sync::{RwLockWriteGuard, RwLock, RwLockReadGuard};

    use rocket::log::private::info;

    use crate::{
        game::{game_instance::GameInstance, GameManager},
        authentication::UserAuth,
    };

    /// Tries to acquire the game_manager read/write lock.
    /// 
    /// If successful the game_manager is returned.
    /// 
    /// Otherwise the following message is send to console: 
    /// 
    /// `{action}: Waiting for game_manager write lock...`
    /// 
    /// After that the game_manager is returned when the write lock can be acquired.
    pub fn get_gm_write_guard<'a>(game_manager: &'a RwLock<GameManager>, action: &'a str) -> RwLockWriteGuard<'a, GameManager> {
        match game_manager.try_write() {
            Ok(manager) => manager,
            Err(_err) => {
                info!("{}: Waiting for game_manager write lock...", action);
                game_manager.write().unwrap()
            }
        }
    }

    /// Tries to acquire the game_manager write lock.
    /// 
    /// If successful the game_manager is returned.
    /// 
    /// Otherwise the following message is send to console: 
    /// 
    /// `{action}: Waiting for game_manager write lock...`
    /// 
    /// After that the game_manager is returned when the write lock can be acquired.
    pub fn get_gm_read_guard<'a>(game_manager: &'a RwLock<GameManager>, action: &'a str) -> RwLockReadGuard<'a, GameManager> {
        match game_manager.try_read() {
            Ok(manager) => manager,
            Err(_err) => {
                info!("{}: Waiting for game_manager read lock...", action);
                game_manager.read().unwrap()
            }
        }
    }
}
