# Type :: 3. OutputFormat

**Purpose:** Selects which of the 6 renderings the query engine produces
for its filtered, projected, sorted result.

**Fundamental Type:** `enum OutputFormat { Table, Markdown, Expanded,
Json, Yaml, Names }` in `src/lib.rs` —
`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, a genuine Rust enum
(unlike the string-realized selector types), carried inside `QueryParams`.

**Constraints:**
- Exactly 6 lowercase spellings accepted: `table`, `markdown`,
  `expanded`, `json`, `yaml`, `names`
- Closed set — no aliases, no case-insensitivity

**Parsing:** `OutputFormat::from_str` (via Rust's `FromStr`). No match →
`CliError::InvalidParam { param: "format", .. }`, reported as `` invalid
`format` value: `<value>` (allowed: table, markdown, expanded, json,
yaml, names) `` on stderr with a non-zero exit.

**Methods:**
- `as_str() -> &'static str` — canonical lowercase spelling; round-trips
  with `from_str`, and supplies the `[default: …]` text in help screens
- `from_str(s) -> Result<Self, CliError>` — the parse above
- Dispatch: `render_chunks` (`src/lib.rs`) matches on the variant to
  select the `data_fmt` pipeline (see [`../format/`](../format/readme.md))

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.list`](../command/01_list.md) | `format::` (default `table`) |
| 2 | [`.get`](../command/02_get.md) | `format::` (default `expanded`) |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`format`](../param/15_format.md) | 2 |
