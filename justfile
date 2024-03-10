#!/usr/bin/env just --justfile
set positional-arguments := true

# publish with a given version, before running this command, make sure you have the version in the Cargo.toml file and have committed the changes but not pushed them
@publish-pypi version:
    echo "Publishing to pypi with version {{version}}"
    python publish.py {{version}}
