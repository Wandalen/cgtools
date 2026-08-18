# BUG-347: `ECSInspector::entity_record` permanently inflates `component_counts` when the same entity is re-recorded

- **Severity:** Medium
- **state:** Verified
- **Affects:** Any code path that calls `ECSInspector::entity_record` more than once for the same
  `entity.id` (e.g., a debug overlay that re-records entities every frame, or any system that
  updates debug info as components change)
- **Component:** `module/helper/tiles_tools` (`src/debug.rs`, `ECSInspector::entity_record`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/ (self)
- **verification_date:** 2026-08-18

## Symptom

Re-recording the same entity via `ECSInspector::entity_record` (e.g., once per frame, or after a
component changes) makes `component_counts` grow without bound, even though `entity_data` itself
correctly holds only one entry per `entity.id`:

```
# Record entity 1 with components [Position] once, then again with [Position, Health]:

inspector.entity_record(entity_1_with_position);
inspector.entity_record(entity_1_with_position_and_health);

inspector.entity_count()        -> 1                         # correct: one entity
inspector.report_generate()     -> "Position: 2 entities"     # wrong: only 1 entity has Position
```

## Impact

**Who is affected:** any consumer of `ECSInspector` that calls `entity_record` more than once for
the same entity — the intended, documented usage for a live debug overlay that refreshes entity
state as the game runs (there is no companion `entity_remove`/`unrecord` method to work around
this by removing-then-re-adding).

**What breaks:** `component_counts` (and therefore `report_generate`'s "Component Statistics"
section, and any other consumer reading `component_counts` directly) silently and permanently
overcounts every component every time an already-recorded entity is re-recorded — the counter
never converges to the correct value even though `entity_data` itself stays correct. No panic, no
error: a purely silent, ever-growing statistics corruption in a debugging tool whose entire job is
to report accurate state.

**Magnitude:** 1 method (`entity_record`); every call site that records the same entity more than
once is affected. `entity_data` (the per-entity detail map) is unaffected — only the aggregate
`component_counts` map is corrupted.

**Entity Scope:** `None` — a code-level counter defect, not entity directory instances.

## How Discovered

```bash
$ cargo test -p tiles_tools --all-features --test debug_test \
    test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts -- --exact

thread 'test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts'
panicked at module/helper/tiles_tools/tests/debug_test.rs:222:3:
re-recording entity 1 should leave Position's count at 1 (one currently-recorded entity), got:
ECS Inspector Report
===================

Total Entities: 1

Component Statistics:
  Position: 2 entities
  Health: 1 entities

Detailed Entity Information:

Entity 1:
  Components: Position, Health
test result: FAILED. 8 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

A prior investigation pass identified the missing decrement by direct reading of `entity_record`
(§ Hypothesis Table below); this report re-confirms it with the permanent reproducer test above,
run against the pre-fix source.

## Minimum Reproducible Example

**Verify Command** (run from repo root; ≤3 lines):
```bash
cargo test -p tiles_tools --all-features --test debug_test \
  test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts -- --exact
```
**What:** re-recording the same `entity.id` must not leave stale contributions from the previous
recording in `component_counts` — a component present in both the old and new recording must
still count as exactly 1, not 2.

**Expected** (fixed): test passes — `test test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts ... ok`.

**Actual** (pre-fix, directly confirmed by running the same test against the current, unfixed
source before applying the fix below):
```
thread 'test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts'
panicked at module/helper/tiles_tools/tests/debug_test.rs:222:3:
re-recording entity 1 should leave Position's count at 1 (one currently-recorded entity), got:
ECS Inspector Report
===================

Total Entities: 1

Component Statistics:
  Position: 2 entities
  Health: 1 entities

Detailed Entity Information:

Entity 1:
  Components: Position, Health
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `entity_record` (debug.rs:783, pre-fix) increments `component_counts` for every component of the newly-passed `entity`, unconditionally, on every call — with no corresponding decrement of whatever the previous call for the same `entity.id` had already contributed | ✅ Root Cause | Direct read: the only mutation of `component_counts` is `*self.component_counts.entry(component.clone()).or_insert(0) += 1;` inside a loop over `entity.components`, reached on every call regardless of whether `entity.id` was already present | E1 |
| H2 | `entity_data.insert(entity.id, entity)` (debug.rs, end of `entity_record`) uses `HashMap::insert`, which silently overwrites any prior entry for the same key — so `entity_data` itself never accumulates stale entries, only `component_counts` does | ✅ Verified | Direct read: `self.entity_data.insert(entity.id, entity);` — standard overwrite-on-insert semantics, confirmed by `entity_count()` (returns `entity_data.len()`) staying at 1 across repeated calls in the reproducer | E2 |
| H3 | No method on `ECSInspector` (`entity_remove`, `unrecord`, or similar) exists to decrement `component_counts` for a previously-recorded entity, so there is no available workaround call sequence (remove-then-re-add) short of fixing `entity_record` itself | ✅ Verified | `grep -n "pub fn" src/debug.rs` inside `impl ECSInspector` lists only `new`, `entity_record`, `system_time_record`, `report_generate`, `entity_count`, `total_system_time`, `slowest_system` — no removal/unrecord method | E3 |
| H4 | The struct's own doc comment / intended usage (a live debug overlay) implies `entity_record` is meant to be called repeatedly for the same entity as its components change over time, making this not an edge case but the primary intended usage pattern | ✅ Verified | `src/debug.rs` module-level doc comment (doctest at `debug.rs:27`) and the method's own `/// Records entity information.` doc frame `entity_record` as the standard per-entity update path, with no caveat about single-call-only usage | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/tiles_tools/src/debug.rs:783` (`entity_record`, pre-fix, direct read via `git show HEAD:...`) | `component_counts` incremented unconditionally for every component of the new `entity`, every call, no prior-state check | H1 |
| E2 | `module/helper/tiles_tools/src/debug.rs` (`entity_record`, direct read) | `self.entity_data.insert(entity.id, entity);` — ordinary `HashMap::insert` overwrite, confirmed correct (no accumulation) via `entity_count() == 1` in the reproducer after 2 calls | H2 |
| E3 | `grep -n "pub fn" src/debug.rs` (direct read) | No `entity_remove`/`unrecord` method exists on `ECSInspector` | H3 |
| E4 | `module/helper/tiles_tools/src/debug.rs:27` (module doctest, direct read) | Module documentation frames `ECSInspector` as a live/refreshable debug tool, not a write-once log | H4 |
| E5 | Terminal output (this report, MRE section; also captured in `-0001_longrun.log:26-43`, pre-fix combined test run) | Reproducer assertion fails exactly as predicted: `report_generate()` shows `Position: 2 entities` after only 1 entity (with `Position`) was ever recorded | H1 |

## Root Cause

```
entity_record(entity_1 { components: [Position] })
  -> component_counts[Position] += 1     => component_counts = { Position: 1 }
  -> entity_data.insert(1, entity_1)     => entity_data = { 1: [Position] }

entity_record(entity_1 { components: [Position, Health] })   # re-record, same id
  -> component_counts[Position] += 1     => component_counts = { Position: 2, Health: 1 }  # WRONG
  -> component_counts[Health]   += 1
  -> entity_data.insert(1, entity_1')    => entity_data = { 1: [Position, Health] }         # correct
```
`entity_record` treats every call as purely additive against `component_counts`, but treats
`entity_data` as replace-on-conflict (via `HashMap::insert`). The two maps are meant to describe
the same underlying fact (which components which entities currently have) but are maintained with
inconsistent semantics: one self-corrects on re-recording, the other does not. Since nothing
decrements the previous call's contribution before applying the new one, `component_counts`
diverges from `entity_data`'s ground truth by exactly the previous recording's component set,
every time the same entity is re-recorded.

## Why Not Caught

Every existing `ECSInspector` test recorded each `entity.id` exactly once before asserting on
`component_counts`/`report_generate` — none exercised the "record the same entity twice with a
changed component set" path that a live debug overlay's actual, intended usage requires. No test
asserted that `component_counts` stays consistent with `entity_data` (the two maps' relationship
was never checked against each other at all).

## Fix Location

**`module/helper/tiles_tools/src/debug.rs:783`** (`entity_record`, pre-fix signature; fix inserted
immediately after the opening brace, before the existing increment loop):

```rust
// Before:
pub fn entity_record(&mut self, entity: EntityDebugInfo) {
  for component in &entity.components {
    *self.component_counts.entry(component.clone()).or_insert(0) += 1;
  }
  self.entity_data.insert(entity.id, entity);
}

// After:
pub fn entity_record(&mut self, entity: EntityDebugInfo) {
  // Fix(BUG-347): decrement the previous entry's component counts (if
  // entity.id was already recorded) before applying the new entity's
  // counts, so re-recording the same entity_id does not permanently
  // inflate component_counts.
  // Root cause: every call incremented component_counts for the new
  // entity's components, then unconditionally overwrote entity_data via
  // HashMap::insert -- the prior call's contribution to component_counts
  // was never removed, and no entity_remove/unrecord method existed to
  // correct it either.
  // Pitfall: a counter incremented on every call of a "record" method that
  // can be called more than once for the same identity needs a matching
  // decrement for whatever it is replacing -- otherwise re-recording
  // silently inflates the counter forever, with no panic to surface it.
  if let Some(previous) = self.entity_data.get(&entity.id) {
    for component in &previous.components {
      if let Some(count) = self.component_counts.get_mut(component) {
        *count = count.saturating_sub(1);
        if *count == 0 {
          self.component_counts.remove(component);
        }
      }
    }
  }

  for component in &entity.components {
    *self.component_counts.entry(component.clone()).or_insert(0) += 1;
  }
  self.entity_data.insert(entity.id, entity);
}
```

## Prevention

Detection command for the general pattern (an aggregate counter mutated by a "record"/"upsert"
method that also maintains an authoritative per-key map, without symmetric decrement-then-insert
handling):
```bash
grep -n "or_insert(0) += 1" module/helper/tiles_tools/src/debug.rs
```
This is a starting point for review, not a precise check — confirming correctness requires a
re-recording test (record the same key twice with different derived aggregate contributions,
assert the aggregate matches what the current per-key state alone implies), which is exactly what
the new reproducer test adds.

**Pitfall:** when a method maintains two data structures describing the same underlying fact — one
that self-corrects on repeated calls (like `HashMap::insert`'s overwrite) and one that is purely
additive (like a counter incremented per call) — repeated calls for the same key will desync them
unless the additive structure is given an explicit decrement-the-old-value step before applying
the new one.

## Generalized Version

**Broken assumption:** "a method that records/upserts an entity by key, and also increments an
aggregate counter derived from that entity's fields, is safe to call more than once for the same
key — because the per-key map 'handles' repeated calls via overwrite."

Fails for any recording/upsert method whenever:
1. The method maintains at least one aggregate structure (a counter, a sum, a running total)
   derived from a per-key entity's fields, AND
2. That aggregate is only ever incremented (never decremented) when the method runs, AND
3. The method can legitimately be called more than once for the same key (an update/refresh path,
   not a strict write-once log).

**Detection invariant:**
```
for every "record(key, value)" method maintaining an aggregate A derived from value's fields:
  calling record(key, v1) then record(key, v2) must leave A equal to
  what a single record(key, v2) call from empty state would have produced
  (i.e., A must reflect only the CURRENT per-key state, never a sum across all historical calls)
```
Confirmed as a single instance in this crate (`entity_record` is the only "record by key + bump an
aggregate counter" method in `debug.rs`; `system_time_record` — the other `record`-named method —
appends to a `Vec` per system name rather than maintaining a counter derived from overwritten
state, so it does not share this failure mode). Dedup search:
`grep -rli "entity_record\|component_counts\|ECSInspector" task/bug/` found no prior hits — not a
duplicate of any existing bug report in this repository.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Re-confirmed via direct source reading and a new permanent re-recording reproducer test, following up a prior investigation pass's finding |
| 2026-08-18 | note | SUBMIT: state Draft -> Unverified; reproducer confirmed FAIL pre-fix and PASS post-fix, fix applied, full scoped suite (`cargo test -p tiles_tools --all-features`) green |
| 2026-08-18 | VERIFY Gate | Reproducer test `test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts` confirmed passing (`cargo test -p tiles_tools --all-features --test debug_test test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts -- --exact`: 1 passed; 0 failed) against current source; fix in `module/helper/tiles_tools/src/debug.rs` confirmed present in `entity_record` (line 783) -- decrement-previous-contribution block at lines 799-808 matches the report's claimed After block. state: Unverified -> Verified |
| 2026-08-18 | note | VERIFY Gate two-pass re-check (Tier 2 Dual-Role Self-Check, `governance/maav.rulebook.md`): adversarial pass found neither `src/debug.rs` nor `tests/debug_test.rs` carried the canonical FI027 backreference (only `Fix(BUG-347)`/`test_kind:` markers existed, matching the same gap BUG-298's own VERIFY Gate previously found and fixed in this repo); added `// BUG-347 task/bug/347_....md -- ...` backreference comment adjacent to each marker, re-verified via `grep -rn 'BUG-347' src/ tests/`; full `tiles_tools` scoped suite re-run (`cargo nextest run -p tiles_tools --all-features`: 272 passed / 0 failed, including this bug's reproducer); `## Verification Record` appended below |

## Refs: src/

- `module/helper/tiles_tools/src/debug.rs` — `entity_record` now decrements the previous recording's component contributions (if `entity.id` was already present) before applying the new one

## Refs: tests/

- `module/helper/tiles_tools/tests/debug_test.rs` — new reproducer: records the same `entity.id` twice with different component sets and asserts `component_counts` (via `report_generate`) reflects only the current state, not a sum across both calls

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE uses an in-repo `cargo test` command, not literal `/tmp/mreNNN/` paths -- deliberate, precedented local adaptation for a crate-internal algorithm defect (matches BUG-298/BUG-300's own already-verified shape in this repo), not an oversight | — |
| D3 | Cross-Reference Integrity | 🟡 | 🟢 | State was already flipped to Verified (with a History row) by a prior pass that left no `## Verification Record`, and neither `src/debug.rs` nor `tests/debug_test.rs` carried the canonical FI027 backreference (only `Fix(BUG-347)`/`test_kind:` markers existed) | Added canonical backreference comment adjacent to each existing marker in both files; re-verified via `grep -rn 'BUG-347' src/ tests/` |
| D4 | Root Cause Quality | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 0 open | 1/1 |

**Reproduced:** YES -- exit 0 (`test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts` ... ok), 2026-08-18. Full `tiles_tools` scoped suite (`cargo nextest run -p tiles_tools --all-features`, 272 passed / 0 failed) re-confirmed post-fix.
