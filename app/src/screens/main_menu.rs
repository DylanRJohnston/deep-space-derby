use leptos::prelude::*;
use leptos_router::components::Form;

use crate::components::layouts::HorizontalStack;

#[component]
pub fn main_menu() -> impl IntoView {
    view! {
        <div class="vertical-stack container full-height">
            <div class="headroom"></div>
            <h1 class="title">"Deep Space Derby"</h1>
            // <div class="splash-image">"Image"</div>
            <HorizontalStack>
                <Form attr:class="button-form" action="/api/create_game" method="POST">
                    <input class="button" type="submit" value="Host" />
                </Form>
                <a class="button" href="/play">
                    "Join"
                </a>
            </HorizontalStack>
        </div>
    }
}
