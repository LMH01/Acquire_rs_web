use std::{sync::{RwLock, RwLockReadGuard}, clone};

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
    /// The urid of the user that thries to recover the authentication.
    pub urid: Urid,
    /// The name of the user that tries to recover the authentication.
    pub name: Option<String>,
}

impl UserRecovery {
    /// Creates a new user recovery
    pub fn new(urid: Urid) -> Self {
        Self {
            urid,
            name: None,
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserRecovery {
    type Error = FromRequestError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        match Self::from_cookie(request.cookies(), "urid") {
            Ok(urid) => Outcome::Success(*urid),
            Err(err) => Outcome::Failure((Status::Forbidden, err)),
        }
    }
}

impl FromCookie for UserRecovery {

    /// Constructs a new `UserRecovery` from a cookie.
    /// 
    /// The `name` field is not set automatically.
    fn from_cookie(cookies: &CookieJar<'_>, name: &str) -> Result<Box<Self>, FromRequestError> {
        match cookies.get(name).map(|cookie| cookie.value().parse::<String>().unwrap()) {
            Some(value) => {
                match Uuid::parse_str(&value) {
                    Ok(uuid) => Ok(Box::new(UserRecovery::new(Urid::from_uuid(uuid)))),
                    Err(_err) => Err(FromRequestError::Invalid(String::from("Unable to construct urid from cookie, value invalid"))),
                }
            },
            None => Err(FromRequestError::Missing(String::from("Cookie named urid missing"))),
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

/// Trait to construct a type with the value of a cookie.
pub trait FromCookie {
    /// Create `Self` from a cookie.
    /// 
    /// # Arguments
    /// - `cookies` all cookies that are submitted in an http request
    /// - `name` the name for the cookie from whichs value `Self` should be constructed
    /// 
    /// # Returns
    /// - `Ok(Box<Self>)` `Self` was successfully constructed
    /// - `Err(FromRequestError)` `Self` could not be constructed, contains detailed information on what failed
    fn from_cookie(cookies: &CookieJar<'_>, name: &str) -> Result<Box<Self>, FromRequestError>;
}
