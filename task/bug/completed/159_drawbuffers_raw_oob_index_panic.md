# BUG-159: `drawbuffers` panics with a raw, undocumented index-out-of-bounds message instead of an attributable one

- **Severity:** Medium (panic is already documented in the function's own `# Panics` contract --
  this is not a silent-corruption or undocumented-crash bug -- but the actual panic message
  produced does not match what the guard that's supposed to catch this condition claims to
  check, making the real failure harder to attribute at the call site)
- **state:** Completed
- **Affects:** `drawbuffers` -- any caller passing an attachment index `>= 16`
  (`MAX_COLOR_ATTACHMENTS`)
- **Component:** `module/min/minwebgl` (`src/drawbuffers.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Co-located in the same investigation batch as BUG-160/BUG-161 (both in
  `texture/d2.rs`'s `sprite_upload`), but shares no root cause with either -- filed and fixed
  separately.

## Symptom

```rust
// buffers is a fixed [u32; 16] array; attachment is a raw caller-supplied index
let index = *attachment as usize; // pre-fix: used directly, no bounds check
buffers[ index ] = attachment; // index 16 -> "index out of bounds: the len is 16 but the index is 16"
```

## Impact

**Who is affected:** Any caller of `drawbuffers(&gl, attachments)` that passes an attachment
index `>= 16`. Every current call site in this repo passes only literal indices 0-3, so this is
not reachable today, but the function takes an arbitrary `&[u32]` slice with no compile-time
bound.

**What breaks:** The process panics -- already an accepted, documented outcome per the
function's own `# Panics` doc comment ("Panics if an attachment index is `>= MAX_COLOR_
ATTACHMENTS`"). What's actually broken is that the pre-fix code had no code path that produces
that documented panic on that documented condition: the only guard present
(`checked_add(GL::COLOR_ATTACHMENT0)` against `u32::MAX`) only fires near `u32::MAX -
COLOR_ATTACHMENT0`, far above 16. An ordinary out-of-range index like `16` instead panics via a
raw Rust slice-index panic with no attribution to `drawbuffers` or its actual precondition.

**Magnitude:** Low in practice (no real call site reaches this today) but the panic message a
future caller would actually see is misleading relative to the function's own documented
contract.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Independent re-derivation: dispatched a background Explore agent to read 100% of `minwebgl`'s
`src/` (36 files) from scratch, without referencing any earlier candidate list. It flagged
`drawbuffers`'s index handling as a candidate; independently re-verified by reading
`src/drawbuffers.rs` directly and hand-tracing the `checked_add` guard's actual trigger
threshold against the array's real bound.

## Minimum Reproducible Example

```bash
cd module/min/minwebgl && cargo test -p minwebgl --test tests drawbuffers_test::validate_color_attachment_index_rejects_out_of_range_values 2>&1 | tail -6
```

**Expected** (post-fix):
```
test drawbuffers_test::validate_color_attachment_index_rejects_out_of_range_values ... ok
```

**Actual** (pre-fix -- confirmed via in-place revert-test-restore against the extracted guard):
```
thread 'validate_color_attachment_index_rejects_out_of_range_values' panicked at module/min/minwebgl/tests/drawbuffers_test.rs:54:5:
index 16 must be rejected with IdOutOfRange, got Ok(16)
Summary [ 0.008s] 1 test run: 0 passed, 1 failed, 0 skipped
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/min/minwebgl && cargo test -p minwebgl --test tests drawbuffers_test::
# 2 "ok" = fixed; IdOutOfRange assertion failure = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `drawbuffers` has no real bounds check on the array index before indexing `buffers[index]`, so an out-of-range index panics via a raw slice-index panic rather than an attributable message. | ✅ Root Cause | Read `src/drawbuffers.rs` directly: the only guard present is `attachment.checked_add(GL::COLOR_ATTACHMENT0)`, which bounds the *sum* against `u32::MAX`, never the index against `MAX_COLOR_ATTACHMENTS` (16). | E1, E2 |
| H2 | The existing `checked_add` guard already covers the `index >= 16` case because `COLOR_ATTACHMENT0` is a large constant. | ❌ Falsified | `COLOR_ATTACHMENT0 = 0x8CE0` (36064); `checked_add` only overflows past `u32::MAX - 36064`, many orders of magnitude above 16 -- an index of exactly 16 passes `checked_add` fine and reaches the raw array index unguarded. | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/drawbuffers.rs` (pre-fix, unedited) | `buffers[*attachment as usize] = attachment.checked_add(GL::COLOR_ATTACHMENT0).unwrap_or_else(...)` -- the checked-add guards the sum, the raw array index is never separately validated. | H1 ✅, H2 ❌ |
| E2 | `tests/drawbuffers_test.rs::validate_color_attachment_index_rejects_out_of_range_values` (real run) | Reverting `color_attachment_index_validate` to unconditional `Ok(index)` and re-running: real failure `got Ok(16)` where `Err(IdOutOfRange)` was expected -- confirms the pre-fix path had no rejection at index 16. | H1 ✅ |

## Root Cause

```rust
// before (drawbuffers, inline, no bounds check on the array index)
let mut buffers = [ gl::NONE; MAX_COLOR_ATTACHMENTS ];
for attachment in attachments
{
  let attachment = attachment.checked_add( gl::COLOR_ATTACHMENT0 ).unwrap_or_else( || panic!( ... ) );
  buffers[ *attachment as usize ] = attachment; // panics via raw slice index if attachment >= 16
}
```

The only guard present (`checked_add` against `u32::MAX`) validates the wrong quantity -- the
attachment id after adding `COLOR_ATTACHMENT0`, not the array index before the add.

## Why Not Caught

`drawbuffers` takes `&GL` and has no pure-logic twin to unit-test, and every current call site in
this repo passes only literal indices 0-3, well within range -- the out-of-range path was never
exercised.

## Fix Location

`module/min/minwebgl/src/drawbuffers.rs`.

```rust
// after: bounds check extracted into a standalone, testable, Result-returning function
pub fn color_attachment_index_validate( index : usize ) -> Result< usize, WebglError >
{
  if index < MAX_COLOR_ATTACHMENTS
  {
    Ok( index )
  }
  else
  {
    Err( WebglError::IdOutOfRange( format!( "Invalid color attachment index {index}: must be < {MAX_COLOR_ATTACHMENTS}" ) ) )
  }
}

// drawbuffers calls it via .expect(...), keeping its own already-documented "Panics if..."
// contract but now panicking on the actual out-of-range condition with an attributable message
let index = color_attachment_index_validate( *attachment as usize ).expect( "Invalid color attachment" );
```

`drawbuffers` itself is `&GL`-bound and still panics on out-of-range input (unchanged, already
documented) -- the fix makes the panic message attributable to the real precondition instead of a
raw slice-index panic, and makes the bounds check itself independently unit-testable.

## Prevention

Added `tests/drawbuffers_test.rs` (new file, `bug_reproducer(BUG-159)`): a happy-path test
(indices 0..16 all accepted) plus a regression test asserting indices `[16, 17, 100]` all return
`Err(WebglError::IdOutOfRange(_))`.

## Pitfall

`MAX_COLOR_ATTACHMENTS` bounds the ARRAY INDEX, not the attachment id after adding
`COLOR_ATTACHMENT0` -- a guard placed on the wrong quantity (the sum, checked only for `u32`
overflow) can look like it's validating the right thing while actually leaving the real bound
(16) completely unchecked.

## Generalized Version

**Broken assumption:** "a `checked_add` guard on a derived value also protects the value it was
derived from." False when the derived value's valid range (near `u32::MAX`) is many orders of
magnitude wider than the source value's real constraint (an array bound of 16) -- the derived
guard's trigger threshold is so far from the source's real bound that it never fires for
ordinary out-of-range input.

**Confirmed general rule:** when a raw index feeds both an array access and a derived
computation, validate the index itself against its real bound (the array length) directly --
never rely on a guard downstream of a transformation (add, multiply, cast) whose own valid range
doesn't match the original value's actual constraint.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Flagged by an independent Explore-agent re-derivation of minwebgl's `src/` (36 files, task #93); independently re-verified by reading `src/drawbuffers.rs` directly and hand-tracing the `checked_add` guard's real trigger threshold. |
| 2026-08-16 | fixed | Extracted the bounds check into `color_attachment_index_validate`, returning `Result<usize, WebglError>`; `drawbuffers` calls it via `.expect(...)`, preserving its documented panic contract with an attributable message. |
| 2026-08-16 | verified | Added `tests/drawbuffers_test.rs` (2 tests) via in-place revert-test-restore against the real guard: captured the real pre-fix failure (`got Ok(16)`), restored, confirmed passing. Scoped crate suite (13 tests) + `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings` clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the test against the actual extracted function; adversarial pass performed a real in-place revert-test-restore (unconditional `Ok(index)`), capturing the actual `got Ok(16)` failure before restoring. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-160/BUG-161 (same investigation batch, different file, different root cause) -- no cross-dependency. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct source reading plus a concrete numeric trace of why `checked_add`'s threshold (near `u32::MAX`) never overlaps the real bound (16). | — |
| D5 | Execution Scope | 🟢 | 🟢 | Only `drawbuffers.rs`'s bounds-check logic touched; `drawbuffers`'s own signature/documented panic contract unchanged. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `minwebgl` src + test + bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is a function extraction with identical logic, no call-site signature change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | New public fn (`color_attachment_index_validate`) has one responsibility (index validation), re-exported via the same `mod_interface!` block as `drawbuffers`. | — |

**Reproduced:** YES -- `validate_color_attachment_index_rejects_out_of_range_values` was confirmed
to fail with the exact predicted `Ok(16)` (instead of `Err(IdOutOfRange)`) when
`color_attachment_index_validate` was temporarily reverted to unconditional `Ok(index)`;
restoring the guard returns the test to passing. Scoped crate suite (13 tests) + `cargo clippy -p
minwebgl --all-targets --all-features -- -D warnings` clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/min/minwebgl/src/drawbuffers.rs` | Extracted `color_attachment_index_validate`, returning `Result<usize, WebglError>`; `drawbuffers` now calls it via `.expect(...)` (full `Fix(BUG-159)` root cause/pitfall comment). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/minwebgl/tests/drawbuffers_test.rs` | New file: 2 tests (happy path + `bug_reproducer(BUG-159)`). |
| `module/min/minwebgl/tests/readme.md` | Added Responsibility Table row for `drawbuffers_test.rs`. |
