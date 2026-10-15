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
#
# The runner gives a whole test binary 20 s by default. A suite that renders
# on the CPU — pmrem_tests runs the 512×512, 1024-sample BRDF integration once
# per test — takes ~14 s under the CI runner's SwiftShader and goes past 20 s
# on a slower machine, which fails it as a webdriver timeout. Allow 120 s
# unless the caller sets its own.
export WASM_BINDGEN_TEST_TIMEOUT="${WASM_BINDGEN_TEST_TIMEOUT:-120}"
cd "$( dirname "$0" )/.." && exec wasm-bindgen-test-runner "$@"
