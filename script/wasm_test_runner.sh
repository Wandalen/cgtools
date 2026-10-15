#!/bin/sh
# Runner shim for `cargo test --target wasm32-unknown-unknown` (wired up in
# `.cargo/config.toml`). `cargo test` runs every suite with the *package*
# directory as cwd, but wasm-bindgen-test-runner serves non-generated files —
# test asset fetches like `/assets/gltf/...`, plus any `webdriver.json`
# capability file — from *its own* cwd. Package cwd therefore 404s every
# workspace-root asset (the loader then fails with "Failed to parse gltf
# file"). Re-anchor to the workspace root, where the assets live. Cargo
# resolves this script's config-relative path to an absolute one before
# spawning, so `$0` is always absolute here.
cd "$( dirname "$0" )/.." && exec wasm-bindgen-test-runner "$@"
