use leptos::*;
use log::{error, info};
use rustic_web_shell::{App, ls};



fn main() {
    // set up logging
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    let test = ls();
    match test {
        Ok(output) => {
            println!("Listing directory contents: {}", output);
        }
        Err(e) => {
            eprintln!("Failed to list directory contents: {}", e);
        }
    }

    mount_to_body(|| {
        view! {
            <App />
        }
    })
}
