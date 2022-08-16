use rocket::FromForm;

use self::game_instance::GameInstance;

/// Contains all base components that are required to run a game
pub mod base_game;

/// Contains the struct that represents a single game
pub mod game_instance;

/// Used to manage all currently running games.
/// 
/// One `GameManager` instance is managed by rocket and given to each request handler.
pub struct GameManager {
    /// Contains all games that are currently running
    games: Vec<GameInstance>,
    /// All player ids that are already in use. 
    /// 
    /// A player id uniquely identifies the given player. 
    /// 
    /// It is also used to authorize the player against the server.
    player_ids: Vec<i32>,
}

impl GameManager {
    pub fn new() -> Self {
        Self { 
            games: Vec::new(), 
            player_ids: Vec::new() 
        }
    }

    /// # Returns
    /// 
    /// `Some(&mut Game)` when the game was found where the user is playing in
    /// 
    /// `None` the player id does not appear to be assigned to a game
    pub fn game_by_player_id(&mut self, id: i32) -> Option<&mut GameInstance> {
        for game in &mut self.games {
            for player in game.players() {
                if player.id == id {
                    return Some(game);
                }
            }            
        }
        None
    }

    /// Generates a new game code that is not yet used by another game
    pub fn generate_game_code(&self) -> GameCode {
        !todo!("This function is not yet implemented!")
    }
}

/// Unique 9 character code that identifies a game
#[derive(Clone, Copy, Debug)]
pub struct GameCode {
    game_code: [char; 9],
}

impl GameCode {
    pub fn new(random_chars: Vec<char>) -> Option<Self> {
        // Check if only 8 characters where submitted
        if random_chars.len() > 8 {
            return None;
        }
        let mut game_code: [char; 9] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I'];
        let mut loops = 0;
        for c in random_chars {
            game_code[loops] = c;
            if loops == 3 {
                loops += 1;
                game_code[loops] = '-';
            }
            loops += 1;
        }
        Some(Self {
            game_code,
        })
    }
}

impl ToString for GameCode {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for c in self.game_code {
            s.push(c);
        }
        s
    }
}
