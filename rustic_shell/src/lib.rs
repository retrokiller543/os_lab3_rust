use anyhow::Result;
use env_logger::Env;
use file_system::prelude::*;
use log::{debug, error, info, trace, warn};
use std::{
    env,
    io::{self, Write},
};

#[derive(Debug, thiserror::Error)]
enum ShellError {
    #[error("Invalid command usage")]
    InvalidUsage,
    #[error("File system error: {0}")]
    FileSystemError(#[from] anyhow::Error),
}

pub struct Shell {
    file_system: FileSystem,
}

impl Shell {
    pub fn new() -> Result<Shell> {
        trace!("Starting shell...");
        Ok(Shell {
            file_system: FileSystem::new()?,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut running = true;
        while running {
            print!("filesystem> ");
            io::stdout().flush()?;
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;
            let cmd_line: Vec<&str> = line.trim().split_whitespace().collect();
            if cmd_line.is_empty() {
                continue;
            }

            let cmd = cmd_line[0];
            let args = &cmd_line[1..];

            match cmd {
                "quit" => running = false,
                "help" => Self::help(),
                _ => {
                    if let Err(e) = self.execute_command(cmd, args) {
                        error!("Error executing command: {}", e);
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }

        trace!("Exiting shell...");
        Ok(())
    }

    fn execute_command(&mut self, cmd: &str, args: &[&str]) -> Result<()> {
        match cmd {
            "format" => self.format(args),
            "create" => {
                if args.len() != 1 {
                    return Err(ShellError::InvalidUsage.into());
                }
                #[cfg(feature = "debug")]
                {
                    trace!("Running create {}", args[0]);
                }
                self.file_system.create_file(args[0]).map_err(Into::into)
            }
            "cat" => {
                if args.len() != 1 {
                    return Err(ShellError::InvalidUsage.into());
                }
                #[cfg(feature = "debug")]
                {
                    trace!("Running cat {}", args[0]);
                }
                self.file_system.read_file(args[0]).map_err(Into::into)
            }
            "ls" => {
                if args.len() != 0 {
                    return Err(ShellError::InvalidUsage.into());
                }
                #[cfg(feature = "debug")]
                {
                    trace!("Running ls");
                }
                self.file_system.list_dir().map_err(Into::into)
            }
            _ => Err(ShellError::InvalidUsage.into()),
        }
    }

    fn format(&mut self, args: &[&str]) -> Result<()> {
        if args.len() != 0 {
            return Err(ShellError::InvalidUsage.into());
        }
        self.file_system.format().map_err(Into::into)
    }

    // Implement other command functions (create, cat, etc.) similarly

    fn help() {
        let commands = [
            "format", "create", "cat", "ls", "cp", "mv", "rm", "append", "mkdir", "cd", "pwd",
            "chmod", "help", "quit",
        ];

        for command in commands {
            println!("{}", command);
        }
    }
}
