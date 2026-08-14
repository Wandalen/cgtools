# Type :: 9. TagSelector

**Purpose:** One element of the `tag::` filter list — either an exact
`group:tag` pair or a bare `tag` matched under any group.

**Fundamental Type:** `String` (element of unilang
`Kind::List(String, ',')`). No wrapper type — the pair/bare distinction
is a `split_once(':')` at match time (`matches_tag_selector` in
`src/lib.rs`), not a parse into a struct.

**Constraints:**
- Any string is structurally valid — a selector matching no declared tag
  simply matches no chunk (empty output, exit 0), never an error
- The first `:` splits group from tag; a selector without `:` is a bare
  tag name compared against the tag part of every declared `group:tag`
- Comparison is exact and case-sensitive on both sides

**Parsing:** `matches_tag_selector(selector, chunk_tags)`
(`src/lib.rs`): `split_once(':')` present → exact match against the full
declared pair; absent → match against the tag part under any group.
Multiple selectors combine per [`TagsMode`](06_tags_mode.md).

**Methods:**
- `matches(selector, tag) -> bool` — conceptually the per-tag comparison
  above; realized inside `chunk_matches`'s tag filter loop
- Discovery counterpart: the `tags` command enumerates every declared
  `group:tag` pair a selector can target

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.list`](../command/01_list.md) | `tag::` |
| 2 | [`.get`](../command/02_get.md) | `tag::` |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`tag`](../param/05_tag.md) | 2 |
