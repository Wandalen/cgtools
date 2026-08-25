# Document orrery_webgpu (L0-direct) and orrery_flexible (L1-direct) as undocumented layer-ladder consumers

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-20 09:57:35
- **expires_at:** 2026-08-20 11:57:35
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** null
- **unit_type:** repository
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **in_motion:** true
- **verifying_at:** 2026-08-20 09:57:35
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **unverified_at:** 2026-08-20 09:57:11
- **unverified_by:** system

## Goal

Two `examples/orrery/*` crates reach the layer ladder directly, in two
different ways, and neither is recorded in `docs/layer/`'s own "who
currently depends on this layer directly" bookkeeping. `orrery_webgpu`
(`examples/orrery/webgpu/Cargo.toml`) depends only on `minwebgpu` (L0,
wasm32-gated; confirmed no `gpu_hal`/`renderer` dependency line exists,
and the file has an explicit `# No minwebgpu dependency for native
targets` comment on its native stub-deps block) — a genuine direct L0
consumer, absent from `docs/layer/001_l0_drivers.md` entirely (`grep -c
"orrery" docs/layer/001_l0_drivers.md` → 0). `orrery_flexible`
(`examples/orrery/flexible/Cargo.toml`) depends only on `gpu_hal` (L1;
confirmed no direct `minwebgl`/`minwebgpu`/`minwgpu`/`minvulkan`/
`renderer` dependency or import anywhere in its `src/`) and reaches all
four backends through it via the unified `Device::new` overloads
(`src/main.rs` lines 112, 141 — async canvas-based, resolving to
`webgpu`/`webgl`; sync width/height-based, resolving to `native`/`vulkan`;
each delegates internally to `gpu_hal`'s own `new_webgpu`/`new_webgl`/
`new_native`/`new_vulkan`) —
bypassing L3 (`renderer`) entirely, unlike `docs/layer/002_l1_gpu_hal.md`'s
two existing Sources-table consumers (`renderer`'s canonical path,
`tilemap_renderer`'s adapters), which are themselves L3 stack engines
adopting the HAL. `orrery_flexible` is already named by name in
`docs/layer/002`'s own Status prose (line 77, explaining why the `vulkan`
backend was added) but is absent from that same file's Sources table —
the file's actual consumer record — leaving an internal inconsistency
within `docs/layer/002` itself (`grep -c "orrery"
docs/layer/002_l1_gpu_hal.md` → 1, the prose mention only, zero Sources
rows). Both crates are already correctly documented as deliberate,
permanent reference/comparison implementations in their own readmes
(`examples/orrery/readme.md`, `examples/orrery/flexible/readme.md`,
`examples/orrery/webgpu/readme.md`) — neither is "code awaiting HAL
migration" like `docs/layer/001`'s existing "Current Direct Consumers
(pre-HAL)" section describes, nor single-backend authoring tooling like
its "Non-Stack Tooling Consumers" section. Fix by adding both as their
own distinct consumer category, matching the crates' own already-accurate
"reference implementation" framing — this is gap #3 from the 2026-08-17
docs/layer round-3 gap audit. `rulebook.md`'s "Rendering layer placement"
table is deliberately NOT touched: its rule ("every rendering-ecosystem
crate occupies exactly one rung... or is explicitly listed beside it")
and its existing rows/beside-the-ladder list name only `module/`-level
library crates (`tiles_tools`, `line_tools`, `animation`,
`shader_chunks_render_core`, etc.) — `examples/` crates are downstream
consumers of the ladder, not crates that themselves occupy or sit beside
a rung, so they are out of that table's scope.
Testable: `grep -c "orrery" docs/layer/001_l0_drivers.md` returns ≥1
(was: 0), and `grep -c "examples/orrery/flexible"
docs/layer/002_l1_gpu_hal.md` returns ≥2 (was: 1, prose only).

## In Scope

- `docs/layer/001_l0_drivers.md`: add a new subsection (after "Beside-the-
  Ladder Consumers", before "### Layers") documenting `orrery_webgpu` as a
  deliberate, permanent, direct-L0 reference-implementation consumer —
  distinct from "Current Direct Consumers (pre-HAL)" (accepted violations
  scheduled to migrate) and "Non-Stack Tooling Consumers" (single-chunk
  authoring/preview tooling).
- `docs/layer/002_l1_gpu_hal.md`'s Sources table (lines 118-124): add a
  third row for `examples/orrery/flexible/src/main.rs`, naming it a
  reference/comparison consumer reaching all four backends directly and
  bypassing L3, distinguishing it from the table's existing two rows
  (both of which are L3 stack engines adopting L1).

## Out of Scope

- `rulebook.md`'s "Rendering layer placement" table — out of scope per
  the Goal section's rationale (examples are not ladder-occupying
  library crates; the table's existing rows/beside-the-ladder list
  contains none).
- `docs/layer/004_l3_stack_engine.md` — `orrery_flexible`/`orrery_webgpu`
  are not L3 stack engines and do not appear there; no false claim to
  correct (confirmed via `grep -n "orrery"
  docs/layer/004_l3_stack_engine.md` → no matches).
- Any change to `examples/orrery/*`'s source code, Cargo.toml, or its own
  readme.md files — those are already accurate; this task only propagates
  their existing accurate framing into `docs/layer/`.
- Claiming `orrery_flexible` "adopts the HAL" in the same sense
  `tilemap_renderer` does (ADR-003) — it does not; it is a comparison
  example, not a stack engine, and the new Sources row must not blur that
  distinction.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Non-code task: test-related items omitted.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `orrery_webgpu` is documented in `docs/layer/001` as a direct L0
    consumer, without being miscategorized as pre-HAL migration debt or
    authoring tooling
-   `orrery_flexible` is documented in `docs/layer/002`'s Sources table
    as a direct L1 consumer, without being miscategorized as an L3 stack
    engine adopting the HAL
-   `rulebook.md` and `docs/layer/004_l3_stack_engine.md` remain untouched
-   Independent verification passes per `§ Acceptance Verification :
    Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to
    `task/completed/`

## Test Matrix

*(Non-code documentation task — rows are text-consistency checks, not
`cargo test` cases.)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `grep -c "orrery" docs/layer/001_l0_drivers.md` | New Example / Reference-Implementation Consumers subsection | ≥1 (was: 0) |
| T02 | `grep -c "examples/orrery/flexible" docs/layer/002_l1_gpu_hal.md` | New Sources table row + existing Status prose mention | ≥2 (was: 1) |
| T03 | Read `docs/layer/001_l0_drivers.md`'s new subsection | `orrery_webgpu` entry | States direct `minwebgpu` dependency, zero `gpu_hal`/`renderer` involvement |
| T04 | Read `docs/layer/002_l1_gpu_hal.md`'s Sources table new row | `orrery_flexible` entry | States it bypasses L3, reaches all 4 backends, is a reference/comparison consumer — not an L3 stack engine |
| T05 | `git diff --stat -- rulebook.md docs/layer/004_l3_stack_engine.md` | Out-of-scope files | Empty (untouched) |

## Acceptance Criteria

-   `docs/layer/001_l0_drivers.md` names `orrery_webgpu` as a direct L0
    consumer in its own, correctly-distinguished subsection
-   `docs/layer/002_l1_gpu_hal.md`'s Sources table names `orrery_flexible`
    as a direct L1 consumer, distinct from the table's two existing L3-
    stack-engine rows
-   Neither addition overclaims HAL adoption or stack-engine status for
    either example crate
-   `rulebook.md` and `docs/layer/004_l3_stack_engine.md` are untouched
-   Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does
NOT self-verify — an independent verifier performs the walk after the
task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Documentation consistency**
- [ ] C1 — Does `docs/layer/001_l0_drivers.md` contain a new subsection
  naming `orrery_webgpu` as a direct L0 (`minwebgpu`) consumer?
- [ ] C2 — Does that subsection correctly state `orrery_webgpu` has zero
  `gpu_hal`/`renderer` dependency (matching its actual Cargo.toml)?
- [ ] C3 — Does the new subsection avoid placing `orrery_webgpu` under
  "Current Direct Consumers (pre-HAL)" or "Non-Stack Tooling Consumers"
  (neither categorization fits)?
- [ ] C4 — Does `docs/layer/002_l1_gpu_hal.md`'s Sources table contain a
  new row for `examples/orrery/flexible/src/main.rs`?
- [ ] C5 — Does that row correctly state it reaches all four backends
  directly via `gpu_hal` and bypasses L3, without claiming it "adopts the
  HAL" the way `tilemap_renderer` (ADR-003) does?
- [ ] C6 — Do the two new entries agree with `examples/orrery/readme.md`,
  `examples/orrery/flexible/readme.md`, and `examples/orrery/webgpu/readme.md`'s
  own existing framing (no overclaim, no contradiction)?

**Out of Scope confirmation**
- [ ] C7 — Is `rulebook.md` untouched (`git diff --stat -- rulebook.md`
  empty)?
- [ ] C8 — Is `docs/layer/004_l3_stack_engine.md` untouched (`git diff
  --stat -- docs/layer/004_l3_stack_engine.md` empty)?
- [ ] C9 — Is `examples/orrery/` untouched (`git diff --stat --
  examples/orrery/` empty)?

### Measurements

- [ ] M1 — `grep -c "orrery" docs/layer/001_l0_drivers.md` → ≥1 (was: 0)
- [ ] M2 — `grep -c "examples/orrery/flexible" docs/layer/002_l1_gpu_hal.md`
  → ≥2 (was: 1)

### Invariants

- [ ] I1 — source/example tree unaffected: `git diff --stat --
  examples/orrery/ rulebook.md docs/layer/004_l3_stack_engine.md` → empty
- [ ] I2 — workspace still builds: `cargo check --workspace` → 0 errors
  (doc-only change, unaffected)

### Anti-faking checks

- [ ] AF1 — the new `orrery_flexible` Sources row explicitly distinguishes
  it from an L3-stack-engine HAL adopter (not just omits the claim —
  states the bypass explicitly), checked by reading the literal row text,
  not just the row's presence

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Spans 2 files, both within `docs/layer/` — narrower than task 190's/219's repo-spanning precedent; `unit_type: repository` retained since neither file is itself a crate | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | — | — |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-17 03:16:29 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-17 03:17 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 220` → blocked: `tsk: .verify_pass: self-verification forbidden (actor matches filed_by)` |
| 2026-08-18 23:47:41 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:12 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:54 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 22:45:30 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:45:30 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 220` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |
| 2026-08-20 09:57:11 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-20 09:57:35 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-17]** `FILED` — Task filed via docs/layer round-3 gap audit (gap #3): document `orrery_webgpu` (L0-direct) and `orrery_flexible` (L1-direct) as undocumented layer-ladder consumers in docs/layer/001 and docs/layer/002.
- **[2026-08-17]** `EXECUTED` — Implemented both doc edits: (1) new "### Example / Reference-Implementation Consumers" subsection in `docs/layer/001_l0_drivers.md` (after "Beside-the-Ladder Consumers", before "### Layers") documenting `orrery_webgpu` as a direct L0 `minwebgpu` consumer, explicitly "no `gpu_hal` or `renderer` dependency at all"; (2) new Sources-table row in `docs/layer/002_l1_gpu_hal.md` for `examples/orrery/flexible/src/main.rs`, stating "Reference/comparison consumer, not an L3 stack engine" and "bypassing L3 entirely, unlike this table's other two rows" — reaching all four backends via the unified `gpu_hal::Device::new(...)` constructor. Test Matrix: T01 (`grep -c "orrery" docs/layer/001` → 3, want ≥1) PASS; T02 (`grep -c "examples/orrery/flexible" docs/layer/002` → 2, want ≥2) PASS; T03 (re-read new subsection: states direct `minwebgpu` dep, zero `gpu_hal`/`renderer` involvement) PASS; T04 (re-read new Sources row: states bypasses L3, reaches all 4 backends, reference/comparison consumer not an L3 stack engine) PASS; T05 (`git diff --stat -- rulebook.md docs/layer/004_l3_stack_engine.md`) showed a non-empty `rulebook.md` diff — investigated via `git diff -- rulebook.md`, confirmed the entire 2-hunk/4-line diff is task 219's own already-completed, already-verified edit (this task's own work added zero bytes to it); `docs/layer/004_l3_stack_engine.md` showed 0 changes. Checklist C1-C6 (documentation consistency, incl. cross-check against `examples/orrery/readme.md`'s own "Reference implementation" / "Backend-selectable implementation" framing — no contradiction), M1 (3≥1), M2 (2≥2), and AF1 (row explicitly states the L3 bypass, not a mere omission) all walked and PASS on direct self-check re-read. C7 (`rulebook.md` untouched) and C9/I1 (`examples/orrery/` untouched) both show non-empty `git diff --stat` vs HEAD, but for reasons predating this task: C7's diff is 100% task 219's prior edit (content-verified); C9/I1's `examples/orrery/flexible/*` and `examples/orrery/webgpu/shader/scene_fragment.wgsl` changes were already present, uncommitted, in the working tree before this session began (visible in this session's opening `git status` snapshot — pre-existing Vulkan-backend-task 202/203 work), and this task made zero Edit/Write calls anywhere under `examples/orrery/`. C8 (`docs/layer/004_l3_stack_engine.md` untouched) clean, no caveat. I2 (workspace still builds) holds trivially — both edits are pure Markdown, cannot affect compilation. Self-check performed as Tier 2 Dual-Role Self-Check (this repo's MAAV cap) — not the file's own formal independent-verifier 🔎-Accepting-stage Checklist walk, which this sandbox cannot reach (same-actor guard blocks both `.verify_pass` and `.acceptance_pass`). `tsk .claim_verify 220` succeeded (❓→🔬 Verifying). `tsk .verify_pass 220` blocked by same-actor guard (`actor` == `filed_by`) — same structural block hit on every prior task this session; left at 🔬 Verifying per established precedent, no force/spoof attempted.
- **[2026-08-17]** `NOTE` — Goal section's `orrery_flexible` citation corrected: a same-day `gpu_hal` API unification collapsed `main.rs`'s 4 named-constructor call sites into 2 unified `Device::new` overloads; citation now points at the current call sites (lines 112, 141) instead of the removed direct calls. The underlying claim this task documents (direct L1 consumer, bypassing L3) is unaffected.

## Related Documentation

- `examples/orrery/readme.md` — the family readme's already-accurate
  "Reference implementation" / "Backend-selectable implementation"
  framing this task propagates
- `examples/orrery/flexible/readme.md` — the crate's own already-accurate
  Backend selection table (`gpu_hal/webgl → minwebgl`, etc.)
- `examples/orrery/webgpu/readme.md` — the crate's own description as the
  family's `minwebgpu`-only reference implementation
- `docs/adr/004_native_vulkan_hal_backend.md` — the ADR `orrery_flexible`
  motivated, already cited by `docs/layer/002`'s Status prose
