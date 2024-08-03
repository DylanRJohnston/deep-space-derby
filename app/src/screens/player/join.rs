use leptos::*;
use leptos_router::*;

use crate::components::molecules::TextInput;

#[component]
pub fn join() -> impl IntoView {
    view! {
        <div class="vertical-stack container full-height">
            <div class="headroom"></div>
            <h1 class="title">"Join a game"</h1>
            <div class="splash-image">"Image"</div>
            <Form action="/api/join_game" method="POST" class="vertical-stack">
                <TextInput
                    id="code"
                    name="Lobby Code"
                    pattern="[a-zA-Z0-9]{6}"
                    minlength=6
                    maxlength=6
                    title="6 alpha-numerical characters e.g. ABC123"
                    uppercase=true
                />
                <input type="text" name="name" prop:value="Bob" hidden/>
                <input class="button" type="submit" value="Join"/>
            </Form>
        </div>
    }
}
