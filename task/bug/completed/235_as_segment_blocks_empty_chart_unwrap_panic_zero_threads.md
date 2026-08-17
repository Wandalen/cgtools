# BUG-235: `as_segment_blocks` panics with `unwrap()` on `None` for designs with zero threads

- **Severity:** Medium (a legitimate, format-valid input -- a stitch-free-of-threads design,
  e.g. a jump-only pattern or a bare `emb.end()` with no `thread_add` calls -- crashes the
  writer with an unhandled panic instead of degrading gracefully; narrower blast radius than
  BUG-234 since it requires a design with literally zero threads, not merely "many")
- **state:** Completed
- **Affects:** `pes::write( ..., PESVersion::V6 )` for any `EmbroideryFile` whose
  `threads()` is empty AND has no `Stitch`/`SewTo`/`NeedleAt` instruction for
  `color_count_fix` to backfill a thread from. `PESVersion::V1` is unaffected
  (`version1_write` never calls `as_segment_blocks`).
- **Component:** `module/helper/embroidery_tools` (`src/format/pes/writer.rs`,
  `as_segment_blocks`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** BUG-234 (same file, same scouting pass; both are unhandled-panic defects
  in `format/pes/writer.rs` on otherwise format-valid input). Shares this crate's established
  "substitute something reasonable instead of erroring" convention with `thread_or_filler_get`'s
  own `.unwrap_or( &thread::random_thread_get() )` fallback.

## Symptom

```rust
// pre-fix
fn as_segment_blocks( emb : &EmbroideryFile, threads : &[ Thread ], adjust_x : i32, adjust_y : i32 )
-> Vec< SegmentBlock >
{
  let chart : Vec< _ > = threads.iter().map( Some ).collect();
  let mut color_index = 0;
  let mut current_thread = emb.thread_or_filler_get( color_index );
  color_index += 1;
  let mut color_code = thread::nearest_color_find( &current_thread.color, &chart ).unwrap(); // panics when chart is empty
  ...
}
```

`threads` is `emb.threads()` for PES v6 (see `version6_write`'s call site), so `chart` is
empty whenever the design never had a thread added. `nearest_color_find` returns `None` by
its own documented contract when `palette` contains no `Some` entries (including when
`palette` itself is empty) -- and `.unwrap()` here (and at the identical call site inside the
`ColorChange` match arm) turns that documented `None` into an unconditional panic.

## Impact

**Who is affected:** Any caller writing an `EmbroideryFile` with zero threads to PES v6 --
e.g. a jump-only design, or any design built and written before a single `thread_add` call is
made. Not as broad as BUG-234 (which affects any design with 129+ threads), but a plausible
caller mistake (forgetting `thread_add`) or legitimate minimal/placeholder design.

**What breaks:** `pes::write( ..., PESVersion::V6 )` panics instead of returning
`Result::Err` or simply succeeding with a placeholder value -- crashing (or, in a
multi-threaded host, poisoning) whatever thread called it, before the block-processing loop
that would otherwise handle the empty-thread design just fine.

**Magnitude:** 1 function (`as_segment_blocks`), 2 call sites of the same unguarded
`.unwrap()`.

**Entity Scope:** None -- a code-level defect.

## How Discovered

This session's scouting pass of `embroidery_tools`'s `format/pes/writer.rs`, reading
`as_segment_blocks` in full immediately after fixing BUG-234 in the same file, and noting the
initial `color_code` computation runs unconditionally against `chart` (built from
`emb.threads()`) before any check that `threads` is non-empty -- combined with `thread.rs`'s
own doc comment on `nearest_color_find` explicitly documenting the `None`-on-empty-palette
contract that `.unwrap()` here ignores.

## Minimum Reproducible Example

```rust
let mut emb = EmbroideryFile::new();
emb.end();
assert!( emb.threads().is_empty() ); // zero threads, and no Stitch/SewTo/NeedleAt to backfill one
let mut memory = vec![ 0_u8; 4096 ];
let mut writer = Cursor::new( &mut memory );
let result = pes::write( &mut emb, &mut writer, pes::PESVersion::V6 );
// pre-fix: panics with "called `Option::unwrap()` on a `None` value" instead of returning Ok
assert!( result.is_ok() );
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/embroidery_tools && cargo nextest run --all-features -E 'test(version6_write_with_zero_threads_does_not_panic)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `as_segment_blocks`'s two `nearest_color_find( ..., &chart ).unwrap()` call sites panic whenever `chart` (built from `emb.threads()`) is empty, which is reachable via an ordinary, format-valid design that never had a thread added, producing an unconditional panic instead of a graceful fallback. | ✅ Root Cause | Direct read confirms `chart` is built straight from `threads` with no emptiness check, and `nearest_color_find`'s own doc comment documents `None` as the exact, expected return for an empty/all-`None` palette. Confirmed empirically via temporary-revert-and-rerun (exact `unwrap()`-on-`None` panic reproduced at the predicted line). | E1, E2, E3, E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/embroidery_tools/src/format/pes/writer.rs`, `as_segment_blocks` (pre-fix, direct read) | `let chart : Vec< _ > = threads.iter().map( Some ).collect();` followed immediately by `thread::nearest_color_find( &current_thread.color, &chart ).unwrap()` -- no check that `chart`/`threads` is non-empty before the call. | H1 ✅ |
| E2 | `module/helper/embroidery_tools/src/thread.rs`, `nearest_color_find` (direct read) | Doc comment states the function "Returns `None` if palette consists only of `None` values" -- confirming `None` (not a panic) is the function's own documented, expected behavior for an empty/all-`None` `chart`. | H1 ✅ |
| E3 | `module/helper/embroidery_tools/src/embroidery_file.rs`, `color_count_fix` (direct read) | Only backfills a thread when the design contains a `Stitch`/`SewTo`/`NeedleAt` instruction -- a jump-only or bare `emb.end()` design reaches `as_segment_blocks` with `emb.threads()` still empty, confirming the empty-`chart` path is reachable through ordinary, format-valid instruction sequences, not just a contrived construction. | H1 ✅ |
| E4 | Temporary direct-source-edit revert-and-rerun (this fix) | Reverting both `.unwrap_or( 0 )` sites to `.unwrap()` and re-running the new zero-thread test reproduced `thread '...' panicked at .../writer.rs:456:86: called \`Option::unwrap()\` on a \`None\` value` exactly, confirming the failure mode and its exact source location. | H1 ✅ |

## Root Cause

`as_segment_blocks` builds `chart` directly from its `threads` parameter (== `emb.threads()`
for PES v6's only caller, `version6_write`), then immediately looks up the current thread's
nearest palette index via `nearest_color_find( &current_thread.color, &chart ).unwrap()` --
once before its main per-block loop, and again on every `ColorChange` instruction inside that
loop. `nearest_color_find` is documented to return `None` when `chart` has no `Some` entries
(including the empty case), which happens whenever the design has zero threads. A design can
legitimately have zero threads and still reach this function: `color_count_fix` (called
earlier in the write pipeline) only backfills a thread when the design contains a
`Stitch`/`SewTo`/`NeedleAt` instruction, so a jump-only design or a bare `emb.end()` reaches
`as_segment_blocks` with `emb.threads()` still empty, and the unconditional `.unwrap()`
converts the documented `None` into an unhandled panic.

## Why Not Caught

Every existing PES v6 test (`write_v6_matches_reference_fixture`,
`v6_roundtrip_preserves_metadata_and_threads`, and BUG-234's own new test) adds at least one
thread via `thread_add` before writing -- none exercised a design with zero threads.

## Fix Location

`module/helper/embroidery_tools/src/format/pes/writer.rs`: both `.unwrap()` call sites in
`as_segment_blocks` changed to `.unwrap_or( 0 )`. This is safe specifically because the PES
v6 CEmbOne/CSewSeg block this data feeds into is write-only/informational for external PES
consumers -- this codebase's own `pes::read` seeks directly to `pec_block_position`, bypassing
this block entirely (confirmed by direct read of `pes/reader.rs`) -- so there is no
"meaningful index into the design's own (empty) thread palette" to preserve; falling back to
`0` mirrors `thread_or_filler_get`'s own existing "substitute something reasonable instead of
erroring" convention.

## Prevention

`tests/pes_test.rs::version6_write_with_zero_threads_does_not_panic` writes a bare
`emb.end()` design (zero threads, confirmed via `emb.threads().is_empty()` in the test body
itself) to PES v6 and asserts the call succeeds instead of panicking.

## Pitfall

`emb.stitches().is_empty()` (checked by `pes_block_write` to skip the whole CEmbOne/CSewSeg
block entirely) is a different condition from "zero threads" -- a jump-only or otherwise
stitch-free-but-non-empty instruction sequence still reaches `as_segment_blocks` with however
many threads the design happens to have, which can independently be zero. Don't conflate "no
stitches" with "no threads" when reasoning about which guard covers which empty-input case.

## Generalized Version

**Broken assumption:** "this palette/chart built from the design's own data will always have
at least one entry, because a design without any of that data wouldn't reach this code path."

**Confirmed general rule:** When a value passed into a function is optional/possibly-empty at
its ultimate source (here: `emb.threads()`, which is empty for a valid design that never
called `thread_add`, or wasn't backfilled by `color_count_fix` because it lacks
Stitch/SewTo/NeedleAt), any downstream computation built from it (`chart`) must handle the
empty case explicitly -- especially when the function it's passed to (`nearest_color_find`)
already documents `None` as its answer for exactly this case. An `.unwrap()` immediately
downstream of a documented `Option`-returning contract is the same class of trap as BUG-234's
`wrapping_sub`-into-allocation-size: a documented "this can fail" signal converted into an
unconditional panic instead of being handled at the one point that actually knows what a safe
fallback looks like.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `embroidery_tools` scouting pass (Batch 2), reading `format/pes/writer.rs`'s `as_segment_blocks` in full immediately after closing BUG-234 in the same file, and cross-referencing `nearest_color_find`'s own documented `None`-on-empty-palette contract in `thread.rs`. |
| 2026-08-17 | fixed | Both `nearest_color_find( ..., &chart ).unwrap()` call sites in `as_segment_blocks` changed to `.unwrap_or( 0 )`. |
| 2026-08-17 | verified | `cargo nextest run -p embroidery_tools --all-features`: 15/15 passed, 0 skipped. `cargo test --doc -p embroidery_tools --all-features`: 0 doctests (crate has none). `cargo clippy -p embroidery_tools --all-targets --all-features -- -D warnings`: clean. Fix verified via a temporary direct-source-edit revert-and-rerun (exact `unwrap()`-on-`None` panic reproduced pre-fix at `writer.rs:456:86`, passed cleanly post-fix). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Confirming pass: deterministic MRE, `result.is_ok()` is an exact, non-flaky check. Adversarial pass: checked whether `emb.threads().is_empty()` could be reachable-but-vacuous (i.e. `color_count_fix` might always backfill a thread, making the test's premise false) -- direct read of `color_count_fix` confirms it only fires on `Stitch`/`SewTo`/`NeedleAt`, and the MRE's `emb.end()`-only design has none of those, so the assertion in the test body (`emb.threads().is_empty()`) is a genuine, non-tautological precondition check, not dead code. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly identified `thread_or_filler_get`'s `.unwrap_or( &thread::random_thread_get() )` as the precedent convention this fix now mirrors, and confirmed via direct read of `pes/reader.rs` that the CEmbOne/CSewSeg block is genuinely never read back (justifying the `0` fallback as safe rather than merely convenient). | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct reads of `as_segment_blocks`, `nearest_color_find`'s doc comment, and `color_count_fix`, plus empirical revert-rerun proof matching the predicted panic location and message exactly. | — |
| D5 | Execution Scope | — | 🟢 | Confirming pass: fix confined to the two `.unwrap()` sites in `as_segment_blocks`. Adversarial pass: grepped `format/pes/writer.rs` for other unguarded `nearest_color_find(...).unwrap()` or similar `Option::unwrap()` calls -- none found beyond the two already fixed. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `as_segment_blocks`; its signature (`Vec< SegmentBlock >`, no `Result`) and all callers are unchanged -- both call sites already sat inside a function with no error-return path, so `.unwrap_or( 0 )` (not a new `Result`/`?`) is the correct-shaped fix, requiring no caller update. | — |

**Reproduced:** Confirmed via `cargo nextest` (exact `unwrap()`-on-`None` panic pre-fix, clean
`Ok` post-fix) and temporary direct-source-edit revert-and-rerun. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/src/format/pes/writer.rs` | `as_segment_blocks`'s two `nearest_color_find( ..., &chart ).unwrap()` call sites changed to `.unwrap_or( 0 )` (full `Fix(BUG-235)` comment block on the first site; a shorter cross-reference comment on the second). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/tests/pes_test.rs` | Added `version6_write_with_zero_threads_does_not_panic` (`bug_reproducer(BUG-235)`, 5-section doc comment), placed after BUG-234's test. |

## Refs: docs/

| File | Change |
|------|--------|
| — | None -- the fix eliminates the trap rather than leaving a permanent API characteristic to document, matching this session's established convention for fixed (not by-design) defects. |
