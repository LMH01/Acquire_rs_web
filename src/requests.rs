use std::path::Path;

use rocket::{http::CookieJar, fs::NamedFile, get, log::private::info};

use crate::game::GameCode;

#[get("/lobby")]
pub async fn lobby() -> Option<NamedFile> {
    let mut x = Vec::new();
    for i in 1..=8 {
        x.push(char::from_digit(i, 10).unwrap());
    }
    let game_code = GameCode::new(x).unwrap();
    info!("Game code: {:?}", game_code.to_string());
    NamedFile::open(Path::new("web/protected/lobby.html")).await.ok()
}

/// Retrieves the user id from the `userid` cookie
/// 
/// # Returns
/// 'Some(i32)' when the id was found
/// 'None' when the user id was not found or the cookie was not set
pub fn user_id_from_cookies(cookies: &CookieJar<'_>) -> Option<i32> {
    cookies.get("userid").map(|cookie| cookie.value().parse().unwrap())
}

/// Some utility functions
mod utils {
    use std::sync::RwLockWriteGuard;

    use crate::{game::{GameManager, game_instance::GameInstance}, request_data::PlayerAuth};

    /// Returns the game a player is assigned to by using the `player_auth`
    pub fn game_by_player_auth<'a>(game_manager: &'a mut RwLockWriteGuard<GameManager>, player_auth: PlayerAuth) -> Option<&'a mut GameInstance> {
        match game_manager.game_by_player_id(player_auth.player_id) {
            Some(game) => Some(game),
            None => None,
        }
    }
}
