# Type :: 8. NonNegativeInteger

**Purpose:** A count-like numeric parameter that can never be negative —
the shared shape of `limit`, `offset`, and `width`, each reserving `0`
as a semantic value (unlimited / start / auto).

**Fundamental Type:** `usize` (unilang `Kind::Integer` at the binding
boundary — signed — then `usize::try_from` in `src/cli.rs`). No wrapper
type; the non-negativity constraint lives in the conversion.

**Constraints:**
- ≥ 0 — negatives pass unilang's integer coercion (which is signed) and
  are rejected by `shader_chunks` itself
- Non-numeric values fail unilang's integer coercion first — non-zero
  exit before the routine is entered
- `0` is always valid and always reserved: `limit::0` = unlimited,
  `offset::0` = start, `width::0` = auto

**Parsing:** `arg_usize(cmd, key)` (`src/cli.rs`) —
`usize::try_from(value)`; a negative →
`CliError::InvalidParam { param: key, .. }`, reported as `` invalid
`<param>` value: `<value>` (allowed: a non-negative integer) `` on stderr
with a non-zero exit.

**Methods:**
- `arg_usize(cmd, key) -> Result<usize, ErrorData>` — the parse above,
  carrying the parameter's own name into the error message

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.list`](../command/01_list.md) | `limit::`, `offset::`, `width::` |
| 2 | [`.get`](../command/02_get.md) | `limit::`, `offset::`, `width::` |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`limit`](../param/18_limit.md) | 2 |
| 2 | [`offset`](../param/19_offset.md) | 2 |
| 3 | [`width`](../param/21_width.md) | 2 |
