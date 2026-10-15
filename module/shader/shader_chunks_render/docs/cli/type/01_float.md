# Type :: 11. Float

**Purpose:** A continuous numeric parameter where fractional values are
meaningful — the shape of `time`, measured in seconds of animation
drift, as opposed to the count-shaped
[`NonNegativeInteger`](../../../../shader_chunks_query/docs/cli/type/08_non_negative_integer.md) where `0`-or-more
whole steps are the domain.

**Fundamental Type:** `f32` at the point of use (the GPU uniform buffer
is `f32`), bound from unilang `Kind::Float` (`f64`) — the narrowing cast
happens in `shader_chunks_render/src/lib.rs`, matching the browser
preview runner, which also feeds the uniform as `f32`.

**Constraints:**
- Must be finite — `shader_chunks_render`'s own guard rejects NaN and
  ±infinity with `` invalid `time` value: `<v>` (allowed: a finite
  number) ``, exit 1, independently of whatever unilang's coercion lets
  through
- Integer tokens are valid — unilang may deliver `time::2` as an
  integer value, and the parse accepts both numeric shapes
- Negatives are valid — `time` carries no non-negativity requirement; a
  negative instant simply drifts the pattern the other way
- Non-numeric tokens fail unilang's `Kind::Float` coercion before the
  command routine is entered — non-zero exit

**Parsing:** `arg_time(cmd)` (`shader_chunks_render/src/lib.rs`) —
matches both `Value::Float(f)` and `Value::Integer(n)` from the parsed
arguments, then requires `is_finite`. Deliberately local to
`shader_chunks_render` rather than promoted into
`shader_chunks_cli_core`'s shared `arg_*` family — it currently has
exactly one consumer.

**Methods:**
- `arg_time(cmd) -> Result<f32, ErrorData>` — the parse above, private
  to the crate, carrying the offending value into the error message

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.render`](../command/01_render.md) | `time::` |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`time`](../param/03_time.md) | 1 |
