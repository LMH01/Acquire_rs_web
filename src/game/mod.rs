use rand::{thread_rng, distributions::Alphanumeric, Rng};
use rocket::FromForm;

use self::game_instance::GameInstance;

/// Contains all base components that are required to run a game
pub mod base_game;

/// Contains the struct that represents a single game
pub mod game_instance;

/// All characters that can be used to generate a game code
const GAME_CODE_CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWZ";

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
    /// Stores all game codes that are already in use
    used_game_codes: Vec<GameCode>
}

impl GameManager {
    pub fn new() -> Self {
        Self { 
            games: Vec::new(), 
            player_ids: Vec::new(),
            used_game_codes: Vec::new(),
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
        let mut rng = thread_rng();
        let code: String = (0..8)
                .map(|_| {
                    let idx = rng.gen_range(0..GAME_CODE_CHARSET.len());
                    GAME_CODE_CHARSET[idx] as char
                })
                .collect();
        let chars: Vec<char> = code.chars().collect();
        let code: [char; 8] = [chars[0], chars[1], chars[2], chars[3], chars[4], chars[5], chars[6], chars[7]];
        GameCode::new(code).unwrap()
    }
}

/// Unique 9 character code that identifies a game
/// 
/// A code will look like this when `to_string` is called: AB2S-B4D2
#[derive(Clone, Copy, Debug)]
pub struct GameCode {
    game_code: [char; 8],
}

impl GameCode {
    /// Construct a new game code
    fn new(random_chars: [char; 8]) -> Option<Self> {
        Some(Self {
            game_code: random_chars,
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
