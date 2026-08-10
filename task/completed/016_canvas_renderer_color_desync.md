# Fix canvas_renderer silent color-desync bug

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/canvas_renderer
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Fix a bug in `canvas_renderer` where rendered color state can silently desynchronize from the logical
state it's meant to track (no panic, no error — just visually wrong output), identified during the
workspace audit (P2 — remaining logic bugs, Fix-in-place). **Carried forward from the audit triage plan —
exact file/line is not re-verified in this filing pass; re-confirm against current
`module/helper/canvas_renderer/src/` before touching.** Write a test that asserts color-state consistency
across the specific operation sequence that triggers the desync before fixing.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P2 (remaining logic
  bugs) tier, Fix-in-place bucket.

- **[2026-08-10]** `IMPLEMENTED` — Confirmed the citation was stale as warned: no cache-based desync
  exists in this crate (grepped `cache|last_|prev|dirty|skip|memo` across `canvas_renderer/src/`,
  zero hits). Real site: `module/helper/canvas_renderer/src/renderer.rs`, `CanvasRenderer::render`
  (pre-fix lines 238, 253-254, 270). The mesh-color lookup index advanced once per *traversed scene
  node* (mesh or not), while `colors` is documented as one entry per *mesh*, in mesh-encounter order
  ("traverses the scene to render all mesh nodes with their corresponding colors from the colors
  array"). Any non-mesh node (`Object3D::Other` — e.g. the transform-only "parent" nodes that
  `primitive_generation::primitives_data_to_gltf` explicitly creates for `PrimitiveData` entries
  without attributes) visited before or between mesh nodes silently shifted every later mesh onto
  the wrong `colors` entry, or past the array end into the magenta fallback — no panic, no error,
  just a wrong-colored mesh.
  Fix: extracted the mesh-to-color resolution into a new pure, GL-free `resolve_mesh_colors`
  function (`renderer.rs`, `mod private`, `pub`-within-file-scope only — not part of the crate's
  external API) that indexes `colors` by `resolved.len()` (grows only when a mesh is actually
  pushed) instead of a counter shared with every traversed node. `render` now resolves
  `mesh_colors` once up front via this function and walks it with its own mesh-only counter during
  the real GL-drawing traversal, removing the old duplicated buggy inline indexing.
  Test: `renderer::tests::resolve_mesh_colors_stays_in_sync_across_non_mesh_siblings` (in-source
  `#[cfg(test)] mod tests`, per this repo's `rulebook.md` "Test placement" rule for private
  helpers — `resolve_mesh_colors` is not externally reachable, so its test can't live in `tests/`
  without polluting the public API). Builds a scene with two top-level groups each owning one mesh
  child, so a non-mesh node sits between the two meshes in traversal order. Confirmed genuinely red
  pre-fix (`cargo nextest run -p canvas_renderer`: mesh 1 resolved to mesh 2's color
  `[0.0, 1.0, 0.0, 1.0]` instead of its own `[1.0, 0.0, 0.0, 1.0]`), green post-fix.
  Verification: `longrun .launch dir::/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/canvas_renderer
  -- will .test l::3` (package-scoped) → exit 0, "Summary: 4/4 commands passed, 0 failed" (elapsed
  61s). Direct follow-up `cargo nextest run -p canvas_renderer` confirms
  `canvas_renderer renderer::tests::resolve_mesh_colors_stays_in_sync_across_non_mesh_siblings`
  PASS, 1 test run: 1 passed, 0 skipped.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | Private helper tested in-source per local `rulebook.md` § Test placement — verified `resolve_mesh_colors` is `pub` only within `mod private`, not re-exported by `mod_interface!` | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| B1 | Rulebook Compliance | — | 🟢 | — | — |
| B2 | Test-First Requirement | — | 🟢 | — | — |
| B3 | Evidence of Failure | — | 🟢 | Cross-checked mechanism independently by reading `Scene::traverse`'s source directly (deterministic `Vec` iteration, same call site used by both the color-resolution and draw closures) rather than solely trusting the reported RED output | — |
| B4 | Proper Fix Only | — | 🟢 | Root-cause fix (index by meshes-actually-resolved, not nodes-traversed), not a symptom patch | — |
| B5 | Fix Verification | — | 🟢 | Independently re-ran myself: `longrun`-launched package-scoped `will .test l::3` → exit 0, 4/4; direct `cargo nextest run -p canvas_renderer resolve_mesh_colors` → 1/1 passed | — |
| B6 | Knowledge Preservation | — | 🟢 | 3-field `Fix(TASK-016)`/`Root cause`/`Pitfall` source comment + 5-section test doc comment confirmed via `git diff` | — |
| B7 | Code Cleanliness | — | 🟢 | Old counter/local removed cleanly; `git status` scoped to canvas_renderer shows only the 2 expected files touched | — |
| **Total** | | 🔴 | 🟢 | 0 | 0/0 |

**Aggregate verdict:** PASS — all 15 dimensions clean on both passes, zero Blocking Findings. Verification independently re-executed (`longrun`, direct `cargo nextest`, `Scene::traverse` source read, `git diff`/`git status`) rather than solely trusted from the implementing subagent's own prose, per this session's Stale Evidence Trust discipline.
