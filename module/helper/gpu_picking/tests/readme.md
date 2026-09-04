# tests

Integration tests for `gpu_picking`, split by what a test can observe rather
than by what it covers.

The crate is almost entirely GL calls: `IdProgram` and `PickBuffer` need a live
`WebGl2RenderingContext` to construct at all, so nothing about them is decidable
in a native `cargo nextest` run. What *is* decidable natively is the small
amount of interpretive logic — the background sentinel, the bounds check, the
pick-id validation — which is why those three sit in their own module and their
own test file. Everything else runs under the headless-browser wasm32 runner.

Both files reach private items through the `test_internals` feature, so run
them with `--features test_internals`; without it each file's own `#![ cfg ]`
compiles it away to nothing.

| File | Responsibility |
| ---- | -------------- |
| pick_logic_test.rs | Sentinel mapping, bounds check, pick-id validation (native) |
| pick_buffer_drop_test.rs | `PickBuffer::drop` frees all three GL handles (wasm32, live context) |
