# BUG-337: `spinning_cube_size_opt`'s doc comment described a single stationary large point, not the rotating wireframe cube it actually renders

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `examples/minwebgl/spinning_cube_size_opt/src/main.rs`
- **Component:** examples/minwebgl/spinning_cube_size_opt
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

The module doc comment described this example as drawing "a single stationary large point" rather
than what it actually renders: a rotating wireframe cube (8 vertices, 24 line indices, animated
angle uniform, perspective projection).

## Impact

**Who is affected:** any reader using the module doc comment to understand what the demo does
before reading further.

**What breaks:** the doc comment describes a capability (a static point) the crate doesn't have,
giving a materially wrong first impression of both the geometry (8-vertex wireframe cube vs. a
point) and the behavior (animated rotation vs. stationary).

**Entity Scope:** `None` -- documentation-only defect.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by checking the doc comment against the crate's own vertex/index buffer setup and rotation
uniform rather than trusting the prose. Independently verified by the orchestrating session: the
crate's vertex data comprises 8 positions with 24 line-mode indices, and a per-frame angle uniform
drives the rotation -- confirming a rotating wireframe cube, not a point.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p minwebgl_spinning_cube_size_opt --test doc_comment_test
```
**Expected** (fixed): the doc comment describes a rotating wireframe cube. **Actual** (pre-fix):
the doc comment claimed a single stationary large point.

## Root Cause

Stale/copy-pasted description, likely carried over from an earlier, simpler revision of this
example before it grew into its current animated wireframe-cube form.

## Why Not Caught

No test tied the module doc comment's factual claim to the crate's actual vertex/index data or
rotation logic -- the two drift independently since nothing cross-checks them.

## Fix Applied (2026-08-18)

Corrected the module doc comment to describe the crate's actual behavior: a rotating wireframe
cube rendered with minimal code and aggressive size optimization, matching the size-optimization
framing the rest of the doc comment (and the crate's own readme.md) already correctly establishes.
Added `tests/doc_comment_test.rs`: `include_str!` + substring assertion confirming the doc comment
no longer claims a stationary point and does describe a rotating cube.

## Verification

- **Pre-fix (RED):** reverted the doc comment to its stationary-point claim; new test failed
  (stale claim detected).
- **Post-fix (GREEN):** `cargo test -p minwebgl_spinning_cube_size_opt` -- new test passes;
  `cargo check --target wasm32-unknown-unknown -p minwebgl_spinning_cube_size_opt` and
  `cargo clippy --all-targets --all-features -p minwebgl_spinning_cube_size_opt -- -D warnings` both clean.

## Generalized Version

Copy-pasted boilerplate doc comments across sibling example crates drift silently once one
crate's actual behavior diverges from the text -- this is at least the third `examples/minwebgl`
crate in this session's bug-hunt found carrying the exact same "Just draw a large point in the
middle of the screen." stale sentence verbatim (alongside `attributes_vao`/BUG-318 and
`attributes_instanced`/BUG-319, both fixed incidentally as part of BUG-318), confirming this exact
sentence originated from one early template file and was copy-pasted broadly. A repo-wide grep for
this phrase after fixing this bug turned up 4 further unfixed instances the assigned forks had
missed entirely (`make_cube_map`, `obj_load`, `obj_viewer`, `trivial` -- filed and fixed as
BUG-340), confirming the audit this note already recommended was in fact necessary, not
hypothetical.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-337 after a fresh on-disk collision scan. Related: BUG-319 (`attributes_instanced`), BUG-336 (`text_msdf`) -- same stale-doc-comment lineage, independent crates. |
