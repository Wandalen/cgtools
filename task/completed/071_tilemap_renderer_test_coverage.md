# Restore test-directory convention in tilemap_renderer (decomposed from task 035)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** crate
- **unit:** module/helper/tilemap_renderer
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Census 2026-08-10 (task 035 — re-derive at pickup): **5 tests/ files with 40 test markers; 80 inline #[test] in src/**. The workspace's LARGEST inline block — 80 inline tests despite an established tests/ suite. Likely adapter-internal unit tests; the expose-or-exception decision dominates. Coordinate with draft 064 (marker resolution in the same crate) to avoid edit collisions.

Per-test procedure (uniform across the 035 decomposition):
1. For each inline `#[ test ]` in `src/`: if it exercises public API only, relocate it to
   `tests/`; if it needs private access, DECIDE — expose the tested item (only when the API
   genuinely warrants it) or keep it in place as a documented exception. Never delete a test to
   satisfy the rule; never widen an API solely to satisfy it either.
2. Consolidate any inline test that duplicates existing tests/ coverage instead of relocating a
   second copy.
3. Verify with `longrun .launch dir::<workspace root> -- cargo test -p tilemap_renderer --all-features` —
   all green before and after each relocation batch.

## Verification

### Checklist

- [x] C1 — Does `src/adapters/svg.rs` now contain exactly the 29 claimed "documented exception" inline tests? `grep -c "#\[ *test *\]" src/adapters/svg.rs` → `29`.
- [x] C2 — Does `tests/svg_backend_test.rs` exist with exactly the 54 claimed relocated tests, feature-gated on `adapter-svg`? `grep -c "#\[ *test *\]" tests/svg_backend_test.rs` → `54`; file's first line → `#![ cfg( feature = "adapter-svg" ) ]`.
- [x] C3 — Does the inline-exception rationale comment name all 8 claimed pinned private helpers and cite this task by number? Read `src/adapters/svg.rs:1938-1946` → cites "task 071" and names `transform_to_svg_static`/`transform_to_svg_local`, `anchor_to_svg`, `path_to_href`, `png_dimensions`, `detect_image_mime`, `bitmap_to_png`, `SvgContentManager`, plus `image_encoded_png_stores_dimensions`'s private-encoder dependency — all 8 present.
- [x] C4 — Are the 4 pre-existing `tests/*.rs` files (the claimed disjoint, untouched 39) still exactly as claimed? `grep -c "#\[ *test *\]"` per file: `assets_test.rs` → `9`, `backend_test.rs` → `17`, `commands_test.rs` → `4`, `types_test.rs` → `9` — sums to `39`.
- [x] C5 — Is `bytemuck` genuinely present in `[dev-dependencies]` (distinct from the pre-existing optional `[dependencies]` entry)? `Cargo.toml` → `[dev-dependencies]` (line 51) contains `bytemuck.workspace = true` (line 52).
- [x] C6 — Does `tests/readme.md` reflect the relocation, with the domain map crediting this task by number? Full read → directory-structure block (line 19) lists `svg_backend_test.rs` with its `adapter-svg` feature gate; domain-map row (line 31) explicitly cites "task 071" and describes the public-surface-only / private-helper-exception split.

### Measurements

