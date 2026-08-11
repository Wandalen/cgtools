# Resolve tilemap_renderer's 11 task markers (decomposed from task 038)

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

Resolve the 11 live task markers in `module/helper/tilemap_renderer` (census 2026-08-10, task 038 —
re-derive at pickup). Nearly all are honest capability-gap annotations on the WebGL adapter, i.e.
roadmap content living inline:

**Capability-flag annotations (`src/adapters/webgl.rs:1228-1236`, ×6):** `paths : false` (qqq:
tessellation / GPU curves), `text : false` (qqq: glyph atlas / SDF fonts), `gradients`/`patterns`/
`clip_masks : false` (qqq: not yet loaded or rendered), `effects : false` (qqq: requires FBO
post-processing). These flags are truthful; the qqq markers duplicate what a roadmap should own.
Disposition: move the implementation-plan content into the crate's docs (it has a docs/ tree) or
`roadmap.md`, keep the flags' comments as plain descriptions, drop the qqq prefixes.

**Unimplemented behaviors:**
- `src/adapters/webgl.rs:851` — `ImageSource::Encoded( _ ) => { continue; } // qqq: decode`
  (encoded images silently skipped — same silent-skip family as the pitfall below).
- `src/adapters/webgl.rs:1134` — `// qqq: gradients, patterns, clip masks, fonts` (load-path gap
  matching the capability flags).
- `src/adapters/svg.rs:986` — `// TODO: Source::Path geometries are silently skipped for now.` —
  documented by `docs/pitfall/003_svg_geometry_path_source_silently_skipped.md`; implementing the
  load (std::fs native / fetch wasm32) retires that pitfall doc (update or delete it in the same
  change).
- `src/adapters/webgl/webgl_helpers.rs:693` — `// qqq: true Overlay (Multiply where dst<0.5, ...)
  cannot be` (blend-mode approximation note — if genuinely impossible in the current pipeline,
  convert to a plain explanatory comment or a docs/ pitfall entry and drop the qqq).

