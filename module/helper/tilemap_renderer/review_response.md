# Review Response

## CORRECTNESS — `full` feature fails `cargo check` on native targets

Not addressed in this PR. The compile error (`expected f64, found i32` in
`minwebgl/src/texture/d2.rs`) is a pre-existing bug in `minwebgl`, not a
structural issue with the `full` feature definition. That bug has already been
fixed in a separate PR where `adapter-webgl` compiles cleanly. Once that PR is
merged the `full` feature will work as-is on its intended WASM target.

The `adapter-webgl` feature pulling in WASM-only dependencies is also an
intentional and documented constraint — see "Known issues" in the PR description:

> The hexagonal_map example depends on `tilemap_renderer` with `adapter-webgl` feature
> and will not compile until the WebGL adapter PR is merged. This is expected.

---

## TESTING — `Backend` trait has zero test coverage

**Addressed in commit `9d3baa30`.**

Added `tests/backend_test.rs` with a minimal `TestBackend` struct that fully
implements the `Backend` trait. The file covers:

- `load_assets()` with valid assets and with an empty `Assets`
- `submit()` with an empty command slice and with a `Clear` command
- `output()` returning `Output::String`
- All three `RenderError` variants (`MissingAsset`, `Unsupported`, `BackendError`)
  formatted correctly via `Display`
- `Capabilities::default()` returning all `false` fields and `max_texture_size = 0`

9 tests added, all passing.

---

## DESIGN — Feature-enabled adapter backends are completely silent

**Addressed in commit `959e2de5`.**

Added a `**Status: stub only** — implementation deferred to a follow-up PR.`
doc comment to the module-level doc of `svg.rs`, `webgl.rs`, and `terminal.rs`.
The empty state is now visible via `cargo doc` and to any developer reading the
source.

Exporting a placeholder `SvgBackend` / `WebGLBackend` / `TerminalBackend` struct
with a panicking `Backend` impl is intentionally deferred to each adapter's own PR,
as described in the PR description:

> Adapter source files (`svg.rs`, `webgl.rs`, `terminal.rs`) are empty stubs —
> implementations will be added in their respective follow-up PRs.

Shipping a panicking stub in this PR would give the false impression that a type
exists and is usable, which is misleading before the real implementation lands.
The doc comment approach documents the intent without that risk.
