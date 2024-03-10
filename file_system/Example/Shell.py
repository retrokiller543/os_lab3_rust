from typing import Any

from RusticFS import FileSystem

from ShellError import ShellError


class Shell:
    fs: FileSystem

    def __init__(self, /, **data: Any):
        super().__init__(**data)
        self.fs = FileSystem()

    def execute_command(self, cmd: str, args: list[str]):
        handlers = {
            "format": self.format,
            "create": self.create_file,
            "cat": self.read_file,
            "ls": self.list_dir,
            "append": self.append_file,
            "rm": self.remove_entry,
            "mv": self.move_entry,
            "cp": self.copy_entry,
            "mkdir": self.create_dir,
            "pwd": self.print_working_dir,
            "cd": self.change_dir,
            "chmod": self.change_permissions,
            "dbg": self.dbg,
            "exec": self.exec,
            "quit": lambda _: None,
        }

        if cmd not in handlers:
            raise ShellError("Invalid command")

        try:
            handlers[cmd](args)
        except Exception as e:
            print(f"Error: {e}")

    def dbg(self, args: list[str]):
        if args:
            raise ShellError("Invalid number of arguments for dbg")
        print(self.fs)

    def format(self, args: list[str]):
        # Implement format logic using RusticFS methods
        if args:
            raise ShellError("Invalid number of arguments for format")
        self.fs.format()

    def create_file(self, args: list[str]):
        # Implement create_file logic using RusticFS methods
        if len(args) != 1:
            raise ShellError(f"Invalid number of arguments for create: {args}")
        self.fs.create_file(args[0])

    def read_file(self, args: list[str]):
        # Implement read_file logic using RusticFS methods
        if len(args) != 1:
            raise ShellError("Invalid number of arguments for cat")
        print(self.fs.read_file(args[0]))

    def list_dir(self, args: list[str]):
        # Implement list_dir logic using RusticFS methods
        if args:
            raise ShellError("Invalid number of arguments for ls")
        print(self.fs.list_dir())

    def append_file(self, args: list[str]):
        # Implement append_file logic using RusticFS methods
        if len(args) != 2:
            raise ShellError("Invalid number of arguments for append")
        self.fs.append_file(args[0], args[1])

    def remove_entry(self, args: list[str]):
        # Implement remove_entry logic using RusticFS methods
        if len(args) != 1:
            raise ShellError("Invalid number of arguments for rm")
        self.fs.remove_entry(args[0])

    def move_entry(self, args: list[str]):
        # Implement move_entry logic using RusticFS methods
        if len(args) != 2:
            raise ShellError("Invalid number of arguments for mv")
        self.fs.move_entry(args[0], args[1])

    def copy_entry(self, args: list[str]):
        # Implement copy_entry logic using RusticFS methods
        if len(args) != 2:
            raise ShellError("Invalid number of arguments for cp")
        self.fs.copy_entry(args[0], args[1])

    def create_dir(self, args: list[str]):
        # Implement create_dir logic using RusticFS methods
        if len(args) != 1:
            raise ShellError("Invalid number of arguments for mkdir")
        self.fs.create_dir(args[0])

    def print_working_dir(self, args: list[str]):
        # Implement print_working_dir logic using RusticFS methods
        if args:
            raise ShellError("Invalid number of arguments for pwd")
        print(self.fs.print_working_dir())

    def change_dir(self, args: list[str]):
        # Implement change_dir logic using RusticFS methods
        if len(args) != 1:
            raise ShellError("Invalid number of arguments for cd")
        self.fs.change_dir(args[0])

    def change_permissions(self, args: list[str]):
        if len(args) != 2:
            raise ShellError("Invalid number of arguments for chmod")
        self.fs.change_permissions(args[0], args[1])

    def exec(self, args: list[str]):
        if len(args) != 1:
            raise ShellError("Invalid number of arguments for exec")
        self.fs.execute_py(args[0])
    # Implement similar handler functions for other commands...

    def run(self):
        while True:
            try:
                command = input("filesystem> ")
                if not command:
                    continue
                cmd, *args = command.split()
                self.execute_command(cmd, args)
                if cmd == "quit":
                    break
            except ShellError as e:
                print(f"Shell Error: {e}")