# BUG-314: PEC reader's `stitch_block_len - 5` underflows for untrusted file data with a length under 5, panicking in debug and corrupting the read position in release

- **Severity:** High (untrusted external file data -- any `.pec` or `.pes` file, not just
  ones this crate wrote itself -- can trigger a debug-build panic; release-build behavior
  silently corrupts a subsequent seek position instead of erroring cleanly)
- **state:** Verified
- **Affects:** any consumer parsing a PEC file (directly) or a PES file (which embeds a PEC
  content block) whose `stitch_block_len` field is less than 5
- **Component:** `module/helper/embroidery_tools`
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/ (self)
- **verification_date:** 2026-08-18
- **Fix Task:** [365](../../verifying/365_pec_stitch_block_len_underflow_fix_registration.md) (renumbered from 356→360→365; two TOCTOU races with a concurrent actor's own BUG-3xx promotion batch, 2026-08-18)

## Symptom

`content_read` reads `stitch_block_len` as an untrusted 24-bit little-endian value directly
from file data (`reader.read_u24::<LE>()?`), then computes
`stitch_block_len - 5 + reader.stream_position()?` using a raw `-` operator. For any
`stitch_block_len` value less than 5, this subtraction underflows: it panics in a debug
build ("attempt to subtract with overflow") and silently wraps to a value near `u64::MAX`
in a release build (Rust's default release profile disables overflow checks), which then
becomes the target of a real `reader.seek( SeekFrom::Start( stitch_block_end ) )` call a few
statements later (after an intervening seek and a `pec_instructions_read` call), at what is
now post-fix line 136.

## Impact

**Who is affected:** anyone parsing a `.pec` file via `pec::file_read`/`memory_read`/`read`,
or a `.pes` file via the equivalent `pes::*` entry points -- PES files embed a PEC content
block that is parsed by this same `content_read` function (confirmed: `pes/reader.rs` calls
`pec::content_read` at 2 separate call sites). Any corrupted, truncated, or maliciously
crafted file with a `stitch_block_len` under 5 reaches this code path.

**What breaks:** in a debug build, the process panics on a value that is entirely
attacker/corruption-controlled file content -- a denial-of-service vector for any
long-running process that parses untrusted embroidery files (e.g. a file-upload service).
In a release build, no panic occurs, but `stitch_block_end` silently becomes a value near
`u64::MAX`, and the subsequent `seek( SeekFrom::Start( stitch_block_end ) )` either fails
with an IO error (for most file/cursor backends, seeking far past the end is either an
error or succeeds and positions past EOF) or leaves the reader in a nonsensical position for
all subsequent parsing -- either way, a malformed file produces confusing failure modes
instead of a clear, catchable decode error.

**Entity Scope:** `None` -- source-level input-validation defect, not entity directory
instances.

## How Discovered

Found during this session's workspace-wide bug-hunt pass, `module/helper` review stage.
Reading `content_read`'s parsing of length-prefixed/offset fields for arithmetic on values
sourced directly from untrusted file bytes (the same category of defect as the
already-fixed, unrelated BUG-234 in this crate's writer path) surfaced `stitch_block_len - 5`
as the one remaining unchecked raw subtraction on such a value in the reader path.

## Minimum Reproducible Example

**Verify Command**:
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/embroidery_tools
cargo nextest run -E 'test(content_read_rejects_stitch_block_len_below_5_instead_of_underflowing)' --all-features
```
**Expected** (fixed): 1 passed / 0 failed -- `content_read` returns
`Err( EmbroideryError::DecodingError( _ ) )`.

**Actual** (the raw pre-fix expression's real behavior): the test builds a minimal, otherwise
well-formed PEC content buffer (`build_pec_content_with_stitch_block_len`) with
`stitch_block_len = 0` placed at its real on-disk offset (byte 514, derived from the exact
sequence of seeks/reads `content_read` performs before reaching this field, with
`color_changes` set to `0` so `count_colors = 1` and the post-color-bytes seek distance is
the full, unreduced `0x1D0`), then calls `content_read` directly. Pre-fix, `0u64 - 5` on this
exact value panics in a debug build (`attempt to subtract with overflow`) -- confirmed by
inspection of the removed code and Rust's documented debug-mode overflow-check default; the
test as written could not safely execute the panicking expression directly (a panic would
abort the test process, not produce a comparable value), so the divergence is demonstrated
via the fix itself: the same buffer that panicked pre-fix now cleanly returns
`DecodingError` post-fix.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `stitch_block_len` is read directly from untrusted file bytes with no validation | ✅ Root Cause | `reader.read_u24::<LE>()?` reads a raw 24-bit value from the file; no range check follows before the subtraction | E1 |
| H2 | The raw `- 5` underflows for any value less than 5, both in debug (panic) and release (wrap) | ✅ Verified | Rust's documented default: debug builds panic on integer overflow/underflow, release builds wrap silently -- `u64` subtraction is no exception | E2 |
| H3 | This code path is reachable via both `pec::*` and `pes::*` public entry points | ✅ Verified | `pes/reader.rs` calls `pec::content_read` at 2 call sites (lines 60, 85), in addition to `pec/reader.rs`'s own `read()` | E3 |
| H4 | No existing test constructs a malformed/corrupted `stitch_block_len` | ✅ Verified | `grep -n "stitch_block" tests/pec_test.rs tests/pes_test.rs` (pre-fix) returns nothing | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/embroidery_tools/src/format/pec/reader.rs:121` (pre-fix) | `let stitch_block_len = u64::from( reader.read_u24::< LE >()? );` -- raw file read, no validation | H1 |
| E2 | `module/helper/embroidery_tools/src/format/pec/reader.rs:122` (pre-fix) | `let stitch_block_end = stitch_block_len - 5 + reader.stream_position()?;` -- raw `-`, no `checked_sub`/`saturating_sub` | H1, H2 |
| E3 | `module/helper/embroidery_tools/src/format/pes/reader.rs:60,85` | `pec::content_read( &mut emb, reader, &[] )?;` / `pec::content_read( &mut emb, reader, &threads )?;` -- both call into the same buggy function | H3 |
| E4 | Terminal output (this section, `grep` command) | Empty output pre-fix -- no test constructs a malformed `stitch_block_len` | H4 |

## Root Cause

```
reader.read_u24::<LE>()?  -- UNTRUSTED, attacker/corruption-controlled file data
  |
  +-- stitch_block_len : u64  (range: 0 .. 0xFFFFFF)
        |
        +-- stitch_block_len - 5 + reader.stream_position()?
               |
               +-- if stitch_block_len < 5:
                     debug:   panics ("attempt to subtract with overflow")
                     release: wraps to a value near u64::MAX (silent)
                              -> fed directly into `seek( SeekFrom::Start( .. ) )`
```
The `- 5` accounts for a fixed 5-byte trailer within the stitch block, a reasonable
computation for a WELL-FORMED file -- but nothing validates that the on-disk length is
actually large enough to hold that trailer before subtracting.

## Why Not Caught

No existing test constructed a PEC buffer with a corrupted or malicious `stitch_block_len`
(confirmed via E4) -- all existing tests use either the reference sample file
(`test_files/read_sample.pec`, a valid, well-formed file) or a buffer freshly written by this
crate's own writer, both of which always produce a valid (`>= 5`) length. This is the same
category of gap that produced BUG-234 in this crate's writer path: arithmetic on a
length/count value with no test exercising the out-of-range case.

## Fix Location

`module/helper/embroidery_tools/src/format/pec/reader.rs:121-122` (pre-fix), now
`:121-131` with the guard and comment:

```rust
// Before:
let stitch_block_len = u64::from( reader.read_u24::< LE >()? );
let stitch_block_end = stitch_block_len - 5 + reader.stream_position()?;

// After:
let stitch_block_len = u64::from( reader.read_u24::< LE >()? );
let stitch_block_len = stitch_block_len.checked_sub( 5 )
.ok_or_else( || EmbroideryError::DecodingError( "PEC stitch block length is too small (must be at least 5 bytes)".into() ) )?;
let stitch_block_end = stitch_block_len + reader.stream_position()?;
```
Source comment (`Fix(BUG-314)`/`Root cause`/`Pitfall`) added immediately above.

**`module/helper/embroidery_tools/tests/pec_test.rs:208-272`** (new): a
`build_pec_content_with_stitch_block_len` helper constructs a minimal, otherwise
well-formed PEC content buffer with a caller-chosen `stitch_block_len` at its real on-disk
offset; `content_read_rejects_stitch_block_len_below_5_instead_of_underflowing` calls
`content_read` with `stitch_block_len = 0` and asserts a `DecodingError` comes back instead
of a panic.

## Prevention

Detection command for the general pattern (a raw arithmetic operator applied to a value
just read from untrusted file/network input via `read_u8`/`read_u16`/`read_u24`/`read_u32`,
with no `checked_*`/`saturating_*` guard):
```bash
grep -n "read_u[0-9]*::<.*>()?" module/helper/embroidery_tools/src/format/pec/reader.rs | \
  grep -v "checked_"
```
This is a starting point for human review, not a precise check -- it flags every untrusted
read, most of which are used safely (as array lengths already validated elsewhere, or in
contexts where any value is valid).

**Pitfall:** a length/offset field's own semantic (e.g. "always includes a fixed trailer, so
subtract it") is only true for well-formed files -- any arithmetic derived from an
untrusted length must use `checked_sub`/`checked_add` and return a decode error on failure,
matching this crate's own established convention (BUG-234's fix, in the writer path,
followed the same "explicit bounds check, real `EmbroideryError`" pattern).

## Generalized Version

**Broken assumption:** a length or offset field read from external file data satisfies the
invariants a well-formed file would guarantee (e.g. "large enough to subtract a fixed
trailer from").

Fails whenever:
1. A value is read directly from untrusted file/network bytes, AND
2. That value is used in arithmetic (subtraction, in this case) without a `checked_*` guard,
   AND
3. The result feeds into a further operation (here, a seek position) that has no
   independent validation of its own

**Detection invariant:**
```
for every value read via a `read_u*`/`read_i*` call on untrusted input:
  any arithmetic on that value must use checked_*/saturating_* and return
  a decode error on failure, never a raw operator that can panic or wrap
```
Second confirmed instance of this exact "untrusted length, raw arithmetic, silent
underflow" defect shape in this crate, after BUG-234 (writer-path `128_usize.wrapping_sub(
count )`, using `wrapping_sub` -- which does NOT panic but silently produces a wrong value
used as an allocation size). This one differs in that the pre-fix code used a raw `-`
(which DOES panic in debug, unlike `wrapping_sub`), and is in the reader path against
attacker-controlled input rather than the writer path against internally-computed values --
a higher-severity variant of the same underlying category BUG-234 already established a fix
convention for.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found during this session's workspace-wide bug-hunt task, `module/helper` review stage, following up on the untrusted-length-arithmetic pattern already established by BUG-234 in this same crate |
| 2026-08-18 | fix_applied | `stitch_block_len - 5` -> `stitch_block_len.checked_sub( 5 ).ok_or_else( .. )?`, returning `EmbroideryError::DecodingError` |
| 2026-08-18 | verified | Tier 2 Dual-Role Self-Check, 8/8 dimensions PASS after correcting one imprecise distance claim in Symptom (D4) |

## Refs: src/

- `module/helper/embroidery_tools/src/format/pec/reader.rs` — replaced raw `stitch_block_len - 5` with a `checked_sub` guard returning `DecodingError`

## Refs: tests/

- `module/helper/embroidery_tools/tests/pec_test.rs` — added `content_read_rejects_stitch_block_len_below_5_instead_of_underflowing` (bug_reproducer) and its `build_pec_content_with_stitch_block_len` helper

## Verification Record

**Tier 2 (Dual-Role Self-Check)** — 8-dimension check (Completeness, MRE Validity &
Reproducibility, Cross-Reference Integrity, Root Cause Quality, Execution Scope, Crate
Scope Unity, Crate Locality, Crate Single Responsibility), reused unchanged from the
BUG-311/312/313 checks earlier this pass.

*Single emoji per cell — see `governance/maav.rulebook.md § MAAV : Surface Rule` for the
🟢🔴🟡🟠 legend.*

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Completeness | 🟢 | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | — | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | — | — |
| D4 | Root Cause Quality | 🟡 | 🟢 | Symptom claimed the corrupted value is used "two lines later"; actual distance (traced against the fixed source) is 3 statements / 5 source lines later, at post-fix line 136 | Reworded to a precise, checkable description citing the exact line |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 1 issue | 1 fix |

**Confirming pass notes:** all 12 FI008 sections + 2 Refs present, matching BUG-313's exact
section structure (`grep -n "^##\|^- \*\*"` diff-checked); MRE verify command executed live
via `longrun` and returned exactly the documented `1 passed` result; both FI027
backreferences (`src/format/pec/reader.rs:122`, `tests/pec_test.rs:254`) use the established
bare `task/bug/NNN_....md` path form (no state-directory segment), matching BUG-312/313's
convention; `git show HEAD:...` confirmed E4's "no pre-existing test mentions `stitch_block`"
claim against both `pec_test.rs` and `pes_test.rs`; `git diff --stat` confirmed no changes
outside the 2 expected files; existing `DecodingError` usages in the same file checked for
message collision (none).

**Adversarial pass notes:** attempted to falsify the MRE's "would panic in debug" claim by
searching for an `overflow-checks` profile override in either the workspace or crate
`Cargo.toml` that could invalidate it — none found, claim holds; re-traced every file:line
citation in Evidence Table, Fix Location, and Refs sections against a fresh read of the
current source and `git diff`'s hunk context rather than trusting the draft's original
citations — caught D4's "two lines later" defect (traced the actual statement sequence:
`stitch_block_end` assignment -> intervening `seek(Current)` -> `pec_instructions_read` ->
the real `seek(Start(stitch_block_end))`, which is 3 statements away, not 2); corrected the
range for the new test in Fix Location (initially miscited as `:206-266`, actually
`:208-272` — line 206 is the closing brace of the *prior* existing test, not part of this
addition, and the test itself closes at 272, not 266) before this gate began, per this
session's now-established precision discipline. No independence: this is a single
authoring entity's own two-pass check, not a dispatched second opinion.
