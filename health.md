# cgtools health

Per-crate workspace health dashboard. Every column is a snapshot with its regeneration command —
re-run the command to refresh a number instead of trusting the table. Live work items are tracked in
[task/readme.md](task/readme.md); this file summarizes state, it does not duplicate the backlog.

- **Snapshot date:** 2026-08-19
- **Workspace build:** ✅ `cargo check --workspace --all-features --exclude orrery_flexible` — exit 0,
  re-confirmed 2026-08-19 (`longrun`-detached, 463s — briefly blocked on a build-directory file lock
  held by concurrent Fleet activity before proceeding). The bare `--workspace --all-features` form
  this row previously documented still fails unconditionally —
  `orrery_flexible`'s 4 backend features (webgl/webgpu/wgpu/vulkan) are mutually exclusive by design
  (`compile_error!` guard, see docs/adr/004), and `--all-features` enables all 4 at once.
  `--exclude orrery_flexible` mirrors what `verb/test` itself already does for this crate; see that
  script's own comment above its native stages for the full per-feature check list.
- **Task system:** 90 completed · 16 draft · 12 cancelled · 8 accepting · 47 verifying · 23 open bugs
  (see task/readme.md for the live table; task counts re-derived 2026-08-20 via
  `grep -oE '\| (✅|🔎|📝|🚫|❓|🔬|⚙️|📦) \([A-Za-z]+\)' task/readme.md | sort | uniq -c`; bug count via
  `awk '/^## Open Bugs/,/^## Closed Bugs/' task/bug/readme.md | grep -c '^| BUG-'`, cross-checked
  against `task/bug/verified/`'s file count).

## Regeneration commands

| Column | Command |
|--------|---------|
| Build | `cargo check --workspace --all-features --exclude orrery_flexible` |
| Tests (files / fns) | `find <crate>/tests -name "*.rs" \| wc -l` · `grep -rc "#\[ test \]\|#\[test\]" <crate>/tests` |
| Inline tests | `grep -rn "#\[ test \]\|#\[test\]" <crate>/src \| wc -l` |
| docs/ | `[ -d <crate>/docs ] && echo yes` |
| Markers | `grep -rn "xxx :\|xxx:\|qqq :\|qqq:\|aaa :\|aaa:\|TODO:" <crate> --include="*.rs" --include="*.toml" \| wc -l` |
| Allows | `grep -rn "#!\?\[ *allow(" <crate>/src <crate>/tests \| wc -l` |

## Per-crate state (module/, snapshot 2026-08-14 for the shader/ rows — plus 2 rows added 2026-08-16,
shader_chunks_render/shader_chunks_render_core, previously missing — 2026-08-13 for the rest)

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
| helper/gpu_hal | 2 / 22 | 1 | yes | 0 | 12 | HAL v0; buy-vs-build ADR closed (docs/adr/002); allows justified by task 058 (14 stale deleted, 6 combo-dependent kept); 19 further clippy::pedantic violations fixed by task 102 (blocking task 086) · Grew substantially since (tasks 087-090, 191, 201-203, 206, 358): native/vulkan backend arms, write_texture/context-loss handling, browser pixel verification, crate-wide 50-line function-length split; sole inline test is a deliberate exception (`pub(crate)` panic contract `expect_vulkan` unreachable from tests/, task 202 T04) |
| helper/line_tools | 5 / 88 | 0 | yes | 0 | 0 | Swept by task 058 (151 latent lint errors fixed; expect survivors since cleared) |
| helper/primitive_generation | 2 / 5 | 0 | — | 0 | 0 | ufo.rs allows justified/fixed by task 036 · Swept by draft 058 increment 2026-08-11 (6 + 47 findings → 0 once minwebgl landed; curve/contour helpers decomposed, phantom Result dropped from make_buffer_attribute_info, stale test-file cast allow deleted; 6 downstream example call sites updated across 4 crates) |
| helper/renderer | 28 / 133 | 0 | yes | 0 | 7 | 6 inline tests previously kept as a documented exception by task 075 (private resolve_asset_uri URI helper) — since relocated (0 inline remain, 2026-08-13) · Allow sweep by draft 058 increment: 87 → 42 (57-line lib.rs blanket wall stripped, ~60 mechanical fixes across 25 files, 8 # Errors + 12 gbuffer docs written) · Policy package 2026-08-11: crate policy block deleted (families central in root Cargo.toml), ~52 # Errors/# Panics sections written instead of suppressing · Decomposition package 2026-08-11: all 6 too_many_lines expects eliminated by real fn splits (skeleton upload, transition set, FramebufferContext::new, render, WebGpuRenderer::new, glTF load → 21 named helpers) · Full clippy::pedantic sweep, tasks 083-093 (~730 issues across every webgl/webgpu/material/animation/post_processing/loaders file): attrs down to 7 reasoned allows, remainder converted to real fixes or `#[expect]` · Test count grew further via later rounds not previously reflected here — browser pixel-verified opaque-path example (task 197), task 223's gltf_animation_loader native test, and 3 concurrent-actor bug fixes each with their own new native unit tests (BUG-252 displacement-texture division-by-zero ×3, BUG-253 projection-matrix validation bypass ×3, BUG-255 spot-light equal-cone-angle NaN ×4) |
| helper/scene_script | 2 / 51 | 0 | yes | 0 | 0 | Swept by task 058 (1 stale allow deleted) · Implements the script-as-data pattern (docs/pattern/004); `codename_space_sandbox` (cross-repo) adopted it as a second consumer via that repo's own task 007 (✅ Completed) · task 107 (🎯 Verified, concurrent/in-progress as of this snapshot) proposes a whole-AST purity check closing the gap `top_level_lint` leaves — calls hidden inside nested control-flow bodies |
| helper/tilemap_renderer | 11 / 149 | 0 | yes | 0 | 4 | Markers resolved by task 064 (Source::Path geometry loading implemented, pitfall/003 retired) · 54 tests relocated to tests/ by task 071; 29 inline previously kept as documented exceptions — since relocated (0 inline remain, 2026-08-13) · Allow re-sweep by draft 058 increment 2026-08-11 · webgl.rs backlog closed 2026-08-11 (verified by draft 058 gates — enabled,adapter-webgl + --all-features clippy green, suite 128/128); ImageSource::Encoded decoding added by tasks 092/093 (Q-02) · Test count grew further via later rounds not previously reflected here — adapter-webgpu/webgl browser pixel-verified examples (task 251, renumbered from 198 2026-08-17 — bug/task ID collision with BUG-198), task 218's WebGPU adapter real pixel-upload test |
| helper/tilemap_scene | 13 / 179 | 0 | yes | 0 | 10 | All 38 inline tests relocated to 2 new domain files by task 073 (hash_test.rs + compile_units_test.rs; zero exceptions — mod_interface root re-exports reach everything); tests/readme.md added · Swept by draft 058 increment 2026-08-11 (35 findings across lib + 7 test binaries; project_to_transform dedup, sampler-type re-exports; suite 169/169) · re-measured 2026-08-17: 13/179 (+1 file/+8 fns since the 2026-08-11 snapshot — includes 3 same-day bug-fix tests, BUG-263/264/265) |
| helper/tiles_tools | 18 / 246 | 0 | yes | 0 | 1 | Markers resolved by task 063 (movement queue implemented, pitfall/002 retired) · 46 tests relocated to 7 new feature-mirrored tests/ files by task 072; 5 inline previously kept as documented exceptions — since relocated (0 inline remain, 2026-08-13) · Flowfield integration module revived by task 078 (5 of 21 dead tests repaired; 16 retired with named reasons) · Allow sweep by draft 058 increment: 460 → 38, then re-greened 2026-08-11 under the expect regime (23 format_push_string → write!/writeln!, 2 must_use, 14 bench --fix sites, 8 similar-name renames); remaining justified attrs since converted almost entirely to `#[expect]` (1 allow survives) |
| math/mdmath_core | 23 / 89 | 0 | — | 0 | 0 | Markers resolved by task 059 (soundness unsafe removed, Ix4 added); workspace-lints inheritance wired (verified 2026-08-11) · draft-058 residue tranche 2026-08-11 justified all suppressions as machine-checked — since fully expressed as `#[expect]` (39), 0 plain `#[allow]` remain; see Regeneration commands caveat below |
| math/ndarray_cg | 37 / 223 | 0 | — | 1 | 0 | Markers resolved by task 060 (typed TryFrom error, IntoVector tests); suite shared with ndarray_tools; workspace-lints inheritance wired (verified 2026-08-11) · draft-058 residue tranche 2026-08-11 justified all suppressions — since fully expressed as `#[expect]` (10), 0 plain `#[allow]` remain; see Regeneration commands caveat below |
| min/mingl | 7 / 54 | 0 | — | 0 | 1 | Markers resolved by task 061 · 13 inline tests previously kept as a documented exception by task 074 (private pure URL helpers) — since relocated (0 inline remain, 2026-08-13) · draft-058 residue tranche 2026-08-11: 10 stale central-family duplicates deleted, 1 `deprecated` expect remains |
| min/minwebgl | 3 / 6 | 0 | yes | 0 | 11 | Markers resolved by task 062 · Runnability story + native data_type tests by task 069; 4 inline previously kept as documented exceptions — since relocated (0 inline remain, 2026-08-13) · Sweep landed 2026-08-11, verified by draft 058 gates (host --all-targets + wasm32 --lib); unexpected_cfgs allow replaced by workspace check-cfg declaration; 11 reasoned allows remain |
| min/minwebgpu | 1 / 0 | 0 | yes | 0 | 0 | Sweep landed 2026-08-11, verified by draft 058 gates (wasm32 --lib + host; 3 findings fixed incl. copy-paste FailedToCreateRenderPipeline → FailedToCreateComputePipeline in compute_pipeline.rs); 6 stale cast duplicates deleted → 0 attrs; wasm32 --all-targets still BUG-079-blocked (getrandom) |
| min/minwgpu | 4 / 30 | 0 | yes | 0 | 0 | Deterministic adapter-error tests/ established by task 070; 21 inline previously kept as documented exceptions (pub(super) builder internals) — since relocated (0 inline remain, 2026-08-13) |
| shader/shader_chunks | 1 / 32 | 0 | yes | 0 | 0 | Aggregation only (2026-08-14 CLI split, was one monolithic crate): `run()` concatenates query→compose→params→preview command sets/help and hands off to shader_chunks_cli_core::run; `src/bin/{shader_chunks,sch}.rs` are one-line delegates. Monolithic `shader_chunks_test.rs` removed, replaced by `cli_subprocess_test.rs` (aggregation-order + help-screen pins). Still bundles hash21/value_noise/fbm3/fullscreen_triangle WGSL chunks, each its own subdirectory with a per-chunk readme.md. See [module/shader/readme.md](module/shader/readme.md) for the full family map |
| shader/shader_chunks_cli_core | 1 / 2 | 0 | — | 0 | 0 | New crate (2026-08-14 CLI split): shared unilang dispatch/help/exit-code layer used by all 5 CLI crates (aggregator + query/compose/params/preview) — `CommandSet`, `CliApp`, `run` (help-spelling routing, exit-code mapping per BUG-103), EPIPE-safe `stdout_print`/`stderr_print` (BUG-108), `names_flatten` (works around unilang's nested List-of-List binding quirk) |
| shader/shader_chunks_compose | 1 / 6 | 0 | — | 0 | 0 | New crate (2026-08-14, split out of the old monolithic shader_chunks CLI): CLI and logic for the `compose` command in one file — deliberately has no separate `_core`, since shader_chunks_core itself is thin enough to serve as this utility's core. `ComposeCliError` (UnknownChunk/Compose, both exit 1) |
| shader/shader_chunks_core | 4 / 33 | 0 | yes | 0 | 0 | Shared chunk-manifest parsing (`//@ name:`/`description:`/`tags:`/`depends_on:`/`export:`), reused by shader_chunks_params_core/query_core/preview_core without duplicating the header-comment parser. +4 tests (2026-08-14): gained `set_resolve` (name-set → descriptors, optional transitive-dependency closure), a shared resolver consolidating logic that would otherwise be duplicated between shader_chunks_compose and shader_chunks_preview_core |
| shader/shader_chunks_params | 1 / 3 | 0 | — | 0 | 0 | Thinned 2026-08-14 from the former combined discovery+CLI crate down to CLI wiring only for the `tunables` command — `ParamsCliError` (UnknownChunk exit 1, Render exit 2) over shader_chunks_params_core below. Its former 2/25 engine tests moved wholesale to that new crate, not lost |
| shader/shader_chunks_params_core | 2 / 25 | 0 | yes | 0 | 0 | New crate (2026-08-14, split out of the old shader_chunks_params): `//@ param:` tunable-parameter discovery engine, carrying forward that crate's full test suite and `docs/` (`algorithm/` + `api/`, migrated intact). Task 105/Q-03 range-inference heuristic unchanged: declared range wins, else a 2-stage heuristic (name-substring pattern, then WGSL-type-keyed default), tagging each result `RangeSource::Declared`/`Inferred` |
| shader/shader_chunks_preview | 1 / 8 | 0 | — | 0 | 0 | New crate (2026-08-14): CLI wiring for `preview` — `bundle_prepare` (build via shader_chunks_preview_core, then naga-validate before any write) → `bundle_write` → serve (browser dev-server, default) or summary-only. Generalizes the 3-slider live-preview capability task 112 first delivered as a hardcoded single-purpose example (that example is now superseded and deleted, see task 112's closing NOTE) to any bundled or `file::`-supplied chunk. Disclosed test gaps (see `tests/docs/cli/command/cmd_007_preview.md`): no test exercises a successful `file::` target read, and none covers giving both `name` and `file::` together (only the neither-given arm is tested) |
| shader/shader_chunks_preview_core | 1 / 11 | 0 | — | 0 | 0 | New crate (2026-08-14): builds a composed, slider-annotated preview bundle from one chunk — two modes (declared-param fragment chunk, or synthesized-harness value chunk); `resolution_index` computes the 16-byte-boundary uniform layout shared with the wasm runner below. None of the 4 bundled chunks are fragment-mode today; that path is exercised only by this crate's own fixture tests |
| shader/shader_chunks_preview_web | 0 / 0 | 0 | — | 0 | 0 | New crate (2026-08-14): wasm32-only WebGPU browser runner — every real dependency gated under `[target.'cfg(target_arch = "wasm32")'.dependencies]`, native `main()` is a stub. 0/0 tests and 0 host-visible deps are expected here, not a defect: nothing in this crate is unit-testable on the host; verify via `cargo check -p shader_chunks_preview_web --target wasm32-unknown-unknown` |
| shader/shader_chunks_query | 0 / 0 | 0 | — | 0 | 0 | New crate (2026-08-14, split out of the old monolithic shader_chunks CLI): CLI wiring only for `list`/`get`/`tags`/`tree` over shader_chunks_query_core below. 0/0 tests is by design, not a gap — this crate carries no logic of its own to unit-test; query_core's 30 tests cover the rendering logic directly, and this crate's dispatch is exercised end-to-end via shader_chunks's `cli_subprocess_test.rs` |
| shader/shader_chunks_query_core | 1 / 30 | 0 | — | 0 | 0 | New crate (2026-08-14, split out of the old monolithic shader_chunks CLI): filter/project/sort/page/render query engine over bundled chunks — `QueryParams` (19 named fields), `chunks_query` pipeline (select→filter→count-shortcut→sort/order→offset/limit→render), `tags_list`, `chunk_tree` |
| shader/shader_chunks_render | 1 / 19 | 0 | yes | 0 | 0 | CLI wiring for the `render` command: reuses `shader_chunks_preview`'s `bundle_prepare` (same target resolution + naga validation the live preview runs), then renders one frame via `shader_chunks_render_core` and writes it as a PNG — every slider at its initial value, `time` frozen at the caller's `time::`. Needs no browser/dev-server/web-runner crate, unlike `.preview` |
| shader/shader_chunks_render_core | 1 / 7 | 0 | — | 0 | 0 | Renders a `shader_chunks_preview_core::PreviewBundle` to raw RGBA pixels on a headless GPU — one static frame of exactly what `shader_chunks_preview_web`'s browser runner shows live. Uses `minwgpu`'s offscreen toolkit (headless context, one uniform buffer laid out via the shared `resolution_index` convention, bufferless fullscreen-triangle pipeline, row-padding-aware readback) |
| shader/shader_chunks_validate | 1 / 4 | 0 | yes | 0 | 0 | CLI wiring for the `validate` command: renders `shader_chunks_validate_core`'s registry-wide checks as a human-readable findings report |
| shader/shader_chunks_validate_core | 1 / 8 | 0 | — | 0 | 0 | Five independent, non-panicking registry-wide integrity checks over `shader_chunks_core::CHUNKS` in one pass: manifest drift, duplicate names, missing/cyclic deps, WGSL compile |

*(Previously noted as missing from this table across earlier snapshots — both crates exist on disk,
fully implemented and tested, and are now rows above like their siblings.)*

Notes column links go through task/readme.md; `—` in Allows = not in the top-count sweep (small or
zero). **Allows-column caveat (2026-08-13):** the regeneration command only matches literal
`#[allow(...)]`; several crates (mdmath_core, ndarray_cg, renderer, tiles_tools among them) have
since converted most or all of their justified suppressions to `#[expect(...)]` (fails loudly if the
lint stops firing), which this column does not count — a low or zero Allows value no longer implies
zero suppression attributes for those crates. To see the current expect-count for a crate:
`grep -rn '#!\?\[ *expect(' <crate>/src <crate>/tests | wc -l`. Examples tree (75 demo crates —
recount: `find examples -name Cargo.toml | wc -l`) is intentionally not tabulated per-crate: demos
carry no tests/ requirement; their marker triage closed with task 065 (✅ Completed) — task 065
decided keep-crate for `diamond` and `make_cube_map`, and tasks 094/095 deleted the two stale
`rid of this crate` markers from their manifests.

## Known issues (workspace level)

- **Shader validation tooling absent: resolved without external tooling.** `glslangValidator` is
  still not installed on this machine, but GLSL ES 3.00 shader validation no longer needs it —
  `module/helper/renderer/tests/legacy_glsl_shader_compile_test.rs` compiles all 28 shipped
  `.vert`/`.frag` files through a real headless WebGL2 context's own compiler, the actual target
  these OpenGL-ES-idiom sources are written for (naga's `front::glsl` targets desktop GLSL
  440+/Vulkan only and rejects them outright — see `shader_validation_tests.rs`'s doc comment).
  Verify: `grep -c 'wasm_bindgen_test( async )'
  module/helper/renderer/tests/legacy_glsl_shader_compile_test.rs` (28).
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
- **291** — gpu_hal mipmap/MSAA/compute support watch item (📝 Draft; same YAGNI-deferred pattern —
  `docs/layer/002`'s own Status section already names the gap, no consumer needs it yet).
- **9 tasks in 🔎 Accepting** (246, 247, 248, 118, 192, 201, 202, 413, 414 — 191 moved back to 🎯
  Verified, not yet claimed for execution) and **44 in 🔬 Verifying** (grown substantially since the
  2026-08-18 snapshot's 14, driven by a wave of formal fix-registration tasks for already-applied bug
  fixes — see task/readme.md's live Tasks Index for the full current list, not duplicated here) — all
  code-complete and independently self-verified (Tier 2 Dual-Role Self-Check). Every attempted
  `tsk .acceptance_pass`/`.verify_pass` transition across this backlog is refused by this sandbox's
  same-actor guard (`self-verification forbidden` — actor matches `executing_by`/`filed_by`). Blocked
  on a genuinely independent verifier, not on further work; see each task's own Journal section for the
  exact refusal.
- **BUG-114** (🎯 Verified, High) — `diamond` example's uv-attribute stride mismatch. Fix applied and
  live-confirmed: Chromium/SwiftShader's software WebGL2 backend performs no `drawElements`-time
  bounds validation at all (blocked round 1's VERIFY Gate), but a re-run of the identical MRE via
  Firefox instead — whose software fallback genuinely validates buffer bounds — reproduced the full
  predicted symptom exactly (see the bug file's `## Verification Record`). VERIFY Gate PASS (8/8);
  promoted to **task 254** (renumbered from 243→252→254 — two same-day collisions: `BUG-243`, then a
  fresh race with a concurrent actor's independently-filed `BUG-252`; formal fix-task registration via
  `bug_promote`/PROC12), itself readiness-gate PASS (8/8) and now folded into the 14-Verifying count
  above, same same-actor-guard block.
- **BUG-298** (🎯 Verified, Medium) — `ndarray_cg`'s `Quat::invert()` returns the bare conjugate
  unconditionally, silently wrong for any non-unit-length quaternion; latent (zero reachable call
  sites currently). Fix applied and registered via **task 357** (`closes: BUG-298`, `bug_promote`/
  PROC12), 🔬 Verifying, readiness-gate PASS 8/8, blocked on the same same-actor guard.
- **BUG-300** (🎯 Verified, Medium) — `minwebgpu`'s `TextureDescriptor::new()` default format is
  incompatible with `.storage_binding()`, silently producing a texture WebGPU rejects; latent (zero
  reachable call sites currently). Fix applied and registered via **task 359** (`closes: BUG-300`),
  🔬 Verifying, readiness-gate PASS 8/8, blocked on the same same-actor guard.
- **BUG-311** (🎯 Verified, Medium) — `Quat::from_angle_y( 90.0 )` called with a raw degree literal
  instead of radians at 3 sibling example call sites (`curve`/`lottie`/`animation_surface_rendering`);
  active, visually-wrong behavior, confined to those 3 examples. Fix applied and registered via
  **tasks 369-372** (split of task 360, one per example plus an `ndarray_cg` regression test,
  `closes: BUG-311`), all 🔬 Verifying, blocked on the same same-actor guard.
- **BUG-312** (🎯 Verified, Medium) — `character_control` example halves the visible character mesh's
  yaw at its `Quat::from_angle_y` call site, desyncing it from the camera's own orbit; active, confined
  to 1 example. Fix applied and registered via **task 363**, 🔬 Verifying, blocked on the same
  same-actor guard.
- **BUG-313** (🎯 Verified, Medium) — `sprite_animation` example's frame-index modulus uses
  `sprite_count - 1` instead of `sprite_count`, permanently skipping the last animation frame; active,
  confined to 1 example. Fix applied and registered via **task 358**, 🔬 Verifying, blocked on the
  same same-actor guard.
- **BUG-314** (🎯 Verified, High) — `embroidery_tools`' PEC reader underflows `stitch_block_len - 5` for
  untrusted file data under 5 bytes, panicking in debug and corrupting the read position in release;
  reachable via both public `pec::*` and `pes::*` entry points, not latent. Fix applied and registered
  via **task 365**, 🔬 Verifying, blocked on the same same-actor guard.

No task in the current backlog is actionable by further autonomous work in this sandbox — the 8
Accepting + 44 Verifying tasks above are code-complete and self-verified pending independent review,
blocked only on a genuinely independent verifier this sandbox's same-actor guard cannot supply.
BUG-114/298/300/311/312/313/314 all have fixes applied and formally registered (bug/readme.md itself
still shows each 🎯 Verified — the linked fix-registration task is the pending step, not the code fix).
The bug registry's own Open Bugs table (task/bug/readme.md) has grown substantially since this
section was last written; treat the bullets above as the reach-consistency-confirmed subset, not an
exhaustive Open Bugs list — re-derive via `awk '/^## Open Bugs/,/^## Closed Bugs/' task/bug/readme.md`
for the current full count (23 as of this snapshot).

