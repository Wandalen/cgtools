# BUG-151: `pec_table_process` silently drops every first-seen color instead of recording it, misaligning `emb.threads()` against the color table

- **Severity:** High (silently corrupts the thread palette read from any PES v6 file whose
  embedded thread chart is nonempty but shorter than its PEC section's color table -- not a
  crash, but every downstream consumer indexing `emb.threads()` by color-change position
  gets wrong or missing data)
- **state:** Completed
- **Affects:** Any `pec::content_read` call (in practice, `pes::read` on a `#PES0060`/v6 file)
  whose `pes_chart` argument is non-empty but shorter than the PEC section's color table
- **Component:** `module/helper/embroidery_tools` (`src/format/pec/reader.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** BUG-150 (same review batch, same crate, independent code path --
  `duplicate_color_interpolate_as_stop`'s OOB guard vs. this bug's dropped-thread merge
  logic). Also related to BUG-152 (`pec_header_write` dropped the caller's first added
  thread when writing) -- now fixed; this bug's own regression test adds 2 threads and both
  land cleanly in the written color table, with no compensation needed. See Prevention.

## Symptom

```rust
use embroidery_tools::format::pec;
use embroidery_tools::thread::Thread;

// A PEC color table with 2 entries, read with a 1-entry PES chart shorter than it.
let chart_thread = Thread { description : "chart thread".into(), ..Default::default() };
let mut result = EmbroideryFile::new();
pec::content_read( &mut result, &mut reader, std::slice::from_ref( &chart_thread ) ).unwrap();

result.threads().len()
// Wrong (pre-fix): 0  -- both first-seen colors silently dropped, nothing ever recorded
// Correct (post-fix): 2  -- one thread per color-table byte, matching the table's own length
```

## Impact

**Who is affected:** Any caller of `pec::content_read` that supplies a non-empty `pes_chart`
shorter than the PEC section's own color table -- in practice, exactly `pes::read` on a
`#PES0060` (v6) file, which reads its own thread chart from the PES header (`header_version6_read`)
and passes it straight through to the embedded PEC section's `content_read` call
(`format/pes/reader.rs:85`).

**What breaks:** `pec_colors_map` routes to `pec_table_process` whenever the chart is
nonempty but shorter than `color_bytes.len()` -- "mixed with default palette" per its own doc
comment. `pec_table_process`'s per-byte loop used a `HashMap` (`thread_map`) to decide *which*
`Thread` value to assign a given `color_index`, but only pushed that value into
`emb.threads()`/`values` on a cache **hit** (`if let Some(thread) = thread_value`) -- the cache
**miss** branch (first sighting of any `color_index`, the common case) computed the thread and
cached it, but never called `emb.thread_add`/`values.push` at all. Every color's first
occurrence vanished; only its second-and-later occurrences (if any) were ever recorded. For
the frequent case where every `color_index` in a design is distinct (no immediately-obvious
repeats), `emb.threads()` ends up completely empty regardless of how many colors the file
actually uses.

**Magnitude:** Silent data corruption, not a crash -- `emb.threads()` ends up either empty or
with far fewer entries than `color_bytes.len()`, breaking the 1-entry-per-color-change
invariant every positional consumer of `emb.threads()` depends on (including
`duplicate_color_interpolate_as_stop`, fixed independently in BUG-150) and silently producing
wrong thumbnail/thread-color data for any `pec_graphics_read` consumer downstream in the same
`content_read` call.

**Entity Scope:** None -- a code-level defect, not an operational-entity concern.

## How Discovered

Surfaced by a background review pass over the `embroidery_tools` crate (task #88, one of five
parallel crate reviews this session). Independently confirmed by direct code reading (the
`else` branch's missing `thread_add`/`values.push` calls, contrasted against the `if let Some`
branch that has them) before filing, then reproduced via a real write→read roundtrip rather
than trusting the code-reading alone.

## Minimum Reproducible Example

```bash
cd module/helper/embroidery_tools && cargo test --test pec_test content_read_with_short_chart_assigns_one_thread_per_color_byte 2>&1 | tail -10
```

**Expected** (post-fix):
```
test content_read_with_short_chart_assigns_one_thread_per_color_byte ... ok
```

**Actual** (pre-fix -- confirmed via in-place revert-test-restore against the real
fixed/unfixed code):
```
assertion `left == right` failed: one thread must be recorded per color-table byte, not silently dropped on first sight of a color
  left: 0
 right: 2
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/embroidery_tools && cargo test --test pec_test content_read_with_short_chart_assigns_one_thread_per_color_byte
# ok = fixed; "left: 0, right: 2" = bug present
```

**Known MRE limitation (check 205, historical):** at filing time, the test wrote a real
design through `pec::write` rather than hand-crafting PEC bytes, so it also depended on the
writer producing a valid color table -- it deliberately compensated for the then-still-open
BUG-152 writer defect (`pec_header_write` dropped the caller's first added thread) by adding
3 threads to reliably get 2 encoded. BUG-152 is now fixed, so the test adds exactly 2 threads
and both land in the written color table with no compensation needed -- see the current
`tests/pec_test.rs`.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The `else` (cache-miss) branch never calls `emb.thread_add`/`values.push`, unlike the `if let Some` (cache-hit) branch which does -- every first-seen color is dropped. | ✅ Root Cause | Direct code reading confirmed the asymmetry; the MRE's captured failure (`left: 0, right: 2`, both color bytes being fresh misses) matches exactly. | E1, E2 |
| H2 | `thread_map`'s dedup caching itself is the wrong design -- repeated colors shouldn't reuse a cached thread at all. | ❌ Rejected (not adopted) | The cache-*hit* branch's existing push behavior is correct and left untouched by the fix; only the cache-*miss* branch was missing its push. Caching *which* thread to reuse for a repeat is orthogonal to *whether* a push happens. | E1 |
| H3 | The bug is actually in `pec_colors_map`'s branch-selection logic (routing to the wrong helper), not inside `pec_table_process` itself. | ❌ Falsified | `pec_colors_map`'s three branches (empty chart / chart >= color count / chart < color count) are mutually exclusive and correctly route the "mixed" case to `pec_table_process` -- the MRE deliberately targets exactly that case and the defect is entirely inside the routed-to function. | E2, E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `format/pec/reader.rs`, `pec_table_process` (pre-fix, both branches) | `if let Some(thread) = thread_value { emb.thread_add(...); values.push(...); }` vs. the `else` branch computing `thread` and calling only `thread_map.insert(...)` -- the asymmetry is directly visible in the diff. | H1 ✅, H2 ❌ |
| E2 | `-0005_longrun.log` (in-place revert-test-restore run against the real unfixed code) | Captured exact pre-fix failure: `assertion left == right failed ... left: 0, right: 2` -- both of the MRE's 2 distinct color bytes were fresh cache-misses, so both were dropped, leaving `emb.threads()` completely empty. | H1 ✅, H3 ❌ |
| E3 | `format/pec/reader.rs`, `pec_colors_map` (lines 154-169, unedited) | The 3-way branch (`chart.is_empty()` / `chart.len() >= color_bytes.len()` / else) correctly isolates the "mixed" case to `pec_table_process` -- confirmed by construction, not just by reading in isolation. | H3 ❌ |

## Root Cause

```
pec_table_process()   (pre-fix)
  for byte in color_bytes
  {
    let color_index = ...;
    match thread_map.get( &color_index )
    {
      Some( thread ) => { emb.thread_add( thread.clone() ); values.push( thread.clone() ); }  // push -- OK
      None =>
      {
        let thread = /* drain chart or fall back to default palette */;
        thread_map.insert( color_index, thread );   // <-- no push at all
      }
    }
  }
```

The cache-miss branch (the *first* time any given `color_index` is seen -- the common case for
any design using more than one or two colors) computed the correct `Thread` value but never
recorded it into `emb`/`values`. Only a *second* sighting of the same `color_index` (a cache
hit) ever produced output.

## Why Not Caught

No existing test supplied a non-empty `pes_chart` shorter than the PEC section's color table
to `pec::content_read` -- the only way to reach `pec_table_process` at all. `pes_test.rs`'s
`v6_roundtrip_preserves_metadata_and_threads` uses a single-thread, single-color design where
`chart.len() >= color_bytes.len()` (1 >= 1), taking the direct-copy branch in `pec_colors_map`
instead; `pec_test.rs`'s existing tests never pass a chart at all (`pec::read`/`memory_read`
always call `content_read` with `&[]`).

## Fix Location

`module/helper/embroidery_tools/src/format/pec/reader.rs`, `pec_table_process`:

```rust
// before
else
{
  let thread = if chart.is_empty()
  {
    threads[ color_index ].clone()
  }
  else
  {
    chart.remove( 0 )
  };
  thread_map.insert( color_index, thread );
}

// after
else
{
  let thread = if chart.is_empty()
  {
    threads[ color_index ].clone()
  }
  else
  {
    chart.remove( 0 )
  };
  emb.thread_add( thread.clone() );
  values.push( thread.clone() );
  thread_map.insert( color_index, thread );
}
```

Added the same `emb.thread_add`/`values.push` pair the cache-hit branch already performs,
before moving `thread` into the cache. No signature change, no field change.

## Prevention

Added `content_read_with_short_chart_assigns_one_thread_per_color_byte`
(`bug_reproducer(BUG-151)`) to `tests/pec_test.rs`: builds a design with 2
color-change-delimited stitch runs, writes it via `pec::write` (adding 2 threads, both of
which land cleanly in the written color table now that BUG-152's writer defect is fixed),
then calls `pec::content_read` directly with a 1-entry chart (shorter than the 2-entry table)
and asserts the recovered thread count and values match the table, not the chart.

## Pitfall

A dedup cache (here, `thread_map` keyed by `color_index`) is only supposed to decide *which*
value to reuse for a repeat -- it must never be allowed to also gate *whether* a side effect
(the push) happens at all. Every entry in the source sequence (`color_bytes`) still needs
exactly one push, matching the sibling `pec_colors_process`'s unconditional per-byte push with
no cache at all. This is the same general shape as BUG-150's `Option::get()` guard in the same
crate -- a plausible-looking conditional silently absorbs a case it wasn't designed to handle
-- but the mechanism here (a cache branch quietly missing its side effect, not a vacuous
equality) is distinct.

## Generalized Version

**Broken assumption:** "adding a dedup/caching layer in front of a per-item side effect is a
drop-in change that only affects *which* value is used, not *whether* the side effect
happens." False here -- introducing `thread_map` to decide which `Thread` to reuse for a
repeated `color_index` accidentally became the sole gate for the `thread_add`/`values.push`
side effect too, since only the cache-hit branch retained it.

**Confirmed general rule:** when adding a memoization/dedup layer around code that has a
required per-iteration side effect, verify the side effect still fires on *every* iteration
(both cache hit and cache miss) -- a cache's job is to pick a value, never to decide whether
downstream state gets updated.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Surfaced by a background review pass over `embroidery_tools` (task #88); independently reproduced via in-place revert-test-restore before filing. |
| 2026-08-16 | fixed | Added `emb.thread_add`/`values.push` to `pec_table_process`'s cache-miss branch, mirroring the cache-hit branch's existing behavior. |
| 2026-08-16 | verified | Added `content_read_with_short_chart_assigns_one_thread_per_color_byte` (written test-first against the unfixed code); confirmed it fails pre-fix with the exact predicted `left: 0, right: 2` and passes post-fix; full crate suite (12 tests, 0 doctests) + `cargo clippy --all-targets --all-features -- -D warnings` clean. |
| 2026-08-16 | completed | Independently re-verified in a later session: fix still present exactly as documented, regression test still passing. Corrected stale documentation (Related Bugs, Known MRE limitation, Prevention) that referenced an obsolete "+3 threads to compensate for task #101" workaround -- BUG-152 (the writer defect) is now fixed, so the current test adds exactly 2 threads with no compensation needed. Full crate suite re-run clean (`pec_test.rs` 5/5) after also closing BUG-150 and BUG-152 from the same review batch. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the test against unfixed code and captured the exact pre-fix failure; adversarial pass specifically checked whether the test's use of `pec::write` (rather than hand-crafted bytes) silently depends on unverified writer behavior -- resolved by reading `pec_header_write` directly, discovering and explicitly documenting the task #101 interaction rather than being surprised by it. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Related to BUG-150 (same batch, independent root cause) and task #101 (separate writer defect this test's setup must compensate for) -- both noted explicitly, not conflated. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct diff-level code comparison (cache-hit vs. cache-miss branch asymmetry) plus a captured real failure matching the predicted mechanism exactly. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Read `pec_colors_map`'s full 3-way branch to confirm `pec_table_process` is the sole defective path and the routing logic itself is correct (H3). | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `embroidery_tools` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to 2 added lines inside one function body; no signature/field change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing function's "one thread per color-table byte" contract restored, matching its own sibling `pec_colors_process`. | — |

**Reproduced:** YES -- `content_read_with_short_chart_assigns_one_thread_per_color_byte` was
written and run against the unfixed function first (test-first), producing the exact predicted
failure (`left: 0, right: 2`); applying the fix and re-running returned the test to passing,
and the full crate suite (12 tests, 0 doctests) + `cargo clippy --all-targets --all-features --
-D warnings` remained clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/src/format/pec/reader.rs` | `pec_table_process`: added `emb.thread_add`/`values.push` to the cache-miss (`else`) branch. `Fix(BUG-151)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/tests/pec_test.rs` | Added `use embroidery_tools::thread::Thread;` import and appended `content_read_with_short_chart_assigns_one_thread_per_color_byte` (`bug_reproducer(BUG-151)`, 5-section doc comment). |
