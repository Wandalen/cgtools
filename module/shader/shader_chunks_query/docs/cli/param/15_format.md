# Parameter :: 15. format

- **Fundamental Type:** [`OutputFormat`](../type/03_output_format.md)
  (unilang `Kind::String`, parsed by `OutputFormat::from_str` in
  `shader_chunks_query_core/src/lib.rs`)
- **Constraints:** Exactly one of `table`, `markdown`, `expanded`,
  `json`, `yaml`, `names`; anything else is `CliError::InvalidParam`
  naming the allowed set, non-zero exit
- **Default:** `Varies` — `table` for `list` (columnar overview),
  `expanded` for `get` (one record block per chunk)
- **Purpose:** Selects the rendering of the filtered, projected,
  sorted result — human tables, machine formats, or a bare name list
  for shell pipelines.

### Examples
```bash
# Valid values
list format::table                       # plain aligned columns (default)
list format::markdown heading::Chunks    # pipe table, optional heading
get hash21 format::json                  # machine-readable
list pattern::noise format::names        # pipeline-friendly name lines

# Invalid values (rejected with error)
list format::bogus   # "invalid `format` value: `bogus` (allowed: table,
                      #  markdown, expanded, json, yaml, names)"
```

### Notes
- Format-to-file mapping: `table` →
  [`table_plain`](../format/01_table_plain.md), `markdown` →
  [`markdown`](../format/04_markdown.md), `expanded` →
  [`expanded`](../format/05_expanded.md), `json` →
  [`json`](../format/06_json.md), `yaml` → [`yaml`](../format/07_yaml.md),
  `names` → [`names`](../format/08_names.md).
- `json`/`yaml` key order within a record is not guaranteed — consumers
  must parse, not string-match (see the format files).
- Member of the [formatting](../param_group/03_formatting.md) parameter
  group; the per-command default is one of the two ways `list` and `get`
  differ (the other being [`fields::`](13_fields.md)).

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `table` | Overview rendering |
| 2 | [.get](../command/02_get.md) | `expanded` | Detail rendering |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [OutputFormat](../type/03_output_format.md) | String (enum) | `OutputFormat` | 6 closed variants, loud rejection otherwise |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
