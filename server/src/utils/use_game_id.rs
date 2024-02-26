use leptos::{SignalGet, SignalGetUntracked};
use leptos_reactive::SpecialNonReactiveZone;
use leptos_router::use_params_map;

use shared::models::game_id::GameID;

pub fn use_game_id() -> GameID {
    let params = use_params_map();

    let code = params.get_untracked().get("game_id").unwrap().clone();

    code.as_str().try_into().unwrap()
}

