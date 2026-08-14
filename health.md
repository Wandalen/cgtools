# cgtools health

Per-crate workspace health dashboard. Every column is a snapshot with its regeneration command —
re-run the command to refresh a number instead of trusting the table. Live work items are tracked in
[task/readme.md](task/readme.md); this file summarizes state, it does not duplicate the backlog.

- **Snapshot date:** 2026-08-13
- **Workspace build:** ✅ `cargo check --workspace --all-features` — exit 0, 108s, all crates
  (module/ + examples/) compile clean.
- **Task system:** 76 completed · 2 draft · 7 cancelled · 6 executed · 1 accepting · 1 verified (see
  task/readme.md for the live table).

## Regeneration commands

| Column | Command |
|--------|---------|
| Build | `cargo check --workspace --all-features` |
| Tests (files / fns) | `find <crate>/tests -name "*.rs" \| wc -l` · `grep -rc "#\[ test \]\|#\[test\]" <crate>/tests` |
| Inline tests | `grep -rn "#\[ test \]\|#\[test\]" <crate>/src \| wc -l` |
| docs/ | `[ -d <crate>/docs ] && echo yes` |
| Markers | `grep -rn "xxx :\|xxx:\|qqq :\|qqq:\|aaa :\|aaa:\|TODO:" <crate> --include="*.rs" --include="*.toml" \| wc -l` |
| Allows | `grep -rn "#!\?\[ *allow(" <crate>/src <crate>/tests \| wc -l` |

## Per-crate state (module/, snapshot 2026-08-13)

