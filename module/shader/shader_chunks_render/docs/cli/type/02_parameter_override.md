# Type :: 12. ParameterOverride

**Purpose:** One element of the `set::` override list — a
`<property>:<value>` pair assigning a new value to one of the target
bundle's declared parameters.

**Fundamental Type:** `String` (element of unilang `Kind::List(String,
',')`) parsed into `(String, f64)`. No wrapper struct — parsing
(`overrides_parse`) and identity resolution against a live bundle
(`overrides_apply`) are two separate free functions in
`shader_chunks_render/src/lib.rs`, not a dedicated type.

**Constraints:**
- Must contain a `:` separator — `split_once(':')`; absent, the whole
  token is rejected (`RenderCliError::InvalidOverride`)
- The side after `:` must parse as `f64` and be finite — `inf`, `-inf`,
  `nan`, and non-numeric text are all rejected the same way as a missing
  separator
- The side before `:` is not validated at parse time — only later, when
  `overrides_apply` matches it against the bundle's actual
  `PreviewParameter::property` names; an unresolved name is a distinct
  error (`RenderCliError::UnknownOverrideParameter`) naming every valid
  property
- No range check against the parameter's declared `min`/`max` — those
  are browser-slider UI hints, not a value constraint enforced here

**Parsing:** `overrides_parse(raw: &[String]) -> Result<Vec<(String,
f64)>, RenderCliError>` (`shader_chunks_render/src/lib.rs`) — maps each
list element through `split_once(':')` then a finiteness-filtered
`f64` parse, short-circuiting on the first failure. Resolution against a
bundle happens separately in `overrides_apply(bundle: &mut
PreviewBundle, overrides: &[(String, f64)]) -> Result<(), RenderCliError>`,
which applies pairs in order — a later pair naming the same property as
an earlier one overwrites it, so list order decides the winner.

**Methods:**
- `overrides_parse(&[String]) -> Result<Vec<(String, f64)>,
  RenderCliError>` — shape validation, bundle-independent
- `overrides_apply(&mut PreviewBundle, &[(String, f64)]) ->
  Result<(), RenderCliError>` — identity resolution and in-place mutation

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.render`](../command/01_render.md) | `set::` |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`set`](../param/04_set.md) | 1 |
