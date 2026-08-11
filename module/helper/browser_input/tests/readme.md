# browser_input tests

Native test suite for the `browser_input` crate. Everything DOM-facing (event-listener wiring,
`wasm-bindgen` callbacks) needs a browser environment and lives in the manual plan instead —
these files cover the pure logic that runs on the native target: `src/` carries no inline test
modules (task 076 relocated the last 6).

## Responsibility Table

| File | Responsibility |
|------|----------------|
| active_pointers_test.rs | Pointer press/release tracking through the Input event queue |
| manual/ | Manual browser-testing plan (DOM wiring not coverable natively) |
| pointer_type_test.rs | PointerType DOM-string parsing and default pins |

## Adding tests

1. Pure logic reachable through `browser_input::*` → a domain file here (add a row above).
2. Behaviour that needs a real browser → extend `manual/readme.md`'s plan instead of writing a
   test that silently passes without DOM resources.
3. Verify with `cargo test -p browser_input --all-features` (run detached via `longrun .launch`
   from the workspace root; check the log's per-suite breakdown).
