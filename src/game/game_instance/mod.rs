use std::{net::IpAddr, collections::HashSet};

use rocket::form::name;
use uuid::Uuid;

use crate::{authentication::UserRecovery, request_data::UserRegistration};

use super::{base_game::Player, User};

/// Functions related to the games logic
///
/// All these function will be called from within a [GameInstance](../struct.GameInstance.html)
mod logic;

/// All characters that can be used to generate a game code
pub const GAME_CODE_CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWZ";

/// Representation of a game
pub struct GameInstance {
    /// All players that play in this game
    players: Vec<Player>,
    /// Unique 9 character id that identifies
    game_code: GameCode,
    /// The current state of the game
    game_state: GameState,
}

impl GameInstance {

    /// Creates a new game instance
    pub fn new(game_code: GameCode) -> Self {
        Self {
            players: Vec::new(),
            game_code,
            game_state: GameState::Lobby,
        }
    }

    /// Creates a new player that is associated to the user and adds them to the game.
    /// 
    /// # Params
    /// `user` the [User](../struct.User.html) associated to this player.
    /// 
    /// # Returns
    /// `true` when the player was added.
    /// 
    /// `false` when the player was not added because the game has already started.
    pub fn add_user(&mut self, user: User) -> bool {
        match self.game_state {
            GameState::Lobby => {
                self.players.push(Player::new(user));
                true
            },
            _ => false,
        }
    }

    /// Sets the game master of the game.
    /// For that the player has to be added already.
    /// 
    /// Only the game master can start the game.
    /// 
    /// If a game master is already assigned, the game master will be replaced.
    /// 
    /// # Params
    /// `id` the id of the player that should become the game master.
    /// 
    /// # Returns
    /// - `true` game master was updated
    /// - `false` game master was not updated because the player was not found
    pub fn set_game_master(&mut self, uuid: Uuid) -> bool {
        match self.player_by_uuid_mut(uuid) {
            Some(new_gm) => {
                new_gm.make_game_master();
                for player in &mut self.players {
                    if player.is_game_master() && player.uuid() != uuid {
                        player.revoke_game_master();
                    }
                }
                true
            },
            None => false,
        }
    }

    /// Returns a vector containing all players
    pub fn players(&self) -> &Vec<Player> {
        &self.players
    }

    /// Returns the games game code
    pub fn game_code(&self) -> &GameCode {
        &self.game_code
    }

    /// Returns the player by uuid if found
    pub fn player_by_uuid(&self, id: Uuid) -> Option<&Player> {
        for player in &self.players {
            if player.uuid() == id {
                return Some(player);
            }
        }
        None
    }

    /// Returns the player by id mutable if found
    pub fn player_by_uuid_mut(&mut self, uuid: Uuid) -> Option<&mut Player> {
        for player in &mut self.players {
            if player.uuid() == uuid {
                return Some(player);
            }
        }
        None
    }

    /// Checks if a player with the name already exists
    pub fn does_player_exist(&self, name: &String) -> bool {
        for player in &self.players {
            if player.user.name() == *name {
                return true;
            }
        }
        false
    }

    /// Checks if the player with the name is connected to the game.
    pub fn is_player_connected(&self, name: &String) -> bool {  
        for player in &self.players {
            if player.user.name() == *name {
                return player.user.connected;
            }
        }
        false
    }

    /// Validates the UserRecovery.
    /// 
    /// # Returns
    /// - `true` user recovery is valid
    /// - `false` user recovery is invalid
    pub fn validate_urid(&self, ur: UserRecovery) -> bool {
        for player in &self.players {
            let user = &player.user;
            if user.urid.value() == ur.urid.value() {
                return true;
            }
        }
        false
    }
    
    /// Updates the user entry to reflect that the user is connected.
    /// 
    /// Returns `false` when the user is not assigned to this game.
    pub fn user_connected(&mut self, uuid: Uuid) -> bool {
        for player in &mut self.players {
            if player.uuid() == uuid {
                player.user.set_connected(true);
                return true;
            }
        }
        false
    }

