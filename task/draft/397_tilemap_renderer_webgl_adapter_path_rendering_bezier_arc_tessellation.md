# 397: tilemap_renderer webgl adapter — path rendering (bezier/arc tessellation)

## Execution State

- **id:** 397
- **title:** tilemap_renderer webgl adapter — path rendering (bezier/arc tessellation)
- **state:** 📝 (Draft)
- **open:** true
- **in_motion:** false
- **round:** 1
- **filed:** 2026-08-19 22:51:09
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tilemap_renderer
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **unverified_at:** 2026-08-19 22:56:37
- **unverified_by:** unknown
- **verifying_at:** 2026-08-19 23:18:22
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **redraft_at:** 2026-08-19 23:19:44
- **redraft_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## MOST Goal

**Tracking placeholder — needs scoping before becoming claimable.** `WebGlBackend::capabilities()`
(`src/adapters/webgl.rs`) declares `paths: false`, pinned by task 246's honest-subset test. The SVG
adapter renders `RenderCommand::Path` natively via SVG `<path>` elements; the WebGL2 adapter has no
equivalent — no CPU-side tessellation of bezier/arc path segments into GPU-drawable primitives (e.g.
line-strips or filled triangle meshes via a flattening algorithm) exists yet. `roadmap.md`'s WebGL2
adapter section lists this as remaining work. Too large for one-pass implementation: needs a tessellation
algorithm choice (flattening tolerance, curve-to-line-segment conversion for Quad/Cubic Bezier and Arc),
a fill-vs-stroke decision, and new GPU buffer/pipeline wiring — real design work, not a mechanical patch.

## In Scope

- Design and implement CPU-side path tessellation for `RenderCommand::Path` (Line/QuadTo/CubicTo/ArcTo
  segments, matching the segment vocabulary the SVG and terminal adapters already handle) in
  `src/adapters/webgl.rs`.
- Update `WebGlBackend::declared_capabilities()` to report `paths: true` once implemented.
- Cross-backend visual parity test (bounded tolerance) against the SVG adapter's existing path output,
  once a rendering test pattern exists (see browsee-based pixel verification precedent, task 191).

## Out of Scope

- Text-on-path, gradient/pattern-filled paths, or clip-masked paths — separate roadmap gaps (see
  sibling draft tasks for webgl text rendering and gradient/pattern/clip-mask asset loading).
- Any other adapter's path support — SVG and terminal already handle paths.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized until this is fleshed out into
  a full Quality-Gate task (Test Matrix, Acceptance Criteria, Delivery Requirements re-derived against
  the actual scoped tessellation approach at that time).

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a tracking placeholder, not yet scoped for execution. Not
  intended to progress through SUBMIT/VERIFY toward 🎯 Verified/claimable state until fleshed out.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Related Documentation

- `module/helper/tilemap_renderer/roadmap.md` — WebGL2 adapter remaining-work section
- `task/accepting/246_tilemap_renderer_webgl_adapter_test_coverage.md` — pins today's honest
  `paths: false` capability baseline this task would flip to `true`

## Verification Findings

Fresh Tier 2 Dual-Role Self-Check (8-dimension Readiness Gate), self-administered, no subagent dispatch.

**Gate Check** · Tier: 2 · Type: Full · Verdict: OPEN · Agents: 0 (self, dual-role) · 6/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | — | 🟢 | In Scope/Out of Scope both present and coherent as prose | — |
| D2 | MOST Goal Quality | — | 🔴 | Goal text itself: "needs scoping before becoming claimable," "too large for one-pass implementation." Scoped and Testable components both explicitly unmet | Not fixed — see rationale below |
| D3 | Value/YAGNI | — | 🟢 | Real, concrete roadmap gap; no speculative-work concern | — |
| D4 | Implementation Readiness | — | 🔴 | No Work Procedure section; Delivery Requirements/Acceptance Criteria/Verification all literally read "N/A while this task remains 📝 Draft" | Not fixed — see rationale below |
| D5 | Execution Scope | — | 🟢 | All referenced paths (`src/adapters/webgl.rs`) resolve inside repo | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`tilemap_renderer`) | — |
| D7 | Crate Locality | — | 🟢 | Targets the leaf crate directly | — |
| D8 | Crate Single Responsibility | — | 🟢 | `tilemap_renderer`'s responsibility stays one-sentence-statable | — |
| **Total** | | — | 🔴 | 2 blocking (D2, D4) | 0/2 |

**Adversarial pass:** Attempted to argue D2/D4 should PASS despite the N/A markers — rejected: the task's own
MOST Goal explicitly self-declares "Tracking placeholder — needs scoping before becoming claimable" and its
own Acceptance Criteria section explicitly states "Not intended to progress through SUBMIT/VERIFY toward
🎯 Verified/claimable state until fleshed out." This is not an ordinary Fix-and-Recheck case — the fix isn't
mine to apply (inventing a tessellation-algorithm design speculatively to force a PASS would itself be a
YAGNI violation and would override the filer's own deliberate deferral). Content is structurally identical
in shape and intent to task 291's watch-item pattern
(`task/draft/291_reconsider_gpu_hal_mipmapmsaacompute_support_if_a_real_consumer_emerges.md`), which
correctly stayed in 📝 Draft. This task's Journal shows it was moved ❓ Unverified via a `SUBMIT` event fired
by actor `unknown` — inconsistent with its own body text. Routing to `tsk .verify_redraft` (🔬/❓→📝) to
correct the lifecycle state to match the task's own explicit self-declared content, mirroring 291's disposition.

## Verification Findings

D2/D4 blocking: self-declared tracking placeholder (mirrors task 291 watch-item pattern); explicitly states it should not progress through SUBMIT/VERIFY; Acceptance Criteria/Verification/Delivery Requirements all literal N/A. Misfiled to Unverified via SUBMIT by actor unknown; redrafting to correct lifecycle state to match content.


## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 22:51:09 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-19 22:56:37 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-19 23:18:22 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 23:19:44 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | REDRAFT | verification retry exhausted; returned to draft |
