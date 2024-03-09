use file_system::prelude::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

// Modules
mod components;
mod pages;

// Top-Level pages
use crate::pages::home::Home;
use crate::pages::not_found::NotFound;

#[derive(Debug, Clone)]
pub struct MemIOHandler {
    pub buffer: Vec<String>,
}

impl MemIOHandler {
    fn new() -> Self {
        MemIOHandler { buffer: Vec::new() }
    }
}

impl IOHandler for MemIOHandler {
    type Input = String;
    type Output = String;

    fn read(&mut self) -> anyhow::Result<String> {
        if let Some(line) = self.buffer.pop() {
            Ok(line)
        } else {
            Err(IOHandlerError::IOError("No more input".to_string()).into())
        }
    }

    fn write(&mut self, content: String) -> anyhow::Result<()> {
        self.buffer.push(content);
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GlobalState {
    pub file_system: RwSignal<FileSystem>,
    pub terminal_output: RwSignal<Vec<String>>,
}

impl GlobalState {
    pub fn new() -> Self {
        let file_system = FileSystem::new(Box::new(MemIOHandler::new())).unwrap();
        let file_system = create_rw_signal(file_system);
        let terminal_output = create_rw_signal(Vec::new());

        GlobalState {
            file_system,
            terminal_output,
        }
    }
}

pub fn read_all(io_handler: &mut dyn IOHandler<Input = String, Output = String>) -> Vec<String> {
    let mut buffer = Vec::new();

    while let Ok(line) = io_handler.read() {
        buffer.push(line);
    }
    buffer.iter().rev().map(|s| s.to_string()).collect()
}

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    provide_context(create_rw_signal(GlobalState::new()));

    view! {
        <Html lang="en" dir="ltr" attr:data-theme="dark"/>

        // sets the document title
        <Title text="RusticOS"/>

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        <Router>
            <Routes>
                <Route path="/" view=Home/>
                <Route path="/*" view=NotFound/>
            </Routes>
        </Router>
    }
}
