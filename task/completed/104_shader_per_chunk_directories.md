# Give each shader chunk its own directory with a documented visualization, parameters, and cross-references

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** self (Tier 2 Dual-Role Self-Check)
- **verification_date:** 2026-08-13
- **blocked_by:** null
- **priority:** 0

## Goal

Restructure the repo-root `shader/` collection so every WGSL chunk lives in its own directory containing its `.wgsl` source, a `readme.md` documenting a visualization of the chunk, all its manifest parameters, its implementation nuances, and cross-references to related chunks/consumers, plus a generated `preview.png` backing that visualization — per explicit user instruction this session: "each shader must have its own dir with readme file with visualization of it, all paramters and nuances and references on relatives. redo."

Motivated: direct, explicit user instruction, framed as a "redo" of the then-flat `shader/` layout (4 bare `.wgsl` files, no per-chunk documentation). Observable: `shader/<name>/` exists for all 4 chunks, each containing `<name>.wgsl` + `readme.md` (Visualization/Parameters/Nuances/Relatives sections) + `preview.png`; `shader_chunks_core` still compiles and its full test suite still passes after the `include_str!` path update the move requires. Scoped: pure reorganization plus new documentation and generated preview images — zero WGSL logic or manifest-field changes to any chunk. Testable: `cargo check`/`cargo test`/`cargo clippy` across the two consuming crates plus `orrery_webgpu`, a `wasm32-unknown-unknown` cross-check of the two wasm-relevant crates, and a relative-link resolution sweep across all 5 new `readme.md` files.

## In Scope

- `shader/hash21.wgsl` -> `shader/hash21/hash21.wgsl`, plus new `shader/hash21/readme.md` and `shader/hash21/preview.png`
- `shader/value_noise.wgsl` -> `shader/value_noise/value_noise.wgsl`, plus new `shader/value_noise/readme.md` and `shader/value_noise/preview.png`
- `shader/fbm3.wgsl` -> `shader/fbm3/fbm3.wgsl`, plus new `shader/fbm3/readme.md` and `shader/fbm3/preview.png`
- `shader/fullscreen_triangle.wgsl` -> `shader/fullscreen_triangle/fullscreen_triangle.wgsl`, plus new `shader/fullscreen_triangle/readme.md` and `shader/fullscreen_triangle/preview.png`
- New `shader/readme.md` top-level index: collection intro, per-chunk table (Responsibility + Depends On), dependency order, consumer note
- `module/shader/shader_chunks_core/src/chunks.rs`: all 4 `include_str!` paths plus both doc comments referencing chunk paths, updated to the new per-directory layout
- `module/shader/shader_chunks_core/readme.md`: chunk-storage description and illustrative example paths updated to reflect the new per-directory layout

## Out of Scope

