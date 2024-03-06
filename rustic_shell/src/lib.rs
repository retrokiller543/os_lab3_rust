use anyhow::Result;
use file_system::prelude::*;
use log::{error, trace};
use std::io::{self, Write};

/// # Shell Errors
/// Represents errors that can occur within the Shell.
///
/// This enum is used to encapsulate various kinds of errors that can arise
/// during the operation of the Shell, such as invalid command usage or file
/// system related errors.
/// It leverages `thiserror` for error definitions to
/// simplify error handling and propagation.
#[derive(Debug, thiserror::Error)]
enum ShellError {
    /// # Invalid usage
    /// Error indicating that a command was used incorrectly.
    ///
    /// This variant is used when a command receives arguments that don't
    /// satisfy its requirements, such as missing arguments or incorrect
    /// option flags.
    #[error("Invalid command usage")]
    InvalidUsage,

    /// # File system error
    /// Represents an error stemming from file system operations.
    ///
    /// This variant encapsulates errors returned by the file system, such as
    /// permissions issues, missing files, or other I/O related errors. It uses
    /// `anyhow::Error` to provide flexibility in the types of errors that can
    /// be included, allowing for easy error conversion and propagation.
    #[error("File system error: {0}")]
    FileSystemError(#[from] anyhow::Error),
}

/// # Shell
/// Represents the main Shell structure.
///
/// This struct holds the state and functionality for the shell's operation,
/// including managing the file system operations. It acts as the core of the
/// shell application, processing commands and handling errors.
pub struct Shell {
    /// # file_system
    /// The file system component of the shell.
    ///
    /// This field manages interactions with the file system, such as executing
    /// commands that involve file and directory operations. It's encapsulated within
    /// the shell to centralize file system access and error handling.
    file_system: FileSystem,
}

/// `command_handler` is a macro that takes four arguments:
/// - `$self`: the instance of the `Shell` struct
/// - `$cmd`: the command to be executed
/// - `$args`: the arguments for the command
/// - a mapping of commands to their handlers and the expected number of arguments
///
/// The macro matches the given command with the provided mapping and executes the corresponding handler.
/// If the command is not found in the mapping, it returns an `InvalidUsage` error.
macro_rules! command_handler {
    ($self:expr, $cmd:expr, $args:expr, { $($command:expr => $handler:ident($($arg_count:expr),*)),* $(,)? }) => {
        match $cmd {
            $(
                $command => {
                    // Validate the number of arguments for the command
                    let expected_args: &[usize] = &[$($arg_count),*];
                    if !expected_args.contains(&$args.len()) {
                        return Err(ShellError::InvalidUsage.into());
                    }

                    // Debug log if the `debug` feature is enabled
                    #[cfg(feature = "debug")]
                    {
                        let args_str = $args.join(" ");
                        trace!("Running {}({})", $command, args_str);
                    }

                    // Call the appropriate function
                    $self.$handler($args)
                },
            )*
            _ => Err(ShellError::InvalidUsage.into()),
        }
    };
}

/// `function_handler` is a macro that generates functions to handle specific commands.
/// It takes two or three arguments:
/// - `$self`: the instance of the `Shell` struct
/// - `$func_ident`: the identifier of the function to be created
/// - `$arg1` and `$arg2` (optional): the indices of the arguments to be passed to the function
///
/// The macro generates a function that calls a method on the `file_system` field of the `Shell` struct.
/// The method to be called and the arguments to be passed are determined by the arguments to the macro.
macro_rules! function_handler {
    ($func_ident:ident) => {
        fn $func_ident(&mut self, _args: &[&str]) -> Result<()> {
            self.file_system.$func_ident().map_err(Into::into)
        }
    };

    ($func_ident:ident, $arg1:expr) => {
        fn $func_ident(&mut self, args: &[&str]) -> Result<()> {
            self.file_system
                .$func_ident(args[$arg1])
                .map_err(Into::into)
        }
    };

    ($func_ident:ident, $arg1:expr, $arg2:expr) => {
        fn $func_ident(&mut self, args: &[&str]) -> Result<()> {
            self.file_system
                .$func_ident(args[$arg1], args[$arg2])
                .map_err(Into::into)
        }
    };
}

impl Shell {
    /// Creates a new instance of the `Shell`.
    ///
    /// Initializes the shell with a new file system. This function logs the start
    /// of the shell using a trace macro and attempts to create a new `FileSystem`
    /// instance. If successful, it returns the shell instance wrapped in a `Result`.
    ///
    /// Returns:
    /// - `Ok(Shell)`: A new instance of `Shell`.
    /// - `Err(e)`: An error if the `FileSystem::new()` call fails.
    pub fn new() -> Result<Shell> {
        trace!("Starting shell...");
        let io_handler = Box::new(StdIOHandler); // This is a mock input handler
        Ok(Shell {
            file_system: FileSystem::new(io_handler)?,
        })
    }

