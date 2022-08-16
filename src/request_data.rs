
use std::sync::RwLock;

use rocket::{FromForm, request::{FromRequest, Outcome}, http::Status};
use serde::{Serialize, Deserialize};

use crate::{game::{GameManager, GameCode}, requests::user_id_from_cookies};


/// Errors that can occur when the player tries to authenticate a request
#[derive(Debug)]
pub enum PlayerAuthError {
    /// The transmitted id-cookie is missing
    Missing,
    /// The transmitted id-cookie is invalid
    Invalid,
}

/// Symbolizes the authentication of a player.
/// 
/// A authenticated player is assigned to a game.
#[derive(Clone, Copy)]
pub struct PlayerAuth {
    /// The unique id that identifies this player
    pub player_id: i32,
    /// The unique code that identifies a game
    pub game_code: GameCode,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PlayerAuth {
    type Error = PlayerAuthError;

    async fn from_request(request: &'r rocket::Request<'_>) ->  Outcome<Self, Self::Error> {
        let player_id = match user_id_from_cookies(request.cookies()) {
            Some(id) => id,
            None => return Outcome::Failure((Status::Forbidden, PlayerAuthError::Missing)),
        };
        let mut game_manager = request.rocket().state::<RwLock<GameManager>>().unwrap().write().unwrap();
        let game = match game_manager.game_by_player_id(player_id) {
            Some(game) => game,
            None => return Outcome::Failure((Status::Forbidden, PlayerAuthError::Invalid)),
        };
        Outcome::Success(PlayerAuth { player_id: player_id, game_code: game.game_code().clone()})
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
    game_id: i32,
    /// Additional data
    data: String,
}

impl EventData {
    pub fn new(player: usize, game_id: i32, data: String) -> Self {
        Self {
            player,
            game_id,
            data, 
        }
    }

    /// # Returns
    /// The game id to which this data event belongs
    pub fn game_id(&self) -> i32 {
        self.game_id
    }
}
