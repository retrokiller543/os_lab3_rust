use crate::GlobalState;
use leptos::*;
use log::info;

#[component]
pub fn Output() -> impl IntoView {
    let state = expect_context::<RwSignal<GlobalState>>();
    let buffer_signal = create_rw_signal(Vec::new());

    create_effect(move |_| {
        // Log the current terminal output from global state
        info!(
            "Current terminal buffer output: {:?}",
            state.get().terminal_output.get()
        );
        buffer_signal.set(state.get().terminal_output.get().clone());
    });

    let output = move || {
        let buffer = buffer_signal.get();
        buffer.iter().map(|line| format!("{}\n", line)).collect::<String>()
    };

    view! {
        <pre>
            {output}
        </pre>
    }
}
