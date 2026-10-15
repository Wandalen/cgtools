# Parameter :: 26. time

- **Fundamental Type:** [`Float`](../type/01_float.md) (unilang
  `Kind::Float`)
- **Constraints:** Any finite number — integer tokens (`time::2`) are
  accepted alongside fractional ones (`time::2.5`), and negatives are
  allowed; non-finite values are rejected by the routine with
  `` invalid `time` value: `<v>` (allowed: a finite number) ``, exit 1;
  non-numeric tokens fail unilang's float coercion before the routine
  runs (non-zero exit).
- **Default:** `0`
- **Purpose:** The value of the bundle's `time` uniform for the single
  rendered frame — the animation instant to freeze. Fragment chunks read
  it directly as `params.time`; a value chunk's synthesized grayscale
  harness uses it to drift the sample position (`time * 0.05`), so two
  renders at different `time::` values capture different slices of the
  same pattern.

### Examples
```bash
# Valid values
render fbm3               # time::0 is the default — the preview's opening frame
render fbm3 time::2.5     # the frame 2.5 seconds into the drift
render fbm3 time::2       # integer tokens coerce fine

# Invalid values (rejected with error)
render fbm3 time::later   # unilang float coercion failure, non-zero exit
```

### Notes
- The only `Kind::Float` parameter in the CLI — it introduces the
  [`Float`](../type/01_float.md) type. Every other numeric parameter
  (`limit`, `offset`, `width`) is a count where fractions are
  meaningless; `time` is a continuous quantity measured in seconds.
- Tested end to end: `render_time_advances_the_synthesized_drift`
  asserts a value chunk's `time::0` and `time::10` frames differ;
  `subprocess_render_with_non_numeric_time_is_rejected_by_coercion`
  pins the coercion failure path.
- A frozen `time` is what separates `.render` from
  [`.preview`](../../../../shader_chunks_preview/docs/cli/command/01_preview.md): the browser preview animates
  the uniform continuously; `.render` bakes exactly one value of it
  into the artifact.
- Member of no [parameter group](../../../../shader_chunks_query/docs/cli/param_group/readme.md).

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.render](../command/01_render.md) | `0` | Value of the `time` uniform at frame capture |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [Float](../type/01_float.md) | Float | `f32` (bound from `f64`) | Must be finite |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
