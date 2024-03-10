#!/usr/bin/env just --justfile

release:
  cargo build --release    

lint:
  cargo clippy

fmt:
  cargo fmt

# publish with a given version, before running this command, make sure you have the version in the Cargo.toml file and have committed the changes but not pushed them
publish-pypi version:
  python publish.py $(version)