    /// Runs the shell loop, processing user input commands.
    ///
    /// This method starts a loop that continuously prompts the user for input,
    /// processes the input as commands, and executes them until the "quit" command
    /// is received. It handles command execution errors and flushes the stdout buffer
    /// to ensure that the prompt is displayed properly.
    ///
    /// Returns:
    /// - `Ok(())`: If the loop exits normally.
    /// - `Err(e)`: If an error occurs while flushing stdout or reading from stdin.
    pub fn run(&mut self) -> Result<()> {
        let mut running = true;
        while running {
            print!("filesystem> ");
            io::stdout().flush()?;
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;
            let cmd_line: Vec<&str> = line.split_whitespace().collect();
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

    /// Executes a given command with arguments.
    ///
    /// This method matches the input command with a predefined set of commands
    /// and executes the corresponding function. It leverages the `command_handler!`
    /// macro (assumed to be defined elsewhere) for mapping commands to their
    /// handlers.
    ///
    /// Parameters:
    /// - `cmd`: The command to execute.
    /// - `args`: A slice of arguments for the command.
    ///
    /// Returns:
    /// - `Ok(())`: If the command is executed successfully.
    /// - `Err(e)`: If an error occurs during command execution.
    fn execute_command(&mut self, cmd: &str, args: &[&str]) -> Result<()> {
        command_handler! {self, cmd, args, {
            "format" => format(0), // No arguments expected for format
            "create" => create_file_stdio(1), // Expects exactly 1 argument
            "cat" => read_file(1), // Expects exactly 1 argument
            "ls" => list_dir(0), // No arguments expected for ls
            "cp" => copy_entry(2), // Expects exactly 2 arguments
            "mv" => move_entry(2), // Expects exactly 2 arguments
            "append" => append_file(2), // Expects exactly 2 arguments
            "mkdir" => create_dir(1), // Expects exactly 1 argument
            "cd" => change_dir(1), // Expects exactly 1 argument
            "pwd" => print_working_dir(0), // No arguments expected for pwd
            //"chmod" => change_permissions(2), // Expects exactly 2 arguments
            "rm" => remove_entry(1), // Expects exactly 1 argument
        }}
    }

    function_handler! {format}
    function_handler! {create_file_stdio, 0}
    function_handler! {read_file, 0}
    function_handler! {list_dir}
    function_handler! {copy_entry, 0, 1}
    function_handler! {move_entry, 0, 1}
    function_handler! {append_file, 0, 1}
    function_handler! {create_dir, 0}
    function_handler! {change_dir, 0}
    function_handler! {print_working_dir}
    //function_handler!{change_permissions, 0, 1}
    function_handler! {remove_entry, 0}

    /// Displays help information for available commands.
    ///
    /// This static method prints a list of available commands to the standard output.
    /// It is intended to provide users with a quick reference to the commands that
    /// the shell supports.
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
