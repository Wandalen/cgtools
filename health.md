# cgtools health

Per-crate workspace health dashboard. Every column is a snapshot with its regeneration command —
re-run the command to refresh a number instead of trusting the table. Live work items are tracked in
[task/readme.md](task/readme.md); this file summarizes state, it does not duplicate the backlog.

- **Snapshot date:** 2026-08-10
- **Workspace build:** ✅ `cargo check --workspace --all-features` — exit 0, 57s, all crates
  (module/ + examples/) compile clean.
- **Task system:** 58 completed · 4 draft · 6 cancelled (see task/readme.md for the live table).

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
| helper/animation | 3 / 29 | 0 | — | 0 | — | |
| helper/behaviour_tree | 1 / 15 | 0 | yes | 0 | — | Tests relocated by task 067 (14 inline → tests/, +1 new pin) |
| helper/browser_input | 2 / 13 | 0 | — | 0 | — | All 6 inline tests relocated to tests/pointer_type_test.rs by task 076 (fully public surface); tests/readme.md added · Decoupled from minwebgl by task 057 (math via ndarray_cg, JsCast via web-sys re-export, own Window/Document web-sys features) |
| helper/browser_log | 2 / 5 | 0 | — | 0 | — | panic.rs qqq markers closed by task 077 (panic_hook_test.rs: Config pins + real-panic native hook test; wasm-only formatting recorded as decision); duplicate `licence`+`license` files remain |
| helper/canvas_renderer | 0 | 1 | — | 0 | — | Inline reproducer kept as documented exception by task 068 (public surface is all live-GL) |
| helper/embroidery_tools | 3 / 10 | 0 | yes | 0 | — | Tests relocated by task 066 (8 inline → tests/, +2 new pins); no workspace-lints inheritance |
| helper/gpu_hal | 1 / 2 | 0 | — | 0 | 28 | HAL v0; buy-vs-build ADR closed |
| helper/line_tools | 5 / 88 | 0 | yes | 0 | 32 | |
| helper/primitive_generation | 2 / 5 | 0 | — | 0 | 41 | ufo.rs allows justified/fixed by task 036 |
| helper/renderer | 16 / 75 | 6 | yes | 0 | 87 | Native backend work in progress (uncommitted working tree — counts reflect it) · 6 inline tests kept as one documented exception by task 075 (private resolve_asset_uri URI helper of the browser-bound glTF loader; run natively) |
| helper/scene_script | 2 / 16 | 0 | yes | 0 | — | |
| helper/tilemap_renderer | 6 / 100 | 29 | yes | 0 | 23 | Markers resolved by task 064 (Source::Path geometry loading implemented, pitfall/003 retired) · 54 tests relocated to tests/ by task 071 (29 inline kept as documented exceptions pinning private helpers) |
| helper/tilemap_scene | 12 / 171 | 0 | yes | 0 | 39 | All 38 inline tests relocated to 2 new domain files by task 073 (hash_test.rs + compile_units_test.rs; zero exceptions — mod_interface root re-exports reach everything); tests/readme.md added |
| helper/tiles_tools | 18 / 257 | 5 | yes | 0 | 460 | Markers resolved by task 063 (movement queue implemented, pitfall/002 retired) · 46 tests relocated to 7 new feature-mirrored tests/ files by task 072 (5 inline kept as documented exceptions; 5 fov duplicates consolidated) · disabled flowfield integration module → draft 078 · largest allow count |
| math/mdmath_core | 23 / 89 | 0 | — | 2 | 83 | Markers resolved by task 059 (soundness unsafe removed, Ix4 added); 2 lint markers → draft 058; no workspace-lints inheritance |
| math/ndarray_cg | 36 / 222 | 0 | — | 2 | 41 | Markers resolved by task 060 (typed TryFrom error, IntoVector tests); 2 lint markers → draft 058; suite shared with ndarray_tools; no workspace-lints inheritance |
| min/mingl | 6 / 38 | 13 | — | 0 | 44 | Markers resolved by task 061 · 13 inline tests kept as one documented exception by task 074 (private pure URL helpers of the wasm-only web loader; proven to run natively under --all-features) |
| min/minwebgl | 1 / 2 | 4 | yes | 0 | 44 | Markers resolved by task 062 · Runnability story + native data_type tests by task 069 (4 inline kept as documented exceptions) |
| min/minwebgpu | 1 / 6 | 0 | yes | 0 | 32 | |
| min/minwgpu | 2 / 5 | 21 | yes | 0 | — | Deterministic adapter-error tests/ established by task 070 (21 inline kept as documented exceptions — pub(super) builder internals) |

Notes column links go through task/readme.md; `—` in Allows = not in the top-count sweep (small or
zero). Examples tree (~50 demo crates) is intentionally not tabulated per-crate: demos carry no
tests/ requirement; their 13 open markers are triaged in draft 065 (6 need human decisions,
including two `rid of this crate` calls on `diamond` and `make_cube_map`).

## Known issues (workspace level)

- **Shader validation tooling absent:** `glslangValidator` is not installed on this machine, which
  blocks offline GLSL validation work. Fix: `sudo apt install glslang-tools`, verify with
  `glslangValidator --version`.
- **5 blank/ crates carry a stale repository URL** (`Wandalen/cg_tools`): cg_tools, mdmath,
  mdmath_ai, mdmath_cg, mdmath_linalg. Verify: `grep -rn "Wandalen/cg_tools" module/*/*/Cargo.toml`.
- **browser_log ships both `licence` and `license` files** and its changelog claims are unverified.
  Verify: `ls module/helper/browser_log/ | grep -i licen`.
- **Lint-policy stragglers:** mdmath_core, ndarray_cg, embroidery_tools (plus most example crates)
  do not inherit `[workspace.lints]`; per-crate allow-justification sweep is draft 058. Verify a
  crate: `grep -A1 "^\[lints\]" <crate>/Cargo.toml`.

## Open work streams (details in task/readme.md)

- **058** — per-crate `#[allow]` justification sweep (census + procedure embedded).
- **065** — task-marker resolution, examples tranche (the per-crate tranches 059–064 are all
  done; 065 needs human decisions on two `rid of this crate` calls).
- **Test-coverage stream (035 decomposition) COMPLETE** — 066–077 closed embroidery_tools,
  behaviour_tree, canvas_renderer, minwebgl, minwgpu, tilemap_renderer, tiles_tools,
  tilemap_scene, mingl, renderer, browser_input, and browser_log; every crate's inline tests
  are relocated or documented exceptions. **078** — re-enable or retire tiles_tools' disabled
  flowfield integration tests.
- **056** — vectorizer revival watch item.

