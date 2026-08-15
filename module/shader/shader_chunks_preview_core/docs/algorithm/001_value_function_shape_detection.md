# Algorithm: Value Function Shape Detection

### Scope

- **Purpose**: Decide whether a chunk export is a previewable "value
  function", which of 3 shapes it matches, and how the harness writes
  that shape's sampled value to the render target.
- **Responsibility**: Document `value_fn_of`'s parsing rule, `bundle_build`'s
  candidate-selection fallback, and `ValueFnKind::write_expr`'s per-shape
  render-target write, exactly as implemented.
- **In Scope**: The 3 previewable export shapes, the candidate-selection
  order among multiple matching exports in one chunk, and the write-out
  expression each shape maps to.
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
preference — and the chosen export's `ValueFnKind` then selects one of 3
fixed write-out expressions in the synthesized harness. No per-chunk
rescaling exists anywhere in this path: every shape writes its raw
sampled value and relies on the render target's own `[0, 1]` clamp, the
same convention the `f32` shape already used before `vec2f`/`vec3f`
existed.

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

**Stage 2 — render-target write** (`ValueFnKind::write_expr`, selects the
synthesized harness's final `return` expression once `value` has been
sampled):

| `ValueFnKind` | Write expression | Notes |
|----------------|-------------------|-------|
| `F32` | `vec4f( vec3f( value ), 1.0 )` | Grayscale; unchanged since before `Vec2`/`Vec3` existed. |
| `Vec2` | `vec4f( value, 0.5, 1.0 )` | Red/green from `value`; blue fixed at a neutral `0.5` pad — not `0.0`, so a `vec2f` field never reads as "half-missing-color" at a glance. |
| `Vec3` | `vec4f( value, 1.0 )` | Direct RGB passthrough. |

No expression here rescales, clamps, or remaps `value` beyond what the
render target's own format already does — the same "write the raw value,
let the target clamp to `[0, 1]`" convention `F32` established, extended
unchanged to the two new shapes.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `ValueFnKind`, `value_fn_of`, `harness_synthesize`, and the candidate-selection site in `bundle_build` — the entire algorithm |

### Tests

| File | Relationship |
|------|--------------|
| `tests/preview_bundle_test.rs` | `vec2_value_chunk_gets_a_synthesized_harness` / `vec3_value_chunk_gets_a_synthesized_harness` exercise Stage 0's `vec2f`/`vec3f` return-type match and Stage 2's write expressions against real bundled chunks; the existing `value_chunk_gets_a_synthesized_grayscale_harness` continues to cover `F32`; an all-chunks sweep confirms every bundled chunk except a small denylist previews successfully end to end |
