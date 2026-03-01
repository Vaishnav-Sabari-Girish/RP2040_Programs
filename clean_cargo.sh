#!/usr/bin/env bash

# 1. -type d -name target -prune: If it sees a folder named 'target', skip it entirely.
# 2. -o: OR (if it wasn't a target folder...)
# 3. -type f -name Cargo.toml -exec ...: Run the clean command on Cargo.toml files.

find . -type d -name target -prune -o -type f -name Cargo.toml -exec cargo +stable clean --manifest-path {} \;