- `module/shader/shader_chunks/readme.md`'s `help`-command documentation changes — an unrelated, concurrently-completed fix for BUG-103 (`sch help`/`sch compose help` dispatch), already present in the working tree before this task began; not authored by this task and not touched by it
- Any change to WGSL logic, manifest fields (`name`/`description`/`tags`/`stage`/`depends_on`/`export`), or shader composition/dependency-resolution behavior — pure reorganization plus documentation, zero behavioral change
- A permanent Rust-side preview-rendering tool or binary — the 4 preview PNGs were produced by a temporary, hyphen-prefixed script (`-generate_previews.py`), deleted immediately after use; no standing preview-generation tooling added, since no second consumer or recurring need exists yet (YAGNI)
- `shader_chunks`' own `docs/cli/`/`tests/docs/cli/` trees — no CLI-visible change, untouched
- A per-leaf-chunk-directory Responsibility Table — this workspace's own established convention (`assets/`, `examples/minwebgl/trivial/readme.md`, both with ≥3 files and no internal table) puts a lightweight index only at the actual new-directory parent-registration point (`shader/readme.md`); each chunk directory's `readme.md` documents that one chunk's own content, not a multi-file index needing a table of its own

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `cargo test -p shader_chunks_core -p shader_chunks -p orrery_webgpu --all-features` passes with zero failures
-   `cargo clippy -p shader_chunks_core -p shader_chunks -p orrery_webgpu --all-targets --all-features -- -D warnings` passes with zero warnings
-   `cargo check --workspace --all-targets` passes
-   `cargo check --target wasm32-unknown-unknown -p orrery_webgpu -p shader_chunks_core` passes — scoped to the two crates actually meant to be wasm32-compatible; `shader_chunks` is a native-only terminal CLI (pulls in the `home` crate transitively via its `assert_cmd` dev-dependency, which is `#[cfg(unix)]`-gated) and is correctly excluded
-   Every relative markdown link in the 5 new `readme.md` files resolves to a real file
-   Each of the 4 `preview.png` files is a real, visually-distinct rendering of its chunk's actual output — not a placeholder
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file resides in `task/completed/`

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|
| Workspace build after restructure | `cargo check --workspace --all-targets` | Exits 0, no errors |
| Core crate test suite | `cargo test -p shader_chunks_core --all-features` | All tests pass — manifest/compose logic untouched by the path change |
| CLI crate test suite | `cargo test -p shader_chunks --all-features` | All tests pass — CLI reads chunks via the Rust `CHUNKS` API, never raw paths |
| Consumer crate test suite | `cargo test -p orrery_webgpu --all-features` | All tests pass |
| Full clippy sweep | `cargo clippy -p shader_chunks_core -p shader_chunks -p orrery_webgpu --all-targets --all-features -- -D warnings` | Zero warnings |
| wasm32 cross-check, correctly scoped | `cargo check --target wasm32-unknown-unknown -p orrery_webgpu -p shader_chunks_core` | Exits 0 |
| Stray flat-path reference sweep | `grep -rn "shader/hash21\.wgsl\|shader/value_noise\.wgsl\|shader/fbm3\.wgsl\|shader/fullscreen_triangle\.wgsl" --include="*.rs" --include="*.toml" .` | Zero hits |
| Relative link resolution | Loop resolving every `](...)` link in the 5 new `readme.md` files against each file's own directory | Zero broken links |

## Acceptance Criteria

-   Every one of the 4 chunks lives at `shader/<name>/<name>.wgsl`; the flat `shader/<name>.wgsl` files no longer exist
-   Every chunk directory contains a `readme.md` with Visualization (embedded `preview.png` plus a caption explaining exactly what is rendered and how), Parameters (manifest-field table), Nuances (implementation-specific prose grounded in the actual WGSL body), and Relatives (Depends on / Depended on by / Collection index / Bundled by / Inspect via CLI / Consumer) sections
-   Every chunk directory contains a `preview.png` that is a real, pixel-accurate-to-the-WGSL-math rendering, visually distinct per chunk's actual behavior
-   `shader/readme.md` exists as a top-level collection index linking to all 4 chunk directories
-   `module/shader/shader_chunks_core/src/chunks.rs`'s `CHUNKS` table's 4 `include_str!` calls resolve to the new per-directory paths, and the crate compiles
-   `cargo check --workspace --all-targets` exits 0
-   `cargo test -p shader_chunks_core -p shader_chunks -p orrery_webgpu --all-features` passes with zero failures
-   `cargo check --target wasm32-unknown-unknown -p orrery_webgpu -p shader_chunks_core` exits 0

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**Structural completeness**
- [x] C1 — Does every one of the 4 original chunks now live in its own `shader/<name>/` directory containing `<name>.wgsl`, `readme.md`, and `preview.png`, with the flat `shader/<name>.wgsl` file no longer present?
- [x] C2 — Does `shader/readme.md` exist as a top-level index listing all 4 chunks with links to each subdirectory's `readme.md`?
- [x] C3 — Does each chunk's `readme.md` contain all 4 required sections (Visualization, Parameters, Nuances, Relatives)?

**Manifest fidelity**
- [x] C4 — Does each chunk readme's Parameters table accurately reflect that chunk's actual `//@`-header fields, cross-checked against the `.wgsl` source?
- [x] C5 — Does each chunk readme's Relatives section correctly state Depends on / Depended on by, matching the actual `depends_on` graph (`hash21` <- `value_noise` <- `fbm3`; `fullscreen_triangle` standalone)?

