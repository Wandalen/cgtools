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
ValueFnKind, Vec<String>)>` — the export's name, its return shape, and the
names of any trailing `f32` arguments after the leading `vec2f` sample
point (the chunk's own tunables, resolved into `own_params` at the
`bundle_build` call site — see Stage 1). It answers one question — "does
this export look like a spatial value function the preview harness can
sample, and if so, what shape does it return and what does it still need?"
— through a single structural parse, no name-based heuristics involved
(unlike `shader_chunks_params_core`'s `range_infer`, which this crate's
design deliberately does not mirror here: a chunk's *shape* is a hard
structural fact, not something worth inferring from naming convention).

At the `bundle_build` call site, when a chunk has more than one matching
export, candidate selection first discards exports whose trailing
arguments aren't fully backed by declared `//@ param:` lines, then applies
one fixed name-preference order — never a shape preference — and the
chosen export's `ValueFnKind`, together with one tag check on the *target
chunk's own* manifest (`category:sdf`, independent of which chunk the
export was collected from), selects the synthesized harness's sampling
point and render-target write. Every shape still writes
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
| Not of the form `fn NAME(ARGS) -> RETURN` (missing `fn `, unbalanced parens, no `->`) | `None` |
| `NAME` is empty | `None` |
| First argument's type is not exactly `vec2f` | `None` |
| Any trailing (2nd, 3rd, ...) argument's type is not exactly `f32` | `None` |
| `RETURN` is not exactly `f32`, `vec2f`, or `vec3f` | `None` |
| First argument is `vec2f`, every trailing argument is `f32`, `RETURN` matches | `Some((NAME, ValueFnKind::{F32\|Vec2\|Vec3}, extra_args))` |

`extra_args` collects the trailing arguments' own names, in signature
order — empty when the export takes only its `vec2f` sample point. The
first-argument type check never widens — the sample point is always 2D
regardless of which value the function returns; trailing arguments are
always `f32` (the harness only ever synthesizes scalar uniform tunables);
only the *return* type match widens across the 3 kinds.

**Stage 1 — candidate selection** (`bundle_build`, only reached when the
target chunk is not `//@ stage: fragment`; runs once per `bundle_build`
call, over every export that passed Stage 0):

| Step | Rule |
|------|------|
| 1 | Collect every `//@ export:` line in the target chunk's *own* manifest ( `exports_parse(target_wgsl)` — never a dependency's exports, regardless of `depends_on`; see Sources ) that Stage 0 matched, preserving file/manifest order. |
| 2 | Discard any candidate with an `extra_args` entry that has no matching `//@ param: NAME argument f32 range(min, max)` declaration ( kind `Argument`, checked by name only ) anywhere in the target chunk's own manifest — the surviving candidates are *viable*. A structurally-matching export with unbacked trailing arguments ( e.g. a primitive like `d2_sdf_circle(p: vec2f, radius: f32) -> f32`, called by dependents with real values, not a preview wrapper ) is simply not a candidate at all; this is not an error by itself. |
| 3 | Among viable candidates, if one is named `"{name}_preview"` ( the target chunk's own `name`, suffixed ), pick it. |
| 4 | Otherwise, among viable candidates, if one's name equals the target chunk's own `name`, pick it. |
| 5 | Otherwise pick the first viable candidate in manifest order. |
| 6 | If no viable candidate exists, fail with `PreviewError::Unpreviewable`. |

Steps 3/4/5 apply identically regardless of `ValueFnKind` — a `vec3f`-shaped
export is never preferred or deprioritized relative to an `f32`-shaped
one. This is deliberate: a chunk exporting both its own `NAME`-matching
value function and some unrelated previewable helper should always
preview itself, not whichever shape happens to look more interesting.

Step 3 exists because a `//@ param:` declaration is scoped to a *name*,
not to one specific export — when a primitive and its dedicated
`NAME_preview` wrapper happen to share a trailing argument name (a natural
pattern, since both conceptually take the same parameter), both become
viable at Step 2, and without Step 3 the Step 4 tie-break always wins for
the primitive (it shares the chunk's own `name` by construction; the
wrapper never does). Checking for a viable `NAME_preview` candidate first
restores the intended fall-through to a dedicated wrapper when the
chunk's own primitive export isn't a meaningful preview on its own.

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
| `tests/preview_bundle_test.rs` | `vec2_value_chunk_gets_a_synthesized_harness` / `vec3_value_chunk_gets_a_synthesized_harness` exercise Stage 0's `vec2f`/`vec3f` return-type match and Stage 2's non-SDF write expressions against real bundled chunks; the existing `value_chunk_gets_a_synthesized_grayscale_harness` continues to cover non-SDF `F32`; `sdf_tagged_value_chunk_gets_filled_banded_visualization_and_stationary_sampling` covers the `F32`+SDF combination (fill/band/isoline color, stationary sample point) against `sdf_op_round`; `non_sdf_value_chunk_keeps_raw_grayscale_and_time_drift` confirms an untagged `F32` chunk keeps the original drifting/raw-value path; `every_value_chunk_preview_carries_a_reference_grid` covers Stage 3 across kinds; `composed_bundle_marks_dependency_target_and_harness_sections` covers the banner comments the composed WGSL text now carries (dependency / target / harness sections — a `docs/`-adjacent concern of the surrounding `bundle_build` composition, not this algorithm itself, but exercised in the same file); `value_chunk_prefers_dedicated_preview_wrapper_over_same_named_primitive_sharing_an_argument_name` covers Stage 1 Step 3 — a viable `NAME_preview` candidate must be chosen over the chunk's own same-named primitive even when both are viable under a shared trailing-argument name (BUG-205); an all-chunks sweep confirms every bundled chunk except a small denylist previews successfully end to end |
