# decisions — cgtools

Decision log for the cgtools task system.

Distribution: project task backlog — visible to all task contributors.

State legend: 🔍 Unverified · ✔️ Verified · 🚧 Blocked · ✅ Decided · ➖ Cancelled

Format rule: ✅ Decided entries show only the selected option as a single collapsed statement with rationale — rejected alternatives are dropped. ✔️ Verified entries retain the full Options section byte-for-byte plus a Confirmed field citing verification evidence. 🔍 Unverified entries show either the full Options section with a recommendation, or (once an assumption is formulated) a single Assumed statement with a verification mechanism and contingency. 🚧 Blocked entries retain the full Options section plus mandatory Blocked-on/Blocks fields. ➖ Cancelled entries preserve the original analysis plus a mandatory Reason field.

1 entry · 1 cancelled

---

## Index

| ID | Question | State | Owner | Date | Gated by |
|----|----------|-------|-------|------|----------|
| Q-01 | Split, narrow, or plan-extract task 001? | ➖ Cancelled | i4@wbox.pro | 2026_08_08 | — |

---

## Q-01 — Task 001 split strategy

**➖ Cancelled · i4@wbox.pro · 2026_08_09**
Should task 001 (SPRAWL) be split into 5 sibling tasks, left as a single broader-scoped task, or extracted into a slim Task plus a Governing Plan with 5 Phases?

Task 001 bundles all 5 Development Milestones (Wasm bridge/canvas; terrain/hydrology/shoreline; hub placement/traffic/roads/bridges; parcel subdivision/labels; AI integration/segmentation/polish) as a single deliverable, which fails the Readiness Verification Gate's D2 (MOST Goal Quality — Scoped) dimension (`tsk.rulebook.md § Task File : Readiness Verification Gate`).

**A: Split into 5 sibling tasks**
One per Development Milestone, linked via `related_tasks`/`blocked_by`. Each milestone already has concrete deliverable bullets in the source spec; achievable without fabricating detail.

**B: Leave as one task, narrow scope**
Accept a single task but shrink Out of Scope until only one milestone's worth of deliverable remains; defer the rest to follow-up tasks filed later.

**C: Extract to a Governing Plan**
Slim Task plus a `pln.rulebook.md`-compliant Plan with 5 Phases (the existing `### Phase 1`–`### Phase 5` headers in the Technical Specification already match Plan's own Phase vocabulary). Requires authoring per-phase Estimated-Time/Outputs/Steps detail (`pln.rulebook.md` TP037) not present in the source material today — needs real estimation input from the task owner, not invention.

→ **Recommended: A** — leverages `tsk.rulebook.md § Core Procedures : Procedure - Decompose by Crate` (adapted from crate-boundary to milestone-boundary partitioning, since no D2-specific split procedure exists); B just relabels 4/5 of the scope as undefined future work rather than resolving it; C is blocked on per-phase estimation data that doesn't exist yet without inventing it.

**Assumed A** based on the milestone boundaries already present in task 001's own Development Milestones section. Verification mechanism: each resulting task's own Readiness Verification Gate — if the milestone boundaries were wrong, at least one resulting task would fail D1/D2/D6/D7/D8. If wrong: fold surviving milestones back toward Option B, or revisit Option C once real per-phase estimation data exists.

**Confirmed** by tasks 002-006's own `## Verification Record` sections (durable, `task/verified/00{2..6}_*.md`) — all 8 Readiness Verification Gate dimensions 🟢 on both passes for all 5 tasks (40/40), including one genuine adversarial catch-and-fix (002's D6, an overbroad dependency-registration bullet). Reconfirmed at the option-selection level (A vs B vs C) by the Gate Check below (2026-08-08, this session).

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 1/1

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| G1 | Option A selection validity (vs B, C) | — | 🟢 | Adversarial checks against A: (1) B merely relabels 4/5 of scope as undefined deferred work rather than resolving it — weaker than A; (2) C is genuinely blocked on real per-phase estimates that don't exist without inventing them, not just deprioritized; (3) milestone boundaries hold empirically — 40/40 dimension passes across 002-006, including D8 Crate Single Responsibility on each; (4) the linear `blocked_by` chain matches the original spec's own data dependencies (M3 needs M2's terrain grid; M4 needs M3's road network), not an arbitrary serialization; (5) sunk-cost bias risk (A was already executed before this check) is partially mitigated by mechanical evidence (40/40 passes, one real D6 catch) rather than narrative self-assurance — full mitigation is deferred to the human ENDORSE step, which this gate does not substitute for | — |
| **Total** | | — | 🟢 | 0 open | — |

**Aggregate verdict:** PASS — Option A holds under adversarial challenge; no rejected finding. State → ✔️ Verified (superseded below).

**Reason (➖ Cancelled, 2026-08-09):** Before the ENDORSE step above ever fired, the filer (i4@wbox.pro) cancelled the entire SPRAWL initiative outright — task 001 and its milestone split 002-006 alike — as exploratory/idea-stage work, not committed for implementation. This moots the question this decision resolved: Option A (split into 5) was the correct decomposition *if* SPRAWL were to proceed, and remains validly confirmed as that (40/40 gate passes stand unchanged) — but with no implementation proceeding at all, no split/narrow/plan-extract strategy is needed. All 6 tasks (001-006) moved to `task/cancelled/`; Tasks Index reindexed accordingly.

---
