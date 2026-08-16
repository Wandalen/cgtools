# Algorithm: Value Function Shape Detection

### Scope

- **Purpose**: Decide whether a chunk export is a previewable "value
  function", which of 3 shapes it matches, and how the synthesized harness
  samples and writes that shape's value to the render target.
- **Responsibility**: Document `value_fn_of`'s parsing rule, `bundle_build`'s
  candidate-selection fallback, and `harness_synthesize`'s per-shape,
  tag-gated sampling point and render-target write, exactly as implemented.
- **In Scope**: The 3 previewable export shapes, the candidate-selection
  order among multiple matching exports in one chunk, the target chunk's
  `category:sdf` tag check, and the sampling point / write-out expression
  each shape+tag combination maps to, including the universal reference
  grid overlay.
- **Out of Scope**: The fragment-chunk mode (`//@ stage: fragment`, its
  own `fs_main` + `//@ param:` uniform requirements) — an entirely
  separate branch of `bundle_build` this algorithm never participates in.
  The `//@ param:` grammar itself (see
  [`shader_chunks_params_core`](../../../shader_chunks_params_core/docs/api/001_tunable_parameter_taxonomy.md)).

### Abstract

`value_fn_of` is a pure, total function with no randomness: the same
export-signature string always yields the same `Option<(&str,
ValueFnKind)>`. It answers one question — "does this export look like a
spatial value function the preview harness can sample, and if so, what
shape does it return?" — through a single structural parse, no name-based
heuristics involved (unlike `shader_chunks_params_core`'s
`range_infer`, which this crate's design deliberately does not mirror
here: a chunk's *shape* is a hard structural fact, not something worth
inferring from naming convention).

At the `bundle_build` call site, when a chunk has more than one matching
export, candidate selection applies one fixed rule — never a shape
preference — and the chosen export's `ValueFnKind`, together with one tag
check on the *target chunk's own* manifest (`category:sdf`, independent of
which chunk the export was collected from), selects the synthesized
harness's sampling point and render-target write. Every shape still writes
a raw sampled value with no per-chunk rescaling — the render target's own
`[0, 1]` clamp is still the only "normalization" applied — except the
`F32`+SDF combination, which additionally derives a fill/band/isoline color
*from* the value rather than writing it directly (Stage 2). A universal
reference grid is composited over every shape afterward, regardless of
kind or tag (Stage 3).

### Algorithm

