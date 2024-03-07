use leptos::*;
use log::info;

#[component]
pub fn Output(buffer: Vec<String>) -> impl IntoView {
    info!("Output buffer: {:?}", buffer);

    view! {
        <pre>
            {buffer.iter().map(|line| format!("{}\n", line)).collect::<String>()}
        </pre>
    }
}
