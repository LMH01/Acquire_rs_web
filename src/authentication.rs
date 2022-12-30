use std::{net::IpAddr, sync::{RwLock, RwLockWriteGuard, RwLockReadGuard}};

use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    FromForm,
};
use serde::{Deserialize, Serialize};

use crate::{
    game::{GameManager, game_instance::GameCode}, paths::utils::get_gm_read_guard,
};

/// Errors that can occur when the user tries to authenticate a request
#[derive(Debug)]
pub enum PlayerAuthError {
    /// The transmitted user_id header is missing
    Missing,
    /// The transmitted user_id header is invalid.
    /// 
    /// `String` contains more information on why the authentication is invalid.
    Invalid(String),
}

/// Symbolizes the authentication of a user.
///
/// A authenticated user is assigned to a game.
/// 
/// For a `UserAuth` so succeed the `user_id` has to be transmitted in an http header
/// and the user has to be assigned to a game.
/// 
/// # Request Guard
/// This struct implements [FromRequest](../../rocket/request/trait.FromRequest.html) and thus is a [Request Guard](../../rocket/request/trait.FromRequest.html#request-guards), 
/// it can only be constructed by the [from_request](#method.from_request) function.
/// 
/// For more information on [Request Guards](../../rocket/request/trait.FromRequest.html#request-guards) take a look [here](../paths/index.html) or [here](../../rocket/request/trait.FromRequest.html#request-guards).
/// 
/// # Guarantees
/// An instance of this [Request Guard](../../rocket/request/trait.FromRequest.html#request-guards) guarantees the following:
/// 
/// - The user with id `user_id` exists and is assigned to the game with the code `game_code`.
#[derive(Clone, Copy)]
pub struct UserAuth {
    /// The unique id that identifies this user
    pub user_id: i32,
    /// The unique code that identifies a game
    pub game_code: GameCode,
}

impl UserAuth {
    
    /// Constructs a new [UserAuth]() by checking if the `user_id` exists and is assigned to a game.
    pub fn from_id(game_manager: RwLockReadGuard<GameManager>, user_id: i32) -> Option<Self> {
        match game_manager.game_by_uuid_read(user_id) {
            Some(game) => Some(UserAuth {
                user_id,
                game_code: game.game_code().clone(),
            }),
            None => None,
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAuth {
    type Error = PlayerAuthError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let user_id = match request.headers().get_one("user_id") {
            Some(header) => header,
            None => return Outcome::Failure((Status::Forbidden, PlayerAuthError::Missing)),
        };
        let user_id = match user_id.parse::<i32>() {
            Ok(id) => id,
            Err(_e) => return Outcome::Failure((Status::Forbidden, PlayerAuthError::Invalid(String::from("user_id is not a number"))))
        };
        match UserAuth::from_id(get_gm_read_guard(request.rocket().state::<RwLock<GameManager>>().unwrap(), "user_auth: from request"), user_id) {
            Some(auth) => Outcome::Success(auth),
            None => return Outcome::Failure((Status::Forbidden, PlayerAuthError::Invalid(String::from("game not found")))),
        }
    }
}

/// Errors that occur when a request requires a `GameCode`.
#[derive(Debug)]
pub enum GameCodeError {
    /// The transmitted game_code header is missing
    Missing,
    /// The transmitted game_code header could not be parsed to a GameCode
    ParseError,
    /// No game was found for the game code
    NotFound,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GameCode {
    type Error = GameCodeError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let mut game_manager = request
            .rocket()
            .state::<RwLock<GameManager>>()
            .unwrap()
            .write()
            .unwrap();
        // Check if header was submitted
        let game_code_string = match request.headers().get_one("game_code") {
            Some(header) => header,
            None => return Outcome::Failure((Status::Forbidden, GameCodeError::Missing)),
        };
        // Check if the game code can be parsed
        let game_code = match GameCode::from_string(game_code_string) {
            Some(code) => code,
            None => return Outcome::Failure((Status::Forbidden, GameCodeError::ParseError,))
        };
        // Check if a game with the game code exists
        if game_manager.does_game_exist(&game_code) {
            Outcome::Success(game_code)
        } else {
            Outcome::Failure((Status::Forbidden, GameCodeError::NotFound))
        }
    }
}
