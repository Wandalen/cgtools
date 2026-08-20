# BUG-319: `attributes_instanced`'s module doc comment claims a single stationary large point, not the instanced-triangle rendering it actually demonstrates

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `examples/minwebgl/attributes_instanced/src/main.rs`
- **Component:** examples/minwebgl/attributes_instanced
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

The module doc comment claimed this example "draw[s] a large point in the middle of the screen."
The crate actually instanced-draws 6 triangles with a per-instance Y offset (divisor 1, 5
instances) -- its own `readme.md` already correctly describes it as an instanced-rendering demo.

## Impact

**Who is affected:** any reader using the module doc comment to understand what the demo does
before reading further.

**What breaks:** the doc comment describes a capability (drawing one point) the crate doesn't
have, giving a materially wrong first impression that also directly contradicts the crate's own
`readme.md`.

**Entity Scope:** `None` -- documentation-only defect.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), which also flagged sibling crate `attributes_vao`'s `main.rs` as having carried the exact
same stale sentence -- corrected there as part of that crate's own VAO-structure fix (BUG-318),
whose rewritten doc comment now correctly reads "Draws 5+5 points via two independent VAOs...".
Independently verified by the orchestrating session against this crate's own
`gl::draw_instanced`/attribute-divisor call sites, its `readme.md`, and `attributes_vao/src/main.rs`'s
current (already-corrected) doc comment.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -c "draw a large point" examples/minwebgl/attributes_instanced/src/main.rs
grep -n "instanc" examples/minwebgl/attributes_instanced/readme.md
```
**Expected** (fixed): the "draw a large point" count is 0, and the doc comment matches the
readme's instanced-rendering description. **Actual** (pre-fix): count was 1, contradicting the
crate's own readme and its actual instanced-triangle draw call.

## Root Cause

Stale/copy-pasted doc comment, never updated as the demo grew past an early single-point sketch
into its current instanced-rendering form -- the same copy-paste lineage that also produced the
identical stale sentence in sibling crate `attributes_vao`, corrected there as an incidental part
of that crate's own VAO-structure fix (BUG-318) rather than as a separately filed doc bug.

## Why Not Caught

No test tied the module doc comment's factual claim to the crate's actual draw calls or its own
readme -- the two drift independently since nothing cross-checks them.

## Fix Applied (2026-08-18)

Replaced the stale doc comment with an accurate description: instanced rendering of 6 triangles
sharing one base geometry, offset per-instance via a divisor-1 Y-offset attribute (5 instances),
matching the crate's own readme. Added `tests/doc_comment_test.rs`
(`main_doc_comment_describes_instanced_triangles_not_a_point`): pure `include_str!` + substring
assertion, following the same pattern established for BUG-306's sibling doc-comment fixes.

## Verification

- **Pre-fix (RED):** doc comment claimed a point; new test would fail against the pristine text.
- **Post-fix (GREEN):** `cargo test -p minwebgl_attributes_instanced` -- new test passes;
  `cargo check --target wasm32-unknown-unknown -p minwebgl_attributes_instanced` and
  `cargo clippy --all-targets --all-features -p minwebgl_attributes_instanced -- -D warnings` both clean.

## Generalized Version

Copy-pasted boilerplate doc comments across sibling example crates drift silently once one
crate's actual behavior diverges from the text -- when one sibling crate's stale doc comment is
found and fixed (here, `attributes_vao`'s, alongside its own BUG-318 fix), check every other
sibling sharing the same copy-paste lineage rather than assuming the defect was isolated to the
first one found.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-ZZZ` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-319 after a fresh on-disk collision scan. |
