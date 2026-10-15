# BUG-341: `as_segment_blocks`'s `Jump` arm never updates the tracked needle position, corrupting the second of two consecutive jump segments

- **Severity:** Medium (no crash; corrupts only the write-only CEmbOne/CSewSeg "informational"
  block, not the authoritative PEC binary section this crate's own reader relies on — but a
  real, currently-manifesting content-correctness defect for any external PES-consuming
  software that does read that block)
- **state:** Verified
- **Affects:** Every PES v6 write (`pes::write(..., PESVersion::V6)`) of a design containing two
  `Jump` command-blocks separated only by non-`Stitch` instructions (`ColorChange`, `Trim`, or
  any other instruction that falls through `as_segment_blocks`'s catch-all `_ => continue`
  arm) with no `Stitch` between them
- **Component:** `module/helper/embroidery_tools` (`src/format/pes/writer.rs`, `as_segment_blocks`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **Fix Task:** [376](../../verifying/376_register_embroidery_tools_jumpsegment_staleneedleposition_fix_closes_bug341.md)

## Symptom

`as_segment_blocks` (`src/format/pes/writer.rs`) builds the CSewSeg block's list of stitch/jump
segments. Its `Instruction::Jump` arm reads a local `stitched_x`/`stitched_y` tracker to compute
a jump segment's *start* point, but — unlike the `Instruction::Stitch` arm, which writes both
variables back after every stitch — never writes them back itself. When a non-moving
instruction (`ColorChange`, `Trim`, etc.) separates two `Jump` command-blocks with no `Stitch`
between them, the second jump's recorded start point is stale: it reads back whatever the
tracker held *before the first jump*, not where the first jump actually ended.

```
# Program: stitch(0,0) stitch(10,10) jump(5,5) color_change(0,0) jump(5,5) stitch(0,0) stitch(1,1) end()
# -> absolute needle path: (0,0) -(stitch)-> (10,10) -(jump #1)-> (15,15) -(no move)-> (15,15) -(jump #2)-> (20,20)

# Expected jump segment #2 (relative to design-bounds adjustment):
(15,15) -> (20,20)

# Actual jump segment #2 (decoded from the written CSewSeg bytes):
(10,10) -> (20,20)   # starts from BEFORE jump #1 fired, not from where jump #1 ended
```

## Impact

**Who is affected:** any external PES-consuming software (embroidery machines, editors such as
the pyembroidery-ecosystem tools this writer was ported from) that reads the CSewSeg block's
jump-segment list for a design containing a jump→(colorchange or trim, no stitch)→jump sequence.
This crate's own reader (`format/pes/reader.rs`) never reads CSewSeg/CEmbOne back (confirmed:
`grep -n "CSewSeg\|CEmbOne" src/format/pes/reader.rs` finds nothing), so no round-trip
(`pes::write` then `pes::read`) consumer inside this crate is affected — the corruption is
externally visible only.

**What breaks:** the second jump segment renders as a spurious, longer diagonal starting from
the needle position recorded *before* the first jump, instead of from where the first jump
actually ended — visually a wrong, extra-long jump stitch path in any tool that renders or
interprets the CSewSeg segment list.

**Magnitude:** 1 function (`as_segment_blocks`), 1 of 4 match arms (`Jump`) missing the tracker
write-back that its sibling `Stitch` arm already performs.

**Entity Scope:** `None` — a code-level generator defect, not entity directory instances.

## How Discovered

A prior investigation pass for this session's bug-hunt built a throwaway probe crate that
constructed the exact repro program below, wrote it to PES v6 via `pes::write`, and decoded the
resulting CSewSeg segment list, observing the second jump segment start at `(10,10)` instead of
the expected `(15,15)`. This report independently re-confirms the defect by direct reading of
the current `as_segment_blocks` source (`Jump` arm at lines 468-474, `Stitch` arm at lines
484-493 — pre-fix line numbers) and by writing a permanent, byte-level reproducer test
(stronger evidence than the original throwaway probe) into this crate's own `tests/pes_test.rs`,
confirmed to fail against the current source (see MRE below).

## Minimum Reproducible Example

**Verify Command** (run from the crate root; ≤3 lines):
```bash
cd module/helper/embroidery_tools
cargo test --test pes_test second_jump_after_colorchange_starts_where_first_jump_ended -- --exact
```
**What:** the second of two consecutive jump segments (separated by a no-op `color_change`, no
intervening `stitch`) must start where the first jump ended, not carry over a stale
pre-first-jump needle position.

**Expected** (fixed): test passes — `test second_jump_after_colorchange_starts_where_first_jump_ended ... ok`.

