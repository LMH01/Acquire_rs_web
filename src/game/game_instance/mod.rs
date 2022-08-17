use super::{base_game::Player, GameCode, GameManager};

/// Functions related to the games logic
///
/// All these function will be called from within a [GameInstance](../struct.GameInstance.html)
mod logic;

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

    /// Creates a new player and adds them to the game.
    /// 
    /// # Returns
    /// `true` when the player was added
    /// `false` when the player was not added because the game has already started
    pub fn add_player(&mut self, name: String, id: i32) -> bool {
        match self.game_state {
            GameState::Lobby => {
                self.players.push(Player::new(name, id));
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
    /// `true` game master was updated
    /// `false` game master was not updated because the player was not found
    pub fn set_game_master(&mut self, id: i32) -> bool {
        match self.player_by_id_mut(id) {
            Some(new_gm) => {
                new_gm.make_game_master();
                for player in &mut self.players {
                    if player.is_game_master() && player.id() != id {
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

    /// Returns the player by id if found
    pub fn player_by_id(&self, id: i32) -> Option<&Player> {
        for player in &self.players {
            if player.id() == id {
                return Some(player);
            }
        }
        None
    }

    /// Returns the player by id mutable if found
    pub fn player_by_id_mut(&mut self, id: i32) -> Option<&mut Player> {
            for player in &mut self.players {
                if player.id() == id {
                    return Some(player);
                }
            }
            None
        }
    }

/// The different states a game can be in
enum GameState {
    /// Signals that this game is still in the lobby and players can join
    Lobby,
}
