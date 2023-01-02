use std::{sync::{RwLock, RwLockReadGuard}, clone, collections::{HashSet, HashMap}, net::IpAddr};

use rocket::{
    http::{Status, CookieJar},
    request::{FromRequest, Outcome},
};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::{
    game::{GameManager, game_instance::GameCode}, paths::utils::get_gm_read_guard,
};

/// Errors that can occur when the user tries to authenticate a request
#[derive(Debug)]
pub enum FromRequestError {
    /// Something is missing for the request guard to succeed.
    /// 
    /// `String` contains more information on what is missing.
    Missing(String),
    /// The transmitted data is invalid.
    /// 
    /// `String` contains more information on why the authentication is invalid.
    Invalid(String),
}

/// Symbolizes the authentication of a user.
///
/// A authenticated user is assigned to a game.
/// 
/// For a `UserAuth` so succeed the `user_id` has to be transmitted in an http header
/// and the user has to be assigned to a game.
/// 
/// # Request Guard
/// This struct implements [FromRequest](../../rocket/request/trait.FromRequest.html) and thus is a [Request Guard](../../rocket/request/trait.FromRequest.html#request-guards), 
/// it can only be constructed by the [from_request](#method.from_request) function.
/// 
/// For more information on [Request Guards](../../rocket/request/trait.FromRequest.html#request-guards) take a look [here](../paths/index.html) or [here](../../rocket/request/trait.FromRequest.html#request-guards).
/// 
/// # Guarantees
/// An instance of this [Request Guard](../../rocket/request/trait.FromRequest.html#request-guards) guarantees the following:
/// 
/// - The user with id `user_id` exists and is assigned to the game with the code `game_code`.
#[derive(Clone, Copy)]
pub struct UserAuth {
    /// The unique id that identifies this user
    pub uuid: Uuid,
    /// The unique code that identifies a game
    pub game_code: GameCode,
}

impl UserAuth {
    
    /// Constructs a new [UserAuth]() by checking if the `user_id` exists and is assigned to a game.
    pub fn from_uuid(game_manager: RwLockReadGuard<GameManager>, user_id: Uuid) -> Option<Self> {
        match game_manager.game_by_uuid_read(user_id) {
            Some(game) => Some(UserAuth {
                uuid: user_id,
                game_code: game.game_code().clone(),
            }),
            None => None,
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAuth {
    type Error = FromRequestError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let user_id = match request.headers().get_one("user_id") {
            Some(header) => header,
            None => return Outcome::Failure((Status::Forbidden, FromRequestError::Missing(String::from("The user_id header is missing")))),
        };
        let user_id = match user_id.parse::<Uuid>() {
            Ok(id) => id,
            Err(_e) => return Outcome::Failure((Status::Forbidden, FromRequestError::Invalid(String::from("user_id is not a number"))))
        };
        match UserAuth::from_uuid(get_gm_read_guard(request.rocket().state::<RwLock<GameManager>>().unwrap(), "user_auth: from request"), user_id) {
            Some(auth) => Outcome::Success(auth),
            None => return Outcome::Failure((Status::Forbidden, FromRequestError::Invalid(String::from("game not found")))),
        }
    }
}

/// Errors that occur when a request requires a `GameCode`.
#[derive(Debug)]
pub enum GameCodeError {
    /// The transmitted game_code header is missing
    Missing,
    /// The transmitted game_code header could not be parsed to a GameCode
    ParseError,
    /// No game was found for the game code
    NotFound,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GameCode {
    type Error = GameCodeError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let mut game_manager = request
            .rocket()
            .state::<RwLock<GameManager>>()
            .unwrap()
            .write()
            .unwrap();
        // Check if header was submitted
        let game_code_string = match request.headers().get_one("game_code") {
            Some(header) => header,
            None => return Outcome::Failure((Status::Forbidden, GameCodeError::Missing)),
        };
        // Check if the game code can be parsed
        let game_code = match GameCode::from_string(game_code_string) {
            Some(code) => code,
            None => return Outcome::Failure((Status::Forbidden, GameCodeError::ParseError,))
        };
        // Check if a game with the game code exists
        if game_manager.does_game_exist(&game_code) {
            Outcome::Success(game_code)
        } else {
            Outcome::Failure((Status::Forbidden, GameCodeError::NotFound))
        }
    }
}

/// Used to recover the user authentication after the connection was lost.
/// 
/// For that a cookie named `urid` is placed when the player connects.
/// 
/// This cookie is constructed into a UserRecovery wich is then validated by the game instance.
/// If this check succeeds the `uuid` is send back to the user with which subsequent 
/// requests can be authenticated again.
///  
/// See [GameInstance::validate_uri()]() for more information.
#[derive(Clone)]
pub struct UserRecovery {
    /// The urid of the user that tries to recover the authentication.
    pub urid: Urid,
    /// The name of the user that tries to recover the authentication.
    pub name: Option<String>,
    /// The ip address of the user that tries to recover the authentication.
    pub ip_addr: Option<IpAddr>,
}

impl UserRecovery {
    /// Creates a new user recovery
    pub fn new(urid: Urid, ip_addr: Option<IpAddr>) -> Self {
        Self {
            urid,
            name: None,
            ip_addr,
        }
    }

}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserRecovery {
    type Error = FromRequestError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let ur = match request.cookies().get("urid").map(|cookie| cookie.value().parse::<String>().unwrap()) {
            Some(value) => {
                match Uuid::parse_str(&value) {
                    Ok(uuid) => Ok(UserRecovery::new(Urid::from_uuid(uuid), request.client_ip())),
                    Err(_err) => Err(FromRequestError::Invalid(String::from("Unable to construct ruid from cookie, value invalid"))),
                }
            }
            None => Err(FromRequestError::Missing(String::from("Cookie named urid missing"))),
        };
        match ur {
            Ok(urid) => Outcome::Success(urid),
            Err(err) => Outcome::Failure((Status::Forbidden, err)),
        }
    }

}

/// User recovery id that is used to recover a lost connection.
#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub struct Urid {
    uuid: Uuid,
}

