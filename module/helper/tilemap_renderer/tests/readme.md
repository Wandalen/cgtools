# tilemap_renderer — test suite

## Organization principles

Each test file covers a single top-level domain of the crate. Tests are named
in `{component}_{action}_{scenario}` order. Every test function carries a
`///` doc comment that states the behavior under test, why the case matters,
and what the expected outcome is.

## Directory structure

```
tests/
  helpers/
    mod.rs              — shared fixtures (empty_assets, …)
  assets_test.rs        — Assets validation domain
  backend_test.rs       — Backend trait contract, RenderError, Capabilities
  commands_test.rs      — RenderCommand Copy invariant, size, stream construction
  svg_backend_test.rs   — SvgBackend adapter behavior via public surface (feature adapter-svg)
  types_test.rs         — Transform, ResourceId, RenderConfig
```

## Domain map

| File | Domain | Key cases |
|---|---|---|
| `types_test.rs` | Core value types | `Transform` identity/translation/scale/rotation, `ResourceId` equality, `RenderConfig` defaults |
| `commands_test.rs` | Command types | `Copy` invariant (compile-time), enum size bound, stream construction, batch params |
| `assets_test.rs` | Asset validation | Empty set, no-duplicate ok, per-type duplicate errors, cross-type id independence |
| `backend_test.rs` | Backend trait | `assets_load`, `submit`, `output`, `resize`, `Capabilities::default`, all `RenderError` variants |
| `svg_backend_test.rs` | SvgBackend adapter (relocated from inline by task 071) | Clear/viewport wrapper, paths, gradients, patterns, clip masks, sprite tint/batches, mesh topologies, effects, blend modes, groups, disk/encoded/bitmap image loading, text flow/anchors/on-path, plus the former private helpers ( transforms, anchors, PNG probing, `SvgContentManager` ) now exposed as documented or `doc( hidden )` pub — `src/` carries no inline test modules |

## Adding new tests

1. Place the test in the file whose domain it belongs to.
2. Name it `{component}_{action}_{scenario}` (e.g. `backend_submit_unsupported`).
3. Add a `///` doc comment before `#[test]` explaining what is tested and why.
4. If the test needs a fixture shared across files, add it to `helpers/mod.rs`.
