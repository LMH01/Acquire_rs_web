use std::{path::Path, sync::RwLock};

use rocket::{
    fs::NamedFile,
    get,
    http::{ContentType, CookieJar},
    log::private::info,
    State,
};

use crate::game::{self, GameCode, GameManager};

#[get("/lobby")]
pub async fn lobby(game_manager: &State<RwLock<GameManager>>) -> Option<NamedFile> {
    NamedFile::open(Path::new("web/protected/lobby.html"))
        .await
        .ok()
}

#[get("/api/debug")]
pub fn debug(game_manager: &State<RwLock<GameManager>>) -> String {
    let game_manager = game_manager.write().unwrap();
    let game_code = game_manager.generate_game_code();
    //info!("Game code: {:?}", game_code.to_string());
    game_code.to_string()
}

/// Retrieves the player id from the `player_id` cookie
///
/// # Returns
/// 'Some(i32)' when the id was found
/// 'None' when the player id was not found or the cookie was not set
pub fn player_id_from_cookies(cookies: &CookieJar<'_>) -> Option<i32> {
    cookies
        .get("player_id")
        .map(|cookie| cookie.value().parse().unwrap())
}

/// Some utility functions
mod utils {
    use std::sync::RwLockWriteGuard;

    use crate::{
        game::{game_instance::GameInstance, GameManager},
        request_data::PlayerAuth,
    };

    /// Returns the game a player is assigned to by using the `player_auth`
    pub fn game_by_player_auth<'a>(
        game_manager: &'a mut RwLockWriteGuard<GameManager>,
        player_auth: PlayerAuth,
    ) -> Option<&'a mut GameInstance> {
        match game_manager.game_by_player_id(player_auth.player_id) {
            Some(game) => Some(game),
            None => None,
        }
    }
}
