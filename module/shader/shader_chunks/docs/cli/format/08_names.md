# Format :: F08. names

| Field | Value |
|-------|-------|
| ID | F08 |
| Output context | `list`/`get` under `format::names` |
| Trigger | `format::names` on either query command |
| Structure | One chunk name per line, nothing else — no header, no alignment, no placeholders |
| Rendering mechanism | Direct join of the result's `name` fields in `render_chunks` (`src/lib.rs`) — no `data_fmt` pipeline needed for a bare line list |
| Example | See below |

### Example

`list pattern::noise format::names`:

```text
value_noise
fbm3
```

The shell-pipeline format: composable with `xargs`, `while read`,
`grep -c`. Deliberately ignores [`fields::`](../param/13_fields.md) —
the contract is "exactly the names, one per line", stable enough to
script against, which is also why the filter tests pin on it.

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.list](../command/01_list.md) | Via `format::names` |
| 2 | [.get](../command/02_get.md) | Via `format::names` |

---

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`format`](../param/15_format.md) | Selects this rendering |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
