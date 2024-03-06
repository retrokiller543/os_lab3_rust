use leptos::*;
use crate::ls;

#[component]
pub fn Home() -> impl IntoView {
    let (terminal_output, set_terminal_output) = create_signal(String::from("hiii"));

    // Attempt to list directory contents and update the terminal output
    if let Ok(contents) = ls() {
        set_terminal_output(contents);
    } else if let Err(e) = ls() {
        set_terminal_output(e);
    }

    view! {
        <div class="terminal">
            {terminal_output}
        </div>
    }
}
