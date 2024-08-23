use leptos::*;
use leptos_router::*;

use crate::components::molecules::TextInput;

#[component]
pub fn join() -> impl IntoView {
    let param_map = use_query_map();
    let game_id = param_map.get_untracked().get("code").cloned();

    view! {
        <div class="vertical-stack container full-height">
            <div class="headroom"></div>
            <h1 class="title">"Join a game"</h1>
            <div class="placeholder-image">"Image"</div>
            <Form action="/api/join_game" method="POST" class="vertical-stack">
                <TextInput
                    id="code"
                    name="Lobby Code"
                    value=game_id
                    pattern="[a-zA-Z0-9]{6}"
                    minlength=6
                    maxlength=6
                    title="6 alpha-numerical characters e.g. ABC123"
                    uppercase=true
                />
                <TextInput
                    id="name"
                    name="Name"
                    pattern="[a-zA-Z]{1,10}"
                    minlength=1
                    maxlength=10
                    title="player name between 1-10 characters long"
                    value=None
                />
                <input class="button" type="submit" value="Join"/>
            </Form>
        </div>
    }
}
