use rocket::FromForm;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::game::game_instance::GameCode;

/// Used to transmit data back to the user when a new game is joined
#[derive(Serialize, Deserialize)]
pub struct UserRegistration {
    /// Unique user id for the user
    uuid: Uuid,
    /// Game code of the game where the user is assigned to
    game_code: String,
    /// Stores if the ip address was send. If it was not sent a warning will be shown to the player.
    ip_address_send: bool,
}

impl UserRegistration {
    /// Construct a new `PlayerRegistration`
    pub fn new(uuid: Uuid, game_code: GameCode, ip_address_send: bool) -> Self {
        Self {
            uuid,
            game_code: game_code.to_string(),
            ip_address_send
        }
    }
}

/// Used to transmit data to the client with server side events
#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
pub struct EventData {
    /// Indicates to which player this request is directed.
    ///
    /// When this is empty the message is meant to be relevant for all players.
    /// 
    /// [Uuid]() is not used here because it does not implement FromForm.
    user_id: String,
    /// Indicates for what game this request is relevant
    ///
    /// Stores the value of [GameCode::to_string()](../game/struct.GameCode.html#method.to_string)
    game_code: String,
    /// Additional data
    data: (String, Option<String>),
}

impl EventData {
    /// Construct new event data.
    /// 
    /// # Arguments
    /// - `uuid` The user to which the message is directed, if `None` the message is directed to everyone.
    /// - `game_code` The game code for the game instance to which this event is directed.
    /// - `data` Some data that should be sent.
    pub fn new(uuid: Option<Uuid>, game_code: GameCode, data: (String, Option<String>)) -> Self {
        let user_id = match uuid {
            None => String::new(),
            Some(uuid) => uuid.to_string(),
        };
        Self {
            user_id,
            game_code: game_code.to_string(),
            data,
        }
    }

    /// # Returns
    /// The game code to which this data event belongs
    pub fn game_code(&self) -> String {
        self.game_code.to_string()
    }

    /// # Returns
    /// The user id for which the event is relevant
    pub fn user_id(&self) -> String {
        self.user_id.clone()
    }
}


/// Used to get the username from a request formatted as json
#[derive(Deserialize)]
pub struct Username<'a> {
    pub username: &'a str,
}