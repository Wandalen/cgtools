# tests

Native tests for `browser_log`, runnable without a browser/wasm target via
`cargo test -p browser_log --all-features`: `panic::hook`/`panic::setup` and `DebugLog` are
target-agnostic (no `#[cfg(target_arch = "wasm32")]` gating in either), so their behavior is
fully exercisable on any host through `std::panic::set_hook`/`catch_unwind` and a custom
`log::Log` capturing implementation. `panic_hook_test.rs` and `debug_log_test.rs` each install
process-global mutable state (`std::panic::set_hook`/`log::set_logger`) and guard against
same-binary test concurrency accordingly (see their own module docs).

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| basic_test.rs | Smoke coverage that `panic::setup`/manual `panic::set_hook` wiring don't panic on repeated calls |
| panic_hook_test.rs | `panic_message` location-inclusion behavior behind `Config.with_location` (BUG-168) |
| debug_log_test.rs | `DebugLog` trait methods report the real external caller's `file:line`, not their own (BUG-167) |
