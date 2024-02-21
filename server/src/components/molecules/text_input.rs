use leptos::*;

#[component]
pub fn text_input(
    id: &'static str,
    name: &'static str,
    #[prop(optional)] pattern: Option<&'static str>,
    #[prop(optional)] minlength: Option<i32>,
    #[prop(optional)] maxlength: Option<i32>,
) -> impl IntoView {
    view! {
        <div class="input full-width">
            <label for=id>{name}</label>
            <input
                id=id
                type="text"
                name=id
                required
                pattern=pattern
                minlength=minlength
                maxlength=maxlength
            />
        </div>
    }
}
