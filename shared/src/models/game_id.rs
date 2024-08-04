use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use anyhow::bail;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Serialize,
};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct GameID([u8; 6]);

pub fn generate_game_code() -> GameID {
    GameID(
        Uniform::from(b'A'..=b'Z')
            .sample_iter(&mut thread_rng())
            .take(6)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    )
}

impl Debug for GameID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("GameID")
            .field(&std::str::from_utf8(&self.0))
            .finish()
    }
}

impl Display for GameID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(std::str::from_utf8(&self.0).unwrap()).unwrap();
        Ok(())
    }
}

impl TryFrom<&str> for GameID {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 6 {
            bail!("failed to convert from String to GameID, incorrect length");
        }

        Ok(GameID(
            value
                .to_uppercase()
                .as_bytes()
                .to_owned()
                .try_into()
                .unwrap(),
        ))
    }
}

impl From<GameID> for String {
    fn from(value: GameID) -> Self {
        value.deref().to_owned()
    }
}

impl Deref for GameID {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        std::str::from_utf8(&self.0).unwrap()
    }
}

struct GameIDVisitor;

impl<'de> Visitor<'de> for GameIDVisitor {
    type Value = GameID;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string containing exactly 6 characters")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.try_into()
            .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(v), &self))
    }
}

impl<'de> Deserialize<'de> for GameID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(GameIDVisitor)
    }
}

impl Serialize for GameID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(std::str::from_utf8(&self.0).unwrap())
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use crate::models::game_id::GameID;

    #[test]
    fn test_deserialize() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Test {
            game_id: GameID,
        }

        let input = "{ \"game_id\": \"ABCDEF\" }";
        let output = serde_json::from_str::<Test>(input).unwrap();

        assert_eq!([b'A', b'B', b'C', b'D', b'E', b'F'], output.game_id.0);
    }

    #[test]
    fn test_serialize() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Test {
            game_id: GameID,
        }

        let input = "{\"game_id\":\"ABCDEF\"}";
        let output = serde_json::from_str::<Test>(input).unwrap();
        let result = serde_json::to_string(&output).unwrap();

        assert_eq!(input, result);
    }

    #[test]
    fn test_uppercase() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Test {
            game_id: GameID,
        }

        let input = "{ \"game_id\": \"abcdef\" }";
        let output = serde_json::from_str::<Test>(input).unwrap();

        assert_eq!([b'A', b'B', b'C', b'D', b'E', b'F'], output.game_id.0);
    }
}
