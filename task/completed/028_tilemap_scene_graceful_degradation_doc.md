# Resolve tilemap_scene's graceful-degradation documentation contradiction

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tilemap_scene
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

`module/helper/tilemap_scene`'s doc set (`docs/algorithm/readme.md`, `docs/api/readme.md`,
`docs/format/readme.md`, `docs/invariant/readme.md`, `docs/pitfall/readme.md`, `readme.md`, `src/lib.rs`
— all touched by this repo's recent docs-entity migration commits) contains a contradiction about how the
crate handles graceful degradation (e.g. malformed/unsupported tilemap input), per the audit triage plan.
P4 (rewrite bucket) — **the exact contradiction's specific claims were not preserved precisely through
this session's context compaction; re-derive by reading the current degradation-handling code against
each doc instance before rewriting.**

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P4 (doc rewrite)
  tier, Rewrite-from-scratch bucket. Flagged: citation detail needs re-derivation at pickup.
- **[2026-08-10]** `IMPLEMENTED` — Re-derived the contradiction, then rewrote every stale site to match
  the code. The doc set told **three mutually-incompatible stories** about missing-sprite degradation:
  - **Story A (warn + magenta-checkerboard placeholder, continue):** `docs/algorithm/002` §Missing-sprite
    handling (plus its Scope lines naming "missing-sprite fallback"/"placeholder handling"),
    `docs/api/001` ("surface as described in `algorithm/002` (render-time warning + placeholder) rather
    than as an `Err` from `render()` … not per-sprite resolution failures"), `docs/format/004` ("contrast
    … `algorithm/002`, which *does* substitute a placeholder"), `docs/invariant/001` Out-of-Scope
    ("render-time missing-sprite fallback … deliberate leniency").
  - **Story B (skip with a warning):** `docs/format/005`'s `External` row — which also miscited the API
    as `set_sprite` while its own cross-ref table says `set_external_sprite`.
  - **Story C (silent skip, no error, no placeholder):** `roadmap.md`'s `External` sprite-source item —
    the only honest witness, but its pending-option pointer ("§12.2 of the spec") dangled: no spec.md
    exists post-migration.
  **Verified actual behavior** (Story C, sharpened): (1) unset `External` slot → that layer of that
  instance silently emits nothing (`src/compile/frame.rs` `compile_instance_layer` unset-slot early
  return + free-pos counterpart; pinned by `tests/renderer_test.rs` "unset External slot must not emit
  any Sprite"); (2) any unresolvable reference during the walk → hard `Err(CompileError::UnresolvedRef)`
  from `render()` (13 `.ok_or_else(UnresolvedRef)` sites in `frame.rs`), incl. a *set* `External` slot
  whose `(asset, frame)` was never pre-allocated (`IdMap::sprite` is pure lookup; `compile/assets.rs`
  explicitly skips External pre-allocation); (3) autotile mask miss cannot degrade —
  `NeighborBitmaskSource::ByMapping.fallback` is a required `Box<SpriteSource>` (`src/source.rs`), not an
  `Option`, so "no matching entry and no fallback" is unrepresentable. No placeholder texture, no
  checkerboard, no logging — the crate has zero log/tracing dependencies (`Cargo.toml`). **Edits:**
  `algorithm/002` Scope + full §Missing-sprite rewrite (3-case actual semantics, placeholder framed as
  roadmap-tracked pending option); `api/001` missing-sprite paragraph promoted to numbered spec
  divergence 4 (intro "three"→"four respects", Compatibility Guarantees "three"→"four divergences");
  `format/004` contrast clause corrected (both layers fail hard; sole leniency = unset-`External` skip);
  `invariant/001` Out-of-Scope reworded (runtime semantics, not "fallback"); `format/005` `External` row
  (silent skip, no warning; set-but-unresolvable = `UnresolvedRef`; API name fixed to
  `set_external_sprite`); `roadmap.md` dangling "§12.2" pointer replaced with a live reference to
  `docs/algorithm/002`. Verification: residue greps clean (no placeholder-as-fact, no "item 4"/"three
  divergences" leftovers); full `tilemap_scene` suite green 169/169 (`-0031_longrun.log`, exit 0).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — 15-dimension Tier 2 gate passed (see Verification
  Record). Adversarial pass caught 1 finding in-loop: confirming pass declared cross-file consistency
  while `api/001`'s Compatibility Guarantees still said "three divergences" and two new references used
  the fragile "roadmap.md item 4" ordinal instead of the item's name — all three residues fixed and
  re-swept clean. Moved draft/ → completed/.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | Docs corrected to code truth; fictional warn+placeholder feature deliberately NOT implemented (stays a roadmap pending option) | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | Zero code changes — 6 doc files only; roadmap edit confined to the dangling pointer inside the contradicting item | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟡 | 🟢 | Adversarial pass found stale "three divergences" at api/001 Compatibility Guarantees + 2 fragile "item 4" ordinal refs after confirming pass had declared consistency | 3 residues fixed; re-grep clean |
| B2 | Test-First | 🟢 | 🟢 | Behavior claims pinned by pre-existing `renderer_test.rs` unset-External assertion; suite re-run green | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | Contradiction proven: 0 placeholder/warn/log hits in src/, no logging dep, required `fallback` field, 13 hard-error sites | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | — | — |
| B5 | Fix Verification | 🟢 | 🟢 | Residue greps clean; `cargo nextest run -p tilemap_scene` 169/169 (`-0031_longrun.log`, exit 0) | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Divergence now a numbered api/001 spec-divergence entry; pending option preserved in roadmap with live pointer | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | No code touched; evidence log hyphen-prefixed | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved | 1/1 |
