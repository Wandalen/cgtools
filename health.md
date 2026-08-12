# cgtools health

Per-crate workspace health dashboard. Every column is a snapshot with its regeneration command —
re-run the command to refresh a number instead of trusting the table. Live work items are tracked in
[task/readme.md](task/readme.md); this file summarizes state, it does not duplicate the backlog.

- **Snapshot date:** 2026-08-10
- **Workspace build:** ✅ `cargo check --workspace --all-features` — exit 0, 57s, all crates
  (module/ + examples/) compile clean.
- **Task system:** 59 completed · 3 draft · 6 cancelled (see task/readme.md for the live table).

## Regeneration commands

| Column | Command |
|--------|---------|
| Build | `cargo check --workspace --all-features` |
| Tests (files / fns) | `find <crate>/tests -name "*.rs" \| wc -l` · `grep -rc "#\[ test \]\|#\[test\]" <crate>/tests` |
| Inline tests | `grep -rn "#\[ test \]\|#\[test\]" <crate>/src \| wc -l` |
| docs/ | `[ -d <crate>/docs ] && echo yes` |
| Markers | `grep -rn "xxx :\|xxx:\|qqq :\|qqq:\|aaa :\|aaa:\|TODO:" <crate> --include="*.rs" --include="*.toml" \| wc -l` |
| Allows | `grep -rn "#!\?\[ *allow(" <crate>/src <crate>/tests \| wc -l` |

## Per-crate state (module/, snapshot 2026-08-10)

