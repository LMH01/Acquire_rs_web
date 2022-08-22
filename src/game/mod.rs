use std::{net::IpAddr, sync::RwLock, collections::HashMap, time::Duration, thread};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket::{FromForm, request::{FromRequest, Outcome}, http::Status, State, tokio::sync::broadcast::Sender, log::private::info};

use crate::{request_data::{UserRegistration, EventData}, authentication::UserAuth, paths::utils::get_gm_write_guard};

use self::{game_instance::{GameInstance, GameCode, GAME_CODE_CHARSET}, base_game::Player};

/// Contains all base components that are required to run a game
pub mod base_game;

/// Contains the struct that represents a single game
pub mod game_instance;

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
    /// All user ids that are already in use.
    /// 
    /// See [User.user_id](struct.User.html#structfield.user_id) for more information.
    used_user_ids: Vec<i32>,
    /// Stores all game codes that are already in use
    used_game_codes: Vec<GameCode>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            games: Vec::new(),
            used_user_ids: Vec::new(),
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
        game.add_player(user);
        game.set_game_master(user_id);
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
    pub fn add_player_to_game(&mut self, event: &State<Sender<EventData>>, game_code: GameCode, username: String, ip_address: Option<IpAddr>) -> Option<UserRegistration> {//TODO Move function to GameInstance
        let user_id = self.generate_user_id();
        let player_added = match self.game_by_code_mut(game_code) {
            Some(game) => {
                game.add_player(User::new(ip_address, username.clone(), user_id));
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
            let _e = event.send(EventData::new(0, game_code, (String::from("AddPlayer"), Some(username))));
            Some(UserRegistration::new(user_id, game_code, ip_address_send))
        } else {
            None
        }
    }

    /// Returns reference to [GameInstance](game_instance/struct.GameInstance.html) where the [User](struct.User.html) with `user_id` is assigned to when found.
    pub fn game_by_user_id(&self, user_id: i32) -> Option<&GameInstance> {
        for game in self.games.iter() {
            for player in game.players() {
                if player.user_id() == user_id {
                    return Some(game);
                }
            }
        }
        None
    }
    
    /// Returns mutable reference to [GameInstance](game_instance/struct.GameInstance.html) where the [User](struct.User.html) with `user_id` is assigned to when found.
    pub fn game_by_user_id_mut(&mut self, id: i32) -> Option<&mut GameInstance> {
        for game in &mut self.games {
            for player in game.players() {
                if player.user_id() == id {
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
    pub fn players_in_game(&self, game_code: GameCode) -> Option<Vec<String>> {// TODO Move to GameInstance
        match self.game_by_code(game_code) {
            Some(game) => {
                let mut player_names = Vec::new();
                for player in game.players() {
                    if player.user.connected() {
                        player_names.push(String::from(player.username()))
                    }
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
}


/// Disconnects the user from the [GameInstance](game_instance/struct.GameInstance.html) and performs cleanup actions if necessary.
/// 
/// This updates the value [User.connected](struct.User.html#structfield.connected) for that user to false.
/// 
/// It is then checked if the [GameInstance](game_instance/struct.GameInstance.html) is abandoned (no more players are marked as connected).
/// If the [GameInstance](game_instance/struct.GameInstance.html) is abandoned, a timer with [GAME_INSTANCE_TIMEOUT](constant.GAME_INSTANCE_TIMEOUT.html) duration is started.
/// 
/// When this timer runs out it is checked again if the [GameInstance](game_instance/struct.GameInstance.html) is abandoned.
/// 
/// If the [GameInstance](game_instance/struct.GameInstance.html) is still abandoned it will be deleted from the server and the [GameCode](game_instance/struct.GameCode.html) is made available again.
/// 
/// Because this thread will be sleeping for some time an `RwLock<GameManager>` is provided to not block access to the [GameManager](struct.GameManager.html) wile sleeping.
/// 
/// When `no_sleep` is set and no more players are connected the game will be deleted directly.
pub fn disconnect_user(game_manager: &RwLock<GameManager>, user_auth: UserAuth, no_sleep: bool) -> UserDisconnectedStatus {
    // Not optimal in terms of runtime when the number of players grows, can be optimized
    {
        let mut game_manager = get_gm_write_guard(game_manager, "disconnect_user: phase 1");
        let game = game_manager.game_by_code_mut(user_auth.game_code).unwrap();
        // 1. Update connection status to false
        game.player_by_id_mut(user_auth.user_id).unwrap().user.set_connected(false);
        // 2. Check if game is abandoned
        if !game.abandoned() {
            return UserDisconnectedStatus::GameAlive;
        }
    }
    if !no_sleep {
        // 3. Wait for some time to check if the game keeps being abandoned
        thread::sleep(GAME_INSTANCE_TIMEOUT);
    }
    {
        // 4. Check again if game is abandoned
        let mut game_manager = get_gm_write_guard(game_manager, "disconnect_user: phase 2");
        if !game_manager.game_by_code_mut(user_auth.game_code).unwrap().abandoned() {
            return UserDisconnectedStatus::GameAlive;
        }
        // 5. Delete game
        game_manager.delete_game(&user_auth.game_code);
        info!("Game instance with code {} was deleted because all players left.", user_auth.game_code.to_string());
        UserDisconnectedStatus::GameDeleted
    }
}

/// The different ways [user_disconnected]() can return.
#[derive(Debug)]
pub enum UserDisconnectedStatus {
    /// Indicates that at least one player is still connected to the game.
    GameAlive,
    /// Indicates that the game was deleted because no players where connected anymore.
    GameDeleted,
}

/// User that is playing in a game.
/// 
/// User is not the same as [Player](base_game/struct.Player.html):
/// 
/// - The [Player](base_game/struct.Player.html) contains all data that is required for the user to play the game.
/// - The [User](struct.User.html) is used for authentication against the server.
#[derive(PartialEq, Eq)]
pub struct User {
    /// The username of this user.
    username: String,
    /// The unique user id of this user, it is used to identify this player.
    ///
    /// It is also used to authorize the player against the server using the [UserAuth](../authentication/struct.UserAuth.html) request guard.
    user_id: i32,
    /// The ip address of the client, used to reconstruct the user id if connection was lost.
    ip_address: Option<IpAddr>,
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