- [x] M1 — Inline `#[test]` count in `src/adapters/svg.rs`: `29` (was: `80` at `git show 4469eafb^:module/helper/tilemap_renderer/src/adapters/svg.rs`, the commit immediately preceding this task's own fix; the 83-vs-80 gap is task 064's 3 new `Source::Path` tests, added between the original task-035 census and this task's pickup — confirmed via that task's own Verification).
- [x] M2 — `tests/svg_backend_test.rs` test count: `54` (was: did not exist — `git show 4469eafb^:module/helper/tilemap_renderer/tests/svg_backend_test.rs` resolves to no such path in that tree).
- [x] M3 — Total preserved test count: `29 + 54 = 83`, matching `83` at `git show 4469eafb:module/helper/tilemap_renderer/src/adapters/svg.rs` (the commit containing this task's own fix, pre-relocation) — confirms zero tests lost.

### Invariants

- [x] I1 — Test suite (crate-scoped, all features): `cargo nextest run -p tilemap_renderer --all-features` → exit 0, 122 tests run, 122 passed, 0 skipped — decomposes as `29` (kept inline) + `54` (`svg_backend_test.rs`) + `39` (pre-existing 4 files) = `122`, matching every count claimed above.
- [x] I2 — Compiler/lints: `cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings` → **exit 101** — genuine current drift, but not from this task's own files. Root cause: the workspace `Cargo.toml`'s `allow_attributes_without_reason` lint was flipped `"allow"` → `"warn"` by the current HEAD commit `5f33be66` (2026-08-11, dated after this task's 2026-08-10 completion; tracked-but-unexecuted in `task/draft/058_workspace_allow_sweep_per_crate.md`, census "1905 sites workspace-wide"), and independently by unrelated pre-existing debt in transitive dependencies pulled in only by the (unrelated) `adapter-webgl` feature (`browser_log`, `minwebgl`). Scoped re-run covering exactly this task's own feature surface — everything except the unrelated `adapter-webgl` chain — (`cargo clippy -p tilemap_renderer --all-targets --no-default-features --features enabled,adapter-svg,adapter-terminal,cli,scene-model -- -D warnings -A clippy::allow_attributes_without_reason`) → exit 0, zero warnings — directly covers `svg.rs` and `tests/svg_backend_test.rs`, this task's only touched files.

### Anti-faking checks

- [x] AF1 — Guards against a future edit silently losing a test during a merge/relocation instead of moving it: re-run M3's `29 + 54` inline+relocated sum after any future `svg.rs`/`svg_backend_test.rs` edit — must still equal the then-current total with no unexplained drop.
- [x] AF2 — Guards against a private helper being exposed "solely for test placement," re-widening the public API for no caller (the exact trade-off this task's own Fix Verification gate rejected): any future move of one of the 8 named exception-list helpers (C3) out of the inline module must be justified by a real external caller, not test convenience alone.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 035's workspace test-coverage census per Crate
  Scope Unity (PROC17).
- **[2026-08-10]** `IMPLEMENTED` — Census re-derived at pickup: **83** inline tests (not 80),
  ALL in one file — the SvgBackend adapter's single test module in `src/adapters/svg.rs` —
  plus one `#[ cfg( test ) ]`-only private fn (`png_dimensions`, kept: its tests are among the
  29 below). Existing tests/ (39 tests over 4 files) covers disjoint domains
  (trait contract via a synthetic TestBackend, assets/commands/types) — zero duplication with
  the inline block, so no consolidation deletions arose. Per-test classification outcome:
  - **54 tests RELOCATED** to new `tests/svg_backend_test.rs` (feature-gated
    `adapter-svg`): they drive the backend purely through public API — `SvgBackend::new`,
    pub `set_viewport_scale`/`set_viewport_offset`, and the `Backend` trait — and assert on
    `output()`'s SVG string. Two viewport-wrapper tests read the private `content` buffer for
    no semantic reason (`output()` is literally `Output::String( content.buffer() )`) and were
    rewritten to the public `render` helper — lossless. The inline `empty_assets` duplicate was
    consolidated onto the existing `tests/helpers::empty_assets`. Moved helpers:
    `svg800x600`/`render`/`body`/`defs`/`mesh_svg`/`begin_text_cmd`.
  - **29 tests KEPT INLINE** as a documented exception (rationale comment on the module): they
    pin private formatting/encoding helpers — `transform_to_svg_static`/`_local` (8+1, Y-flip
    math), `anchor_to_svg` (9), `path_to_href` (1), `png_dimensions` (2), `detect_image_mime`
    (1), `bitmap_to_png` (5), `SvgContentManager` (1), plus `image_encoded_png_stores_dimensions`
    (1) whose PNG fixture is built via the private encoder. None are in the `mod_interface`
    exports; publishing them solely for test placement would widen the API for no caller.
  - **`bytemuck` added to `[dev-dependencies]`** (the relocated disk-geometry and mesh tests
    cast fixture slices; the crate does not re-export it). Relocation performed by an
    assertion-guarded partition script (91 parsed units, per-unit fn-close matching, moved-side
    dedent, 29/54 count asserts); `tests/readme.md` structure + domain map updated.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Suite green: log `-0040` exit 0 — unit
  29/29 (exception keepers), `tests/svg_backend_test.rs` 54/54 RELOCATED, existing suites
  39/39 untouched (assets 9, backend 17, commands 4, types 9), doc-tests 8 pre-existing
  ignored. 29 + 54 = 83 — every original test preserved. In-loop adversarial catches: (1) the
  partition script's tail splice re-included the old module-close brace (`lines[ 3759: ]`
  instead of `[ 3760: ]`) — caught by the first compile's unexpected-delimiter error, repaired
  by an assertion-guarded single-line delete; (2) the parser assumed fn-only units and tripped
  its own loose-line assert on a module-level `const PNG_MAGIC` — extended with single-line
  const units routed KEEP; (3) one relocated test carried a fn-local `use crate::assets::…`
  which re-resolves to the TEST crate after relocation — rewritten to
  `use tilemap_renderer::assets::…`. Pre-authoring check worth recording: the new test file
  carries `#![ cfg( feature = "adapter-svg" ) ]` because the adapter layer is feature-gated in
  `src/adapters/mod.rs` — without it a default-features run would fail to resolve
  `adapters::svg` (verified against the mod_interface cfg attributes before writing, not
  caught by a failing run).

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | Draft's 064-collision warning moot — 064 completed earlier, no concurrent edits | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | — | — |
| D4 | Implementation Readiness | 🟡 | 🟢 | Census drift (83 real vs 80 filed); partition script's const-unit gap and tail-splice off-by-one | Script extended + assertion-guarded repair; census corrected in record |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | No test deleted (29+54=83); no mocks; loud failures preserved | — |
| B2 | Test-First | 🟢 | 🟢 | — | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | — | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Private-helper exposure rejected (API widening for zero callers); wrapper tests rewritten to public `render` only because `output()` is literally the same buffer — lossless | — |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0040` exit 0: 29/29 unit + 54/54 relocated + 39/39 existing | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Exception comment names all 8 pinned private helpers + both-sides-helpers rationale; readme domain map updated | — |
| B7 | Code Cleanliness | 🟡 | 🟢 | Relocated file initially carried a fn-local `use crate::assets::…` that re-resolves to the test crate | Rewritten to `use tilemap_renderer::assets::…`; `adapter-svg` gate was included from the start (verified against `src/adapters/mod.rs` cfg before authoring) |
| **Total** | | 🔴 | 🟢 | 4 findings resolved in-loop | 15/15 |
