use leptos::SignalGet;
use leptos_reactive::SpecialNonReactiveZone;
use leptos_router::use_params_map;

pub fn use_game_id() -> String {
    let params = use_params_map();

    let prev = SpecialNonReactiveZone::enter();
    let code = params.get().get("game_id").unwrap().clone();
    SpecialNonReactiveZone::exit(prev);

    code
}
