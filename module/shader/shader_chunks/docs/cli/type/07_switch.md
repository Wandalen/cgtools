# Type :: 7. Switch

**Purpose:** A boolean toggle parameter — the shared shape of the 5
on/off query parameters (`case`, `transitive`, `roots`, `leaves`,
`count`).

**Fundamental Type:** `bool` (unilang `Kind::Boolean`). No wrapper type —
unilang performs the string→bool coercion during argument binding, before
any `shader_chunks` code runs; the routines read an already-typed
`Value::Boolean`.

**Constraints:**
- Truthy spellings: `1`, `true`, `yes`; falsy: `0`, `false`, `no`
  (unilang lowercases first, so `TRUE`/`Yes` also coerce)
- Anything else fails unilang's boolean coercion — non-zero exit before
  the command routine is entered

**Parsing:** unilang's argument binding (`coerce_arg_value`); because
every Switch declares a `"false"` default in its `ArgumentDefinition`,
the bound argument map always carries a typed value — routines never see
an absent Switch.

**Methods:**
- `arg_bool(cmd, key)` (`src/cli.rs`) — reads the bound
  `Value::Boolean` for a Switch parameter into the `QueryParams` field

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.list`](../command/01_list.md) | `case::`, `transitive::`, `roots::`, `leaves::`, `count::` |
| 2 | [`.get`](../command/02_get.md) | `case::`, `transitive::`, `roots::`, `leaves::`, `count::` |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`case`](../param/04_case.md) | 2 |
| 2 | [`transitive`](../param/09_transitive.md) | 2 |
| 3 | [`roots`](../param/11_roots.md) | 2 |
| 4 | [`leaves`](../param/12_leaves.md) | 2 |
| 5 | [`count`](../param/14_count.md) | 2 |
