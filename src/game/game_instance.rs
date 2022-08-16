use super::{GameManager, base_game::Player, GameCode};

/// Representation of a game
pub struct GameInstance {
    players: Vec<Player>,
    /// Unique 9 character id that identifies 
    game_code: GameCode,
}

impl GameInstance {

    pub fn new(game_manager: &GameManager) -> Self {
        Self {
            players: Vec::new(),
            game_code: game_manager.generate_game_code(),
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

}