**Actual** (pre-fix, directly confirmed by running the same test against the current, unfixed
source before applying the fix below):
```
thread 'second_jump_after_colorchange_starts_where_first_jump_ended' panicked at tests/pes_test.rs:...:
second jump segment must start where the first jump ended ((15, 15)), not carry over a stale
pre-first-jump needle position (found (10, 10))
test second_jump_after_colorchange_starts_where_first_jump_ended ... FAILED
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `as_segment_blocks`'s `Instruction::Jump` arm (writer.rs:468-474, pre-fix) never writes back to `stitched_x`/`stitched_y` after computing a jump segment, unlike the `Instruction::Stitch` arm (writer.rs:484-493, pre-fix; now 499-508 post-fix) which updates both on every stitch | ✅ Root Cause | Direct read of both match arms confirms the asymmetry: `Stitch` assigns `stitched_x = stitch.x; stitched_y = stitch.y;` inside its loop; `Jump` only reads the two variables, never assigns them | E1 |
| H2 | `Instruction::ColorChange` (writer.rs:475-483, pre-fix; now 490-498 post-fix) and every other non-`Stitch`/non-`Jump` instruction fall through a catch-all `_ => continue` (writer.rs:494, pre-fix; now 509 post-fix) that also never touches the tracker | ✅ Verified | Direct read: `ColorChange`'s arm body only updates `current_thread`/`color_index`/`color_code` before `continue`; the catch-all arm is a bare `continue` | E1 |
| H3 | The defect requires two separate `Jump` *command-blocks* (not two `jump()` calls in a row, which `as_command_blocks` merges into one block) with a non-`Stitch` instruction between them and no `Stitch` in between | ✅ Verified | `EmbroideryFile::as_command_blocks` (embroidery_file.rs:314-333) splits blocks only where the instruction *type* changes between consecutive stitches — two consecutive `jump()` calls stay in one block (whose own end point is always its own last instruction, never stale); only an intervening different-typed instruction (`ColorChange`/`Trim`/etc.) creates the second, independently-processed `Jump` block that then reads the un-updated tracker | E2 |
| H4 | This crate's own PES reader never reads the CSewSeg/CEmbOne block back, so the corruption is invisible to any write→read roundtrip test in this crate | ✅ Verified | `grep -n "CSewSeg\|CEmbOne" src/format/pes/reader.rs` returns no matches (same fact BUG-235 already established for this same reader) | E3 |
| H5 | No existing PES writer test exercises two `Jump` blocks separated only by a non-`Stitch` instruction with no `Stitch` between them | ✅ Verified | `pes_test.rs`'s `fixture_program()` and every other existing fixture always place a `Stitch` between any two jumps (trim/color_change always precede a jump that is itself followed by a stitch before the next jump), so the tracker is always freshly correct when the only pre-existing tests' jumps read it | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/embroidery_tools/src/format/pes/writer.rs:468-494` (`as_segment_blocks`, pre-fix, direct read) | `Jump` arm (468-474) reads `stitched_x`/`stitched_y` but never assigns them; `Stitch` arm (484-493) assigns both inside its loop; `ColorChange` (475-483) and the catch-all (494) both `continue` without touching either variable | H1 ✅, H2 ✅ |
| E2 | `module/helper/embroidery_tools/src/embroidery_file.rs:314-333` (`as_command_blocks`, direct read) | Splits `self.stitches()` into blocks only when consecutive instructions differ in type — confirms two back-to-back `jump()` calls stay in one block, while an intervening `color_change()`/`trim()` produces a second, separately-processed `Jump` block | H3 ✅ |
| E3 | `module/helper/embroidery_tools/src/format/pes/reader.rs` (`grep -n "CSewSeg\|CEmbOne"`, direct read) | No output — confirms the PES reader never parses either section back | H4 ✅ |
| E4 | `module/helper/embroidery_tools/tests/pes_test.rs:11-32` (`fixture_program`, direct read) | Every jump in the shared fixture is immediately followed by a `stitch` before the next jump occurs, so the stale-tracker path was never exercised | H5 ✅ |
| E5 | Terminal output (this report, MRE section) | Running the new reproducer test against the current (unfixed) source panics with the exact stale value `(10, 10)` predicted by H1-H3's mechanism, instead of the expected `(15, 15)` | H1 ✅ |

## Root Cause

Pre-fix line numbers throughout the diagram below — it describes the defective mechanism as it
existed before the fix documented in Fix Location.

