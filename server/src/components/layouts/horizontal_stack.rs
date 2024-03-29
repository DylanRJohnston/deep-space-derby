use leptos::*;

#[component]
pub fn horizontal_stack(children: Children) -> impl IntoView {
    view! { <div class="horizontal-stack full-width">{children()}</div> }
}
