use std::sync::RwLock;

use rocket::{
    http::{Status, hyper::request},
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
            Some(id) => id,
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

/// Used to transmit data to the client with server side events
#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
pub struct EventData {
    /// Indicates to which player this request is directed.
    /// This is the player turn number and not the player id.
    ///
    /// When this is 0 the message is meant to be relevant for all players.
    player: usize,
    /// Indicates for what game this request is relevant
    ///
    /// Stores the value of [GameCode::to_string()](../game/struct.GameCode.html#method.to_string)
    game_code: String,
    /// Additional data
    data: String,
}

impl EventData {
    /// Construct new event data
    pub fn new(player: usize, game_code: GameCode, data: String) -> Self {
        Self {
            player,
            game_code: game_code.to_string(),
            data,
        }
    }

    /// # Returns
    /// The game code to which this data event belongs
    pub fn game_code(&self) -> String {
        self.game_code.to_string()
    }
}
