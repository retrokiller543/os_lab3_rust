# Building the filesystem locally

This guide will walk you through the process of building the filesystem locally.

We can build the filesystem to work either in a Python environment or as a Rust library.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Maturin](https://github.com/PyO3/maturin)
- [Python](https://www.python.org/downloads/)
- A python virtual environment (optional)

## Building the filesystem (Rust)

To build the filesystem as a Rust library, you can use cargo to build the library. If you build the debug release, there will be more debug printouts via `log` and `env_logger`.

```bash
cargo build
```

```bash
cargo build --release
```

## Building the filesystem (Python)

When building the filesystem as a Python library, you can use maturin to build the library.

### With a virtual environment

```bash
maturin develop # This will only work if you have a virtual environment activated
```

```bash
maturin develop --release # This will only work if you have a virtual environment activated
```

### Without a virtual environment

```bash
maturin build
pip install --force-reinstall ../target/wheels/*.whl # this might need to be adjusted to the correct path and file name
```

```bash
maturin build --release
pip install --force-reinstall ../target/wheels/*.whl # this might need to be adjusted to the correct path and file name
```
