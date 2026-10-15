# BUG-498: PEC/PES writers truncate `&str` fields by raw byte length with no UTF-8 char-boundary check, embedding invalid UTF-8

- **Severity:** Low (only reachable for a design name / metadata string containing a multi-byte
  UTF-8 character that straddles the field's fixed byte-length limit; no crash, but the written
  file embeds invalid UTF-8 bytes in a fixed-width field, corrupting any downstream reader that
  interprets that field as text)
- **state:** Completed
- **Affects:** Any embroidery file whose `Metadata::name` (PEC) or written string field (PES
  `pes_string16_write`/`pes_string8_write`) contains a multi-byte UTF-8 character positioned so
  the raw byte-length limit falls strictly inside that character's byte sequence.
- **Component:** `module/helper/embroidery_tools` (`src/format/pec/writer.rs`,
  `src/format/pes/writer.rs`, `src/format.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Found in the same sweep as BUG-497 (same crate, same writer path) but a
  different mechanism -- filed separately, no shared root cause.

## Symptom

```rust
// pre-fix -- src/format/pec/writer.rs, pec_header_write
let name = emb.metadata_get().name_get().unwrap_or( "Untitled" );
if name.len() >= 16
{
  writer.write_all( &name.as_bytes()[ ..16 ] )?; // raw byte slice, no char-boundary check
}
```

```rust
// pre-fix -- src/format/pes/writer.rs, pes_string16_write / pes_string8_write
let len = str.len().min( usize::from( u16::MAX ) );
writer.write_all( &str.as_bytes()[ ..len ] )?; // same defect, different limit
```

3 sites total (1 in `pec/writer.rs`, 2 in `pes/writer.rs`) slice a `&str`'s UTF-8 bytes at a raw
byte-length limit with no check that the limit lands on a character boundary -- if it falls
mid-character, `&name.as_bytes()[ ..16 ]` silently produces a byte slice that is not valid UTF-8
(a truncated multi-byte sequence with no continuation bytes).

## Impact

**Who is affected:** Any caller writing an embroidery file whose design name (or other written
string field) contains a non-ASCII character positioned near a field's byte-length boundary (16
bytes for PEC's `name`, `u16::MAX` bytes for PES's `string16`/`string8` fields -- the latter
astronomically unlikely to occur by chance but reachable via crafted/adversarial input).

**What breaks:** The written file's fixed-width name field contains invalid UTF-8 bytes. Rust's
own `&str` slicing (`[ ..16 ]`) would have panicked loudly on a bad boundary -- but this code
slices `&[u8]` (`.as_bytes()[ ..16 ]`), which has no such check and always succeeds, silently
producing corrupt output instead of erroring.

**Consumer audit:** Grepped both writer files for raw `.as_bytes()[ ..N ]` / `.len().min(...)`
patterns writing string data -- exactly 3 sites, all fixed by this same change (routed through
one shared helper).

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX sweep of `module/helper/embroidery_tools`.

## Minimum Reproducible Example

```rust
// module/helper/embroidery_tools/tests/pec_test.rs
// name = 15 ASCII 'a' bytes + one 3-byte '€' (byte 15..18) + more text;
// truncating naively at byte 16 lands 1 byte into '€', producing invalid UTF-8.
let name = format!( "{}{}", "a".repeat( 15 ), "€rest" );
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/embroidery_tools && cargo nextest run -E 'test(straddling_field_limit)'
```

## Root Cause

All 3 sites truncated by indexing raw bytes (`&str.as_bytes()[ ..limit ]` or an equivalent
`len().min(limit)` computed from `.len()`, Rust's UTF-8 byte length, not a character count) with
no check that the truncation point falls on a UTF-8 character boundary. `str`'s own safe slicing
operator would panic on a bad boundary; slicing the byte-level `&[u8]` view instead bypasses that
guard entirely.

## Why Not Caught

No existing test constructed a design name containing a multi-byte UTF-8 character positioned
near either field's byte-length limit -- all pre-existing PEC/PES fixture and roundtrip tests use
plain ASCII names.

## Fix Location

Added `str_truncate_char_boundary( s : &str, max_bytes : usize ) -> &str` to
`module/helper/embroidery_tools/src/format.rs` (the shared parent module both `pec` and `pes` are
`layer`s of, exposed via `mod_interface`'s `own use`) -- scans backward from `max_bytes` to the
nearest valid `str::is_char_boundary` position, returning a shorter (but always valid-UTF-8)
slice when the exact limit falls mid-character. All 3 sites now call this one shared helper
instead of duplicating the boundary-scan logic 3 times:

- `pec::writer::pec_header_write`: `let truncated = format::str_truncate_char_boundary( name, 16
  ); writer.write_all( truncated.as_bytes() )?;` followed by space-padding computed from
  `truncated.len()` (not a fixed `16 - name.len()`), so the field's total width stays exactly 16
  bytes even when `truncated` is shorter than the limit.
- `pes::writer::pes_string16_write`/`pes_string8_write`: same helper, with the length prefix
  (`u16`/`u8`) computed from `truncated.len()` instead of the raw pre-truncation `len`.

## Prevention

New tests in `pec_test.rs`: `str_truncate_char_boundary_backs_off_to_valid_utf8` (direct unit
test of the helper -- asserts the straddling-`€`-name case backs off to the full 15-byte ASCII
prefix, not a partial `€`, plus 2 edge cases: an already-short string returned unchanged, and an
exact-boundary limit needing no back-off) and
`pec_write_with_multibyte_name_straddling_field_limit_stays_valid_utf8` (integration test --
writes a PEC file with the straddling name, locates the `"LA:"` marker, and asserts the following
16-byte name field is valid UTF-8 via `str::from_utf8`).

## Pitfall

`&str`'s own indexing operator (`&s[ ..n ]`) already panics loudly on a bad char boundary --
it is only unsafe to truncate blindly once code drops to the byte-slice view (`s.as_bytes()[
..n ]`), which has no such check. A raw byte-length limit derived from a wire-format spec (a
fixed 16-byte field, a `u16` length prefix) needs an explicit boundary-aware truncation step
before it can safely bound a `&str` slice -- the byte limit and the character boundary are two
different units that happen to coincide for ASCII-only input, masking the gap until non-ASCII
text is used.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of `module/helper/embroidery_tools`. |
| 2026-08-20 | fixed | Added shared `str_truncate_char_boundary` helper; wired into all 3 sites. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily reverted `pec_header_write`'s name-writing block to the pre-fix raw `[ ..16 ]` byte slice and confirmed `pec_write_with_multibyte_name_straddling_field_limit_stays_valid_utf8` fails (`the 16-byte name field must always be valid UTF-8 ... got raw bytes [... 0xe2]`, a truncated 3-byte sequence's lead byte); restored the fix and confirmed 20/20 crate tests pass. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-498)`/`Root cause`/`Pitfall` 3-field comment applied at the helper's definition, plus shorter pointer comments at each of the 3 call sites. | — |
| D3 | Scope containment | — | 🟢 | Changes confined to `format.rs` (new helper) and the 3 call sites in `pec/writer.rs`/`pes/writer.rs`; no unrelated files touched. | — |

**Reproduced:** YES -- temporarily reverted `pec/writer.rs`'s name-truncation block to the raw
pre-fix `if name.len() >= 16 { write [..16] } else { write + pad }` form (helper left intact);
`pec_write_with_multibyte_name_straddling_field_limit_stays_valid_utf8` failed with raw bytes
ending `[..., 0x61, 0xe2]` (a bare UTF-8 lead byte with no continuation, confirmed invalid via
`str::from_utf8`'s panic message). Restored the fix; full crate suite (20/20) passes with 0
warnings. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/src/format.rs` | Added `str_truncate_char_boundary`, exposed via `mod_interface` `own use`. |
| `module/helper/embroidery_tools/src/format/pec/writer.rs` | `pec_header_write`'s name field now truncates via the shared helper; padding computed from the truncated length. |
| `module/helper/embroidery_tools/src/format/pes/writer.rs` | `pes_string16_write`/`pes_string8_write` now truncate via the shared helper before computing their length prefix. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/tests/pec_test.rs` | Added `str_truncate_char_boundary_backs_off_to_valid_utf8` and `pec_write_with_multibyte_name_straddling_field_limit_stays_valid_utf8`. |