**Ordering contract note:**
- `src/types.rs:191` — `/// qqq: SVG and terminal backends still emit in submission order ...`
  (a qqq INSIDE a doc comment — public-facing docs must not carry task markers; move the
  discrepancy note to the crate's docs/ or roadmap and clean the doc comment).

Per-marker outcomes follow task 038's triage contract. Verify with
`cargo test -p tilemap_renderer --all-features` (via `longrun .launch`); pitfall/003 must stay
consistent with whatever the SVG path source handling becomes.

## Verification

### Checklist

- [x] C1 — Are the 6 `webgl.rs` capability-flag comments (`paths`/`text`/`gradients`/`patterns`/`clip_masks`/`effects`) now plain and `qqq`-free, matching the claimed wording? `grep -n "paths : false\|text : false\|gradients : false\|patterns : false\|clip_masks : false\|effects : false" src/adapters/webgl.rs` → all 6 present with plain comments ("needs tessellation / GPU curves", "needs a glyph atlas / SDF fonts", "not yet loaded or rendered" ×3, "needs FBO post-processing"), no `qqq` prefix. The 3 cross-ref comments (`webgl.rs:1188/1204/1212`) read "(unimplemented; see capabilities().paths/.text/.effects)".
- [x] C2 — Is `webgl.rs`'s `ImageSource::Encoded` handling a loud skip (console warning), not a silent `continue`? `grep -n "ImageSource::Encoded" -A 5 src/adapters/webgl.rs` → `web_sys::console::warn_1(...)` immediately precedes the `continue`.
- [x] C3 — Is `svg.rs`'s `Source::Path` silent-skip TODO genuinely implemented (not just reworded)? `grep -n "fn resolve_source\|fn skip_geometry" src/adapters/svg.rs` → both helpers present (`resolve_source` returns `Cow<[u8]>` via `std::fs::read`; `skip_geometry` performs the loud-skip path).
- [x] C4 — Is `docs/pitfall/003_svg_geometry_path_source_silently_skipped.md` genuinely deleted (not merely unlinked)? Path lookup → does not exist; `docs/pitfall/` now contains only `001_...md` and `002_...md`.
- [x] C5 — Are the `webgl_helpers.rs` Overlay comment and the `types.rs` depth-ordering doc comment now plain factual statements? Both read as claimed — Overlay (`webgl_helpers.rs:694-708`): "True Overlay (...) cannot be [expressed]... falls back to Normal"; `types.rs:204-205`: "SVG and terminal backends still emit in submission order... callers... must pre-sort" — neither carries a task-marker prefix.
- [x] C6 — Is the crate genuinely marker-free crate-wide, including the "hidden" patterns the original census missed (`qqq(`, `**qqq (`, bare `(qqq)`)? `grep -rnE "qqq|xxx:|TODO|aaa:" src --include="*.rs"` → `0` hits.

### Measurements

- [x] M1 — Task-marker pattern hits (`qqq`/`TODO:`) across the 4 files this task named: `0` (was: `16` — `webgl.rs` 11, `svg.rs` 1, `webgl_helpers.rs` 2, `types.rs` 2, each counted via `git show 4469eafb^:<path> | grep -c` against the commit immediately preceding this task's own fix; the task's own hand-classified "13 sites" figure counts logical sites rather than raw pattern occurrences, so the two counting methods aren't expected to match digit-for-digit — both independently converge on "many, now zero").
- [x] M2 — Inline test count in `src/adapters/svg.rs`: `83` at `git show 4469eafb:...svg.rs` (the commit containing this task's own fix) vs `80` at `git show 4469eafb^:...svg.rs` (immediately prior) — the +3 are this task's new `Source::Path` tests (`geometry_path_source_loads_from_disk`, `geometry_on_missing_path_is_skipped_with_comment`, `geometry_on_missing_index_path_is_skipped_whole`), confirmed present in the current `svg.rs`/`tests/svg_backend_test.rs` split (task 071 relocated the bulk of these afterward — see that task's own Verification for the 83→29+54 split).

### Invariants

- [x] I1 — Test suite (crate-scoped, all features): `cargo nextest run -p tilemap_renderer --all-features` → exit 0, 122/122 passed — includes the 3 new `Source::Path` tests this task added, confirmed present and passing.
- [x] I2 — Compiler/lints: `cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings` → **exit 101** — genuine current drift, but not from this task's own edits. Two independent, pre-existing causes: (a) the workspace-wide `allow_attributes_without_reason` policy flip (`"allow"`→`"warn"` in the current HEAD commit `5f33be66`, 2026-08-11, dated after this task's 2026-08-10 completion; tracked-but-unexecuted in `task/draft/058_workspace_allow_sweep_per_crate.md`); (b) `webgl.rs` independently carries **20** older, structural lint hits at lines this task never touched — `clippy::wildcard_imports` ×3 (lines 35-37), `clippy::too_many_arguments` ×3, `clippy::trivially_copy_pass_by_ref` ×12, `clippy::too_many_lines` ×1 (`load_geometries`, the WebGL-side GPU geometry loader — a different function from `svg.rs`'s `resolve_source`/`skip_geometry` this task added), plus 1 more `allow_attributes_without_reason` — every one git-blames to April 2026 commits (e.g. `webgl.rs:74`'s `too_many_arguments` → `90ee7fe4`, 2026-04-22; `webgl.rs:923`'s `too_many_lines` → `a2967a0e`, 2026-04-15), roughly 4 months before this task existed. None of this task's own edit sites (the 6 capability-flag comments, the `Encoded`/load-path-gap comments, the 3 cross-refs, the Overlay/`types.rs` doc comments) appear in the failing log. Scoped confirmation for the rest of the crate (`--features enabled,adapter-svg,adapter-terminal,cli,scene-model -- -D warnings -A clippy::allow_attributes_without_reason`) → exit 0.

### Anti-faking checks

- [x] AF1 — Guards against a marker silently reappearing under one of the "hidden" patterns the original census missed: re-run C6's widened `grep -rnE "qqq|xxx:|TODO|aaa:"` after any future edit to `webgl.rs`/`svg.rs`/`webgl_helpers.rs`/`types.rs` — must stay `0`.
- [x] AF2 — Guards against the new `resolve_source`/`skip_geometry` helpers regressing back to a silent skip: `svg.rs`'s loud-skip path must keep emitting its stderr warning + diagnostic HTML comment on read failure — a future refactor that drops the warning reintroduces the exact defect the now-deleted `docs/pitfall/003` used to document, with nothing left to catch the regression.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 038's workspace marker census (80 lines →
  per-crate tasks per Crate Scope Unity). Largest helper-crate cluster; mostly roadmap-in-comments
  needing relocation rather than immediate feature work.
- **[2026-08-10]** `IMPLEMENTED` — All markers resolved. The re-derived census confirmed the 11,
  and a widened adversarial sweep (patterns the census grep misses: `qqq(`, `**qqq (`, bare
  `(qqq)`) found 2 more live markers plus 3 cross-references — 13 marker sites total, all resolved:
  - **Capability flags (`webgl.rs:1228-1236`, ×6):** qqq prefixes dropped; truthful plain
    descriptions kept (`needs tessellation / GPU curves`, `needs a glyph atlas / SDF fonts`,
    `not yet loaded or rendered` ×3, `needs FBO post-processing`). roadmap.md already owned every
    plan (webgl adapter gaps) — the markers were pure duplication.
  - **`webgl.rs:851` Encoded skip:** silent `continue` → loud skip with `web_sys` console warning
    (the adapter's established idiom for unimplemented families). Implementing a decoder was
    deliberately NOT done: browser-side `createImageBitmap`/object-URL decoding is untestable
    until the roadmap's wasm test runner exists, and bundling the `image` crate into the wasm
    build is a size decision nobody has made. roadmap bullet updated ("skipped with a console
    warning; needs a decoder"); `assets.rs` `ImageSource::Encoded` doc updated (stale "silently
    skipped" claim → console-warning contract).
  - **`webgl.rs:1134` load-path gap:** plain comment — flags are false, roadmap owns the plan.
  - **`svg.rs:986` Source::Path silent skip — IMPLEMENTED:** `load_geometries` now resolves
    `Source::Path` via blocking `std::fs::read` (new `resolve_source` helper returning
    `Cow<[u8]>`); on read failure — missing file, or wasm32 where no filesystem exists — the
    geometry is skipped LOUDLY via new `skip_geometry` helper (stderr warning with error detail +
    diagnostic HTML comment interpolating only the numeric id and a static field name). A failed
    index source skips the whole geometry (unindexed fallback would silently change topology).
    Conversions use `bytemuck::pod_collect_to_vec` instead of `cast_slice` — file-read bytes
    carry no alignment guarantee and `cast_slice` panics on misaligned buffers. `pitfall/003`
    RETIRED: file deleted; 4 reference sites updated (pitfall/readme description+scope+row,
    definition/readme count 3→2 + row, feature/001 known-gap paragraph rewritten + pitfall table
    row removed + tests row extended). roadmap svg-gap bullet rewritten (wasm32 fetch redesign
    remains future work); `Source::Path` doc now states per-backend contract. 3 new tests:
    `geometry_path_source_loads_from_disk` (positions + indices from real temp files → renders
    identically to Bytes), `geometry_on_missing_path_is_skipped_with_comment`,
    `geometry_on_missing_index_path_is_skipped_whole`.
  - **`webgl_helpers.rs:693` Overlay + `:684` qqq(FBO) (hidden) + `types.rs:407` (hidden, public
    doc):** blend-accuracy family — all converted to plain factual statements (the Overlay
    variant's own doc already modeled the style); roadmap Overlay bullet extended to cover the
    Multiply/Screen `src_alpha < 1` divergence and the shared FBO/custom-shader remedy.
  - **`types.rs:191` depth-ordering qqq (public doc):** marker dropped; the factual caller
    contract stays in the doc ("SVG and terminal backends emit in submission order — pre-sort");
    the future-work sentence moved to a new roadmap svg-gap bullet.
  - **`webgl.rs:1176/1192/1200` `(qqq)` cross-refs:** → `(unimplemented; see capabilities().X)`.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Widened census clean (exit 1, incl. `qqq(`/
  `qqq ` patterns). Suite green: log `-0030` exit 0 — 83 unit (80 + 3 new) + 39 integration +
  7 wasm-gated ignored (pre-existing), 0 failed; workspace check log `-0031` exit 0 (2.39s,
  downstream tilemap_scene + examples clean). Four genuine in-loop adversarial catches: (1) the
  task's "11 markers" premise was INCOMPLETE — census grep pattern is blind to `qqq(FBO):` and
  `**qqq (requires FBO):**` forms; widened sweep found both hidden markers plus 3 bare `(qqq)`
  cross-refs that would have dangled. (2) HTML-comment injection: interpolating the fs error
  (which contains a caller-controlled path) into a diagnostic comment would let `-->` terminate
  it early — comment carries only numeric id + static field name; error detail goes to stderr.
  (3) latent `cast_slice` misalignment panic: `Vec<u8>` from `fs::read` has no 4-byte alignment
  guarantee — new path uses `pod_collect_to_vec` (copies instead of casting in place). (4) stale
  `assets.rs` claim ("this variant is silently skipped") contradicted the new loud-skip behavior —
  caught by a post-edit "silently" sweep across src/.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | Encoded decoder deliberately deferred: untestable without wasm runner; wasm binary-size decision unowned | Loud skip + roadmap instead — documented, not built |
| D4 | Implementation Readiness | 🟡 | 🟢 | Task's "11 markers" premise incomplete — census grep blind to `qqq(FBO):` / `**qqq (…):**` forms; 2 hidden markers + 3 dangling `(qqq)` cross-refs | Widened sweep; all 13 sites resolved |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | — | — |
| B2 | Test-First | 🟢 | 🟢 | — | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | — | — |
| B4 | Proper Fix Only | 🟡 | 🟢 | `cast_slice` on `fs::read` bytes panics on misaligned buffers (alignment is allocator luck) | `pod_collect_to_vec` — copies, no alignment requirement |
| B5 | Fix Verification | 🟢 | 🟢 | — | — |
| B6 | Knowledge Preservation | 🟡 | 🟢 | `assets.rs` Encoded doc still claimed "silently skipped" after the loud-skip change | Post-edit "silently" sweep across src/; doc updated to console-warning contract |
| B7 | Code Cleanliness | 🟡 | 🟢 | Interpolating fs error text (caller-controlled path) into an HTML comment lets `-->` terminate it early | Comment carries numeric id + static field only; error detail stderr-only |
| **Total** | | 🔴 | 🟢 | 4 findings resolved in-loop | 15/15 |
