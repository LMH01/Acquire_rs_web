use super::User;

/// Player in the game.
/// 
/// Contains all information that is required for a user to play the game.
pub struct Player {
    /// The [User](../struct.User.html) that is associated to this player.
    pub user: User,
    /// Signals that this player is the game master and can start the game.
    game_master: bool,
}

impl Player {
    /// Creates a new player
    pub fn new(user: User) -> Self {
        Self {
            user,
            game_master: false,
        }
    }

    pub fn username(&self) -> String {
        self.user.name()
    }

    pub fn user_id(&self) -> i32 {
        self.user.id()
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
