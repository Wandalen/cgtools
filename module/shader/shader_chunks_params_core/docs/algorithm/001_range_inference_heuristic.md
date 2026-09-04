# Algorithm: Range Inference Heuristic

### Scope

- **Purpose**: Resolve a numeric range for a `//@ param:` line that declares none.
- **Responsibility**: Document `range_infer`'s deterministic two-stage decision procedure — every name-substring pattern rule, every WGSL-type fallback rule, and the two unconditional `None` short-circuits — exactly as decided in [Q-03](../../../../../task/decisions.md#q-03--shader-chunk-tunable-parameter-declaration-discovery-and-range-resolution-strategy).
- **In Scope**: `range_infer`'s full rule table and evaluation order.
- **Out of Scope**: When `range_infer` is even called — a declared `range(min, max)` clause always wins outright and `range_infer` never runs in that case (see [`api/001`](../api/001_tunable_parameter_taxonomy.md), which states the WHAT this algorithm's HOW only partially determines).

### Abstract

`range_infer` is a pure, total function with no randomness: the same `(kind, value_type, name)` triple always yields the same `Option<Range>`. It answers one question — "if this parameter's author didn't bother declaring a range, what's a reasonable default?" — through two ordered stages, evaluated in this order every time:

1. Two unconditional `None` short-circuits, checked first: a `texture` kind, or a `bool` type, never carries a numeric range regardless of `name`.
2. A name-substring pattern match: does `name` contain any of a fixed set of common shader-parameter vocabulary words? If so, that word's associated range wins, regardless of `value_type`.
3. A WGSL-type-keyed fallback, used only when no name pattern matched: a default range keyed purely on `value_type`.

Every returned `Range` is tagged `RangeSource::Inferred` by the caller (`discover`) — `range_infer` itself has no concept of "declared", it only ever produces inferred values.

### Algorithm

**Stage 0 — kind/type short-circuits** (checked before anything else):

| Condition | Result |
|-----------|--------|
| `kind == ParameterKind::Texture` | `None` — unconditional, regardless of `value_type` or `name` |
| `value_type == ValueType::Bool` | `None` — unconditional, regardless of `kind` or `name` |

**Stage 1 — name-substring pattern match** (checked only if Stage 0 did not short-circuit; `name.contains(needle)` for each needle in a pattern's list, first matching pattern wins, patterns tried in the order below):

| Name substrings | Range |
|------------------|-------|
| `octaves`, `count`, `steps`, `iterations` | `[1.0, 8.0]` |
| `seed` | `[0.0, 65535.0]` |
| `angle`, `rotation` | `[0.0, τ]` (`std::f64::consts::TAU`) |
| `scale`, `frequency`, `freq` | `[0.1, 10.0]` |
| `amplitude`, `weight`, `opacity`, `alpha`, `mix`, `blend` | `[0.0, 1.0]` |
| `radius`, `size`, `width`, `height` | `[0.0, 100.0]` |

**Stage 2 — WGSL-type fallback** (checked only if Stage 0 did not short-circuit and no Stage 1 pattern matched `name`; keyed on `value_type` alone, exhaustive over every non-`Bool` `ValueType` variant since `Bool` is already excluded by Stage 0):

| `value_type` | Range |
|---------------|-------|
| `U32`, `Vec2U`, `Vec3U`, `Vec4U` | `[0.0, 16.0]` |
| `I32`, `Vec2I`, `Vec3I`, `Vec4I` | `[-16.0, 16.0]` |
| `F32`, `Vec2F`, `Vec3F`, `Vec4F` | `[0.0, 1.0]` — matches this codebase's own bundled noise chunks' unit-interval convention |
| `Texture2d` | Unreachable in practice: a `texture` `<kind>` line already returns `None` at Stage 0 before `value_type` is even consulted; included here only because the match must be exhaustive over all 14 `ValueType` variants |

If Stage 0 does not short-circuit and neither Stage 1 nor Stage 2 apply, the function has no remaining case — this cannot occur, since Stage 2's fallback is exhaustive over every `ValueType` variant Stage 0 doesn't already exclude.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `range_infer`, `range_by_name_infer`, `range_by_type_infer` — the entire algorithm |

### Tests

| File | Relationship |
|------|--------------|
| `tests/range_inference_test.rs` | Exercises every name-pattern rule, every type-fallback rule, both `None` short-circuits (including precedence over a name that would otherwise pattern-match), and name-pattern-beats-type-fallback precedence |
| `tests/discovery_test.rs` | `discover_declared_range_overrides_name_pattern_inference` / `discover_declared_range_overrides_type_fallback_inference` confirm a declared `range(min, max)` wins outright and `range_infer` never runs in that case |
