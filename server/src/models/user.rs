use rocket::request::FromParam;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone, Eq, PartialEq, Hash, Copy)]
pub struct UserID(Uuid);

impl<'a> FromParam<'a> for UserID {
    type Error = <Uuid as FromParam<'a>>::Error;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Uuid::from_param(param).map(UserID)
    }
}

impl UserID {
    pub fn new() -> Self {
        UserID(Uuid::now_v7())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct User {
    pub id: UserID,
}

impl User {
    pub fn new() -> Self {
        User { id: UserID::new() }
    }
}
