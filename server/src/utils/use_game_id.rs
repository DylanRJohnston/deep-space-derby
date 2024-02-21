use leptos::SignalGet;
use leptos_reactive::SpecialNonReactiveZone;
use leptos_router::use_params_map;

use crate::models::game_id::GameID;

pub fn use_game_id() -> GameID {
    let params = use_params_map();

    let prev = SpecialNonReactiveZone::enter();
    let code = params.get().get("game_id").unwrap().clone();
    SpecialNonReactiveZone::exit(prev);

    code.as_str().try_into().unwrap()
}
