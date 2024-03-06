use leptos::*;

#[component]
pub fn Output(buffer: Vec<String>) -> impl IntoView {

    view! {
        <pre>
            {buffer.iter().map(|line| format!("{}\n", line)).collect::<String>()}
        </pre>
    }
}
