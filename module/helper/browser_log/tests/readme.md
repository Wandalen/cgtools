# tests

Native tests for `browser_log`, runnable without a browser/wasm target via
`cargo test -p browser_log --all-features`: `panic::hook`/`panic::setup` and `DebugLog` are
target-agnostic (no `#[cfg(target_arch = "wasm32")]` gating in either), so their behavior is
fully exercisable on any host through `std::panic::set_hook`/`catch_unwind` and a custom
`log::Log` capturing implementation. `panic_hook_test.rs` and `debug_log_test.rs` each install
process-global mutable state (`std::panic::set_hook`/`log::set_logger`) and guard against
same-binary test concurrency accordingly (see their own module docs). `static_max_level_test.rs`
also installs its own `log::set_logger` captor and is wasm-agnostic like the two above, but its
subject (`log::STATIC_MAX_LEVEL`, gated by `Cargo.toml`'s `log` feature list) depends on a
different axis entirely — BUILD PROFILE (`cfg(debug_assertions)`), not target — so one of its
two tests only exercises its real assertion under `cargo test -p browser_log --release` (see
its own module doc for the exact command).

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| basic_test.rs | Smoke coverage that `panic::setup`/manual `panic::set_hook` wiring don't panic on repeated calls |
| panic_hook_test.rs | `panic_message` location-inclusion behavior behind `Config.with_location` (BUG-168) |
| debug_log_test.rs | `DebugLog` trait methods report the real external caller's `file:line`, not their own (BUG-167) |
| static_max_level_test.rs | `log::STATIC_MAX_LEVEL`/`log::debug!` delivery is never capped below `Trace` in a release-profile build (BUG-354) |
