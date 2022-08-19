use std::{net::IpAddr, sync::RwLock};

use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    FromForm,
};
use serde::{Deserialize, Serialize};

use crate::{
    game::{GameCode, GameManager},
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
#[derive(Clone, Copy)]
pub struct UserAuth {
    /// The unique id that identifies this user
    pub user_id: i32,
    /// The unique code that identifies a game
    pub game_code: GameCode,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAuth {
    type Error = PlayerAuthError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let mut game_manager = request
            .rocket()
            .state::<RwLock<GameManager>>()
            .unwrap()
            .write()
            .unwrap();
        let user_id = match request.headers().get_one("user_id") {
            Some(header) => header,
            None => return Outcome::Failure((Status::Forbidden, PlayerAuthError::Missing)),
        };
        let user_id = match user_id.parse::<i32>() {
            Ok(id) => id,
            Err(_e) => return Outcome::Failure((Status::Forbidden, PlayerAuthError::Invalid(String::from("user_id is not a number"))))
        };
        let game = match game_manager.game_by_user_id(user_id) {
            Some(game) => game,
            None => return Outcome::Failure((Status::Forbidden, PlayerAuthError::Invalid(String::from("game not found")))),
        };
        Outcome::Success(UserAuth {
            user_id,
            game_code: game.game_code().clone(),
        })
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
