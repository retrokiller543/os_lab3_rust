use crate::components::output::Output;
use crate::{read_all, GlobalState};
use file_system::prelude::*;
use leptos::ev::Event;
use leptos::ev::SubmitEvent;
use leptos::wasm_bindgen::JsCast;
use leptos::*;
use log::{debug, info};
use web_sys::HtmlInputElement;

// functions to handle commands, each function must have access to the file system and the terminal output signal

// function to parse the command and execute the appropriate function

// Adjusted `ls` function based on the provided mock-up
pub fn ls(fs: &mut FileSystem) -> Result<Vec<String>, String> {
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

fn format(fs: &mut FileSystem) -> Result<(), String> {
    match fs.format().map_err(|e| e.to_string()) {
        Ok(_) => info!("Formatted file system"),
        Err(e) => {
            let error = format!("Failed to format file system: {}", e);
            eprintln!("{}", error);
            return Err(error);
        }
    };
    Ok(())
}

#[derive(Debug)]
struct InputHandler {
    data: String,
}

impl InputConstructor for InputHandler {
    fn new(io: Box<dyn IOHandler<Input = String, Output = String>>) -> Self {
        Self { data: String::new() }
    }
}

impl Input for InputHandler {
    fn read_lines(&mut self) -> anyhow::Result<String> {
        Ok(self.data.clone())
    }
}

fn create(fs: &mut FileSystem, path: &str) -> Result<(), String> {
    match fs.create_file::<InputHandler>(path, &mut InputHandler::new(fs.io_handler.clone_box())) {
        Ok(_) => info!("Created file: {}", path),
        Err(e) => {
            let error = format!("Failed to create file: {}", e);
            eprintln!("{}", error);
            return Err(error);
        }
    }

    Ok(())
}

// Adjusted `execute_command` function
fn execute_command(
    command: &str,
    file_system: &mut FileSystem,
    terminal_output_writer: impl Fn(Vec<String>) + 'static,
) {
    let args = command.split_whitespace().collect::<Vec<&str>>();

    match args[0] {
        "ls" => match ls(file_system) {
            Ok(files) => terminal_output_writer(files),
            Err(e) => terminal_output_writer(vec![e]),
        },
        "format" => {
            match format(file_system) {
                Ok(_) => terminal_output_writer(vec!["format".to_string(), "Formatting file system...".to_string()]),
                Err(e) => terminal_output_writer(vec![e]),
            };
        },
        "help" => terminal_output_writer(vec![
            "Available commands:".to_string(),
            "ls: List directory contents".to_string(),
            "format: Format the file system".to_string(),
            "help: Display this help message".to_string(),
        ]),
        "create" => {
            if args.len() < 2 {
                terminal_output_writer(vec!["create".to_string(), "Missing file name".to_string()]);
                return;
            }
            match create(file_system, args[1]) {
                Ok(_) => terminal_output_writer(vec!["create".to_string(), "Created file".to_string()]),
                Err(e) => terminal_output_writer(vec![e]),
            };
        }
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
        terminal_output.update(|curr| {
            curr.extend(output); // Update the current output
        }); // Set the updated vector
        debug!("terminal_output: {:?}", terminal_output.get());
    });
}

#[component]
pub fn Home() -> impl IntoView {
    let (input_value, set_input_value) = create_signal(String::new()); // State for the user input

    let state = expect_context::<RwSignal<GlobalState>>();

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
            <Output />

            <form on:submit=move|e: SubmitEvent| {
                e.prevent_default(); // Prevent form submission from reloading the page
                handle_command(input_value.get()); // Execute the command
            }>
                <input type="text"
                       value={input_value.get()}
                       on:input=move |e: Event| {
                           if let Some(input) = e.target().unwrap().dyn_into::<HtmlInputElement>().ok() {
                               set_input_value(input.value()); // Update input value as user types
                           }
                       }
                       placeholder="Enter command"/>
                //<button type="submit">{"Execute"}</button>
            </form>
            </ErrorBoundary>
        </div>
    }
}
