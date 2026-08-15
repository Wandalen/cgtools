# Parameter :: 21. width

- **Fundamental Type:**
  [`NonNegativeInteger`](../type/08_non_negative_integer.md) (unilang
  `Kind::Integer`, then `usize::try_from` in `shader_chunks_cli_core/src/lib.rs`)
- **Constraints:** ≥ 0; a negative value is `CliError::InvalidParam`
  ("allowed: a non-negative integer"), non-zero exit
- **Default:** `0` — the reserved "auto" value: columns size to their
  widest cell
- **Purpose:** Caps every column's width in `table` and `markdown`
  output, truncating longer cells — keeps wide fields (`description`,
  `source`) terminal-friendly.

### Examples
```bash
# Valid values
list width::12               # cells longer than 12 chars truncate with `...`
list format::markdown width::30
list format::json width::12  # accepted, ignored (documented no-op)

# Invalid values (rejected with error)
list width::-1   # "invalid `width` value: `-1` (allowed: a non-negative integer)"
```

### Notes
- Truncation is `data_fmt`'s `with_max_column_width` behavior — an
  ellipsis marks cut cells; the underlying data is untouched.
- Shapes only the `table` and `markdown` formats; under `expanded`,
  `json`, `yaml`, and `names` it is accepted and ignored — machine
  formats must carry full values.
- Member of the [formatting](../param_group/03_formatting.md) parameter
  group.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `0` (auto) | Max column width |
| 2 | [.get](../command/02_get.md) | `0` (auto) | Max column width |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [NonNegativeInteger](../type/08_non_negative_integer.md) | Integer | `usize` | ≥ 0, loud rejection of negatives |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