```
Block A (Stitch, [0,10])   -> stitched_x/y updated to (10,10) after each stitch          (Stitch arm, writer.rs:484-493)
Block B (Jump #1)          -> segment = [ (stitched_x,stitched_y)=(10,10), last=(15,15) ]
                               stitched_x/y NOT updated after this                        (Jump arm, writer.rs:468-474)
Block C (ColorChange)      -> continue; stitched_x/y untouched                            (ColorChange arm, writer.rs:475-483)
Block D (Jump #2)          -> segment = [ (stitched_x,stitched_y)=(10,10) <- STALE, last=(20,20) ]
                               correct start would be (15,15), where jump #1 ended
```
The `Jump` arm computes a jump segment's start point by reading the same `stitched_x`/
`stitched_y` tracker the `Stitch` arm maintains, but only the `Stitch` arm ever writes back to
it. Every instruction type other than `Stitch` (`ColorChange`, `Trim`, and the general catch-all)
falls through without touching the tracker either. A jump's own *end* point is always correct
(read directly from `command_block.last()`'s own absolute coordinates, not from the tracker), so
the bug is confined exclusively to a jump segment's *start* point, and only manifests when the
immediately preceding segment was itself a `Jump` (whose end point was never propagated) rather
than a `Stitch` (which always propagates correctly).

## Why Not Caught

Every existing PES writer test (`pes_test.rs`'s shared `fixture_program()`, and every other
fixture in this crate's test suite) places a `Stitch` between any two `Jump` command-blocks — no
existing test exercises two jumps separated only by a non-moving instruction (`ColorChange`,
`Trim`) with zero stitches between them. Additionally, the CSewSeg block this bug corrupts is
write-only/informational: this crate's own `pes::read` never parses it back (confirmed by direct
grep, matching BUG-235's precedent for the same reader), so no write→read roundtrip test — the
main verification strategy used elsewhere in this test file — could ever have caught it either.

## Fix Location

**`module/helper/embroidery_tools/src/format/pes/writer.rs:468-489`** (`as_segment_blocks`,
`Instruction::Jump` arm, post-fix — was lines 468-474 pre-fix; the arm now spans 22 lines instead
of 7 because of the inserted fix comment and the two new assignment lines):

```rust
// Before:
Instruction::Jump =>
{
  block.push( ( stitched_x - adjust_x, stitched_y - adjust_y ) );
  let last_instruction = command_block.last().unwrap();
  block.push( ( last_instruction.x - adjust_x, last_instruction.y - adjust_y ) );
  flag = 1;
},

// After:
Instruction::Jump =>
{
  block.push( ( stitched_x - adjust_x, stitched_y - adjust_y ) );
  let last_instruction = command_block.last().unwrap();
  block.push( ( last_instruction.x - adjust_x, last_instruction.y - adjust_y ) );
  stitched_x = last_instruction.x;
  stitched_y = last_instruction.y;
  flag = 1;
},
```
Source comment (`Fix(BUG-341)`/`Root cause`/`Pitfall`) added immediately above the two new
assignment lines.

**`module/helper/embroidery_tools/tests/pes_test.rs`** (new test appended): writes the repro
program to PES v6, decodes the CSewSeg segment bytes directly (this block is write-only and
never read back by this crate — see Why Not Caught), and asserts the second jump segment's start
point equals the first jump segment's end point.

## Prevention

Detection command for the general pattern (a per-instruction-type match arm in
`as_segment_blocks` that reads a shared position tracker without also being one of the arms that
writes it):
```bash
grep -n "stitched_x\|stitched_y" module/helper/embroidery_tools/src/format/pes/writer.rs
```
This is a starting point for human review, not a precise check — it cannot by itself confirm
every arm that *reads* the tracker also *writes* it; that judgment still requires reading each
match arm. Confirmed by direct execution against the current file: post-fix, the `Jump` arm
gains two new writes, `Stitch` already had them, and `ColorChange`/the catch-all still have
none (correctly, since neither arm ever computes a segment from the tracker).

**Pitfall:** when a shared mutable tracker variable is read by multiple match arms of the same
`match`, only one of which (the "obvious" one) is ever seen being written to, it is easy to
assume every other arm either doesn't need the tracker or correctly maintains it by omission —
here, `Jump` genuinely *needs* a correct tracker value (it's a segment's own start point) but was
never given a write-back, while the truly write-back-free arms (`ColorChange`, `Trim`, the
catch-all) are only safe because they never appear as `ret.push`-producing segments themselves.

## Generalized Version

**Broken assumption:** "a local mutable position tracker read by one match arm to seed a new
segment's start point is kept fresh by whichever *other* arm the codebase's author had in mind
when the tracker was introduced (here, `Stitch`), regardless of how many *other* arms can also
produce segments that consume the same tracker."

**Confirmed general rule:** any match arm that reads a shared "last known position" tracker to
compute output must also be checked for whether it needs to write that tracker back for the
*next* consumer — a tracker's write-back obligation belongs to every arm capable of moving the
"position" forward and being followed by another tracker-consuming arm, not only to the first
arm the tracker was written for.

**Detection invariant:**
```
for every mutable position/state tracker read by more than one match arm in the same function:
  every arm that can legitimately change "where we are" must write the tracker back,
  not only the arm the tracker was originally introduced alongside
```
Single confirmed instance in this workspace (the `stitched_x`/`stitched_y` tracker is local to
`as_segment_blocks`; no other function in this crate maintains an analogous shared tracker across
match arms — confirmed by `grep -rn "let mut stitched_x\|let mut stitched_y"` returning only this
one declaration). Not a duplicate of any prior bug in this repo's `task/bug/` history (dedup
search: `grep -rli "as_segment_blocks\|stitched_x\|stitched_y\|jump segment\|CSewSeg" task/bug/`
found no prior hits against this function; BUG-150/151/152/234/235/314 all target other functions
in this same crate's PEC/PES/EmbroideryFile code, none touching `as_segment_blocks`'s jump-arm
tracker).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Re-confirmed via direct source reading and a new permanent byte-level reproducer test, following up a prior investigation pass's throwaway-probe-crate finding |
| 2026-08-18 | VERIFY Gate | Reproducer test `second_jump_after_colorchange_starts_where_first_jump_ended` (`cd module/helper/embroidery_tools && cargo test --test pes_test second_jump_after_colorchange_starts_where_first_jump_ended -- --exact`) confirmed passing against current source (4/4 clean runs on the fully-built binary; one anomalous FAIL was observed only on the very first invocation during a fresh 34s compile and did not reproduce on any subsequent run, consistent with a build-cache race rather than a source defect). Fix confirmed present in `module/helper/embroidery_tools/src/format/pes/writer.rs`: the `Instruction::Jump` arm's write-back (`stitched_x = last_instruction.x; stitched_y = last_instruction.y;`) is present at lines 486-487, matching this report's Fix Location "After" block. `state:` field found already set to `Verified` at the start of this gate (flipped without a corresponding History entry); this entry backfills that missing record. |
| 2026-08-18 | verified | Independent Tier 2 Dual-Role Self-Check re-run in full per the assigned VERIFY Gate task, confirming and strengthening the prior entry's PASS verdict with empirical proof: fix temporarily reverted in place (`Jump` arm's 2 write-back lines removed) → `cargo nextest run -p embroidery_tools` showed exactly 1 failure (`second_jump_after_colorchange_starts_where_first_jump_ended`, matching the documented Actual block); fix restored → 17/17 passed; `git diff` confirmed the restored source is byte-identical to the pre-existing fix. Fresh direct re-read of current `writer.rs:468-489` and `pes_test.rs:252-316` confirms no drift since. Adversarial pass caught one real defect: Evidence Table's Hypothesis column cited bare H-IDs with no state symbols (checklist 304) — fixed by annotating all 5 rows with their Hypothesis Table state symbols (✅). Backfills the still-missing `## Verification Record` section required by checklist 106. |

