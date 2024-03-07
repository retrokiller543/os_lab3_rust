use anyhow::Result;
use chrono::Local;
use env_logger::{Builder, Env};
use rustic_shell::Shell;
use std::fs::File;
use std::io::Write;

pub fn setup_logger() -> Result<()> {
    // Get the current timestamp
    let now = Local::now();
    // Format the timestamp as a string in the desired format
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    // Create the log filename with the timestamp
    let log_filename = format!("logs/{}.log", timestamp);
    // Create the log file and directory if needed
    std::fs::create_dir_all("logs")?;

    let file = File::create(log_filename)?;

    Builder::from_env(Env::default().default_filter_or("info"))
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
        .target(env_logger::Target::Pipe(Box::new(file)))
        .init();
    Ok(())
}

pub fn run_shell() -> Result<()> {
    setup_logger()?;

    let mut shell = Shell::new()?;
    shell.run()?;
    Ok(())
}

fn main() -> Result<()> {
    run_shell()?;

    Ok(())
}
