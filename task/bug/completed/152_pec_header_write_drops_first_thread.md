# BUG-152: `pec_header_write` unconditionally drops the caller's first added thread from the written color table

- **Severity:** High (silently corrupts the thread palette of every PEC file this crate
  writes whenever the design has at least one thread -- not a crash, but the first thread is
  never encoded and every other thread's position in the roundtripped file shifts down by one)
- **state:** Completed
- **Affects:** Every `pec::write`/`pec::content_write` call on an `EmbroideryFile` with one or
  more threads added (in practice, both raw PEC output and PES v6 output, since `pes::write`'s
  v6 path embeds a PEC section written by this same function)
- **Component:** `module/helper/embroidery_tools` (`src/format/pec/writer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** BUG-151 (same review batch, same crate, opposite direction -- BUG-151 is a
  *reader*-side merge defect, this is a *writer*-side slicing defect; BUG-151's own regression
  test had to compensate for this bug by adding an extra throwaway thread, since both defects
  existed simultaneously when BUG-151 was fixed -- that compensation was removed as part of
  this bug's own fix, see Prevention). BUG-150 (same review batch, same crate, independent code
  path -- `duplicate_color_interpolate_as_stop`'s OOB guard vs. this bug's slicing logic).

## Symptom

```rust
use embroidery_tools::embroidery_file::EmbroideryFile;
use embroidery_tools::format::pec;

let mut emb = EmbroideryFile::new();
emb.stitch( 0, 0 );
emb.color_change( 0, 0 );
emb.stitch( 1, 1 );
emb.end();

let default_palette = pec::pec_threads();
emb.thread_add( default_palette[ 1 ].clone() );
emb.thread_add( default_palette[ 2 ].clone() );

let mut memory = vec![ 0_u8; 2048 ];
pec::write( &mut emb, &mut Cursor::new( &mut memory ) ).unwrap();
let result = pec::memory_read( &memory ).unwrap();

result.threads()
// Wrong (pre-fix):   [ default_palette[ 2 ] ]                       -- only 1 thread, the
//                                                                       first is gone and the
//                                                                       second shifted to index 0
// Correct (post-fix): [ default_palette[ 1 ], default_palette[ 2 ] ] -- both threads, in order
```

## Impact

**Who is affected:** Every caller of `pec::write`/`pec::content_write` (directly, or via
`pes::write`'s v6 path, which embeds a PEC section written by the same `pec_header_write`)
whose `EmbroideryFile` has at least one thread added -- i.e. essentially every real-world
write of a non-empty design.

**What breaks:** `pec_header_write` built its color table from `emb.threads()[ 1.. ]`,
unconditionally skipping the first entry. The caller's first-added thread was never written to
the color table at all, and every subsequent thread's written position shifted down by one
relative to the caller's own ordering. `current_thread_count` (used both for the `add_value`
byte and the 463-byte padding loop) was computed from this already-truncated slice, so the
truncation was internally consistent -- nothing else in the format detected a mismatch, making
this a silent, self-consistent corruption rather than a decodable error.

**Magnitude:** Silent data loss on every non-trivial write. A single-thread design (the most
common case for a simple test or minimal file) loses its *only* thread and is written with an
empty color table (`current_thread_count == 0`), taking `pec_header_write`'s alternate
"no colors" branch instead of encoding the one color the caller actually specified.

**Entity Scope:** None -- a code-level defect, not an operational-entity concern.

## How Discovered

Surfaced by a background review pass over the `embroidery_tools` crate (task #88, one of five
parallel crate reviews this session), independently of BUG-150 and BUG-151 found in the same
pass. Initially deferred as task #101 pending BUG-150/151 closure (both touched the same
reader-side code paths this bug interacts with); confirmed still real -- not intentional
sentinel-avoidance behavior -- by reading `pec_threads()`'s own source directly (its index 0 is
documented `// This one is for indicating invalid value`, a *default-palette* sentinel, an
unrelated concept from the caller's own first thread) and cross-checking against
`read_sample_threads_resolve_from_default_palette`, which shows the reader side already treats
palette index 0 as an ordinary, meaningful thread.

## Minimum Reproducible Example

```bash
cd module/helper/embroidery_tools && cargo test --test pec_test encoding_roundtrip_preserves_first_added_thread 2>&1 | tail -10
```

**Expected** (post-fix):
```
test encoding_roundtrip_preserves_first_added_thread ... ok
```

**Actual** (pre-fix -- confirmed via in-place revert-test-restore against the real unfixed
code):
```
assertion `left == right` failed
  left: 1
 right: 2
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/embroidery_tools && cargo test --test pec_test encoding_roundtrip_preserves_first_added_thread
# ok = fixed; "left: 1, right: 2" = bug present (first thread dropped)
```

**Known MRE limitation (check 205):** the reproducer deliberately uses `default_palette[ 1 ]`
and `default_palette[ 2 ]` rather than `default_palette[ 0 ]` for its two threads. This is not
a workaround for this bug -- it sidesteps a separate, independent `nearest_color_find` tie-break
property (`thread.rs`'s `dist <= current_distance` resolves color ties to the *last* matching
palette index; `default_palette[ 0 ]`'s color (0,0,0) ties with `default_palette[ 20 ]`, also
pure black, so a thread added by *value* equal to index 0 round-trips back as index 20's
struct, not index 0's -- an orthogonal palette-matching behavior, not something this writer bug
or its reproducer is about). See Prevention for the pre-existing test this also required fixing.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `pec_header_write` slices `emb.threads()[ 1.. ]` before building the color table, unconditionally dropping the caller's first thread. | ✅ Root Cause | Direct code reading confirmed the slice; the MRE's captured failure (`left: 1, right: 2`, exactly one thread missing) matches precisely. | E1, E2 |
| H2 | The `[ 1.. ]` slice is intentional: index 0 of `emb.threads()` is meant to mirror `pec_threads()[ 0 ]`'s "invalid value" sentinel status and should never be written. | ❌ Rejected | `pec_threads()[ 0 ]`'s sentinel status is a property of the *default palette array* (a fixed lookup table), not of the *caller's own thread list* -- `emb.threads()[ 0 ]` is simply whatever thread the caller added first, carrying no such status. `read_sample_threads_resolve_from_default_palette` confirms the reader side already treats palette index 0 as an ordinary thread when resolving colors, refuting any reader/writer symmetry argument for skipping it. | E3, E4 |
| H3 | The bug is in `unique_palette_build`/`nearest_color_find` (the color-matching logic), not in `pec_header_write`'s own slicing. | ❌ Falsified | `unique_palette_build`'s own contract (confirmed by direct source reading) returns exactly one index per input thread -- `color_indices.len() == threadlist.len()`. The truncation happens strictly before that call, at the `[ 1.. ]` slice site; `unique_palette_build` itself faithfully processes whatever slice it's given. | E1, E5 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `format/pec/writer.rs`, `pec_header_write` (pre-fix) | `unique_palette_build( &thread_palette, &emb.threads()[ 1.. ] )` -- the slice is visible directly in the call site, with no conditional or comment explaining the skip. | H1 ✅, H3 ❌ |
| E2 | `-0012_longrun.log` (in-place revert-test-restore run against the real unfixed code) | Captured exact pre-fix failure: `assertion left == right failed ... left: 1, right: 2` -- of the 2 added threads, only 1 (the second) survived the roundtrip. | H1 ✅ |
| E3 | `format/pec.rs`, `pec_threads()` (lines 32-40) | Index 0 of the *default palette* carries an explicit `// This one is for indicating invalid value` comment -- a documented property of that fixed 65-entry array, not of any caller-supplied thread list. | H2 ❌ |
| E4 | `tests/pec_test.rs`, `read_sample_threads_resolve_from_default_palette` (pre-existing, unedited) | Reads a real reference fixture and confirms the reader side resolves palette index 0 to an ordinary thread when appropriate -- no special-casing on the read side either. | H2 ❌ |
| E5 | `src/thread.rs`, `unique_palette_build` (lines 61-87, unedited) | Two-loop structure: the second loop iterates the full, non-deduplicated `threadlist` and pushes exactly one `nearest_color_find` result per input -- confirms `color_indices.len() == threadlist.len()` always, with no internal truncation. | H3 ❌ |

## Root Cause

```
pec_header_write()   (pre-fix)
  let thread_palette = pec_threads();
  let color_indices = unique_palette_build( &thread_palette, &emb.threads()[ 1.. ] );
  //                                                          ^^^^^^^^^^^^^^^^^^^^^
  //                                                          drops emb.threads()[0]
  //                                                          unconditionally
  let current_thread_count = color_indices.len();
```

The slice `[ 1.. ]` was applied to the *caller's own thread list* on every write, with no
condition distinguishing "this happens to be the palette's sentinel value" from "this is simply
the first thread the caller added." Since `unique_palette_build` returns one index per input
thread, the truncation propagated directly into `current_thread_count` and the written color
table -- every write silently lost exactly one thread.

## Why Not Caught

The one existing roundtrip test (`encoding_roundtrip_preserves_stitches_and_threads`) added
`pec_threads()[ 0 ]` (the default palette's own sentinel-value entry) as its first thread. Its
disappearance after the roundtrip was indistinguishable from "the sentinel value specifically
doesn't survive" -- a plausible-sounding property given the sentinel's documented status -- so a
rationalizing comment recorded that reading instead of catching the actual defect (an
unconditional slice that would drop *any* first thread, sentinel or not).

## Fix Location

`module/helper/embroidery_tools/src/format/pec/writer.rs`, `pec_header_write`:

```rust
// before
let color_indices = unique_palette_build( &thread_palette, &emb.threads()[ 1.. ] );

// after
let color_indices = unique_palette_build( &thread_palette, emb.threads() );
```

Removed the `[ 1.. ]` slice -- the full thread list is now passed through unconditionally,
mirroring `unique_palette_build`'s own one-index-per-input contract. No signature change.

## Prevention

Added `encoding_roundtrip_preserves_first_added_thread` (`bug_reproducer(BUG-152)`) to
`tests/pec_test.rs`: adds two ordinary, non-sentinel default-palette threads (indices 1 and 2,
deliberately avoiding index 0's color-tie ambiguity, see MRE Limitation), round-trips through
`pec::write`/`pec::memory_read`, and asserts both survive in order.

Also corrected two pre-existing tests whose behavior depended on this bug, now that it's fixed:
- `encoding_roundtrip_preserves_stitches_and_threads`: previously added `threads[ 0 ]` (the
  palette's sentinel value) as its first thread and asserted a single surviving thread landed
  at position 0. Post-fix, both added threads survive; the test now adds `threads[ 1 ]` (not
  `threads[ 0 ]`, to avoid the unrelated `nearest_color_find` color-tie property described in
  the MRE Limitation) and `threads[ 2 ]`, asserting both come back in order.
- `content_read_with_short_chart_assigns_one_thread_per_color_byte` (BUG-151's own reproducer):
  previously added 3 threads specifically to compensate for this bug dropping one of them
  before the color table was ever written. Now adds exactly 2, matching the design's 2
  color-change-delimited stitch runs directly.

## Pitfall

A documented sentinel *value* inside a fixed default palette (`pec_threads()[ 0 ]`, "Unknown",
a lookup-table entry) must never be confused with a structural *position* in a caller-supplied,
arbitrary-content list (`emb.threads()[ 0 ]`, whatever the caller added first). The two only
appeared related here because a pre-existing test happened to use the sentinel's own value as
its first thread, making an unconditional slice look like deliberate sentinel-skipping instead
of the unrelated-to-content off-by-one it actually was. This is the same general shape as
BUG-150's `Option::get()` vacuous-equality guard and BUG-151's cache-gated side effect -- a
plausible-looking piece of logic silently mishandling a case its author didn't consciously
intend to special-case -- but the mechanism here (a positional slice conflating two distinct
"index 0" concepts) is distinct from either.

## Generalized Version

**Broken assumption:** "a fixed offset/slice applied to a caller-supplied collection is safe
because index 0 of some *other*, related fixed table has special meaning." False here -- the
special meaning belonged to `pec_threads()[ 0 ]` (a specific value within a 65-entry constant
array), not to position 0 of whatever list a caller happens to pass in.

**Confirmed general rule:** when a fixed lookup table has a documented sentinel *value*,
never let that fact justify slicing or special-casing a *position* in an unrelated,
caller-controlled collection -- a value-level property of one data structure does not transfer
to a position-level property of another, even when a coincidental test fixture makes it look
that way.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Surfaced by a background review pass over `embroidery_tools` (task #88); deferred as task #101 pending BUG-150/151 closure, then independently re-confirmed real (not intentional) by reading `pec_threads()`'s own source before filing. |
| 2026-08-16 | fixed | Removed the `[ 1.. ]` slice from `pec_header_write`, passing `emb.threads()` through unconditionally. |
| 2026-08-16 | verified | Added `encoding_roundtrip_preserves_first_added_thread` (written test-first against the unfixed code); confirmed it fails pre-fix with the exact predicted `left: 1, right: 2` and passes post-fix. Corrected 2 pre-existing tests whose setup/assertions depended on this bug's behavior. Full crate suite (13 tests, 0 doctests) + `cargo clippy --all-targets --all-features -- -D warnings` clean. |
| 2026-08-16 | completed | Independently re-verified in a later session: fix and both test corrections still present exactly as documented; full `pec_test.rs` suite re-run clean (5/5). Resolved a same-ID duplicate: a separate retroactive `task/bug/completed/152_*.md` had been filed independently (mistakenly inferring from `bug/readme.md`'s already-updated Closed Bugs table that no BUG-152 file existed yet, without checking `verified/` directly) -- that duplicate described the identical fix with a less-complete narrative and has been deleted in favor of this file, the contemporaneous original. Also corrected BUG-151's own file, whose "+3 threads to compensate" cross-reference text (referenced here in Prevention as already removed) had not actually been updated -- now fixed. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the test against unfixed code and captured the exact pre-fix failure; adversarial pass specifically re-ran the two pre-existing tests this fix touches to check for regressions rather than assuming the writer change was isolated -- found and fixed both (documented in Prevention), including discovering the unrelated `nearest_color_find` tie-break property along the way rather than misattributing that failure to this fix. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Related to BUG-150 (same batch, independent root cause) and BUG-151 (same batch, opposite reader/writer direction, with an explicit compensation dependency now removed) -- both noted explicitly. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct code reading (the unconditional slice) plus a captured real failure matching the predicted mechanism exactly; H2's sentinel-symmetry hypothesis explicitly tested and rejected against `pec_threads()`'s own source and the reader's existing sentinel-handling test. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Read `unique_palette_build`'s full two-loop implementation to confirm the truncation originates at the slice site, not inside the color-matching logic (H3). | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `embroidery_tools` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to one slice expression inside one function body; no signature/field change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing function's "encode every added thread" contract restored. | — |

**Reproduced:** YES -- `encoding_roundtrip_preserves_first_added_thread` was written and run
against the unfixed function first (test-first), producing the exact predicted failure
(`left: 1, right: 2`); applying the fix and re-running returned the test to passing, and the
full crate suite (13 tests, 0 doctests) + `cargo clippy --all-targets --all-features -- -D
warnings` remained clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/src/format/pec/writer.rs` | `pec_header_write`: removed the `[ 1.. ]` slice, passing `emb.threads()` through unconditionally. `Fix(BUG-152)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/tests/pec_test.rs` | Appended `encoding_roundtrip_preserves_first_added_thread` (`bug_reproducer(BUG-152)`, 5-section doc comment). Corrected `encoding_roundtrip_preserves_stitches_and_threads` (uses `threads[1]`/`threads[2]` now, asserts both survive) and `content_read_with_short_chart_assigns_one_thread_per_color_byte` (now adds exactly 2 threads, its BUG-152 compensation removed). |
