use std::{path::Path, sync::RwLock, net::IpAddr};

use rocket::{
    fs::NamedFile,
    get,
    http::{ContentType, CookieJar, Status},
    log::private::info,
    State, Response, response::Redirect,
};

use crate::game::{self, GameCode, GameManager};

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
    if game_manager.write().unwrap().does_game_exist(game_code) {
        Ok(NamedFile::open(Path::new("web/protected/lobby.html"))
            .await
            .ok())
    } else {
        Err(Redirect::to("/lobby"))
    }
}


#[get("/api/debug")]
pub fn debug(game_manager: &State<RwLock<GameManager>>, ip_addr: IpAddr) -> String {
    let mut game_manager = game_manager.write().unwrap();
    String::from("Hello, World!")
}

/// Some utility functions
mod utils {
    use std::sync::RwLockWriteGuard;

    use crate::{
        game::{game_instance::GameInstance, GameManager},
        request_data::UserAuth,
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
