use std::{net::IpAddr, sync::RwLock};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket::{FromForm, request::{FromRequest, Outcome}, http::Status};

use crate::{request_data::UserRegistration};

use self::{game_instance::GameInstance, base_game::Player};

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
    /// All users that are currently playing in a game.
    /// 
    /// Users are added to this vector by either calling [GameManager::create_game()](#method.create_game) or [GameManager::add_player_to_game()](#method.add_player_to_game).
    users: Vec<User>,
    /// All player ids that are already in use.
    ///
    /// A player id uniquely identifies the given player.
    ///
    /// It is also used to authorize the player against the server.
    used_user_ids: Vec<i32>,
    /// Stores all game codes that are already in use
    used_game_codes: Vec<GameCode>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            games: Vec::new(),
            used_user_ids: Vec::new(),
            users: Vec::new(),
            used_game_codes: Vec::new(),
        }
    }    

    /// Some debug functionality, should be deleted from final version
    pub fn debug(&mut self) -> GameCode {
        //let code = *game.game_code();
        //let mut game = GameInstance::new(self);
        //self.used_game_codes.push(*game.game_code());
        //game.players.push(Player {name: String::from("Louis"), id: 1});
        //game.players.push(Player {name: String::from("Markus"), id: 1});
        //game.players.push(Player {name: String::from("David"), id: 1});
        //self.games.push(game);
        //code
        self.generate_game_code()
    }

    /// Creates a new game.
    /// 
    /// # Params
    /// `username` the username of the user that creates the game
    /// `ip_address` the ip address of the user that creates the game. See [User]() for reason why `ip_address` is required.
    /// 
    /// # Returns
    /// `Some(GameCode)` when the game was created
    /// `None` when the game was not created
    pub fn create_game(&mut self, username: String, ip_address: Option<IpAddr>) -> Option<UserRegistration> {
        let code = self.generate_game_code();
        let mut game = GameInstance::new(code);
        let user_id = self.generate_user_id();
        let user = User::new(ip_address, username, user_id);
        game.add_player(user.name(), user.id());
        game.set_game_master(user.id());
        self.users.push(user);
        self.used_game_codes.push(code.clone());
        self.used_user_ids.push(user_id);
        self.games.push(game);
        let ip_address_send = match ip_address{
            Some(_e) => true,
            None => false,
        };
        Some(UserRegistration::new(user_id, code, ip_address_send))
    }

    /// Tries to add the player to the game.
    /// 
    /// This will fail when the game does not exist or the game was already started.
    /// 
    /// # Params
    /// `username` the username of the user that should be added to the game
    /// `ip_address` the ip address of the user that should be added to the game. See [User]() for reason why `ip_address` is required.
    /// 
    /// # Returns
    /// `Some(i32)` when the user was added to the game, contains the unique user id
    /// `None` when the player was not added to the game, because the game does not exist
    pub fn add_player_to_game(&mut self, game_code: GameCode, username: String, ip_address: Option<IpAddr>) -> Option<UserRegistration> {
        let user_id = self.generate_user_id();
        let player_added = match self.game_by_code_mut(game_code) {
            Some(game) => {
                let user = User::new(ip_address, username, user_id);
                game.add_player(user.name(), user.id());
                self.users.push(user);
                true
            },
            None => false,
        };
        if player_added {
            // Only if the player was added will the new user id be pushed to the vector
            self.used_user_ids.push(user_id);
            let ip_address_send = match ip_address{
                Some(_e) => true,
                None => false,
            };
            Some(UserRegistration::new(user_id, game_code, ip_address_send))
        } else {
            None
        }
    }

    /// # Returns
    ///
    /// `Some(&mut Game)` when the game was found where the user is playing in
    ///
    /// `None` the user id does not appear to be assigned to a game
    pub fn game_by_user_id(&mut self, id: i32) -> Option<&mut GameInstance> {
        for game in &mut self.games {
            for player in game.players() {
                if player.id() == id {
                    return Some(game);
                }
            }
        }
        None
    }

    /// # Returns
    /// 
    /// `Some(&mut Game)` when the game with the game code exists
    /// `None` the game does not exist
    pub fn game_by_code(& self, game_code: GameCode) -> Option<& GameInstance> {
        for game in & self.games {
            if *game.game_code() == game_code {
                return Some(game);
            }
        }
        None
    }

    /// # Returns
    /// 
    /// `Some(&mut Game)` when the game with the game code exists
    /// `None` the game does not exist
    pub fn game_by_code_mut(&mut self, game_code: GameCode) -> Option<&mut GameInstance> {
        for game in &mut self.games {
            if *game.game_code() == game_code {
                return Some(game);
            }
        }
        None
    }

    /// Checks if a game with the game code exists
    pub fn does_game_exist(&self, game_code: &GameCode) -> bool {
        self.used_game_codes.contains(game_code)
    }

    /// Returns the names of the players that are currently joined in the selected game
    /// 
    /// # Returns
    /// `Some(Vec<String>)` when the game exists. Vector of string contains the currently joined players.
    /// `None` the game does not exist
    pub fn players_in_game(&self, game_code: GameCode) -> Option<Vec<String>> {
        match self.game_by_code(game_code) {
            Some(game) => {
                let mut player_names = Vec::new();
                for player in game.players() {
                    player_names.push(String::from(player.name()))
                }
                Some(player_names)
            },
            None => None,
        }
    }

    /// Generates a new game code that is not yet used by another game
    /// 
    /// This does not add the generated game code to the used_game_codes vector.
    fn generate_game_code(&self) -> GameCode {
        let mut rng = thread_rng();
        loop {
            let code: String = (0..8)
                .map(|_| {
                    let idx = rng.gen_range(0..GAME_CODE_CHARSET.len());
                    GAME_CODE_CHARSET[idx] as char
                })
                .collect();
            let chars: Vec<char> = code.chars().collect();
            let code: [char; 8] = [
                chars[0], chars[1], chars[2], chars[3], chars[4], chars[5], chars[6], chars[7],
            ];
            let game_code = GameCode::new(code).unwrap();
            if self.used_game_codes.contains(&game_code) {
               continue; 
            }
            return GameCode::new(code).unwrap()
        }
    }

    /// Generates a unique user id that is not yet registered in the [user_ids]() vector.
    /// 
    /// This does not add the generated id to the [user_ids]() vector.
    fn generate_user_id(&mut self) -> i32 {
        let mut number = rand::thread_rng().gen_range(0..=i32::MAX);
        while self.used_user_ids.contains(&number) {
            number = rand::thread_rng().gen_range(0..=i32::MAX);
        }
        number
    }
}

