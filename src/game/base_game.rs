/// Player in the game
pub struct Player {
    /// The players name
    name: String,
    /// The unique id of this player
    id: i32,
    /// Signals that this player is the game master and can start the game
    game_master: bool,
}

impl Player {
    /// Creates a new player
    pub fn new(name: String, id: i32) -> Self {
        Self {
            name,
            id,
            game_master: false,
        }        
    }

    /// Returns the players name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the players id
    pub fn id(&self) -> i32 {
        self.id
    }

    /// Updates this player to be the game master
    pub fn make_game_master(&mut self) {
        self.game_master = true
    }

    /// Revokes the title of game master from this player
    pub fn revoke_game_master(&mut self) {
        self.game_master = false
    }

    /// Checks if this player is a game master.
    /// 
    /// Returns true when this player is a game master.
    pub fn is_game_master(&self) -> bool {
        self.game_master
    }
}
