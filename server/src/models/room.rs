use std::collections::HashMap;

use rocket::request::FromParam;
use serde::Serialize;
use uuid::Uuid;

use super::user::{User, UserID};

#[derive(Debug, Serialize, Clone, Eq, PartialEq, Hash, Copy)]
pub struct RoomID(Uuid);

impl<'a> FromParam<'a> for RoomID {
    type Error = <Uuid as FromParam<'a>>::Error;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Uuid::from_param(param).map(RoomID)
    }
}

impl RoomID {
    fn new() -> Self {
        RoomID(Uuid::now_v7())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Room {
    pub id: RoomID,
    pub join_code: 
    pub users: HashMap<UserID, User>,
}

impl Room {
    pub fn new() -> Self {
        let id = RoomID::new();

        Room {
            id,
            users: HashMap::new(),
        }
    }
}
