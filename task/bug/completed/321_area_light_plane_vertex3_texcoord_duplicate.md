# BUG-321: `area_light`'s plane geometry duplicates vertex 2's texcoord at vertex 3 instead of computing its own corner, breaking the bilinear UV grid (dormant until a non-constant texture is bound)

- **Severity:** Low (currently invisible -- both textures are a constant 1x1 texel today)
- **state:** Completed
- **Affects:** `examples/minwebgl/area_light/src/plane.rs`
- **Component:** examples/minwebgl/area_light
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`plane_vao`'s vertex 3 carried texcoord `(1.0, 0.0)` -- an exact duplicate of vertex 2's row --
instead of `(1.0, 1.0)`, the value required to complete the bilinear UV grid the other 3 vertices
already establish (`uv.x` tracks `-z`, `uv.y` tracks `x`).

## Impact

**Who is affected:** dormant today -- `plane_material` currently fills both textures with a single
constant 1x1 texel, so every UV sample returns the same color under `wrap_clamp`/`filter_nearest`
regardless of which texcoord is wrong. Becomes visible the moment a real (non-1x1) texture is
bound to this plane.

**What breaks:** the plane's UV grid is internally inconsistent -- vertex 3 (corner diagonal from
vertex 0, at `x=1, z=-1`) samples the same UV as vertex 2 instead of its own distinct corner,
which would visibly distort any non-constant texture mapped onto the plane.

**Entity Scope:** `None` -- confined to this crate's own plane-geometry construction.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by checking each vertex's texcoord against the UV-axis convention the other 3 vertices
establish rather than trusting the full vertex table at face value. Independently verified by the
orchestrating session: vertices 0/1/2's texcoords do form a consistent `(-z, x)`-tracking grid,
and vertex 3's position (`x=1, z=-1`) requires `(1.0, 1.0)` to complete it.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p area_light --test plane_texcoord_test
```
**Expected** (fixed): vertex 3's texcoord row reads `(1.0, 1.0)`, distinct from vertex 2's
`(1.0, 0.0)`. **Actual** (pre-fix): vertex 3's row was an exact duplicate of vertex 2's.

## Root Cause

Vertex 3's texcoord row was copy-pasted from vertex 2's instead of being computed for its own
corner -- the other 3 vertices (0, 1, 2) already form a correct, consistent grid.

## Why Not Caught

No test exercised the plane geometry's texcoord values against its own position data, and the
demo's current constant-texel materials mask any UV distortion visually -- nothing renders
differently whether this row is correct or duplicated.

## Fix Applied (2026-08-18)

Corrected vertex 3's texcoord from `(1.0, 0.0)` to `(1.0, 1.0)`, completing the bilinear UV grid
vertices 0/1/2 already establish. Vertices 0/1/2 were left untouched -- only vertex 3's row was
wrong. Added `tests/plane_texcoord_test.rs`: asserts all 4 vertices' texcoords are distinct and
that vertex 3 specifically equals `(1.0, 1.0)`, guarding against the duplicate regressing.

## Verification

- **Pre-fix (RED):** reverted vertex 3's texcoord to `(1.0, 0.0)`; new test failed (duplicate of
  vertex 2 detected).
- **Post-fix (GREEN):** `cargo test -p area_light` -- new test passes;
  `cargo check --target wasm32-unknown-unknown -p area_light` and
  `cargo clippy --all-targets --all-features -p area_light -- -D warnings` both clean.

## Generalized Version

A dormant geometry defect masked by a currently-constant material (uniform color, 1x1 texture) has
no visible symptom today but will surface the moment a more realistic material is bound -- when
auditing hand-written vertex tables, check each vertex's texcoord against the UV-axis convention
the rest of the table establishes, not just for internal consistency of the table as a whole
(a duplicated row can still "look" like valid data at a glance).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-VVV` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-321 after a fresh on-disk collision scan. |
