use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use log::{error, info};
use file_system::FileSystem;
use file_system::prelude::{Directory, IOHandler, IOHandlerError};

// Modules
mod components;
mod pages;

// Top-Level pages
use crate::pages::home::Home;
use crate::pages::not_found::NotFound;

#[derive(Debug)]
pub struct MemIOHandler {
    buffer: Vec<String>,
}

impl MemIOHandler {
    fn new() -> Self {
        MemIOHandler {
            buffer: Vec::new(),
        }
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

pub fn ls() -> Result<String, String> {
    let mut fs = FileSystem::new(Box::new(MemIOHandler::new())).map_err(|e| e.to_string())?;
    match fs.list_dir().map_err(|e| e.to_string()) {
        Ok(_) => {
            println!("Listing directory contents...");
        }
        Err(e) => {
            let error = format!("Failed to list directory contents: {}", e);
            eprintln!("{}", error);
            return Err(error);
        }
    };
    let output = fs.io_handler.read().map_err(|e| e.to_string())?;
    Ok(output)
}

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Html lang="en" dir="ltr" attr:data-theme="light"/>

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
