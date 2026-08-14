# Type :: 6. TagsMode

**Purpose:** Selects how multiple [`TagSelector`](09_tag_selector.md)s
combine — union or intersection.

**Fundamental Type:** `enum TagsMode { Any, All }` in `src/lib.rs` —
`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, carried inside
`QueryParams`.

**Constraints:**
- Exactly 2 lowercase spellings accepted: `any`, `all`
- Closed set — no aliases, no case-insensitivity

**Parsing:** `TagsMode::from_str`. No match →
`CliError::InvalidParam { param: "tags_mode", .. }`, reported as
`` invalid `tags_mode` value: `<value>` (allowed: any, all) `` on stderr
with a non-zero exit.

**Methods:**
- `as_str() -> &'static str` — canonical spelling, round-trips with
  `from_str`
- Semantics (`chunk_matches` in `src/lib.rs`): `Any` keeps a chunk when
  *at least one* selector matches (`iter().any`), `All` when *every*
  selector matches (`iter().all`)

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.list`](../command/01_list.md) | `tags_mode::` |
| 2 | [`.get`](../command/02_get.md) | `tags_mode::` |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`tags_mode`](../param/06_tags_mode.md) | 2 |