## Verification Record

**VERIFY Gate (2026-08-18) — Tier 2 Dual-Role Self-Check, 8 dimensions, verdict: PASS (8/8).**

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Completeness | — | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | — | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Evidence Table Hypothesis column had bare H-IDs, no state symbols (304) | Added ✅ to all 5 rows |
| D4 | Root Cause Quality | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 1 issue | 1 fix |

**Reproduced:** YES — empirical revert/restore proof against `cargo nextest run -p embroidery_tools`: fix temporarily reverted (2 write-back lines removed from the `Jump` arm) → `second_jump_after_colorchange_starts_where_first_jump_ended` FAILED (1 failed, exit 100, matching the documented Actual block exactly); fix restored → 17/17 passed (exit 0); `git diff` confirmed the restore is byte-identical to the pre-existing fix. Re-confirmed 2026-08-18 via fresh direct read of current `writer.rs:468-489` and `pes_test.rs:252-316` (test and fix both present, unchanged since the empirical proof was gathered).

## Refs: src/

- `module/helper/embroidery_tools/src/format/pes/writer.rs` — `as_segment_blocks`'s `Instruction::Jump` arm now writes `stitched_x`/`stitched_y` back after computing a jump segment, mirroring the `Instruction::Stitch` arm's existing behavior

## Refs: tests/

- `module/helper/embroidery_tools/tests/pes_test.rs` — new reproducer: writes two jumps separated by a no-op `color_change` to PES v6, decodes the actual CSewSeg segment bytes, and asserts the second jump segment starts where the first ended
