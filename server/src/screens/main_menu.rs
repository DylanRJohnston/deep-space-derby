use leptos::{component, view, IntoView};
use leptos_router::Form;

#[component]
pub fn main_menu() -> impl IntoView {
    view! {
        <Form action="/api/create_game" method="POST" class="main-menu">
            <h1 class="title">"Deep Space Derby"</h1>
            <input class="host" type="submit" value="Host" />
            <div class="play">
                <a href="/play">
                    <h2>"Play"</h2>
                </a>
            </div>
            <div class="exit">
                <h2>"Exit"</h2>
            </div>
        </Form>
    }
}