**Cross-references**
- [x] C6 — Do all relative markdown links inside the 4 new chunk readmes and the top-level `shader/readme.md` resolve to real files?
- [x] C7 — Does `module/shader/shader_chunks_core/src/chunks.rs`'s `CHUNKS` table point at the new per-directory `.wgsl` paths, and does the crate compile?

### Measurements

- [x] M1 — `grep -rn "shader/hash21\.wgsl\|shader/value_noise\.wgsl\|shader/fbm3\.wgsl\|shader/fullscreen_triangle\.wgsl" --include="*.rs" --include="*.toml" /home/user1/pro/lib/yrd_gamedev/cgtools` -> 0 hits
- [x] M2 — Relative-link resolution loop over all 5 new `readme.md` files -> 0 `BROKEN:` lines
- [x] M3 — `file shader/*/preview.png` -> 4 files, all `256 x 256`, non-interlaced

### Invariants

- [x] I1 — `cargo test -p shader_chunks_core -p shader_chunks -p orrery_webgpu --all-features` -> 0 failures (54/54 passed)
- [x] I2 — `cargo clippy -p shader_chunks_core -p shader_chunks -p orrery_webgpu --all-targets --all-features -- -D warnings` -> 0 warnings
- [x] I3 — `cargo check --workspace --all-targets` -> 0 errors
- [x] I4 — `cargo check --target wasm32-unknown-unknown -p orrery_webgpu -p shader_chunks_core` -> 0 errors, correctly scoped (excludes the native-only `shader_chunks` CLI)

### Anti-faking checks

- [x] AF1 — The 4 `preview.png` files are real rendered pixel data derived from the actual WGSL math (`fract`/`floor`/`dot`/`mix` reimplemented exactly in the generation script, then the script deleted), verified via direct visual inspection distinguishing each chunk's expected qualitative look (uncorrelated static for `hash21`, smooth blobby noise for `value_noise`, richer fractal detail for `fbm3`, a clean R/G gradient for `fullscreen_triangle`) — not placeholder or blank images. `file`'s reported color mode independently corroborates this: the 3 noise-family previews are grayscale (single-channel), `fullscreen_triangle` is full RGB (matching its two-channel-plus-implicit-third-gradient description).
- [x] AF2 — The wasm32 check's first attempt (`-0007_longrun.log`) failed on the transitive `home` crate (`#[cfg(unix)]`-gated `home_dir_inner`, unavailable on `wasm32-unknown-unknown`) because that invocation incorrectly bundled the native-only CLI crate `shader_chunks` (whose `assert_cmd` dev-dependency pulls in `home`) into the sweep. This was root-caused, not silently dropped or rerun with a suspiciously narrower target chosen to dodge a real regression: re-running with `shader_chunks` excluded (`-0008_longrun.log`, scoped to only `orrery_webgpu` and `shader_chunks_core` — the two crates whose own `readme.md` documents them as intended to be wasm-compatible) passed cleanly, confirming the failure was a scope error in the check itself, unconnected to this task's changes.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| G1 | Scope Coherence | — | 🟢 | In Scope enumerates every file group touched (4 chunk moves + their new readme/preview pairs, the top-level index, `chunks.rs`, `shader_chunks_core/readme.md`) with the exact old->new mapping; Out of Scope explicitly excludes the concurrent BUG-103 diff, behavioral changes, permanent preview tooling, and a redundant leaf-level Responsibility Table. Meaningful observable outcome: every chunk browsable as a self-contained, documented, visualized unit. | — |
| G2 | MOST Goal Quality | — | 🟢 | Motivated (direct user quote this turn, explicitly framed as a "redo"); Observable (directory/file existence, compile+test success, link resolution); Scoped (reorganization + documentation only, zero WGSL/manifest changes); Testable (Test Matrix's 8 rows, each with a concrete command already run with real output). | — |
| G3 | Value/YAGNI | — | 🟢 | Null Hypothesis: without this restructure, a reader wanting to understand one chunk's behavior had only a bare `.wgsl` file and its terse `//@` header — no visualization, no prose explanation of *why* a constant or technique was chosen, no explicit cross-reference map. Not speculative: user directly requested this exact structure (own dir + readme + visualization + parameters + nuances + relatives) for every chunk, no more and no less. | — |
| G4 | Implementation Readiness | — | 🟢 | Every touched file was read in full before editing (all 4 `.wgsl` sources, `chunks.rs`, `shader_chunks_core/readme.md`); the CPU-side preview approach was validated against WGSL semantics before use and each output visually inspected; the one dependency-graph fact needed (`hash21 <- value_noise <- fbm3`, `fullscreen_triangle` standalone) was read directly from each chunk's own `//@ depends_on:` line, not assumed. | — |
| G5 | Execution Scope | — | 🟢 | Every path (`shader/`, `module/shader/shader_chunks_core/`) resolves inside this repository. | — |
| G6 | Crate Scope Unity | — | 🟢 | `unit_type: workspace` / `unit: lib/yrd_gamedev/cgtools`, matching task 099's precedent for this exact shape (a repo-root asset directory not owned by any single crate, plus one crate's source file that consumes it) — `shader/` sits outside any crate boundary by design, so a single-crate `unit_type` would misdescribe the deliverable's actual footprint. | — |
| G7 | Crate Locality | — | 🟢 | Chunk documentation lives beside each chunk's own source (`shader/<name>/`, not pushed up into `module/shader/shader_chunks_core/docs/`); the one Rust-source edit (`chunks.rs`'s `include_str!` paths) lives in the crate that owns the bundling logic. | — |
| G8 | Crate Single Responsibility | — | 🟢 | `shader_chunks_core`'s responsibility is unchanged ("manifest-driven WGSL shader-chunk composer", still statable without "and") — this task only updates 4 string literals and two doc-comment sentences inside it; the responsibility-bearing logic (`compose`/`try_compose`/`chunk_get`) is untouched. | — |
| **Total** | | — | 🟢 | — | — |

