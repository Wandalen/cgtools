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
  none_backend_test.rs  — NoneBackend no-op contract (feature adapter-none)
  native_backend_test.rs — NativeBackend real-GPU pixel-readback contract (feature adapter-native)
  webgpu_backend_test.rs — WebGpuBackend compile-and-construct-level contract (feature adapter-webgpu, wasm32)
  webgl_backend_test.rs — WebGlBackend::declared_capabilities pure-function contract (feature adapter-webgl)
  command_consistency_test.rs — cross-backend capabilities-vs-submit() consistency (none/svg/native)
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
| `none_backend_test.rs` | NoneBackend adapter | `Capabilities::default` field-by-field pin, unconditional `Ok` on `submit`/`assets_load` regardless of command/asset content |
| `native_backend_test.rs` | NativeBackend adapter | Real `gpu_hal` device construct/load/submit/output, exact pixel readback, resize |
| `webgpu_backend_test.rs` | WebGpuBackend adapter | `declared_capabilities` honest subset, `sprite_draw_params` anti-hardcoding, `command_classify` family rejection (wasm32 only) |
| `webgl_backend_test.rs` | WebGlBackend adapter | `declared_capabilities` honest-subset pin and `max_texture_size` anti-hardcoding pin — no live `WebGl2RenderingContext` |
| `command_consistency_test.rs` | Cross-backend command/capabilities consistency | `none`/`svg`/`native` each accept a `Sprite` (all declare `sprites: true`); `none`/`native` each reject or gracefully skip a `paths`-family command they declare `false` (never panic) |

## Adding new tests

1. Place the test in the file whose domain it belongs to.
2. Name it `{component}_{action}_{scenario}` (e.g. `backend_submit_unsupported`).
3. Add a `///` doc comment before `#[test]` explaining what is tested and why.
4. If the test needs a fixture shared across files, add it to `helpers/mod.rs`.
