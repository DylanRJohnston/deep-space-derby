use leptos::*;
use leptos_meta::provide_meta_context;

#[server]
pub async fn hello_world_fn() -> Result<String, ServerFnError> {
    Ok("Hello, World, from the server!".into())
}

#[component]
pub fn app() -> impl leptos::IntoView {
    provide_meta_context();

    let click = create_action(|_| hello_world_fn());

    let title = move || {
        click
            .value()
            .get()
            .unwrap_or(Ok("Hello, World".into()))
            .unwrap_or("Hello, World".into())
    };

    leptos::view! {
        <div>
            <h1>{title}</h1>
            <button on:click=move |_|  click.dispatch(())>"What does the server think?"</button>
        </div>
    }
}
