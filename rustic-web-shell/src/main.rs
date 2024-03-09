use chrono::Local;
use env_logger::{Builder, Env};
use leptos::*;
use log::info;
use rustic_web_shell::App;
use std::io::Write;

pub fn setup_logger() -> anyhow::Result<()> {
    Builder::from_env(Env::default().default_filter_or("trace"))
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {} - {}:{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record
                    .file()
                    .unwrap_or(record.module_path().unwrap_or("unknown")),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .target(env_logger::Target::Stdout)
        .init();
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // set up logging
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    //setup_logger()?;

    info!("Starting up...");

    mount_to_body(|| {
        view! {
            <App />
        }
    });

    Ok(())
}
