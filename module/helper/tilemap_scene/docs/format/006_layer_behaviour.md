# Format: Layer Behaviour

### Scope

- **Purpose**: Define `LayerBehaviour` — what the renderer does to a layer's sampled sprite (tint, blend, effects, alpha, viewport parallax).
- **Responsibility**: Document `LayerBehaviour`'s fields, their defaults, and the anchor-scoping restriction on `parallax`/`scroll_velocity`.
- **In Scope**: `TintBehaviour` (`None`/`Flat`/`Masked`), `blend` modes, `effects`, `alpha`, `parallax`/`scroll_velocity`.
- **Out of Scope**: Which sprite is sampled in the first place (see `format/005`); the order tint/effects/pipeline-tint/global-tint compose in during a render pass (see `algorithm/002`).

### Abstract

`LayerBehaviour` is the second of an `ObjectLayer`'s two independent fields (see `format/001`) — declared with no coupling to `sprite_source`, so any sprite-selection rule can pair with any behaviour. Every field is optional; the all-defaults behaviour is "draw the sampled sprite as-is, normal blending, full opacity." `parallax`/`scroll_velocity` are the one anchor-scoped exception — they are meaningful only for `Viewport`-anchored objects (see `format/003`) and the format declares using them elsewhere a load-time error.

### Data Model

| Field | Type | Default | Meaning |
|-------|------|---------|---------|
| `tint` | `TintBehaviour` | `None` | See below. |
| `blend` | `BlendMode` | `Normal` | Standard compositing mode over the accumulated layer stack (`Normal`/`Multiply`/`Screen`/`Add`/`Overlay`); type owned by `tilemap_renderer`. |
| `alpha` | `f32` | `1.0` | Static scalar applied before blending; an `AlphaPulse` effect (see `format/004`) modulates on top of this base value, it does not replace it. |
| `effects` | `Vec<EffectRef>` | `[]` | Declared `Effect` resources (see `format/004`), applied after sampling and tinting. |
| `parallax` | `Option<f32>` | `None` | Viewport only. `0.0` pins to screen, `1.0` moves with the world at 1:1, values between produce depth, `>1.0` produces foreground parallax. |
| `scroll_velocity` | `Option<(f32, f32)>` | `None` | Viewport only. Autonomous world-pixel-per-second drift independent of the camera, added to the texture offset every frame. |

`TintBehaviour`:

| Variant | Fields | Meaning |
|---------|--------|---------|
| `None` | — | Sample the sprite unmodified (default). |
| `Flat` | `TintRef` | Multiply the whole sprite by the named tint. |
| `Masked` | `mask: Box<SpriteSource>`, `tint: MaskTint` | Sample a second sprite from `mask` (any `format/005` source, typically `Static` or `Animation`) and apply `tint` only where the mask's alpha is nonzero — a `Masked` layer samples two textures per draw. |

`MaskTint`: `Ref(TintRef)` | `TeamColor` | `FogDependent` (the same two symbolic tints defined in `format/004`).

### Encoding Structure

`behaviour: ()` in RON is the shorthand for "all fields at their default" (seen throughout `format/008`'s worked `Object` example). A non-default behaviour is written as a partial field list — only the fields being overridden need appear, since every field defaults. `Masked`'s `mask` sub-source is itself a full `SpriteSource` value, nested one level — since `Masked` is a leaf-source-consuming construct rather than a composite source itself, this nesting does not trip the "composites cannot nest inside composites" restriction from `format/005`.

**Intra-object sync constraint**: a `Masked` behaviour whose `mask` is an `Animation` source MUST declare a frame count compatible with the body layer it's masking, so the mask and body advance in lock-step (see `algorithm/001`'s intra-object sync discussion) rather than drifting out of phase.

### Version Compatibility

New `BlendMode` variants are owned by `tilemap_renderer`, not this crate — a blend mode this format can name is bounded by what that crate exposes. The `parallax`/`scroll_velocity`-on-non-`Viewport`-anchor restriction is declared as a MUST in the specification this doc replaces, but **`src/validate.rs` has no check enforcing it today** — a `Hex`-anchored object with a nonzero `parallax` passes `load()` silently rather than producing the documented load-time error (see `pitfall/001`).

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/001_animation_phase_and_frame_selection.md](../algorithm/001_animation_phase_and_frame_selection.md) | Governs how a `Masked` mask's `Animation` frame stays in sync with the body layer's |
| [algorithm/002_scene_rendering_pass.md](../algorithm/002_scene_rendering_pass.md) | `apply_behaviour` step; stages 2–4 of the tint composition order |

### Formats

| File | Relationship |
|------|--------------|
| [format/004_declared_resources.md](../format/004_declared_resources.md) | `TintRef`/`EffectRef`/`MaskTint` reference resources declared there |
| [format/005_sprite_sources.md](../format/005_sprite_sources.md) | `TintBehaviour::Masked`'s mask slot accepts any source from this doc; independent of `sprite_source` |

### Sources

| File | Relationship |
|------|--------------|
| `src/layer.rs` | `LayerBehaviour`, `TintBehaviour`, `MaskTint` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/scene_model_compile_test.rs` | Behaviour field coverage, including `Masked` mask/body pairing |
