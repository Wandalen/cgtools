# Parameter :: 28. set

- **Fundamental Type:** [`ParameterOverride`](../type/02_parameter_override.md)
  list (unilang `Kind::List(String, ',')`) — each element a
  `<property>:<value>` pair
- **Constraints:**
  - Each comma-separated element must contain a `:` separator; the side
    after it must parse as a finite `f64` — either failure rejects the
    whole `set::` value with `RenderCliError::InvalidOverride`, exit 1,
    quoting the offending element verbatim
  - The side before `:` must name a parameter the target's bundle
    actually declares (its own `//@ param:` exports plus the
    harness-synthesized `preview_scale`); an unrecognized name rejects
    with `RenderCliError::UnknownOverrideParameter`, exit 1, listing
    every valid property
  - A later element overriding the same property as an earlier one wins —
    overrides apply in list order, not deduplicated beforehand
  - Values are baked in as-is, never clamped to the parameter's declared
    `min`/`max` — those describe the browser slider's UI range only, not
    a hard constraint on the underlying uniform
- **Default:** none — omitting `set::` leaves every parameter at its
  bundle-declared starting value (the same value the browser preview's
  sliders start at)
- **Purpose:** Lets `.render` capture a frame at any parameter
  combination, not just the defaults — freezing a specific slider
  position (found live in `.preview`) as a static, committable image
  without touching a browser.

### Examples
```bash
# Valid values
render fbm3 set::lacunarity:2.5,gain:0.75   # both overridden, other parameters keep their defaults
render fbm3 set::gain:0.1,gain:0.9          # later wins: gain ends at 0.9

# Invalid values (rejected with error)
render fbm3 set::bogus:1.0    # error: unknown parameter: `bogus` (valid parameters: lacunarity, gain, preview_scale), exit 1
render fbm3 set::gain         # error: invalid `set` override: `gain` (allowed: `<property>:<finite number>`), exit 1
render fbm3 set::gain:inf     # error: invalid `set` override: `gain:inf` (allowed: `<property>:<finite number>`), exit 1
```

### Notes
- Parsed by `overrides_parse` (element shape, independent of any bundle)
  then applied by `overrides_apply` (identity resolution against the
  live bundle's declared parameters), both in
  `shader_chunks_render/src/lib.rs` — the CLI's only parameter validated
  in two independent stages.
- Run [`tunables <name>`](../../../../shader_chunks_params/docs/cli/command/01_tunables.md)
  first to discover a target's exact property names and declared
  `min`/`max` hints before writing a `set::` value.
- Same comma-list-of-colon-pairs grammar as
  [`tag`](../../../../shader_chunks_query/docs/cli/param/05_tag.md) —
  `split_once(':')` per element — but the value side is a required
  finite number here, never an arbitrary string, and an unmatched
  property is a hard error (exit 1) rather than `tag`'s silent
  empty-match.
- Member of no [parameter group](../../../../shader_chunks_query/docs/cli/param_group/readme.md) —
  like `out`/`size`/`time`, it shapes the rendered artifact rather than
  filtering/projecting/formatting query output.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.render](../command/01_render.md) | none (bundle defaults) | Applies after target resolution and naga validation, before the headless GPU render |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [ParameterOverride](../type/02_parameter_override.md) | List element | `(String, f64)` | `<property>:<finite number>`, property must match a declared parameter |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
