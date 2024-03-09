# Example Shell

This is an example shell that uses the `RusticFS` library to create a simple shell that can be used to interact with a virtual filesystem.

## Usage

### Step 1: Install RusticFS

First, you need to install the `RusticFS` library. You can do this by running the following command:

```shell
pip install rusticfs
```

#### Note

If you are on a system that's not supported by RusticFS, you can install build the wheel by following this guide: [RusticFS Installation](https://gitlab.com/dv1629-os/lab_3_rust/-/blob/master/file_system/building.md)

### Step 2: Run the Example Shell

Once you have installed the `RusticFS` library, you can run the example shell by running the following command in this directory:

```shell
python .
```

This will start the example shell, and you can start interacting with the virtual filesystem.

### Step 3: Interact with the Virtual Filesystem

You can use the following commands to interact with the virtual filesystem:

- `append <source> <destination>`: Write the specified data to the specified file.
- `create <file>`: Creates a new file.
- `cat <file>`: Print the contents of the specified file.
- `cd <directory>`: Change the current directory to the specified directory.
- `chmod <file> <permissions>`: Change the permissions of the specified file.
- `cp <source> <destination>`: Copy the specified file or directory to the specified destination.
- `format`: Formats the disk.
- `help`: Display a help message with the available commands.
- `ls`: List the contents of the current directory.
- `mkdir <directory>`: Create a new directory with the specified name.
- `mv <source> <destination>`: Move the specified file or directory to the specified destination.
- `pwd`: Print the current working directory.
- `quit`: Exit the example shell.
- `rm <file>`: Remove the specified file or directory.
- `dbg`: Prints the current state of the filesystem.