    /// Checks if players are still connected to this game
    /// 
    /// # Returns
    /// `true` when no player is connected to the game
    /// 
    /// `false` when at least one player is still connected to the game
    pub fn abandoned(&mut self) -> bool {
        let mut player_connected = false;
        for player in self.players.iter_mut() {
            if player.user.connected {
                player_connected = true;
            }
        }
        !player_connected
    }

    /// Returns the current game state
    pub fn game_state(&self) -> &GameState {
        &self.game_state
    }

    /// Returns a HashSet containing all uuids of the players that are assigned to this game instance.
    pub fn player_uuids(&self) -> HashSet<Uuid> {
        let mut set = HashSet::new();
        for player in &self.players {
            set.insert(player.uuid());
        }
        set
    }

    /// Returns the user registration for the user with `name` if that user exists.
    pub fn user_registration(&self, name: &str) -> Option<UserRegistration> {
        for player in &self.players {
            if player.user.name() == name {
                return Some(UserRegistration::from_user(&player.user));
            }
        }
        None
    }
}

/// The different states a game can be in
pub enum GameState {
    /// Signals that this game is still in the lobby and players can join
    Lobby,
}

/// Unique 9 character code that identifies a game
///
/// A code will look like this when [to_string](#method.to_string) is called: AB2S-B4D2
/// 
/// # Request Guard
/// This struct implements [FromRequest](../../../rocket/request/trait.FromRequest.html) and thus is a [Request Guard](../../../rocket/request/trait.FromRequest.html#request-guards).
/// 
/// For more information on [Request Guards](../../../rocket/request/trait.FromRequest.html#request-guards) take a look [here](../../paths/index.html) or [here](../../../rocket/request/trait.FromRequest.html#request-guards).
/// 
/// # Guarantees
/// An instance of this [Request Guard](../../../rocket/request/trait.FromRequest.html#request-guards) guarantees the following:
/// 
/// - The game with `game_code` exists.
/// 
/// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
/// <strong>Warning:</strong> This is only true when the <a href="">GameCode</a> was constructed by using <a href="#method.from_request">from_request</a>!
/// </p>
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameCode {
    game_code: [char; 8],
}

impl GameCode {
    /// Construct a new game code
    pub fn new(random_chars: [char; 8]) -> Option<Self> {
        Some(Self {
            game_code: random_chars,
        })
    }

    /// Construct a new game code from string
    /// 
    /// Input should be a in the format like the result of [GameCode::to_string()](#method.to_string).
    /// 
    /// # Returns
    /// `Some(Self)` when the string was valid and the game code was constructed
    /// `None` when the string could not be constructed into a game code
    pub fn from_string(string: &str) -> Option<Self> {
        let mut game_code: [char; 8] = ['a','a','a','a','a','a','a','a'];
        if string.len() > 9 {
            return None;
        }
        let mut second_half = false;
        for (index, char) in string.chars().enumerate() {
            let charset: Vec<char> = GAME_CODE_CHARSET.iter().map(|s| *s as char).collect();
            if index != 4 {
                if charset.contains(&char) {
                    if second_half {
                        game_code[index-1] = char;
                    } else {
                        game_code[index] = char;
                    }
                } else {
                    return None;
                }
            } else {
                if char != '-' {
                    return None;
                }
                second_half = true;
            }
        } 
        Some(Self {
            game_code
        })
    }
}

impl ToString for GameCode {
    /// Converts the given value to `String`.
    ///
    /// An example output of this function might be: `A23B-9FRT`
    fn to_string(&self) -> String {
        let s: String = self.game_code.iter().collect();
        let parts = s.split_at(4);
        let mut print = String::from(parts.0);
        print.push('-');
        print.push_str(parts.1);
        print
    }
}
