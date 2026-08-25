# Pitfall: Load-Time Validation Is Only Partially Enforced

### Scope

- **Purpose**: Warn a spec author or integrator that a successful `RenderSpec::load()` is not proof the spec is well-formed against the full validation checklist the format declares.
- **Responsibility**: Document the concrete, worked failure mode (anchor↔source compatibility) and the narrower class of unenforced rules it's representative of.
- **In Scope**: What currently passes `load()` silently despite being invalid per the format's own MUST-requirements; where the resulting failure actually surfaces instead.
- **Out of Scope**: The full enumerated list of enforced-vs-unenforced rules (see `invariant/001`, which is the exhaustive reference this doc's Trap section only summarizes).

### Trap

It is natural to treat `RenderSpec::load(...).is_ok()` as "this spec is valid" — the format's own contract states the loader MUST verify and report all violations of its validation checklist at load time (see `invariant/001`). Most of that checklist is now wired into `src/validate.rs`: id uniqueness, pipeline-layer/asset/tint/animation/effect reference resolution, `connects_with` resolution, composite-source nesting, `default_state` existence, reserved ids, and the `Square4`/`Square8` tiling whitelist are all enforced today, each raising a corresponding `ValidationError` at `load()` time. One rule remains declared but not yet wired in: **anchor↔source compatibility** — `format/003`'s table of which `SpriteSource` variants each `Anchor` permits. The `ValidationError::AnchorSourceMismatch` variant exists in `src/error.rs` for exactly this case, but nothing in `validate.rs` constructs it (see that file's own `// TODO SPEC §16` comment on the subject, which explains the gap is deliberate — the format docs and the actual compile-layer behavior disagree on several specifics, so implementing the check as literally documented would flag intentionally-passing tests as invalid). `RenderSpec.version` compatibility (see `invariant/001`'s Version Compatibility discussion) is unchecked in the same way.

### Failure

A spec declaring an `External` sprite source on an `Edge`-anchored object passes `load()` and returns `Ok(())` — `format/003`'s compatibility table and `format/005`'s per-source description both say `External` is valid on every anchor, including `Edge`, and nothing in `validate.rs` checks anchor/source pairing, so there is no diagnostic at load time. The failure instead surfaces much later and far less legibly: at the *first render call* against that scene, as `Err(CompileError::UnsupportedSource { object, source_kind: "External" })`, thrown by `edge_sprite_source_resolve`'s catch-all arm in `src/compile/frame.rs` — that function has explicit match arms only for `EdgeConnectedBitmask`, `Static`, `Animation`, and `Variant`; every other source, `External` included, falls into a generic "unsupported for this compile path" branch. An integrator who only checks `load()`'s `Result` and assumes a clean `Ok` means "renderable" will ship a scene that silently fails the first time it's actually drawn — in an automated pipeline (asset validation in CI, for instance) that only calls `load()` and not a full render, this class of error passes undetected entirely. `tests/scene_model_compile_test.rs::edge_rejects_external_source` pins both halves of this behavior: `spec.validate()` returns `Ok(())`, then compiling a scene that uses the object returns `CompileError::UnsupportedSource`.

### Mitigation

Until `validate.rs`'s anchor↔source TODO is implemented (see `invariant/001` for the current enforcement table), do not treat a successful `load()` as proof of renderability for anchor/source pairings. Concretely: when introducing a new object, cross-check its `anchor` against its layers' `sprite_source` variants by hand against `format/003`'s compatibility table rather than trusting a load-time rejection, and treat a first successful `Renderer::render()` call in a smoke test — not merely `load()` — as the actual well-formedness gate for any spec that exercises anchor/source pairing. Tiling and composite-nesting no longer need this workaround — `validate.rs` now rejects both at `load()` time.

### APIs

| File | Relationship |
|------|--------------|
| [api/001_renderer_integration_api.md](../api/001_renderer_integration_api.md) | `CompileError::UnsupportedSource` is the actual error surfaced, at `Renderer::render`, not at load |

### Formats

| File | Relationship |
|------|--------------|
| [format/003_anchor_placement_types.md](../format/003_anchor_placement_types.md) | Its Sprite-source compatibility table (line listing `Edge`'s permitted sources) is the claim contradicted by `edge_sprite_source_resolve` — the concrete worked example of this pitfall |
| [format/005_sprite_sources.md](../format/005_sprite_sources.md) | `External`'s "Applicable to all anchors" claim is the other half of the documented-vs-actual mismatch |
| [format/002_grid_coordinate_system.md](../format/002_grid_coordinate_system.md) | `Square4`/`Square8` — formerly this pitfall's worked example; `validate.rs`'s tiling whitelist now rejects both at load time, so this class of failure no longer reaches render |
| [format/008_top_level_file_structure.md](../format/008_top_level_file_structure.md) | `RenderSpec.version`'s unenforced compatibility contract is the same unenforced-validation pattern as anchor↔source |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_renderspec_referential_integrity.md](../invariant/001_renderspec_referential_integrity.md) | Exhaustive enforced-vs-unenforced rule breakdown this doc's Trap section summarizes |

### Sources

| File | Relationship |
|------|--------------|
| `src/validate.rs` | Where the anchor↔source check would be implemented; see its `AnchorSourceMismatch` `// TODO SPEC §16` comment for why it's deliberately deferred |
| `src/compile/frame.rs` | `edge_sprite_source_resolve`'s catch-all match arm is where `External`-on-`Edge` actually fails, as `CompileError::UnsupportedSource` |
| `src/error.rs` | `ValidationError::AnchorSourceMismatch` (declared, never constructed), `CompileError::UnsupportedSource` (the error actually raised) |

### Tests

| File | Relationship |
|------|--------------|
| `tests/scene_model_compile_test.rs` | `edge_rejects_external_source` pins both halves of the gap: `spec.validate()` returns `Ok(())` for an `External` source on an `Edge`-anchored object, then compiling a scene using it returns `CompileError::UnsupportedSource` |
