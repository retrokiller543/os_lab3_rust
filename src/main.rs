use env_logger::Builder;
use log::LevelFilter;
use rustic_shell::Shell;
use anyhow::Result;

fn setup_logger() {
    Builder::new()
        .filter(None, LevelFilter::Debug)
        .init();
}



fn main() -> Result<()> {
    setup_logger();

    let mut shell = Shell::new()?;

    shell.run()?;
    Ok(())
}
