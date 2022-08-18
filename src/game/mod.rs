use std::{net::IpAddr, sync::RwLock, collections::HashMap, time::Duration, thread};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket::{FromForm, request::{FromRequest, Outcome}, http::Status, State, tokio::sync::broadcast::Sender, log::private::info};

use crate::{request_data::{UserRegistration, EventData}};

use self::{game_instance::GameInstance, base_game::Player};

/// Contains all base components that are required to run a game
pub mod base_game;

/// Contains the struct that represents a single game
pub mod game_instance;

/// All characters that can be used to generate a game code
const GAME_CODE_CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWZ";
/// This is the time a game instance is kept alive when no more players are connected
/// 
/// When this time runs out the `GameInstance` and `User`s that where assigned to that instance will be deleted from the `GameManager`.
//const GAME_INSTANCE_TIMEOUT: Duration = Duration::from_secs(60);
const GAME_INSTANCE_TIMEOUT: Duration = Duration::from_secs(20);

/// Used to manage all currently running games.
///
/// One `GameManager` instance is managed by rocket and given to each request handler.
pub struct GameManager {
    /// Contains all games that are currently running
    games: Vec<GameInstance>,
    /// All users that are currently playing in a game.
    /// 
    /// Users are added to this vector by either calling [GameManager::create_game()](#method.create_game) or [GameManager::add_player_to_game()](#method.add_player_to_game).
    /// Stores all users that are currently connected to the server.
    /// Users will be removed from this list when the event stream breaks.
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