/// Unique 9 character code that identifies a game
///
/// A code will look like this when `to_string` is called: AB2S-B4D2
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// User that is playing in a game.
/// 
/// User is not the same as [Player](base_game/struct.Player.html):
/// 
/// - The `Player` contains all data that is required for the user to play the game.
/// - The `User` is used for authentication against the server.
#[derive(PartialEq, Eq)]
struct User {
    /// The ip address of the client, used to reconstruct the user id if connection was lost.
    ip_address: Option<IpAddr>,
    /// The username of this user.
    username: String,
    /// The unique user id of this user.
    /// 
    /// This user id is used to uniquely identify each user.
    user_id: i32,
}

impl User {
    /// Creates a new user
    /// 
    /// # Params
    /// `ip_address` the ip address of the client
    /// `username` the username of the user
    /// `user_id` a unique user id
    pub fn new(ip_address: Option<IpAddr>, username: String, user_id: i32) -> Self {
        Self {
            ip_address,
            username,
            user_id 
        }
    }

    /// Returns the name of this user
    pub fn name(&self) -> String {
        self.username.clone()
    }

    /// Returns the ip address of this user
    pub fn ip_address(&self) -> &Option<IpAddr> {
        &self.ip_address
    }

    /// Returns the users id
    pub fn id(&self) -> i32 {
        self.user_id
    }
}

#[cfg(test)]
mod tests {
    use super::GameCode;

    #[test]
    fn test_game_code_from_string() {
        assert_eq!("ABCD-1234", GameCode::from_string("ABCD-1234").unwrap().to_string());
    }
}
