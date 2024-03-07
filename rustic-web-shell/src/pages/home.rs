use crate::components::output::Output;
use crate::{read_all, GlobalState, MemIOHandler};
use file_system::prelude::*;
use file_system::FileSystem;
use leptos::ev::Event;
use leptos::ev::SubmitEvent;
use leptos::wasm_bindgen::JsCast;
use leptos::*;
use log::{debug, info};
use std::sync::{Arc, Mutex};
use web_sys::HtmlInputElement;

// functions to handle commands, each function must have access to the file system and the terminal output signal

// function to parse the command and execute the appropriate function

// Adjusted `ls` function based on the provided mock-up
pub fn ls(fs: &mut FileSystem) -> Result<Vec<String>, String> {
    match fs.format().map_err(|e| e.to_string()) {
        Ok(_) => info!("Formatted file system"),
        Err(e) => {
            let error = format!("Failed to format file system: {}", e);
            eprintln!("{}", error);
            return Err(error);
        }
    };
    match fs
        .create_file_with_content("file1.txt", "hello world!".repeat(100).as_str())
        .map_err(|e| e.to_string())
    {
        Ok(_) => info!("Created file1.txt with content"),
        Err(e) => {
            let error = format!("Failed to create file1.txt with content: {}", e);
            eprintln!("{}", error);
            return Err(error);
        }
    };
    match fs.list_dir().map_err(|e| e.to_string()) {
        Ok(_) => println!("Listing directory contents..."),
        Err(e) => {
            let error = format!("Failed to list directory contents: {}", e);
            eprintln!("{}", error);
            return Err(error);
        }
    };

    let io_handler = fs.io_handler.as_mut();

    let output = read_all(io_handler);
    #[cfg(debug_assertions)]
    debug!("ls output: {:?}", output);
    Ok(output)
}

// Adjusted `execute_command` function
fn execute_command(
    command: &str,
    file_system: &mut FileSystem,
    terminal_output_writer: impl Fn(Vec<String>) + 'static,
) {
    match command {
        "ls" => match ls(file_system) {
            Ok(files) => terminal_output_writer(files),
            Err(e) => terminal_output_writer(vec![e]),
        },
        _ => terminal_output_writer(vec![format!("Unknown command: {}", command)]),
    }
}

fn handle_input(
    input: String,
    file_system: &mut FileSystem,
    terminal_output: RwSignal<Vec<String>>,
) {
    execute_command(&input, file_system, move |output| {
        let current_output = terminal_output.get(); // Get current output
        debug!("current_output: {:?}", current_output);
        debug!("terminal_output: {:?}", terminal_output.get());
        terminal_output.update(|mut curr| {
            curr.extend(output); // Update the current output
        }); // Set the updated vector
        debug!("terminal_output: {:?}", terminal_output.get());
    });
}

#[component]
pub fn Home() -> impl IntoView {
    let file_system = Arc::new(Mutex::new(
        FileSystem::new(Box::new(MemIOHandler::new())).unwrap(),
    ));
    let (input_value, set_input_value) = create_signal(String::new()); // State for the user input

    let state = expect_context::<RwSignal<GlobalState>>();

    create_effect(move |_| {
        // Log the current terminal output from global state
        info!(
            "Current terminal output: {:?}",
            state.get().terminal_output.get()
        );
    });

    let handle_command = {
        move |command: String| {
            handle_input(
                command,
                &mut state.get().file_system.get(),
                state.get().terminal_output,
            );
        }
    };

    view! {
        <div class="terminal">
            <ErrorBoundary
                fallback=|errors| view! {
                    <div class="error">
                        <p>"Errors: "</p>
                        <ul>
                            {move || errors.get()
                                .into_iter()
                                .map(|(_, e)| view! { <li>{e.to_string()}</li>})
                                .collect_view()
                            }
                        </ul>
                    </div>
                }
            >

            <form on:submit=move|e: SubmitEvent| {
                e.prevent_default(); // Prevent form submission from reloading the page
                handle_command(input_value.get()); // Execute the command
                set_input_value(String::new()); // Reset input field after command execution
            }>
                <input type="text"
                       value={input_value.get()}
                       on:input=move |e: Event| {
                           if let Some(input) = e.target().unwrap().dyn_into::<HtmlInputElement>().ok() {
                               set_input_value(input.value()); // Update input value as user types
                           }
                       }
                       placeholder="Enter command"/>
                <button type="submit">{"Execute"}</button>
            </form>
            <Output />
            </ErrorBoundary>
        </div>
    }
}
