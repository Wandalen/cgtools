# BUG-150: `duplicate_color_interpolate_as_stop` panics via an unguarded `Vec::remove` when there are more color changes than recorded threads

- **Severity:** High (unguarded panic reachable from the public `pec::content_read` entrypoint
  on malformed/unusual real file input; not a silent-wrong-data class defect)
- **state:** Completed
- **Affects:** Any `EmbroideryFile::duplicate_color_interpolate_as_stop()` call -- in practice,
  every `pec::content_read` -- on a file whose stitch-instruction block encodes more
  color-change-delimited stitch runs than its color/thread table declares
- **Component:** `module/helper/embroidery_tools` (`src/embroidery_file.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Same review batch as BUG-151 (`pec_table_process` drops first-seen
  colors) and BUG-152 (`pec_header_write` drops first thread) -- both fixed and completed, both
  in this same crate's PEC format code, but neither shares this bug's root cause or code path
  (thread-palette parsing and header writing vs. this bug's stop/color-change interpolation
  guard).

## Symptom

```rust
use embroidery_tools::embroidery_file::EmbroideryFile;

let mut emb = EmbroideryFile::new();
emb.color_change( 0, 0 );
emb.stitch( 1, 1 );
emb.color_change( 0, 0 );
emb.stitch( 1, 1 );

emb.duplicate_color_interpolate_as_stop();
// Pre-fix: panics -- "removal index (is 1) should be < len (is 0)"
// Post-fix: returns normally -- 0 threads are recorded, so no duplicate-color merge is possible
//           and the function correctly falls through to "nothing to merge" instead of crashing.
```

## Impact

**Who is affected:** Any caller of `duplicate_color_interpolate_as_stop` -- in practice exactly
one call site, `format/pec/reader.rs:133`, invoked unconditionally at the end of every
`pec::content_read`.

**What breaks:** The function's merge guard compares `self.threads().get( thread_index )`
against `self.threads().get( thread_index - 1 )` with no bounds check. The PEC format's thread
count and its stitch-run count come from two independently-parsed sections -- thread count from
the header's `color_bytes`/`pec_colors_map` (a fixed-size array read from a single header byte),
run count from `pec_instructions_read`'s separate stitch-instruction stream. Nothing enforces
that these two counts agree. When a file's stitch stream encodes more color-change-delimited
runs than its header declares threads, `thread_index` advances past `self.threads().len()`, both
`.get()` calls return `None`, `None == None` evaluates `true` in Rust, the guard is satisfied,
and `self.threads.remove( thread_index )` panics with an out-of-bounds removal instead of
gracefully skipping the merge.

**Magnitude:** Hard panic, not silently-wrong output -- reachable directly from the public
`pec::content_read` entrypoint on realistic malformed or unusual input, with no validation
anywhere upstream that would prevent the thread-count/run-count divergence.

**Entity Scope:** None -- a code-level defect, not an operational-entity concern.

## How Discovered

Surfaced by a background review pass over the `embroidery_tools` crate (task #88, one of five
parallel crate reviews this session). Independently reproduced and confirmed via the MRE below
before filing -- the review agent's finding was not taken on trust; the exact panic text and
location were captured directly from a real test run against the unfixed code.

## Minimum Reproducible Example

```bash
cd module/helper/embroidery_tools && cargo test --test embroidery_file_test duplicate_color_interpolate_as_stop_does_not_panic_with_fewer_threads_than_color_changes 2>&1 | tail -10
```

**Expected** (post-fix):
```
test duplicate_color_interpolate_as_stop_does_not_panic_with_fewer_threads_than_color_changes ... ok
```

**Actual** (pre-fix -- confirmed via in-place revert-test-restore against the real fixed/unfixed
code, not a separate reconstruction):
```
thread '...' (584356) panicked at module/helper/embroidery_tools/src/embroidery_file.rs:181:32:
removal index (is 1) should be < len (is 0)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/embroidery_tools && cargo test --test embroidery_file_test duplicate_color_interpolate_as_stop_does_not_panic_with_fewer_threads_than_color_changes
# ok = fixed; panic "removal index (is 1) should be < len (is 0)" = bug present
```

**Known MRE limitation (check 205):** none -- pure, synchronous, dependency-free state; the
regression test runs as an ordinary native `cargo test` against the real crate directly, no PEC
file fixture needed to reach the defect.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The guard's `Option::get()` equality comparison has no bounds check, so `None == None` (both indices out-of-bounds) satisfies it and `Vec::remove` runs out-of-bounds. | ✅ Root Cause | Adding a `thread_index < self.threads().len()` guard before the comparison stops the panic and, by hand-traced arithmetic, produces the correct "nothing to merge" fallthrough. | E1, E3 |
| H2 | The removal/merge logic itself (`self.threads.remove( thread_index )`) is fundamentally wrong, even for legitimately in-bounds cases, and needs replacing. | ❌ Falsified | The removal logic is the exact inverse of the crate's own `stop_interpolate_as_duplicate_color`, which performs a symmetric `threads.insert` under the same style of guard -- the merge semantics are correct; only the missing bounds check is defective. | E2 |
| H3 | The correct fix is to panic loudly on the malformed-input case (treat thread/run-count divergence as an unrecoverable data-integrity error) rather than silently skip the merge. | ❌ Rejected (not adopted) | `stop_interpolate_as_duplicate_color` already establishes the crate's own "not enough threads -> skip, `return`, don't panic" convention for the symmetric inverse operation (lines 237-241) -- a hard panic here would be a newly-invented, inconsistent policy, not a fix following existing precedent. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `-0002_longrun.log` (in-place revert-test-restore run against the real unfixed code) | Captured exact pre-fix panic: `embroidery_file.rs:181:32: removal index (is 1) should be < len (is 0)` on the MRE input | H1 ✅ |
| E2 | `embroidery_file.rs`, `stop_interpolate_as_duplicate_color` (lines 209-248, esp. 230-241) | Sibling function's existing `if thread_index < self.threads.len() { ... } else { return; }` guard establishes the crate's own "not enough threads -> skip, don't panic" convention for the inverse merge direction | H2 ❌, H3 ❌ |
| E3 | Hand-traced arithmetic through the MRE against the fixed guard | With 0 threads recorded and 2 color-change-delimited runs: at the second run `thread_index == 1`, and `thread_index < self.threads().len()` (`1 < 0`) is `false`, so the match falls through to `_ => thread_index += 1` -- no removal, no panic, confirming correct behavior beyond mere absence-of-panic | H1 ✅ |
| E4 | `format/pec/reader.rs:133` (grep-confirmed single call site) | `duplicate_color_interpolate_as_stop` runs unconditionally at the end of every `pec::content_read`, after independently-parsed thread-count (`pec_colors_map`, header bytes) and stitch-run structure (`pec_instructions_read`, stitch stream) -- confirms the OOB scenario is reachable from real malformed/unusual file input, not contrived-only | Impact / Who Affected |

## Root Cause

```
duplicate_color_interpolate_as_stop()   (pre-fix)
  match last_change
  {
    Some( last_change ) if thread_index != 0
    && self.threads().get( thread_index ) == self.threads().get( thread_index - 1 ) =>   // <-- no bounds check
    {
      self.threads.remove( thread_index );   // <-- panics once thread_index >= threads.len()
      self.stitches[ last_change ].instruction = Instruction::Stop;
    }
    _ => thread_index += 1,
  }

Fixed guard
  Some( last_change ) if thread_index != 0
  && thread_index < self.threads().len()                                                 // <-- added
  && self.threads().get( thread_index ) == self.threads().get( thread_index - 1 ) =>
```

When `thread_index >= self.threads().len()`, both `.get()` calls return `None`. `None == None`
is `true` in Rust, so the pre-fix guard read "neither index is valid" as "these two threads
match" and proceeded to remove out-of-bounds.

## Why Not Caught

No existing test called `duplicate_color_interpolate_as_stop` at all. Its only production call
site (`pec::content_read`) is exercised solely by the crate's own reference-fixture PEC files
(`read_sample_stitches_match_reference_decoder`, `read_sample_threads_resolve_from_default_palette`),
which are well-formed and never produce the thread-count/stitch-run-count divergence needed to
trigger the defect.

## Fix Location

`module/helper/embroidery_tools/src/embroidery_file.rs`, `duplicate_color_interpolate_as_stop`:

```rust
// before
Some( last_change ) if thread_index != 0
&& self.threads().get( thread_index ) == self.threads().get( thread_index - 1 ) =>

// after
Some( last_change ) if thread_index != 0
&& thread_index < self.threads().len()
&& self.threads().get( thread_index ) == self.threads().get( thread_index - 1 ) =>
```

Added a single `thread_index < self.threads().len()` guard clause before the `.get()` equality
comparison, mirroring `stop_interpolate_as_duplicate_color`'s existing bounds-check pattern. No
signature change, no field change.

## Prevention

Added `duplicate_color_interpolate_as_stop_does_not_panic_with_fewer_threads_than_color_changes`
(`bug_reproducer(BUG-150)`) to `tests/embroidery_file_test.rs`, constructing a file with more
color-change-delimited stitch runs than recorded threads (zero threads at all) -- exactly the
shape `pec::content_read` can produce from a malformed or unusual PEC file, since it calls this
function automatically on every read.

## Pitfall

`None == None` reads as "these two threads match" instead of "neither index is valid" -- any
`Option`-returning `.get()` comparison used as an equality check must first confirm at least one
side is genuinely in-bounds, or two absences will silently compare as a match. Same general
family as BUG-050/122's shared-cursor aliasing in spirit (a structurally-plausible comparison
hides a state it was never designed to handle), but the mechanism here -- `Option`-equality
vacuous truth -- is distinct from those bugs' shared-mutable-cursor mechanism.

## Generalized Version

**Broken assumption:** "two `Option::get( i )` calls compared for equality is a safe way to
check whether index `i` and `i - 1` refer to matching elements." False -- when both indices are
out-of-bounds, `.get()` returns `None` on both sides, and `None == None` is `true`, so the
comparison silently reports "match" for a condition that actually means "neither side exists."

**Confirmed general rule:** whenever an equality check is built from two `Option`-returning
accessor calls, verify at least one side's index is confirmed in-bounds before trusting the
comparison's result -- otherwise "both absent" and "both present and equal" become
indistinguishable.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Surfaced by a background review pass over `embroidery_tools` (task #88); independently reproduced via in-place revert-test-restore before filing. |
| 2026-08-16 | fixed | Added a `thread_index < self.threads().len()` guard before the `.get()` equality comparison, mirroring the sibling `stop_interpolate_as_duplicate_color`'s existing bounds-check convention. |
| 2026-08-16 | verified | Added `duplicate_color_interpolate_as_stop_does_not_panic_with_fewer_threads_than_color_changes` (written test-first against the unfixed code); confirmed it fails pre-fix with the exact predicted panic (`embroidery_file.rs:181:32: removal index (is 1) should be < len (is 0)`) and passes post-fix; full crate suite (11 tests, 0 doctests) + `cargo clippy --all-targets --all-features -- -D warnings` clean. |
| 2026-08-16 | completed | Independently re-verified in a later session: fix still present exactly as documented (`thread_index < self.threads().len()` guard intact), regression test still passing (5/5 in `embroidery_file_test.rs`). Full crate suite re-run clean (`pec_test.rs` 5/5, all others green) after also closing the two sibling bugs (BUG-151, BUG-152) from the same review batch. Related Bugs note updated -- both siblings are now fixed/completed, not open. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the test against unfixed code and captured the exact pre-fix panic text/location; adversarial pass specifically checked whether the 0-thread/2-color-change construction is a contrived-only shape rather than something real input can produce -- resolved by grepping the single call site (E4) and confirming thread-count and stitch-run-count are parsed independently, not by assumption. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Related tasks #100/#101 noted explicitly as same-batch-but-independent-root-cause, not conflated. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by a captured real panic matching the predicted mechanism exactly, plus hand-traced post-fix arithmetic confirming correct fallthrough, not asserted from the diff alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped the whole crate for `duplicate_color_interpolate_as_stop` -- exactly one call site (`format/pec/reader.rs:133`), confirmed unconditional. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `embroidery_tools` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to one added guard clause inside one method body; no signature/field change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing method's "skip when data doesn't support a merge" contract restored, mirroring its own sibling's established convention. | — |

**Reproduced:** YES -- `duplicate_color_interpolate_as_stop_does_not_panic_with_fewer_threads_than_color_changes`
was written and run against the unfixed function first (test-first), producing the exact
predicted panic (`embroidery_file.rs:181:32: removal index (is 1) should be < len (is 0)`);
applying the fix and re-running returned the test to passing, and the full crate suite (11 tests,
0 doctests) + `cargo clippy --all-targets --all-features -- -D warnings` remained clean,
2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/src/embroidery_file.rs` | `duplicate_color_interpolate_as_stop`: added `thread_index < self.threads().len()` guard clause before the `.get()` equality comparison. `Fix(BUG-150)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/tests/embroidery_file_test.rs` | Appended `duplicate_color_interpolate_as_stop_does_not_panic_with_fewer_threads_than_color_changes` (`bug_reproducer(BUG-150)`, 5-section doc comment). |
