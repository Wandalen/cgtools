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

## Verification

### Checklist

- [x] C1 — Does `docs/algorithm/002`'s Missing-sprite handling section still describe exactly the 3-case Story-C semantics (unset-`External` silent skip; any other unresolved ref → hard `UnresolvedRef`; autotile mask miss cannot degrade)? Read confirms all 3 cases present verbatim at `docs/algorithm/002_scene_rendering_pass.md:44-50`.
- [x] C2 — Does `docs/api/001` still disclose "four" (not "three") divergences, with the missing-sprite behavior as divergence 4? `grep -n "four divergences\|four respects" docs/api/001_renderer_integration_api.md` → 2 hits (lines 34, 43); `grep -c "three divergences"` → `0`.
- [x] C3 — Does `docs/format/005`'s `External` row still cite the correct API name `set_external_sprite` (not the stale `set_sprite` typo the audit found)? `grep -n "set_external_sprite" docs/format/005_sprite_sources.md` → present (line 23); `grep -rn "\bset_sprite\b" docs/ roadmap.md` → `0` hits doc-wide.
- [x] C4 — Does `roadmap.md`'s `External` item still point live at `docs/algorithm/002` instead of the dangling `§12.2 of the spec` reference? `grep -rn "§12.2\|12\.2 of the spec\|spec\.md" roadmap.md docs/` → `0` hits; the item's body now reads "`docs/algorithm/002` documents actual behavior."
- [x] C5 — Do the underlying code facts the docs now assert still hold? (a) `frame.rs` still has exactly `13` `.ok_or_else( || CompileError::UnresolvedRef` sites (`grep -n UnresolvedRef src/compile/frame.rs` → 13 hits); (b) `NeighborBitmaskSource::ByMapping.fallback` is still a required `Box< SpriteSource >`, not `Option` (`src/source.rs:180`); (c) the crate still has zero log/tracing dependencies (`grep -iE "^log|tracing" Cargo.toml` → 0 hits).

### Measurements

- [x] M1 — Disclosed spec divergences in `docs/api/001`: `4` (was: `3`, per `git show 4469eafb^:module/helper/tilemap_scene/docs/api/001_renderer_integration_api.md`, whose Compatibility Guarantees paragraph reads "Beyond the three divergences disclosed above").

### Invariants

- [x] I1 — Test suite (crate-scoped, `longrun`-launched): `cargo nextest run -p tilemap_scene --all-features` → exit `0`, "169 tests run: 169 passed, 0 skipped" (`-0141_longrun.log`) — matches the 169/169 this task's own History cites.
- [ ] I2 — Compiler/lints (crate-scoped, `longrun`-launched): `cargo clippy -p tilemap_scene --all-targets --all-features -- -D warnings` → exit `101` (FAIL). Root cause is entirely pre-existing and outside this task's own edits — TASK-028 touched 6 doc files plus `roadmap.md` only (confirmed zero `src/` changes via its own D5 gate). The build aborts on `1` site in dependency `browser_log` plus `40` sites in dependency `tilemap_renderer` (all `#[allow(clippy::exhaustive_structs)]` missing `reason=`), before ever reaching `tilemap_scene`'s own lint pass (`-0141_longrun.log`). Separately worth flagging, though also not a TASK-028 regression since it touched no `src/`: `tilemap_scene`'s own `src/` independently carries 7 `#[allow(...)]` attributes with no inline `reason=` (`compile/frame.rs:967,971`; `compile/vertex.rs:90`; `compile/neighbors.rs:139,193`; `compile/edges.rs:178,183`), each justified only by a preceding `//` comment per this workspace's `rulebook.md` convention — a convention that does not satisfy clippy's `allow_attributes_without_reason` lint, which requires the reason inline in the attribute itself.

### Anti-faking checks

- [x] AF1 — Guards against the 3-story contradiction silently creeping back in if only one doc file is touched in a future change: re-running `grep -rn -i "placeholder\|checkerboard" docs/ roadmap.md` must keep showing every hit framing the placeholder as a roadmap-tracked *pending* option, never as current behavior.
- [x] AF2 — Guards against the exact residue class this task's own adversarial pass caught ("three divergences" / "item 4" ordinal references) recurring after a future partial edit: re-running `grep -rn "three divergences\|item 4" docs/ roadmap.md` must stay at `0` hits.

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
