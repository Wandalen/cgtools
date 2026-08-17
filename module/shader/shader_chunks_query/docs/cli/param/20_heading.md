# Parameter :: 20. heading

- **Fundamental Type:** `String` (unilang `Kind::String`, wrapped into
  `data_fmt::Heading` at render time)
- **Constraints:** None — any non-empty string renders; empty/absent
  means no heading line
- **Default:** off (no heading line)
- **Purpose:** Prints a heading line above `table` and `markdown` output
  — labeling a query's result when several queries are combined into one
  report.

### Examples
```bash
# Valid values
list heading::Chunks                          # heading above the plain table
list format::markdown heading::"All Chunks"   # heading above the pipe table
list format::json heading::Chunks             # accepted, ignored (documented no-op)

# No invalid values — any string is a legal heading.
```

### Notes
- Shapes only the `table` and `markdown` formats; under `expanded`,
  `json`, `yaml`, and `names` it is accepted and ignored — machine
  formats must stay parseable and un-decorated.
- Member of the [formatting](../param_group/03_formatting.md) parameter
  group.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | off | Table/markdown heading line |
| 2 | [.get](../command/02_get.md) | off | Table/markdown heading line |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| — (plain string) | String | `String` | None — any heading text |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