Adversarial pass: strongest challenge is "does the CPU-side numpy/PIL preview generator violate `shader_chunks_core`'s own documented 'No Rust mirror of any chunk's math' principle (its readme, § line 66-71: 'a parallel Rust body would be a second implementation... that never runs on the GPU path it mirrors')?" — checked directly against that principle's own stated scope: it governs the crate's *shipped* source (no `hash21`/`value_noise`/`fbm3` Rust ports living in `src/`), motivated by drift risk between a permanent duplicate and the GPU path it's meant to mirror. The preview generator was a temporary, hyphen-prefixed script, never part of any crate's source, deleted immediately after producing 4 static PNGs — there is no standing duplicate to drift, and nothing in the shipped crate changed shape. Second challenge: "was the wasm32 finding actually resolved, or just argued away?" — checked against concrete evidence: `-0007_longrun.log` (broad scope, failed on `home`) versus `-0008_longrun.log` (narrowed to the two crates whose own readmes claim wasm-compatibility, passed clean) is a controlled before/after, not an assertion. No blocking finding survives.

## Outcomes

Implementation completed: all 4 chunks (`hash21`, `value_noise`, `fbm3`, `fullscreen_triangle`)
moved from flat `shader/<name>.wgsl` files into `shader/<name>/<name>.wgsl`, each paired with a new
`readme.md` (Visualization/Parameters/Nuances/Relatives) and a generated `preview.png`; a new
top-level `shader/readme.md` index was added; `shader_chunks_core/src/chunks.rs`'s 4 `include_str!`
paths and doc comments, and `shader_chunks_core/readme.md`'s illustrative paths, were updated to match.

