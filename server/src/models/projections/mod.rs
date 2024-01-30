use super::events::Event;
use im::Vector;
use std::collections::HashMap;
use uuid::Uuid;

pub fn player_count(events: &Vector<Event>) -> usize {
    let mut count = 0;

    for event in events {
        if let Event::PlayerJoined { .. } = event {
            count += 1
        }
    }

    count
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct PlayerInfo {
    pub session_id: Uuid,
    pub name: String,
    pub ready: bool,
}

pub fn players(events: &Vector<Event>) -> HashMap<Uuid, PlayerInfo> {
    let mut map = HashMap::new();

    for event in events {
        match event {
            Event::PlayerJoined { name, session_id } => {
                map.insert(
                    *session_id,
                    PlayerInfo {
                        session_id: *session_id,
                        name: name.clone(),
                        ready: false,
                    },
                );
            }
            Event::ChangedProfile { session_id, name } => {
                if let Some(info) = map.get_mut(session_id) {
                    info.name = name.clone();
                }
            }
            Event::PlayerReady { session_id } => {
                if let Some(info) = map.get_mut(session_id) {
                    info.ready = true
                }
            }
            _ => {}
        }
    }

    map
}

pub fn game_has_started(events: &Vector<Event>) -> bool {
    for event in events {
        if let Event::GameStarted = event {
            return true;
        }
    }

    false
}

pub fn player_exists(events: &Vector<Event>, session_id: Uuid) -> bool {
    for event in events {
        if let Event::PlayerJoined {
            session_id: inner_session_id,
            ..
        } = event
        {
            if session_id == *inner_session_id {
                return true;
            }
        }
    }

    false
}