impl Urid {
    /// Creates a new `Urid`
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self {
            uuid,
        }
    }

    /// Returns the value of this urid.
    pub fn value(&self) -> Uuid {
        self.uuid
    }
}

/// Stores all used user recovery ids
pub struct Urids {
    /// All user recovery ids
    used_urids: HashSet<Urid>,
    /// All user recovery ids mapped to an ip address
    urid_by_ip: HashMap<IpAddr, Urid>,
}

impl Urids {
    pub fn new() -> Self {
        Self {
            used_urids: HashSet::new(),
            urid_by_ip: HashMap::new(),
        }
    }

    /// Adds the urid to the `used_urids` set.
    /// 
    /// If an [IpAddr]() is provided, the urid will also be added to the
    /// `urid_by_ip` map.
    /// If the map already contains this address the value is not updated.
    /// 
    /// Returns whether the urid was newly added. That is:
    /// - If the urid was newly added, `true` is returned.
    /// - If the urid was already added, `false` is returned.
    pub fn add_urid(&mut self, urid: Urid, ip_addr: Option<IpAddr>) -> bool {
        if ip_addr.is_some() && !self.urid_by_ip.contains_key(&ip_addr.unwrap()) {
            self.urid_by_ip.insert(ip_addr.unwrap(), urid);
        }
        self.used_urids.insert(urid)
    }

    /// Generate a new [Urid]() and register it or return the [Urid]() linked to
    /// the `ip_addr`.
    /// 
    /// Registering means that the newly genreated [Urid]() is placed in the
    /// fields of this struct.
    /// 
    /// # Arguments
    /// - `ip_addr` the ip address of the user that needs a new urid.
    /// 
    /// # Returns
    /// Returns new [Urid]() or the [Urid]() linked to the `ip_addr`.
    pub fn register(&mut self, ip_addr: Option<IpAddr>) -> Urid {
        let urid = self.generate_urid();
        match ip_addr {
            Some(value) => {
                if self.urid_by_ip.contains_key(&value) {
                    *self.urid_by_ip.get(&value).unwrap()
                } else {
                    self.urid_by_ip.insert(value, urid);
                    self.used_urids.insert(urid);
                    urid
                }
            },
            None => {
                self.used_urids.insert(urid);
                urid
            },
        }
    }

    /// Unregisters the `urid`.
    /// 
    /// Takes O(n) time, `n` being the amount of elements in the `urid_by_ip` field.
    pub fn unregister(&mut self, urid: Urid) {
        self.used_urids.remove(&urid);
        let mut ip_to_remove: Option<IpAddr> = None;
        for (k, v) in &self.urid_by_ip {
            if *v == urid {
                ip_to_remove = Some(*k);
            }
        }
        if ip_to_remove.is_some() {
            self.urid_by_ip.remove(&ip_to_remove.unwrap());
        }
    }

    /// Unregisters all provided `urids`.
    /// 
    /// Takes O(n) time, `n` being the amount of elements in the `urid_by_ip` field.
    pub fn unregister_all(&mut self, urids: &HashSet<Urid>) {
        let mut ips_to_remove: HashSet<IpAddr> = HashSet::new();
        for urid in urids {
            self.used_urids.remove(&urid);
        }
        for (k, v) in &self.urid_by_ip {
            for urid in urids {
                if *v == *urid {
                    ips_to_remove.insert(*k);
                }
            }
        }
        for ip in ips_to_remove {
            self.urid_by_ip.remove(&ip);
        }
    }

    /// Generates a uniqe recovery id that is not yet in use.
    /// 
    /// This does not add the generated id to the `used_urid` set.
    pub fn generate_urid(&self) -> Urid {
        let mut urid = Urid::from_uuid(Uuid::new_v4());
        while self.used_urids.contains(&urid) {
            urid = Urid::from_uuid(Uuid::new_v4());
        }
        urid
    }

}
