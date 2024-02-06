use leptos::*;
use leptos_router::*;

#[component]
pub fn join_screen() -> impl IntoView {
    view! {
        <Form action="/api/join_game" method="POST" class="join-screen">
            <h1>"Join Game"</h1>
                <label for="code">"Lobby Code"</label>
                <input
                    id="code"
                    type="text"
                    name="code"
                    required
                    pattern="[A-Z0-9]{6}"
                    minlength=6
                    maxlength=6
                />
                <input
                    type="text"
                    name="name"
                    prop:value="Bob"
                    hidden
                />
            <div class="button-tray">
                <button>
                    <a href="/">"Back"</a>
                </button>
               <input type="submit" value="Join" />
            </div>
        </Form>
    }
}