| Crate | Tests (files/fns) | Inline tests | docs/ | Markers | Allows | Notes |
|-------|-------------------|--------------|-------|---------|--------|-------|
| alias/browser_tools | 1 / include | 0 | — | 0 | 0 | Runs browser_log's suite by path-include |
| alias/ndarray_tools | 1 / 257 | 0 | — | 0 | 0 | Runs ndarray_cg's full suite by path-include (enabled by task 038) |
| blank/cg_tools | stub | 0 | — | 0 | 0 | Placeholder; stale `Wandalen/cg_tools` repo URL |
| blank/cgtools | stub | 0 | — | 0 | 0 | Placeholder |
| blank/d3_scene | stub | 0 | — | 0 | 0 | Placeholder |
| blank/frame_graph | stub | 0 | — | 0 | 0 | Placeholder |
| blank/mdmath | 0 | 0 | — | 2 | 0 | Placeholder; template markers; stale repo URL |
| blank/mdmath_ai | 0 | 0 | — | 2 | 0 | Placeholder; template markers; stale repo URL |
| blank/mdmath_cg | 0 | 0 | — | 2 | 0 | Placeholder; template markers; stale repo URL |
| blank/mdmath_linalg | 0 | 0 | — | 2 | 0 | Placeholder; template markers; stale repo URL |
| helper/animation | 3 / 29 | 0 | — | 0 | 3 | Swept by task 058 (5 stale attrs deleted, 64 latent findings fixed; `EasingBuilder::new` renamed → `build`, user-approved, 94 call sites) |
| helper/behaviour_tree | 1 / 15 | 0 | — | 0 | — | Tests relocated by task 067 (14 inline → tests/, +1 new pin) |
| helper/browser_input | 2 / 13 | 0 | — | 0 | 6 | All 6 inline tests relocated to tests/pointer_type_test.rs by task 076 (fully public surface); tests/readme.md added · Decoupled from minwebgl by task 057 (math via ndarray_cg, JsCast via web-sys re-export, own Window/Document web-sys features) · draft-058 residue tranche 2026-08-11: 6 justified (3 expect + 3 cfg-dependent allow-with-reason), 3 stale duplicates deleted |
| helper/browser_log | 2 / 5 | 0 | — | 0 | 0 | Swept by task 058 (1 stale allow deleted) · panic.rs qqq markers closed by task 077 (panic_hook_test.rs: Config pins + real-panic native hook test; wasm-only formatting recorded as decision); duplicate `licence`+`license` files remain |
| helper/canvas_renderer | 0 | 1 | — | 0 | 0 | Audit-gated by task 058 (11 latent findings fixed, 0 suppressions) · Inline reproducer kept as documented exception by task 068 (public surface is all live-GL) |
| helper/embroidery_tools | 3 / 10 | 0 | — | 0 | 0 | Tests relocated by task 066 (8 inline → tests/, +2 new pins); workspace-lints inheritance wired (verified 2026-08-11) · draft-058 residue tranche 2026-08-11: all 12 stale central-family duplicates deleted, 0 attrs remain |
| helper/gpu_hal | 1 / 2 | 0 | — | 0 | 6 | HAL v0; buy-vs-build ADR closed; allows justified by task 058 (14 stale deleted, 6 combo-dependent kept) |
| helper/line_tools | 5 / 88 | 0 | yes | 0 | 4 | Swept by task 058 (151 latent lint errors fixed; 4 expect survive) |
| helper/primitive_generation | 2 / 5 | 0 | — | 0 | 3 | ufo.rs allows justified/fixed by task 036 · Swept by draft 058 increment 2026-08-11 (6 + 47 findings → 0 once minwebgl landed; curve/contour helpers decomposed, phantom Result dropped from make_buffer_attribute_info, stale test-file cast allow deleted; 3 expect survive; 6 downstream example call sites updated across 4 crates) |
| helper/renderer | 16 / 75 | 6 | yes | 0 | 27 | 6 inline tests kept as one documented exception by task 075 (private resolve_asset_uri URI helper of the browser-bound glTF loader; run natively) · Allow sweep by draft 058 increment: 87 → 42 (57-line lib.rs blanket wall stripped, ~60 mechanical fixes across 25 files, 8 # Errors + 12 gbuffer docs written; 9 crate-policy + 33 scoped justified attrs remain, each with a reason; user directive applied mid-increment — fix over allow wherever mechanical, 9 first-pass allows converted to real fixes incl. 2 dead Camera fields deleted) · Policy package 2026-08-11: crate policy block deleted (families central in root Cargo.toml), 33 scoped attrs = 26 expect + 7 allow, all with reason; ~52 # Errors/# Panics sections written instead of suppressing (missing_errors_doc/missing_panics_doc stay warned) · Decomposition package 2026-08-11: all 6 too_many_lines expects eliminated by real fn splits (skeleton upload, transition set, FramebufferContext::new, render, WebGpuRenderer::new, glTF load → 21 named helpers), scoped attrs now 27 = 20 expect + 7 allow |
| helper/scene_script | 2 / 16 | 0 | yes | 0 | 0 | Swept by task 058 (1 stale allow deleted) |
| helper/tilemap_renderer | 6 / 100 | 29 | yes | 0 | 5 | Markers resolved by task 064 (Source::Path geometry loading implemented, pitfall/003 retired) · 54 tests relocated to tests/ by task 071 (29 inline kept as documented exceptions pinning private helpers) · Allow re-sweep by draft 058 increment 2026-08-11: 44 stale central-family duplicates deleted + 1 converted to expect-with-reason + 2 test-file crate docs moved above `#![ cfg ]`; src/ minus webgl.rs now carries exactly 1 justified expect, tests/ 0 · webgl.rs backlog closed 2026-08-11 (fixed in concurrent minwebgl-tranche work; verified by draft 058 gates — enabled,adapter-webgl + --all-features clippy green, suite 128/128; webgl.rs now 4 reasoned allows) |
| helper/tilemap_scene | 12 / 171 | 0 | yes | 0 | 15 | All 38 inline tests relocated to 2 new domain files by task 073 (hash_test.rs + compile_units_test.rs; zero exceptions — mod_interface root re-exports reach everything); tests/readme.md added · Swept by draft 058 increment 2026-08-11 (35 findings across lib + 7 test binaries; project_to_transform dedup, sampler-type re-exports, 15 justified attrs = 5 expect + 10 allow-with-reason; suite 169/169) |
| helper/tiles_tools | 18 / 241 | 5 | yes | 0 | 19 | Markers resolved by task 063 (movement queue implemented, pitfall/002 retired) · 46 tests relocated to 7 new feature-mirrored tests/ files by task 072 (5 inline kept as documented exceptions; 5 fov duplicates consolidated) · Flowfield integration module revived by task 078 (5 of 21 dead tests repaired to hex + Ord on hexagonal::Coordinate un-deadened calculate_flow; 16 retired with named reasons — fn count dropped because the old grep counted tests that never ran) · Allow sweep by draft 058 increment: 460 → 38 (449 blanket lines stripped, ~330 machine + ~30 manual fixes, missing_docs + 17 # Errors sections cleared; 37 justified attrs remain, each with a reason — 1 of the 38 matches is a comment mention) · Re-greened under expect regime 2026-08-11 (committed state gated red under the new ratchet): 23 format_push_string → write!/writeln!, 2 must_use, 14 bench --fix sites, 8 similar-name renames, 1 match-arms merge, 2 stale cast attrs deleted; 18 attrs remain (15 expect + 3 allow incl. 2 file-level criterion missing_docs), all with reason — 1 of the 19 matches is a comment mention) |
| math/mdmath_core | 23 / 89 | 0 | — | 2 | 44 | Markers resolved by task 059 (soundness unsafe removed, Ix4 added); 2 lint markers → draft 058; workspace-lints inheritance wired (verified 2026-08-11) · draft-058 residue tranche 2026-08-11: 44 justified, all machine-checked expects |
| math/ndarray_cg | 36 / 222 | 0 | — | 2 | 22 | Markers resolved by task 060 (typed TryFrom error, IntoVector tests); 2 lint markers → draft 058; suite shared with ndarray_tools; workspace-lints inheritance wired (verified 2026-08-11) · draft-058 residue tranche 2026-08-11: 22 justified, all machine-checked expects; 2 stale duplicates deleted |
| min/mingl | 6 / 38 | 13 | — | 0 | 1 | Markers resolved by task 061 · 13 inline tests kept as one documented exception by task 074 (private pure URL helpers of the wasm-only web loader; proven to run natively under --all-features) · draft-058 residue tranche 2026-08-11: 10 stale central-family duplicates deleted, 1 `deprecated` expect remains |
| min/minwebgl | 1 / 2 | 4 | yes | 0 | 11 | Markers resolved by task 062 · Runnability story + native data_type tests by task 069 (4 inline kept as documented exceptions) · Sweep landed 2026-08-11, verified by draft 058 gates (host --all-targets + wasm32 --lib); 6 stale central-family duplicates deleted, unexpected_cfgs allow replaced by workspace check-cfg declaration; 11 reasoned allows remain |
| min/minwebgpu | 1 / 6 | 0 | yes | 0 | 0 | Sweep landed 2026-08-11, verified by draft 058 gates (wasm32 --lib + host; 3 findings fixed incl. copy-paste FailedToCreateRenderPipeline → FailedToCreateComputePipeline in compute_pipeline.rs); 6 stale cast duplicates deleted → 0 attrs; wasm32 --all-targets still BUG-079-blocked (getrandom) |
| min/minwgpu | 2 / 5 | 21 | yes | 0 | — | Deterministic adapter-error tests/ established by task 070 (21 inline kept as documented exceptions — pub(super) builder internals) |

Notes column links go through task/readme.md; `—` in Allows = not in the top-count sweep (small or
zero). Examples tree (72 demo crates — see examples/readme.md) is intentionally not tabulated
per-crate: demos carry no tests/ requirement; their 13 open markers are triaged in draft 065 (6 need
human decisions, including two `rid of this crate` calls on `diamond` and `make_cube_map`).

## Known issues (workspace level)

- **Shader validation tooling absent:** `glslangValidator` is not installed on this machine, which
  blocks offline GLSL validation work. Fix: `sudo apt install glslang-tools`, verify with
  `glslangValidator --version`.
- **5 blank/ crates carry a stale repository URL** (`Wandalen/cg_tools`): cg_tools, mdmath,
  mdmath_ai, mdmath_cg, mdmath_linalg. Verify: `grep -rn "Wandalen/cg_tools" module/*/*/Cargo.toml`.
- **browser_log ships both `licence` and `license` files** and its changelog claims are unverified.
  Verify: `ls module/helper/browser_log/ | grep -i licen`.
- **Lint-policy stragglers: none.** All module/ crates inherit `[workspace.lints]` (mdmath_core,
  ndarray_cg, embroidery_tools verified wired 2026-08-11), and the last 23 stragglers — the
  minwebgl demo crates (the earlier "~43" figure was a stale estimate) — were wired and gated
  green under wasm32 `-D warnings` 2026-08-11 (draft 058 History). Every example crate now
  inherits. Verify: `grep -rL "^\[lints\]" examples/*/*/Cargo.toml module/*/*/Cargo.toml`
  (empty output = all wired).

## Open work streams (details in task/readme.md)

- **058** — per-crate `#[allow]` justification sweep (census + procedure embedded).
- **065** — task-marker resolution, examples tranche (the per-crate tranches 059–064 are all
  done; 065 needs human decisions on two `rid of this crate` calls).
- **Test-coverage stream (035 decomposition) COMPLETE** — 066–077 closed embroidery_tools,
  behaviour_tree, canvas_renderer, minwebgl, minwgpu, tilemap_renderer, tiles_tools,
  tilemap_scene, mingl, renderer, browser_input, and browser_log; every crate's inline tests
  are relocated or documented exceptions. Follow-up 078 (tiles_tools' disabled flowfield
  integration module) closed too — zero disabled tests remain in the workspace.
- **056** — vectorizer revival watch item.

