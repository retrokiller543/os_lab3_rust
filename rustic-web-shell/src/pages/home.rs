use std::sync::{Arc, Mutex};
use leptos::*;
use file_system::FileSystem;
use file_system::prelude::{Directory, File, Format};
use crate::{MemIOHandler, read_all};
use crate::components::output::Output;

// functions to handle commands, each function must have access to the file system and the terminal output signal

// function to parse the command and execute the appropriate function

// Adjusted `ls` function based on the provided mock-up
pub fn ls(mut fs: &mut FileSystem) -> Result<Vec<String>, String> {
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
    Ok(output)
}

// Adjusted `execute_command` function
fn execute_command(command: &str, file_system: &mut FileSystem, terminal_output_writer: impl Fn(Vec<String>) + 'static) {
    match command {
        "ls" => {
            match ls(file_system) {
                Ok(files) => terminal_output_writer(files),
                Err(e) => terminal_output_writer(vec![e]),
            }
        },
        _ => terminal_output_writer(vec![format!("Unknown command: {}", command)]),
    }
}

fn handle_input(input: String, file_system: &mut FileSystem, terminal_output: ReadSignal<Vec<String>>, set_terminal_output: WriteSignal<Vec<String>>) {
    execute_command(&input, file_system, move|output| {
        let mut current_output = terminal_output.get();
        current_output.extend(output);
        set_terminal_output(current_output);
    });
}

#[component]
pub fn Home() -> impl IntoView {
    let (terminal_output, set_terminal_output) = create_signal(Vec::new());
    let file_system = Arc::new(Mutex::new(FileSystem::new(Box::new(MemIOHandler::new())).unwrap()));
    let file_system_clone = file_system.clone();

    handle_input("ls".to_string(), &mut file_system_clone.lock().unwrap(), terminal_output.clone(), set_terminal_output.clone());

    view! {
        <div class="terminal">
            <ErrorBoundary
                    // the fallback receives a signal containing current errors
                    fallback=|errors| view! {
                        <div class="error">
                            <p>"Errors: "</p>
                            // we can render a list of errors as strings, if we'd like
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
                <Output buffer={terminal_output.get().clone()} />
        //create a input form to get user commands
            </ErrorBoundary>
        </div>
    }
}
