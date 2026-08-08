# Pitfall: Load-Time Validation Is Only Partially Enforced

### Scope

- **Purpose**: Warn a spec author or integrator that a successful `RenderSpec::load()` is not proof the spec is well-formed against the full validation checklist the format declares.
- **Responsibility**: Document the concrete, worked failure mode (`Square4`/`Square8` tiling) and the broader class of unenforced rules it's representative of.
- **In Scope**: What currently passes `load()` silently despite being invalid per the format's own MUST-requirements; where the resulting failure actually surfaces instead.
- **Out of Scope**: The full enumerated list of enforced-vs-unenforced rules (see `invariant/001`, which is the exhaustive reference this doc's Trap section only summarizes).

### Trap

It is natural to treat `RenderSpec::load(...).is_ok()` as "this spec is valid" — the format's own contract states the loader MUST verify and report all violations of its validation checklist at load time (see `invariant/001`). In the current implementation, several checklist rules are declared (as `ValidationError` variants in `src/error.rs`) but not yet wired into `src/validate.rs`'s actual checks — most concretely: a `RenderPipeline.hex.tiling` of `Square4` or `Square8` is reserved-but-unimplemented schema surface (see `format/002`) that the format's own text says MUST be rejected at load time with a clear error, and yet nothing in `validate.rs` constructs the `ValidationError::UnsupportedTiling` variant that exists for exactly this case. The same pattern applies more broadly: anchor↔source compatibility (e.g. a `Multihex` object declaring a `NeighborBitmask` source, which `format/003` says MUST be rejected), composite-source nesting, and `RenderSpec.version` compatibility (see `invariant/001`'s Version Compatibility discussion) are all currently unchecked in the same way.

### Failure

A spec naming `Square4`/`Square8` passes `load()` and returns `Ok(())` — no diagnostic, no warning, nothing distinguishing it from a fully valid `HexFlatTop` spec at load time. The failure instead surfaces much later and far less legibly: at the *first render call* against that scene, as `Err(CompileError::UnsupportedAnchor)` — an error variant named after anchors, not tiling, giving an integrator debugging a render-time crash no direct signal that the actual root cause is an unsupported *tiling strategy* declared at the pipeline level, several structural layers away from wherever the anchor-shaped error message points them to look. An integrator who only checks `load()`'s `Result` and assumes a clean `Ok` means "renderable" will ship a scene that silently fails the first time it's actually drawn — in an automated pipeline (asset validation in CI, for instance) that only calls `load()` and not a full render, this class of error passes undetected entirely.

### Mitigation

Until `validate.rs`'s TODOs are implemented (see `invariant/001` for the exact list), do not treat a successful `load()` as proof of renderability for any of the currently-unenforced rules. Concretely: exclude `Square4`/`Square8` from spec-generation tooling entirely rather than relying on the loader to reject them (grep authored specs for `tiling: Square` as a stand-in CI check); when introducing a new object, cross-check its `anchor` against its layers' `sprite_source` variants by hand against `format/003`'s compatibility table rather than trusting a load-time rejection; and treat a first successful `Renderer::render()` call in a smoke test — not merely `load()` — as the actual well-formedness gate for any spec that exercises tiling, anchor/source pairing, or composite-source nesting.

### APIs

| File | Relationship |
|------|--------------|
| [api/001_renderer_integration_api.md](../api/001_renderer_integration_api.md) | `CompileError::UnsupportedAnchor` is the actual error surfaced, at `Renderer::render`, not at load |

### Formats

| File | Relationship |
|------|--------------|
| [format/002_grid_coordinate_system.md](../format/002_grid_coordinate_system.md) | `Square4`/`Square8` — the concrete worked example of this pitfall |
| [format/003_anchor_placement_types.md](../format/003_anchor_placement_types.md) | Anchor↔source compatibility is declared but not yet enforced, the same unenforced-validation pattern as the tiling gap |
| [format/007_render_pipeline.md](../format/007_render_pipeline.md) | `HexConfig.tiling` is declared at the pipeline level, one layer removed from where the failure surfaces |
| [format/008_top_level_file_structure.md](../format/008_top_level_file_structure.md) | `RenderSpec.version`'s unenforced compatibility contract is the same unenforced-validation pattern |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_renderspec_referential_integrity.md](../invariant/001_renderspec_referential_integrity.md) | Exhaustive enforced-vs-unenforced rule breakdown this doc's Trap section summarizes |

### Sources

| File | Relationship |
|------|--------------|
| `src/validate.rs` | Where the missing checks would be implemented (see `// TODO SPEC §16` comments) |
| `src/pipeline.rs` | `TilingStrategy` doc comment explicitly notes `ValidationError::UnsupportedTiling` is declared but not yet constructed |
| `src/error.rs` | `ValidationError::UnsupportedTiling`, `CompileError::UnsupportedAnchor` |

### Tests

No dedicated regression test currently pins "a `Square4`/`Square8` spec fails `load()`" — because it doesn't fail `load()` today. A test asserting the current (silent-pass) behavior would itself be documenting the gap rather than the intended contract, so none was added as part of this migration.
