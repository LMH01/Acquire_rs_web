use std::collections::HashMap;

use rocket::FromForm;
use serde::{Serialize, Deserialize};

use crate::game::GameCode;

/// Used to transmit data back to the user when a new game is joined
#[derive(Serialize, Deserialize)]
pub struct UserRegistration {
    /// Unique user id for the user
    user_id: i32,
    /// Game code of the game where the user is assigned to
    game_code: String,
    /// Stores if the ip address was send. If it was not sent a warning will be shown to the player.
    ip_address_send: bool,
}

impl UserRegistration {
    /// Construct a new `PlayerRegistration`
    pub fn new(user_id: i32, game_code: GameCode, ip_address_send: bool) -> Self {
        Self {
            user_id,
            game_code: game_code.to_string(),
            ip_address_send
        }
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
    /// Additional data 1
    data: (String, String),
}

impl EventData {
    /// Construct new event data
    pub fn new(player: usize, game_code: GameCode, data: (String, String)) -> Self {
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


/// Used to get the username from a request formatted as json
#[derive(Deserialize)]
pub struct Username<'a> {
    pub username: &'a str,
}