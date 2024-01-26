use im::Vector;

use super::events::Event;

pub fn player_count(events: &Vector<Event>) -> usize {
    let mut count = 0;

    for event in events {
        if let Event::PlayerJoined { .. } = event {
            count += 1
        }
    }

    count
}

pub fn game_has_started(events: &Vector<Event>) -> bool {
    for event in events {
        if let Event::GameStarted = event {
            return true;
        }
    }

    false
}
