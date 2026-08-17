# Type :: 7. Switch

**Purpose:** A boolean toggle parameter — the shared shape of the 5
on/off query parameters (`case`, `transitive`, `roots`, `leaves`,
`count`) plus `.preview`'s `serve`.

**Fundamental Type:** `bool` (unilang `Kind::Boolean`). No wrapper type —
unilang performs the string→bool coercion during argument binding, before
any `shader_chunks` code runs; the routines read an already-typed
`Value::Boolean`.

**Constraints:**
- Truthy spellings: `1`, `true`, `yes`; falsy: `0`, `false`, `no`
  (unilang lowercases first, so `TRUE`/`Yes` also coerce)
- Anything else fails unilang's boolean coercion — non-zero exit before
  the command routine is entered

**Parsing:** unilang's argument binding (`coerce_arg_value`); every
Switch declares an explicit default in its `ArgumentDefinition` — `false`
for the 5 query switches, `true` for `.preview`'s `serve` — so the bound
argument map always carries a typed value — routines never see an absent
Switch.

**Methods:**
- `arg_bool(cmd, key)` (`shader_chunks_cli_core/src/lib.rs`) — reads the bound
  `Value::Boolean` for a Switch parameter into the `QueryParams` field

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.list`](../command/01_list.md) | `case::`, `transitive::`, `roots::`, `leaves::`, `count::` |
| 2 | [`.get`](../command/02_get.md) | `case::`, `transitive::`, `roots::`, `leaves::`, `count::` |
| 3 | [`.compose`](../../../../shader_chunks_compose/docs/cli/command/01_compose.md) | `transitive::` |
| 4 | [`.preview`](../../../../shader_chunks_preview/docs/cli/command/01_preview.md) | `serve::` |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`case`](../param/04_case.md) | 2 |
| 2 | [`transitive`](../param/09_transitive.md) | 3 |
| 3 | [`roots`](../param/11_roots.md) | 2 |
| 4 | [`leaves`](../param/12_leaves.md) | 2 |
| 5 | [`count`](../param/14_count.md) | 2 |
| 6 | [`serve`](../../../../shader_chunks_preview/docs/cli/param/02_serve.md) | 1 |