| Crate | Tests (files/fns) | Inline tests | docs/ | Markers | Allows | Notes |
|-------|-------------------|--------------|-------|---------|--------|-------|
| alias/browser_tools | 1 / include | 0 | — | 0 | 0 | Runs browser_log's suite by path-include |
| alias/ndarray_tools | 1 / include | 0 | — | 0 | 0 | Runs ndarray_cg's full suite by path-include (enabled by task 038) |
| blank/cg_tools | stub | 0 | — | 0 | 0 | Placeholder; stale `Wandalen/cg_tools` repo URL fixed |
| blank/cgtools | stub | 0 | — | 0 | 0 | Placeholder |
| blank/d3_scene | stub | 0 | — | 0 | 0 | Placeholder |
| blank/frame_graph | stub | 0 | — | 0 | 0 | Placeholder |
| blank/mdmath | 0 | 0 | — | 2 | 0 | Placeholder; template markers |
| blank/mdmath_ai | 0 | 0 | — | 2 | 0 | Placeholder; template markers |
| blank/mdmath_cg | 0 | 0 | — | 2 | 0 | Placeholder; template markers |
| blank/mdmath_linalg | 0 | 0 | — | 2 | 0 | Placeholder; template markers |
| helper/animation | 3 / 29 | 0 | — | 0 | 0 | Swept by task 058 (5 stale attrs deleted, 64 latent findings fixed; `EasingBuilder::new` renamed → `build`, user-approved, 94 call sites) |
| helper/behaviour_tree | 1 / 15 | 0 | — | 0 | 0 | Tests relocated by task 067 (14 inline → tests/, +1 new pin) |
| helper/browser_input | 2 / 13 | 0 | — | 0 | 3 | All 6 inline tests relocated to tests/pointer_type_test.rs by task 076 (fully public surface); tests/readme.md added · Decoupled from minwebgl by task 057 (math via ndarray_cg, JsCast via web-sys re-export, own Window/Document web-sys features) · draft-058 residue tranche 2026-08-11: 6 justified (3 expect + 3 cfg-dependent allow-with-reason), 3 stale duplicates deleted |
| helper/browser_log | 2 / 5 | 0 | — | 0 | 0 | Swept by task 058 (1 stale allow deleted) · panic.rs qqq markers closed by task 077 (panic_hook_test.rs: Config pins + real-panic native hook test; wasm-only formatting recorded as decision); `licence`/`license` duplication fixed (only `license` remains) |
| helper/canvas_renderer | 1 / 1 | 0 | — | 0 | 0 | Audit-gated by task 058 (11 latent findings fixed, 0 suppressions) · Inline reproducer previously kept as a documented exception by task 068 — since relocated to a real tests/ file (0 inline remain, 2026-08-13) |
| helper/embroidery_tools | 3 / 10 | 0 | — | 0 | 0 | Tests relocated by task 066 (8 inline → tests/, +2 new pins); workspace-lints inheritance wired (verified 2026-08-11) · draft-058 residue tranche 2026-08-11: all 12 stale central-family duplicates deleted, 0 attrs remain |
| helper/gpu_hal | 1 / 3 | 0 | yes | 0 | 7 | HAL v0; buy-vs-build ADR closed (docs/adr/002); allows justified by task 058 (14 stale deleted, 6 combo-dependent kept); 19 further clippy::pedantic violations fixed by task 102 (blocking task 086) |
| helper/line_tools | 5 / 88 | 0 | yes | 0 | 0 | Swept by task 058 (151 latent lint errors fixed; expect survivors since cleared) |
| helper/primitive_generation | 2 / 5 | 0 | — | 0 | 0 | ufo.rs allows justified/fixed by task 036 · Swept by draft 058 increment 2026-08-11 (6 + 47 findings → 0 once minwebgl landed; curve/contour helpers decomposed, phantom Result dropped from make_buffer_attribute_info, stale test-file cast allow deleted; 6 downstream example call sites updated across 4 crates) |
| helper/renderer | 17 / 81 | 0 | yes | 0 | 7 | 6 inline tests previously kept as a documented exception by task 075 (private resolve_asset_uri URI helper) — since relocated (0 inline remain, 2026-08-13) · Allow sweep by draft 058 increment: 87 → 42 (57-line lib.rs blanket wall stripped, ~60 mechanical fixes across 25 files, 8 # Errors + 12 gbuffer docs written) · Policy package 2026-08-11: crate policy block deleted (families central in root Cargo.toml), ~52 # Errors/# Panics sections written instead of suppressing · Decomposition package 2026-08-11: all 6 too_many_lines expects eliminated by real fn splits (skeleton upload, transition set, FramebufferContext::new, render, WebGpuRenderer::new, glTF load → 21 named helpers) · Full clippy::pedantic sweep, tasks 083-093 (~730 issues across every webgl/webgpu/material/animation/post_processing/loaders file): attrs down to 7 reasoned allows, remainder converted to real fixes or `#[expect]` |
| helper/scene_script | 2 / 51 | 0 | yes | 0 | 0 | Swept by task 058 (1 stale allow deleted) · Implements the script-as-data pattern (docs/pattern/004); `codename_space_sandbox` (cross-repo) adopted it as a second consumer via that repo's own task 007 (✅ Completed) · task 107 (🎯 Verified, concurrent/in-progress as of this snapshot) proposes a whole-AST purity check closing the gap `top_level_lint` leaves — calls hidden inside nested control-flow bodies |
| helper/tilemap_renderer | 9 / 133 | 0 | yes | 0 | 4 | Markers resolved by task 064 (Source::Path geometry loading implemented, pitfall/003 retired) · 54 tests relocated to tests/ by task 071; 29 inline previously kept as documented exceptions — since relocated (0 inline remain, 2026-08-13) · Allow re-sweep by draft 058 increment 2026-08-11 · webgl.rs backlog closed 2026-08-11 (verified by draft 058 gates — enabled,adapter-webgl + --all-features clippy green, suite 128/128); ImageSource::Encoded decoding added by tasks 092/093 (Q-02) |
| helper/tilemap_scene | 12 / 171 | 0 | yes | 0 | 10 | All 38 inline tests relocated to 2 new domain files by task 073 (hash_test.rs + compile_units_test.rs; zero exceptions — mod_interface root re-exports reach everything); tests/readme.md added · Swept by draft 058 increment 2026-08-11 (35 findings across lib + 7 test binaries; project_to_transform dedup, sampler-type re-exports; suite 169/169) |
| helper/tiles_tools | 18 / 246 | 0 | yes | 0 | 1 | Markers resolved by task 063 (movement queue implemented, pitfall/002 retired) · 46 tests relocated to 7 new feature-mirrored tests/ files by task 072; 5 inline previously kept as documented exceptions — since relocated (0 inline remain, 2026-08-13) · Flowfield integration module revived by task 078 (5 of 21 dead tests repaired; 16 retired with named reasons) · Allow sweep by draft 058 increment: 460 → 38, then re-greened 2026-08-11 under the expect regime (23 format_push_string → write!/writeln!, 2 must_use, 14 bench --fix sites, 8 similar-name renames); remaining justified attrs since converted almost entirely to `#[expect]` (1 allow survives) |
| math/mdmath_core | 23 / 89 | 0 | — | 0 | 0 | Markers resolved by task 059 (soundness unsafe removed, Ix4 added); workspace-lints inheritance wired (verified 2026-08-11) · draft-058 residue tranche 2026-08-11 justified all suppressions as machine-checked — since fully expressed as `#[expect]` (39), 0 plain `#[allow]` remain; see Regeneration commands caveat below |
| math/ndarray_cg | 37 / 223 | 0 | — | 1 | 0 | Markers resolved by task 060 (typed TryFrom error, IntoVector tests); suite shared with ndarray_tools; workspace-lints inheritance wired (verified 2026-08-11) · draft-058 residue tranche 2026-08-11 justified all suppressions — since fully expressed as `#[expect]` (10), 0 plain `#[allow]` remain; see Regeneration commands caveat below |
| min/mingl | 7 / 54 | 0 | — | 0 | 1 | Markers resolved by task 061 · 13 inline tests previously kept as a documented exception by task 074 (private pure URL helpers) — since relocated (0 inline remain, 2026-08-13) · draft-058 residue tranche 2026-08-11: 10 stale central-family duplicates deleted, 1 `deprecated` expect remains |
| min/minwebgl | 3 / 6 | 0 | yes | 0 | 11 | Markers resolved by task 062 · Runnability story + native data_type tests by task 069; 4 inline previously kept as documented exceptions — since relocated (0 inline remain, 2026-08-13) · Sweep landed 2026-08-11, verified by draft 058 gates (host --all-targets + wasm32 --lib); unexpected_cfgs allow replaced by workspace check-cfg declaration; 11 reasoned allows remain |
| min/minwebgpu | 1 / 0 | 0 | yes | 0 | 0 | Sweep landed 2026-08-11, verified by draft 058 gates (wasm32 --lib + host; 3 findings fixed incl. copy-paste FailedToCreateRenderPipeline → FailedToCreateComputePipeline in compute_pipeline.rs); 6 stale cast duplicates deleted → 0 attrs; wasm32 --all-targets still BUG-079-blocked (getrandom) |
| min/minwgpu | 4 / 30 | 0 | yes | 0 | 0 | Deterministic adapter-error tests/ established by task 070; 21 inline previously kept as documented exceptions (pub(super) builder internals) — since relocated (0 inline remain, 2026-08-13) |
| shader/shader_chunks | 2 / 68 | 0 | yes | 0 | 0 | WGSL chunk library + `sch`/`shader_chunks` unilang CLI (renamed from `shader_chunks_cli` by task 102). Bundles hash21/value_noise/fbm3/fullscreen_triangle, each its own subdirectory with a per-chunk readme.md (task ~165-168 restructure). CLI gained a `tunables` command (task 106, discovers `//@ param:` lines via shader_chunks_params) |
| shader/shader_chunks_core | 4 / 29 | 0 | yes | 0 | 0 | Shared chunk-manifest parsing (`//@ name:`/`description:`/`tags:`/`depends_on:`/`export:`) extracted out of shader_chunks into its own crate so shader_chunks_params could reuse the same header-comment parser without duplicating it |
| shader/shader_chunks_params | 2 / 25 | 0 | yes | 0 | 0 | New crate (task 105, Q-03 decision): discovers repeatable `//@ param: <name> <kind> <type> [range(min,max)]` lines; resolves a missing range via a deterministic 2-stage heuristic (name-substring pattern, then WGSL-type-keyed default), tagging each result `RangeSource::Declared`/`Inferred` |

Notes column links go through task/readme.md; `—` in Allows = not in the top-count sweep (small or
zero). **Allows-column caveat (2026-08-13):** the regeneration command only matches literal
`#[allow(...)]`; several crates (mdmath_core, ndarray_cg, renderer, tiles_tools among them) have
since converted most or all of their justified suppressions to `#[expect(...)]` (fails loudly if the
lint stops firing), which this column does not count — a low or zero Allows value no longer implies
zero suppression attributes for those crates. To see the current expect-count for a crate:
`grep -rn '#!\?\[ *expect(' <crate>/src <crate>/tests | wc -l`. Examples tree (70 demo crates —
recount: `find examples -name Cargo.toml | wc -l`) is intentionally not tabulated per-crate: demos
carry no tests/ requirement; their marker triage closed with task 065 (✅ Completed) — task 065
decided keep-crate for `diamond` and `make_cube_map`, and tasks 094/095 deleted the two stale
`rid of this crate` markers from their manifests.

## Known issues (workspace level)

- **Shader validation tooling absent:** `glslangValidator` is not installed on this machine, which
  blocks offline GLSL validation work. Fix: `sudo apt install glslang-tools`, verify with
  `glslangValidator --version`.
- **Lint-policy stragglers: none.** All module/ crates inherit `[workspace.lints]` (mdmath_core,
  ndarray_cg, embroidery_tools verified wired 2026-08-11), and the last 23 stragglers — the
  minwebgl demo crates (the earlier "~43" figure was a stale estimate) — were wired and gated
  green under wasm32 `-D warnings` 2026-08-11 (draft 058 History). Every example crate now
  inherits. Verify: `grep -rL "^\[lints\]" examples/*/*/Cargo.toml module/*/*/Cargo.toml`
  (empty output = all wired).

*(Two previously-tracked issues are now fixed and dropped from this list: the stale
`Wandalen/cg_tools` repo URL across 5 blank/ crates, and browser_log's duplicate `licence`/`license`
files — verified clean 2026-08-13.)*

## Open work streams (details in task/readme.md)

- **056** — vectorizer revival watch item (📝 Draft; explicitly YAGNI-deferred, no action unless a
  real consumer emerges).
- **098** — obj_viewer example proposal watch item (📝 Draft; same YAGNI-deferred pattern).
- **094, 095, 096, 097, 106** — 📦 Executed, implementation complete and self-verified, but blocked
  on independent acceptance: `tsk .acceptance_pass` mechanically refuses same-session
  self-acceptance (Separation of Concerns, TA142/BUG-197) — needs a human (or a genuinely separate
  actor identity) to run the acceptance gate before these reach ✅ Completed.
- **105** — 🔎 Accepting, one step further than the group above: the acceptance walk itself is
  already complete (`## Outcomes` — Verdict PASS, 20/20 Checklist/Measurements/Invariants/
  Anti-faking items, with a B1 disclosure on the coarse actor-identity collision). `tsk
  .acceptance_pass 105` was attempted directly and confirmed still mechanically refused
  ("self-verification forbidden (actor matches executing_by)") — same root cause as the group
  above, just already past the walk and waiting only on the CLI transition itself.
- **107** — `scene_script` whole-AST purity check, 🎯 Verified. Filed and verified by a different
  concurrent actor in this same workspace (not this session) — as of this snapshot it has fresh
  uncommitted edits (a cross-repo status correction), so treat it as actively owned rather than
  free to claim.

*(058 and 065, previously listed here, are now ✅ Completed — dropped from this list; see their
Notes-column entries in the per-crate table above for what each one closed.)*

