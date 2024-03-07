use leptos::*;
use log::info;
use crate::GlobalState;

#[component]
pub fn Output() -> impl IntoView {
    let state = expect_context::<RwSignal<GlobalState>>();
    let mut buffer = Vec::new();

    create_effect(move |_| {
        // Log the current terminal output from global state
        info!("Current terminal buffer output: {:?}", state.get().terminal_output.get());
        buffer.extend(state.get().terminal_output.get().iter().cloned());
    });

    view! {
        <pre>
            {buffer.iter().map(|line| format!("{}\n", line)).collect::<String>()}
        </pre>
    }
}
