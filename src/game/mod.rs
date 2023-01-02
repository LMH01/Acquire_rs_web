use std::{net::IpAddr, sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}, collections::{HashMap, HashSet}, time::Duration, thread};

use rand::{thread_rng, Rng};
use rocket::{State, tokio::sync::broadcast::Sender, log::private::info, Responder, serde::json::Json};
use uuid::Uuid;

use crate::{request_data::{UserRegistration, EventData}, authentication::{UserAuth, UserRecovery, Urid, Urids}, paths::utils::get_gm_write_guard};

use self::{game_instance::{GameInstance, GameCode, GAME_CODE_CHARSET, GameState}};

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
    /// Contains all games that are currently running.
    /// 
    /// - `key` is the game code for the specified game
    /// - `value` is the game instance, wrapped in an [RwLock]() so that multiple game instances can be accessed with write right simultaneously
    games: HashMap<GameCode, RwLock<GameInstance>>,
    /// All user ids that are already in use.
    /// 
    /// All uuids that are already in use, mapped to the [GameCode]() in which the player with the specified uuid is playing in.
    used_uuids: HashMap<Uuid, GameCode>,
    /// Stores and manages all user recovery ids that are already in used.
    urids: Urids,
    /// Stores all game codes that are already in use
    used_game_codes: HashSet<GameCode>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            games: HashMap::new(),
            used_uuids: HashMap::new(),
            urids: Urids::new(),
            used_game_codes: HashSet::new(),
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
    /// `ip_addr` the ip address of the user that creates the game. See [User]() for reason why `ip_address` is required.
    /// 
    /// # Returns
    /// `Some(UserRegistration)` when the game was created
    /// `None` when the game was not created
    pub fn create_game(&mut self, username: String, ip_addr: Option<IpAddr>) -> Option<UserRegistration> {
        let code = self.generate_game_code();
        let mut game = GameInstance::new(code);
        let uuid = self.generate_uuid();
        let urid = self.urids.register(ip_addr);
        let user = User::new(username, uuid, urid, code);
        game.add_user(user);
        game.set_game_master(uuid);
        self.used_game_codes.insert(code);
        self.used_uuids.insert(uuid, code);
        self.games.insert(code, RwLock::new(game));
        Some(UserRegistration::new(uuid, urid, code))
    }

    /// Deletes the game instance for the game code from the server.
    /// 
    /// This will also delete all users and players assigned to the game.
    /// The `GameCode` under wich the game is registered is also freed.
    /// # Returns
    /// `true` when the game was deleted
    /// `false` when the game was not found
    pub fn delete_game(&mut self, game_code: &GameCode) -> bool {
        if !self.games.contains_key(game_code) {
            return false;
        }
        
        // Free uuids and urids
        let mut urids_to_remove = HashSet::new();
        for player in self.game_by_code_read(*game_code).unwrap().players() {
            urids_to_remove.insert(player.user.urid.clone());
        }
        self.urids.unregister_all(&urids_to_remove);
        let uuids = self.game_by_code_read(*game_code).unwrap().player_uuids();
        for uuid in uuids {
            self.used_uuids.remove(&uuid);
        }
        // Remove game_code from used game codes
        self.used_game_codes.remove(game_code);
        // Remove game instance
        self.games.remove(game_code);
        true
    }

    /// Tries to add the player to the game.
    /// 
    /// This will fail when the game does not exist, the game was already started or when a player with that name was already registered.
    /// 
    /// # Params
    /// - `username` the username of the user that should be added to the game
    /// - `ur` used to recover the user session when the user has lost connection.
    /// 
    /// # Returns
    /// - `Ok(UserRegistration)` when the user was added to the game.
    /// - `Err(UserRegistrationError)` when the player was not added to the game, contains the reason why the player was not added.
    pub fn add_player_to_game(&mut self, event: &State<Sender<EventData>>, game_code: GameCode, username: String, ur: Option<UserRecovery>, ip_addr: Option<IpAddr>) -> Result<UserRegistration, UserRegistrationError> {//TODO Move function to GameInstance
        let uuid = self.generate_uuid();
        let urid = self.urids.register(ip_addr);
        match self.games.get(&game_code) {
            Some(game) => {
                let mut game_write = game.write().unwrap();
                if !game_write.does_player_exist(&username) {
                    match game_write.game_state() {
                        GameState::Lobby => {
                            game_write.add_user(User::new(username.clone(), uuid, urid, game_code));
                        }
                        _ => return Err(UserRegistrationError::GameAlreadyStarted(())),
                    }
                } else {
                    if game_write.is_player_connected(&username) {
                        if ur.is_some() && game_write.validate_urid(ur.unwrap()) {
                            return Ok(game_write.user_registration(&username).unwrap());
                        } else {
                            return Err(UserRegistrationError::NameTaken(Json(String::from("name_taken"))));
                        }
                    } else {
                        let _e = event.send(EventData::new(None, game_code, (String::from("AddPlayer"), Some(username.clone()))));
                        return Ok(game_write.user_registration(&username).unwrap());
                    }
                }
            },
            None => return Err(UserRegistrationError::GameDoesNotExist(())),
        }
        self.used_uuids.insert(uuid, game_code);
        //if ur.is_some() {
        //    self.urids.add_urid(urid, ur.unwrap().ip_addr);
        //} else {
        //    self.urids.add_urid(urid, None);
        //}
        //self.used_urids.insert(urid);
        let _e = event.send(EventData::new(None, game_code, (String::from("AddPlayer"), Some(username))));
        Ok(UserRegistration::new(uuid, urid, game_code))
    }

    /// Returns reference to [GameInstance](game_instance/struct.GameInstance.html) wrapped inside an [RwLock]() where the [User](struct.User.html) with `uuid` is assigned to when found.
    /// 
    /// # Returns
    /// - `Some(&RwLock<GameInstance>)` when a game for the specified user exists.
    /// - `None` the game does not exist.
    pub fn game_by_uuid(&self, uuid: Uuid) -> Option<&RwLock<GameInstance>> {
        if self.used_uuids.contains_key(&uuid) {
            let code = self.used_uuids.get(&uuid).unwrap();
            self.games.get(code)
        } else {
            None
        }
    }
    
    /// Returns [RwLockReadGuard]() for the [GameInstance]() where the `uuid` is assigned to.
    pub fn game_by_uuid_read(&self, uuid: Uuid) -> Option<RwLockReadGuard<GameInstance>> {
        match self.game_by_uuid(uuid) {
            Some(game) => Some(game.read().unwrap()),
            None => None,
        }
    }

    /// Returns [RwLockWriteGuard]() for the [GameInstance]() where the `uuid` is assigned to.    
    pub fn game_by_uuid_write(&self, uuid: Uuid) -> Option<RwLockWriteGuard<GameInstance>> {
        match self.game_by_uuid(uuid) {
            Some(game) => Some(game.write().unwrap()),
            None => None,
        }
    }

    /// Returns reference to [GameInstance](game_instance/struct.GameInstance.html) wrapped inside an [RwLock]() when a [GameInstance]() for this code exists.
    /// 
    /// # Returns
    /// - `Some(&RwLock<GameInstance>)` when the game with the game code exists.
    /// - `None` the game does not exist.
    pub fn game_by_code(&self, game_code: GameCode) -> Option<&RwLock<GameInstance>> {
        self.games.get(&game_code)
    }

    /// Returns [RwLockReadGuard]() for the [GameInstance]() with the specified `game_code`.
    pub fn game_by_code_read(&self, game_code: GameCode) -> Option<RwLockReadGuard<GameInstance>> {
        match self.game_by_code(game_code) {
            Some(game) => Some(game.read().unwrap()),
            None => None,
        }
    }

    /// Returns [RwLockWriteGuard]() for the [GameInstance]() with the specified `game_code`.    
    pub fn game_by_code_write(&self, game_code: GameCode) -> Option<RwLockWriteGuard<GameInstance>> {
        match self.game_by_code(game_code) {
            Some(game) => Some(game.write().unwrap()),
            None => None,
        }
    }

    /// Returns the game a user is assigned to by using the `user_auth`, wrapped in an [RwLock]().
    pub fn game_by_user_auth(&self, user_auth: UserAuth,) -> Option<&RwLock<GameInstance>> {
        self.game_by_code(user_auth.game_code)
    }

    
    /// Returns [RwLockReadGuard]() for the [GameInstance]() where the `user_auth` is assigned to.
    pub fn game_by_user_auth_read(&self, user_auth: UserAuth) -> Option<RwLockReadGuard<GameInstance>> {
        match self.game_by_user_auth(user_auth) {
            Some(game) => Some(game.read().unwrap()),
            None => None,
        }
    }

    /// Returns [RwLockWriteGuard]() for the [GameInstance]() where the `user_auth` is assigned to.    
    pub fn game_by_user_auth_write(&self, user_auth: UserAuth) -> Option<RwLockWriteGuard<GameInstance>> {
        match self.game_by_user_auth(user_auth) {
            Some(game) => Some(game.write().unwrap()),
            None => None,
        }
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
        match self.game_by_code_read(game_code) {
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

    /// Generates a unique user id that is not yet registered in the `used_uuids` vector.
    /// 
    /// This does not add the generated id to the `user_uuids` vector.
    fn generate_uuid(&mut self) -> Uuid {
        let mut uuid = Uuid::new_v4();
        while self.used_uuids.contains_key(&uuid) {
            uuid = Uuid::new_v4();
        }
        uuid
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
        let game_manager = get_gm_write_guard(game_manager, "disconnect_user: phase 1");
        let mut game = game_manager.game_by_code_write(user_auth.game_code).unwrap();
        // 1. Update connection status to false
        game.player_by_uuid_mut(user_auth.uuid).unwrap().user.set_connected(false);
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
        if !game_manager.game_by_code_write(user_auth.game_code).unwrap().abandoned() {
            return UserDisconnectedStatus::GameAlive;
        }
        // 5. Delete game
        game_manager.delete_game(&user_auth.game_code);
        info!("Game instance with code {} was deleted because all players left.", user_auth.game_code.to_string());
        UserDisconnectedStatus::GameDeleted
    }
}

/// The different ways a user registration can fail.
#[derive(Responder)]
pub enum UserRegistrationError {
    #[response(status = 403, content_type = "json")]
    NameTaken(Json<String>),
    #[response(status = 403)]
    GameDoesNotExist(()),
    #[response(status = 403)]
    GameAlreadyStarted(()),
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
    /// The uuid of this user, it is used to identify this player.
    ///
    /// It is also used to authorize the player against the server using the [UserAuth](../authentication/struct.UserAuth.html) request guard.
    uuid: Uuid,
    /// An id that is used to reconstruct the user's session when the user gets disconnected.
    urid: Urid,
    /// The [GameCode]() for the game to which the user is assigned.
    game_code: GameCode,
    /// Stores if this user has an open sse stream currently.
    connected: bool,
}

impl User {
    /// Creates a new user
    /// 
    /// # Params
    /// `username` the username of the user
    /// `uuid` a unique user id
    pub fn new(username: String, uuid: Uuid, urid: Urid, game_code: GameCode) -> Self {
        Self {
            username,
            uuid: uuid,
            urid: urid,
            game_code,
            connected: false,
        }
    }

    /// Returns the name of this user
    pub fn name(&self) -> String {
        self.username.clone()
    }

    /// Returns the users uuid
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// Returns the users urid
    pub fn urid(&self) -> Urid {
        self.urid
    }

    /// Returns the game code for the game to which the user is assigned
    pub fn game_code(&self) -> GameCode {
        self.game_code
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
