# BUG-340: `make_cube_map`, `obj_load`, `obj_viewer` all carried the identical stale "draws a large point" doc comment, unfixed despite being explicitly in-scope for this session's own bug-hunt pass

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `examples/minwebgl/make_cube_map/src/main.rs` (A), `examples/minwebgl/obj_load/src/main.rs` (B), `examples/minwebgl/obj_viewer/src/main.rs` (C)
- **Component:** examples/minwebgl/make_cube_map, examples/minwebgl/obj_load, examples/minwebgl/obj_viewer
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

All three crates' module doc comments read the identical byte-for-byte sentence "Just draw a
large point in the middle of the screen." -- none of them actually do this: (A) generates a 6-face
cube map texture from a loaded glTF model and displays a textured, orbiting cube sampling it; (B)
loads and renders the Suzanne OBJ model as a diffuse-lit, rotating triangle mesh; (C) is a full
interactive OBJ viewer with orbit-camera controls, multi-material/multi-texture loading, and
opaque/transparent render-pass splitting.

## Impact

**Who is affected:** any reader using the module doc comment to understand what each demo does
before reading further.

**What breaks:** all three doc comments describe a trivial single-point demo instead of their
actual (in two of the three cases, substantially more complex) rendering pipelines, giving a
materially wrong first impression of what each crate demonstrates.

**Entity Scope:** `None` -- documentation-only defect, confined to each crate's own `src/main.rs`.

## How Discovered

While correcting a similar bug's (`spinning_cube_size_opt`/BUG-337) "Generalized Version" section,
a repo-wide grep for the exact phrase `"Just draw a large point in the middle of the screen"`
turned up 4 further hits beyond the 3 already-fixed instances (`attributes_vao`/BUG-318,
`attributes_instanced`/BUG-319, `spinning_cube_size_opt`/BUG-337): `make_cube_map`, `obj_load`,
`obj_viewer`, and `trivial`. Cross-checked against task #184's original fork assignment list:
`make_cube_map` was assigned to Fork B, `obj_load`+`obj_viewer` to Fork C, and `trivial` to Fork
D -- all four were explicitly in-scope, but none of the three forks flagged this defect in their
own assigned crates despite it being trivially greppable and already a known, repeatedly-recurring
pattern at that point in the bug-hunt.

Of the 4 hits, direct verification of `trivial/src/main.rs` and its shaders confirmed its doc
comment is **actually accurate** (`gl_PointSize = 250.0`, `gl_Position = vec4(0,0,0,1)`,
`gl.draw_arrays(GL::POINTS, 0, 1)` -- a genuine single large centered point) and is **not a
defect**; it was correctly excluded from this report after direct source verification rather than
trusted at face value from the grep match alone.

While fixing the remaining 3, a concurrent session/actor independently and correctly fixed
`obj_load` and `obj_viewer`'s doc comments mid-session (observed via file mtimes ~7 minutes apart
and an `Edit` tool rejection on a stale read) with different but equally accurate wording before
this session's own fix could be applied to those two files -- only `make_cube_map` was fixed
directly by this session.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p minwebgl_make_cube_map -p minwebgl_obj_load -p minwebgl_obj_viewer --test doc_comment_test --no-fail-fast
```
**Expected** (fixed): each doc comment describes its crate's actual demo (cube map generation /
OBJ loading / interactive OBJ viewer). **Actual** (pre-fix): all three claimed a single stationary
large point.

## Root Cause

Stale/copy-pasted description, carried over from an early template file and never updated as each
crate grew into its current, more complex form -- the same lineage already documented in
BUG-306/318/319/337.

## Why Not Caught

No test tied any of the three module doc comments' factual claims to their crate's actual
rendering logic -- the two drift independently since nothing cross-checks them. Additionally, the
per-crate fork review process used earlier in this bug-hunt (task #184) checked each assigned
crate's logic and shaders but did not systematically grep for this already-known stale-sentence
pattern across the full crate list, despite the pattern having already recurred 3 times by the
point these forks ran.

## Fix Applied (2026-08-18)

Corrected each of the 3 module doc comments to describe its crate's actual behavior:
- `make_cube_map` (fixed this session): describes the glTF-model-to-cube-map-texture generation
  pipeline and the textured orbiting cube that samples it.
- `obj_load` (fixed concurrently by another session/actor): describes Wavefront OBJ loading and
  parsing of vertices/normals/texcoords/face-indices.
- `obj_viewer` (fixed concurrently by another session/actor): describes the interactive OBJ
  viewer with orbit-camera rotation/zoom controls.

Added `tests/doc_comment_test.rs` to all 3 crates (none had a pre-existing `tests/` directory):
`include_str!` + substring assertions confirming each doc comment no longer claims a stationary
point and does describe its own crate's actual demo (cube map / OBJ / viewer respectively).

## Verification

- **Pre-fix (RED):** confirmed via
  `cargo test -p minwebgl_make_cube_map -p minwebgl_obj_load -p minwebgl_obj_viewer --test doc_comment_test --no-fail-fast`
  against the original stale doc comments -- all 3 new tests failed (stale claim detected in each).
- **Post-fix (GREEN):** same command -- all 3 new tests pass;
  `cargo clippy -p minwebgl_make_cube_map -p minwebgl_obj_load -p minwebgl_obj_viewer --all-targets --all-features -- -D warnings`
  and `cargo check --target wasm32-unknown-unknown -p minwebgl_make_cube_map -p minwebgl_obj_load -p minwebgl_obj_viewer`
  both clean.

## Generalized Version

Once a stale-doc-comment pattern is confirmed to recur across sibling crates (as it already had,
3 times, before this report), a targeted repo-wide grep for the exact offending phrase is strictly
more reliable than trusting each per-crate reviewer to notice it independently -- per-crate review
scales with reviewer attention, a single grep sweep scales with the pattern's own distinctiveness.
This is the second time in this same bug-hunt (task #184) that an orchestrator-level direct sweep
for a known defect's exact signature found instances every assigned fork missed (the first was the
`lil_gui.rs` "getTitle" defect, BUG-339); this recurrence is itself worth flagging as a systemic
gap in the per-fork review approach used for task #184, not merely two isolated coincidences.
Also: a grep hit should never be trusted as a confirmed defect without direct source verification
-- `trivial`'s identical-looking match was in fact accurate and correctly excluded from this
report after checking its actual shader/draw-call behavior.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found by the orchestrating session via a repo-wide grep sweep after fixing BUG-337, following up on a defect pattern already established by BUG-306/318/319/337. `make_cube_map` fixed directly by this session; `obj_load` and `obj_viewer` were found already fixed (independently, with different accurate wording) by a concurrent session/actor observed working the same repo during this session. `trivial`'s identical-looking match was verified accurate and excluded, not filed. Filed as BUG-340 after a fresh on-disk collision scan (highest prior ID: 337). Related: BUG-306, BUG-318, BUG-319, BUG-337 -- same stale-doc-comment lineage, independent crates. |