    /// Deletes the game instance for the game code from the server.
    /// 
    /// This will also delete all users and players assigned to the game.
    /// The `GameCode` under wich the game is registered is also freed.
    /// # Returns
    /// `true` when the game was deleted
    /// `false` when the game was not found
    pub fn delete_game(&mut self, game_code: &GameCode) -> bool {
        let mut game_found = false;
        let mut game_to_remove = 0;
        for (index, game) in self.games.iter().enumerate() {
            if game.game_code() == game_code {
                game_found = true;
                game_to_remove = index;
            }
        }
        if game_found {
            // Remove game_code from used game codes
            let mut code_to_remove = 0;
            for (index, code) in self.used_game_codes.iter().enumerate() {
                if code == game_code {
                    code_to_remove = index;
                }
            }
            self.used_game_codes.remove(code_to_remove);
            // Remove users
            let mut users_to_remove = Vec::new();
            for player in self.game_by_code(*game_code).unwrap().players() {
                for (index, user) in self.users.iter().enumerate() {
                    if player.id() == user.id() {
                        users_to_remove.push(index);
                    }            
                }
            }
            for user_index in users_to_remove {
                self.users.remove(user_index);
            }
            // Remove game instance
            self.games.remove(game_to_remove);
            true
        } else {
            false
        }
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
    pub fn add_player_to_game(&mut self, event: &State<Sender<EventData>>, game_code: GameCode, username: String, ip_address: Option<IpAddr>) -> Option<UserRegistration> {
        let user_id = self.generate_user_id();
        let player_added = match self.game_by_code_mut(game_code) {
            Some(game) => {
                let user = User::new(ip_address, username.clone(), user_id);
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
            let _e = event.send(EventData::new(0, game_code, (String::from("AddPlayer"), username)));
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
        //TODO make input require &GameCode
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

    /// Generates a unique user id that is not yet registered in the `user_ids` vector.
    /// 
    /// This does not add the generated id to the `user_ids` vector.
    fn generate_user_id(&mut self) -> i32 {
        let mut number = rand::thread_rng().gen_range(1..=i32::MAX);
        while self.used_user_ids.contains(&number) {
            number = rand::thread_rng().gen_range(1..=i32::MAX);
        }
        number
    }    

    /// Updates the user entry to reflect that the user is connected.
    pub fn user_connected(&mut self, user_id: i32) -> bool {
        for user in &mut self.users {
            if user.id() == user_id {
                user.set_connected(true);
                return true;
            }
        }
        false
    }

    /// Checks if players are still connected to the game
    /// 
    /// # Returns
    /// `Some(true)` when at least one player is still connected to the game
    /// 
    /// `Some(false)` when no player is connected to the game
    /// 
    /// `None` when the game does not exist
    pub fn users_connected(&self, game_code: GameCode) -> Option<bool> {
        match self.game_by_code(game_code) {
            Some(game) => {
                let mut connected_player = false;
                // I know that this is a terrible idea runtime wise, it will probably sometime reworked by putting a reference to user in the player struct.
                for player in game.players() {
                    for user in self.users.iter().as_ref() {
                        if player.id() == user.id() {
                            if user.connected {
                                connected_player = true;
                            }
                        }
                    }
                }
                Some(connected_player)
            },
            None => None
        }
    }
}


/// Notifies the `GameManager` that this user has disconnected and performs cleanup actions if necessary.
/// 
/// This updates the value `User.connected` for that user to false if the user exists.
/// 
/// It is then checked if there are still players connected to the same `GameInstance` where the user disconnected from.
/// If no players are connected to the server a timer with [GAME_INSTANCE_TIMEOUT](constant.GAME_INSTANCE_TIMEOUT.html) is started.
/// 
/// When this timer runs out it is checked again if players are connected to the `GameInstance.
/// 
/// When the answer is no the `GameInstance` and all users associated to that game will be deleted from the server and the game_id will be freed.
/// 
/// Because this thread will be sleeping for some time an `RwLock<GameManager>` is provided to not block access to the `GameManager` wile waiting.
/// 
/// # Returns
/// `Some(true)` when the user exists and was marked as disconnected
/// 
/// `Some(false)` when the user was not found
/// 
/// `None` when the user_id is not assigned to a game
pub fn user_disconnected(game_manager: &RwLock<GameManager>, user_id: i32) -> UserDisconnectedStatus {
    // Not optimal in terms of runtime when the number of players grows, can be optimized
    // 1. Check if user exists
    let game_code: GameCode;
    {
        let mut game_manager = game_manager.write().unwrap();
        let mut user_exists = false;
        for user in &mut game_manager.users {
            if user.id() == user_id {
                // 2. set connection status to false
                user.set_connected(false);
                user_exists = true;
            }
        }
        if !user_exists {
            return UserDisconnectedStatus::UserNotFound;
        }
        // 3. Check if user is assigned to game
        game_code = match game_manager.game_by_user_id(user_id) {
            Some(game) => game.game_code().clone(),
            None => return UserDisconnectedStatus::GameNotFound,
        };
        // 4. Check if at least one player is still connected to the game
        if game_manager.users_connected(game_code).unwrap() {
            return UserDisconnectedStatus::GameAlive;
        }
    }
    // 5. Wait for some time to check if the game keeps being abandoned
    thread::sleep(GAME_INSTANCE_TIMEOUT);
    {
        // 6. Check again if no player is connected
        let mut game_manager = game_manager.write().unwrap();
        if game_manager.users_connected(game_code).unwrap() {
            return UserDisconnectedStatus::GameAlive;
        }
        // 7. Delete game
        game_manager.delete_game(&game_code);
        info!("Game instance with code {} was deleted because all players left.", game_code.to_string());
        UserDisconnectedStatus::GameDeleted
    }
}

/// The different ways [user_disconnected]() can return.
#[derive(Debug)]
pub enum UserDisconnectedStatus {
    /// Indicates that the game was not found.
    GameNotFound,
    /// Indicates that the user with the id was not found.
    UserNotFound,
    /// Indicates that at least one player is still connected to the game.
    GameAlive,
    /// Indicates that the game was deleted because no players where connected anymore.
    GameDeleted,
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
    /// Stores if this user has an open sse stream currently.
    connected: bool,
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
            user_id,
            connected: false,
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

    /// Returns if the user is currently connected to the server
    pub fn connected(&self) -> bool {
        self.connected
    }

    /// Updates the connection status
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected
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
