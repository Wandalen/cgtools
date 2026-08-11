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

## Verification

### Checklist

- [x] C1 — Does `resolve_mesh_colors` still index `colors` by `resolved.len()` (count of meshes already resolved), never a counter shared with the raw traversal? `src/renderer.rs:109-130` — `resolved.push( *colors.get( resolved.len() ).unwrap_or( &default_color() ) )`, executed only inside the `Object3D::Mesh` branch of the traversal closure.
- [x] C2 — Does `render()` still resolve `mesh_colors` once up front and walk it with its own mesh-only counter, with no leftover duplicate of the pre-fix shared-counter indexing? `src/renderer.rs:295-335` — `let mesh_colors = resolve_mesh_colors( scene, colors );` (line 297) runs once before the draw traversal; the draw closure indexes `mesh_colors.get( mesh_i )` (line 313) with `mesh_i` incremented only inside its own `Object3D::Mesh` branch (line 316) — no second/legacy indexing path exists anywhere else in the function.
- [x] C3 — Is `resolve_mesh_colors` still excluded from the crate's public API (private-by-design, not re-exported)? `grep -n resolve_mesh_colors src/lib.rs` → `0` hits; the crate-root `crate::mod_interface!` block in `renderer.rs` (lines 487-493) exposes only `CanvasRenderer` via `orphan use`.
- [x] C4 — Is the regression test still present, still exercising a non-mesh node between two mesh nodes, and still passing? `src/renderer.rs:449-484` builds `group_1`→`mesh_1` and `group_2`→`mesh_2` (two non-mesh groups, one mesh child each); `cargo nextest run -p canvas_renderer --all-features` (see I1) → `resolve_mesh_colors_stays_in_sync_across_non_mesh_siblings` PASS.
- [x] C5 — Does the source still carry the required 3-field `Fix(TASK-016)` / `Root cause` / `Pitfall` comment immediately above `resolve_mesh_colors`? `src/renderer.rs:101-108` — all 3 fields present verbatim, matching this workspace's bug-fix documentation convention.

### Measurements

- [x] M1 — Live reference count to `resolve_mesh_colors` in `renderer.rs`: `2` (1 definition at line 109, 1 call site at line 297) — confirms `render()` resolves mesh colors through exactly one path, with no leftover duplicate of the pre-fix inline indexing. No historical "was" is cited here: `git show 4469eafb^:module/helper/canvas_renderer/src/renderer.rs` already contains the fix, so the true introducing commit sits further back than this pass could cheaply isolate — no number is fabricated in its place.

### Invariants

- [x] I1 — Test suite (crate-scoped, `longrun`-launched): `cargo nextest run -p canvas_renderer --all-features` → exit `0`, "1 test run: 1 passed, 0 skipped" (`-0138_longrun.log`).
- [ ] I2 — Compiler/lints (crate-scoped, `longrun`-launched): `cargo clippy -p canvas_renderer --all-targets --all-features -- -D warnings` → exit `101` (FAIL) — but root-caused entirely outside `canvas_renderer`. The build aborts while checking the transitive dependency `browser_log` (`module/helper/browser_log/src/panic.rs:82`, `#[allow(clippy::exhaustive_structs)]` missing the `reason = "..."` that this workspace's `allow_attributes_without_reason = "warn"` lint now demands once escalated to error by `-D warnings`) — clippy never reaches `canvas_renderer`'s own source in this run. Confirmed `canvas_renderer`'s own `src/` carries zero `#[allow(...)]` attributes at all (`grep -rn "allow(" module/helper/canvas_renderer/src/` → 0 hits), so this crate's own code is not implicated. `browser_log`'s working tree is clean/committed (last touched by unrelated commit `5f33be66`) — this is pre-existing workspace drift, unrelated to and predating both TASK-016 and TASK-068 (`-0138_longrun.log`).

### Anti-faking checks

- [x] AF1 — Guards against the shared-counter regression silently reappearing: re-running C1/C2's read of `renderer.rs:109-130` and `:295-335` after any future edit must still show `resolved.len()` and `mesh_i` as the only two counters, each incrementing exclusively inside its own `Object3D::Mesh` branch — a counter shared with the raw, unfiltered traversal is exactly the TASK-016 bug.
- [x] AF2 — Guards against the regression test being weakened back to an all-mesh scene (the exact shape that let the original bug hide, per this file's own "Why Not Caught"): re-running the test must still exercise a scene where a non-mesh node sits between two mesh nodes; `grep -n "group_node()\|mesh_node()" src/renderer.rs` inside `mod tests` must keep showing 2 non-mesh groups, each wrapping exactly 1 mesh child, never a flat all-mesh list.

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