**Stage 0 — structural shape match** (`value_fn_of`, checked for every
export string in the chunk's manifest):

| Condition | Result |
|-----------|--------|
| Not of the form `fn NAME(ARG) -> RETURN` (missing `fn `, unbalanced parens, no `->`) | `None` |
| More than one argument, or the sole argument's type is not exactly `vec2f` | `None` |
| `NAME` is empty | `None` |
| Argument type is `vec2f` and `RETURN` is `f32` | `Some((NAME, ValueFnKind::F32))` |
| Argument type is `vec2f` and `RETURN` is `vec2f` | `Some((NAME, ValueFnKind::Vec2))` |
| Argument type is `vec2f` and `RETURN` is `vec3f` | `Some((NAME, ValueFnKind::Vec3))` |
| Argument type is `vec2f` and `RETURN` is anything else | `None` |

The argument type check never widens — the sample point is always 2D
regardless of which value the function returns; only the *return* type
match widens across the 3 kinds.

**Stage 1 — candidate selection** (`bundle_build`, only reached when the
target chunk is not `//@ stage: fragment`; runs once per `bundle_build`
call, over every export that passed Stage 0):

| Step | Rule |
|------|------|
| 1 | Collect every export in the chunk's own manifest ( plus any it composes via `depends_on` — see Sources ) that Stage 0 matched, preserving file/manifest order. |
| 2 | If any candidate's name equals the target chunk's own `name`, pick it. |
| 3 | Otherwise pick the first candidate in manifest order. |
| 4 | If no candidate exists, fail with `PreviewError::Unpreviewable`. |

Step 2/3 apply identically regardless of `ValueFnKind` — a `vec3f`-shaped
export is never preferred or deprioritized relative to an `f32`-shaped
one. This is deliberate: a chunk exporting both its own `NAME`-matching
value function and some unrelated previewable helper should always
preview itself, not whichever shape happens to look more interesting.

**Stage 2 — sampling point and color** (`harness_synthesize`, gated by both
`ValueFnKind` and the target chunk's own `category:sdf` tag, checked once
per `bundle_build` call via `tags_parse( target_wgsl )`):

| `ValueFnKind` | `category:sdf`? | Sample point `p` | Color from `value` |
|----------------|:---:|-------------------|---------------------|
| `F32` | no | `q * scale + vec2f( time * 0.05, 0.0 )` (drifts) | `vec3f( value )` — raw grayscale, unchanged since before `Vec2`/`Vec3` existed |
| `F32` | yes | `q * scale` (stationary) | filled inside (`value < 0`) / light outside, multiplicatively banded by `cos( value * 40.0 )`, dark isoline where `abs( value )` crosses `0` within one anti-aliased pixel width |
| `Vec2` | either | `q * scale + vec2f( time * 0.05, 0.0 )` (drifts) | `vec3f( value, 0.5 )` — red/green from `value`; blue fixed at a neutral `0.5` pad so a `vec2f` field never reads as "half-missing-color" at a glance |
| `Vec3` | either | `q * scale + vec2f( time * 0.05, 0.0 )` (drifts) | `value` — direct RGB passthrough |

Only the `F32`+SDF combination samples at a stationary point and derives
color *from* the value (fill/band/isoline) rather than writing it
directly — every other combination keeps the original convention: an
unbounded horizontal drift (`time * 0.05`) and a raw value written
straight into the color, still relying on Stage 3's final clamp for
`[0, 1]` range, never rescaled or remapped otherwise. The stationary
sample point exists because a drifting point eventually carries a
finite-footprint SDF shape off-screen permanently (the shape never
re-enters frame, since nothing wraps or bounds the drift) — a field like
noise or a color gradient has no edge to drift past, so drift is harmless
there and left unchanged; an SDF shape does have one, so SDF-tagged chunks
hold the point still instead.

**Stage 3 — reference grid overlay** (`harness_synthesize`, applied after
Stage 2's color is computed, unconditionally — every shape and every tag):

A world-space grid is composited over the Stage 2 color before the final
clamp: unit-spaced minor lines (`fract( p - 0.5 ) - 0.5`, thin,
low-opacity) plus emphasized axis lines through the world origin
(`abs( p.x )` / `abs( p.y )`, thicker, higher-opacity), both
anti-aliased in screen-pixel units via `px = scale / resolution.y` so
line thickness stays constant in screen space regardless of zoom. The
grid is alpha-blended toward black (`mix( color, black, grid )`), then
`clamp( ·, 0, 1 )` produces the final `vec4f`. This exists so a preview's
scale and center are always legible — previously nothing on screen
indicated where the world origin was or how large one unit looked, at any
zoom level or for any chunk.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `ValueFnKind`, `value_fn_of`, `harness_synthesize`, and the candidate-selection site in `bundle_build` — the entire algorithm |

### Tests

| File | Relationship |
|------|--------------|
| `tests/preview_bundle_test.rs` | `vec2_value_chunk_gets_a_synthesized_harness` / `vec3_value_chunk_gets_a_synthesized_harness` exercise Stage 0's `vec2f`/`vec3f` return-type match and Stage 2's non-SDF write expressions against real bundled chunks; the existing `value_chunk_gets_a_synthesized_grayscale_harness` continues to cover non-SDF `F32`; `sdf_tagged_value_chunk_gets_filled_banded_visualization_and_stationary_sampling` covers the `F32`+SDF combination (fill/band/isoline color, stationary sample point) against `sdf_op_round`; `non_sdf_value_chunk_keeps_raw_grayscale_and_time_drift` confirms an untagged `F32` chunk keeps the original drifting/raw-value path; `every_value_chunk_preview_carries_a_reference_grid` covers Stage 3 across kinds; `composed_bundle_marks_dependency_target_and_harness_sections` covers the banner comments the composed WGSL text now carries (dependency / target / harness sections — a `docs/`-adjacent concern of the surrounding `bundle_build` composition, not this algorithm itself, but exercised in the same file); an all-chunks sweep confirms every bundled chunk except a small denylist previews successfully end to end |
