use std::{path::Path, sync::RwLock, net::IpAddr, collections::HashMap, time::Duration, thread};

use rocket::{
    fs::NamedFile,
    get,
    http::{ContentType, CookieJar, Status},
    log::private::info,
    State, Response, response::{Redirect, stream::{EventStream, Event}}, serde::json::Json, post, Shutdown, tokio::sync::broadcast::Sender,
    tokio::{sync::broadcast::error::RecvError, select},
};

use crate::{game::{GameCode, GameManager, user_disconnected, UserDisconnectedStatus}, request_data::{UserRegistration, Username, EventData}};

use self::utils::{get_gm_read_guard, get_gm_write_guard};

#[get("/lobby")]
pub async fn lobby() -> Option<NamedFile> {
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
    if get_gm_read_guard(game_manager, "lobby_join").does_game_exist(&game_code) {
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
    let mut game_manager = get_gm_write_guard(game_manager, "create_game");
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
    let mut game_manager = get_gm_write_guard(game_manager, "create_game_without_ip");
    match game_manager.create_game(String::from(username.username), None) {
        Some(registration) => Some(Json(registration)),
        None => None,
    }
}

/// 
/// # Requires
/// The user needs to send a username formatted in a json string in the post request body.
#[post("/api/join_game", data = "<username>", rank = 1)]
pub fn join_game(game_manager: &State<RwLock<GameManager>>, event: &State<Sender<EventData>>, username: Json<Username<'_>>, ip_addr: IpAddr, game_code: GameCode) -> Option<Json<UserRegistration>> {
    let mut game_manager = get_gm_write_guard(game_manager, "join_game");
    match game_manager.add_player_to_game(event, game_code, String::from(username.username), Some(ip_addr)) {
        Some(registration) => Some(Json(registration)),
        None => None,
    }
}

/// 
/// # Requires
/// The user needs to send a username formatted in a json string in the post request body.
#[post("/api/join_game", data = "<username>", rank = 2)]
pub fn join_game_without_ip(game_manager: &State<RwLock<GameManager>>, event: &State<Sender<EventData>>, username: Json<Username<'_>>, game_code: GameCode) -> Option<Json<UserRegistration>> {
    let mut game_manager = get_gm_write_guard(game_manager, "join_game_without_ip");
    match game_manager.add_player_to_game(event, game_code, String::from(username.username), None) {
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
pub fn events<'a>(event: &'a State<Sender<EventData>>, game_manager: &'a State<RwLock<GameManager>>, mut end: Shutdown, game_code: String, user_id: i32) -> Option<EventStream![Event + 'a]> {
    let mut rx = event.subscribe();
    match GameCode::from_string(&game_code) {
        Some(code) => {
            // Mark user as connected
            get_gm_write_guard(game_manager, "Set user connected").user_connected(user_id);
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
                                user_disconnected(game_manager.inner(), user_id);
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
                    if msg_game_code == code.to_string() && ((msg_user_id == user_id) || msg_user_id == 0) {
                        yield Event::json(&msg);
                    }
                }
            })
        },
        None => None,
    }
}

#[get("/api/debug/<user_id>")]
pub fn debug(game_manager: &State<RwLock<GameManager>>, ip_addr: IpAddr, event: &State<Sender<EventData>>, user_id: i32) -> String {
    let status = user_disconnected(game_manager, user_id);
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

/// Some utility functions
mod utils {
    use std::sync::{RwLockWriteGuard, RwLock, RwLockReadGuard};

    use rocket::log::private::info;

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
