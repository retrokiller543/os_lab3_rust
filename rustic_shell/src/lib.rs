use anyhow::Result;
use file_system::prelude::*;
use log::{info, warn, error};
use std::{io::{self, Write}, env};
use env_logger::Env;

#[derive(Debug, thiserror::Error)]
enum ShellError {
    #[error("Invalid command usage")]
    InvalidUsage,
    #[error("File system error: {0}")]
    FileSystemError(#[from] anyhow::Error), // Assume FileSystem errors are anyhow::Error for simplicity
}

pub struct Shell {
    file_system: FileSystem,
}

macro_rules! run_command {
    ($cmd:expr, $args:expr, $req_args:expr) => {
        if $args.len() != $req_args {
            return Err(ShellError::InvalidUsage.into());
        } else {
            self.file_system.$cmd($args).map_err(Into::into)
        }
    }
}

impl Shell {
    pub fn new() -> Result<Shell> {
        info!("Starting shell...");
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
                    }
                }
            }
        }

        info!("Exiting shell...");
        Ok(())
    }

    fn execute_command(&mut self, cmd: &str, args: &[&str]) -> Result<()> {
        match cmd {
            "format" => self.format(args),
            "create" => {
                if args.len() != 1 {
                    return Err(ShellError::InvalidUsage.into());
                }
                self.file_system.create_file(args[0]).map_err(Into::into)
            }
            "cat" => {
                if args.len() != 1 {
                    return Err(ShellError::InvalidUsage.into());
                }
                self.file_system.read_file(args[0]).map_err(Into::into)
            }
            "ls" => {
                if args.len() != 0 {
                    return Err(ShellError::InvalidUsage.into());
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