**Preview generation approach:** a temporary script (`-generate_previews.py`, hyphen-prefixed
per this repo's temp-file convention, deleted immediately after use) reimplemented each chunk's
WGSL math exactly in numpy (`fract`/`floor`/`dot`/`mix`, non-negative-domain `fract` matching
WGSL's floor-based definition) to render pixel-accurate 256×256 PNGs without needing an actual
GPU pipeline. Each output was visually confirmed via direct image inspection before being moved
into its permanent location: `hash21` shows uncorrelated "TV static", `value_noise` shows smooth
blobby noise with faint grid-cell boundaries, `fbm3` shows visibly richer fractal detail layered
over `value_noise`'s look, and `fullscreen_triangle` shows a clean R/G gradient with no visible
triangle edge (the overshoot is clipped off-screen by construction).

**Documentation-structure precedent:** rather than mechanically applying a per-directory
Responsibility Table everywhere, this workspace's own existing conventions were inspected first
(`assets/`, `module/shader/` had no readme at all; `examples/minwebgl/trivial/readme.md` embeds
its showcase image with a plain `![image](./showcase.webp)`, no internal table, despite ≥3 files) —
concluding a lightweight index table belongs only at the actual new-directory parent-registration
point (`shader/readme.md`, registering the 4 new subdirectories), not duplicated inside every leaf
chunk directory.

**wasm32 scope-correction finding (adversarial-pass discovery, own initiative):** an initial
wasm32 sweep bundling all 3 candidate crates (`orrery_webgpu`, `shader_chunks_core`,
`shader_chunks`) failed with `error[E0425]` inside the transitive `home` crate
(`-0007_longrun.log`). Root-caused rather than accepted at face value: `home` is pulled in only
via `shader_chunks`' `assert_cmd` dev-dependency (used for locating the built CLI binary in
subprocess tests) and is `#[cfg(unix)]`-gated — `shader_chunks` is a native-only terminal CLI
never intended to target wasm32 in the first place, unlike `shader_chunks_core` and
`orrery_webgpu`, whose own readmes describe them as usable identically from native and browser
consumers. Re-running scoped to just those two crates (`-0008_longrun.log`) passed cleanly,
confirming this was an incorrectly-scoped check on this task's own part, not a regression
introduced by the restructuring.

**Acceptance walk performed as a self-administered Tier 2 Dual-Role Self-Check**, per this repo's
standing convention (verification capped at Tier 2, never escalated —
[[feedback_maav_tier_cap]]). Confirming pass: every one of C1-C7/M1-M3/I1-I4/AF1-AF2 checked
directly against fresh command output — `cargo check --workspace --all-targets`, `cargo clippy
-p shader_chunks_core -p shader_chunks -p orrery_webgpu --all-targets --all-features -- -D
warnings`, and `cargo nextest run -p shader_chunks_core -p shader_chunks -p orrery_webgpu
--all-features` (54/54 passed, 0 skipped — `-0006_longrun.log`); `cargo check --target
wasm32-unknown-unknown -p orrery_webgpu -p shader_chunks_core` (`-0008_longrun.log`, exit 0); a
fresh `find shader -type f` inventory (exactly 13 files: 4×(wgsl+readme+preview)+1 top-level
readme); a fresh flat-path grep sweep (0 hits); a fresh relative-link resolution loop across all
5 new readmes (0 broken); `file shader/*/preview.png` (4×256×256, correct color modes per chunk).
Adversarial pass: see the Verification Record's own adversarial-pass paragraph above (No-Rust-Mirror
principle scope check; wasm32 finding controlled-comparison check). No blocking finding survives.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-08-13** `FILED` — Task filed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/. Goal: give each shader chunk its own directory with readme (visualization, parameters, nuances, relatives) and preview image, per explicit user instruction this session. Gate ran 8/8 PASS at filing time (self-administered Tier 2 Dual-Role Self-Check).
- **2026-08-13** `COMPLETED` — Acceptance walk passed (Tier 2 Dual-Role Self-Check, 16/16 checklist items PASS — see Outcomes). All 4 chunks restructured into per-directory layout with full documentation and generated previews; `shader_chunks_core` compiles and 54/54 tests pass across the 3 affected crates; clippy clean; wasm32 cross-check clean once correctly scoped; zero stray flat-path references; zero broken relative links. Moved directly to `task/completed/`.
