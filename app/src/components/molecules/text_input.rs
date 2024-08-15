use leptos::*;

#[component]
pub fn text_input(
    id: &'static str,
    name: &'static str,
    value: Option<String>,
    #[prop(optional)] pattern: Option<&'static str>,
    #[prop(optional)] minlength: Option<i32>,
    #[prop(optional)] maxlength: Option<i32>,
    #[prop(optional)] title: Option<&'static str>,
    #[prop(default = false)] uppercase: bool,
) -> impl IntoView {
    view! {
        <div class="input full-width" class:uppercase=uppercase>
            <label for=id>{name}</label>
            <input
                id=id
                value=value
                type="text"
                name=id
                required
                pattern=pattern
                minlength=minlength
                maxlength=maxlength
                title=title
            />

        </div>
    }
}
