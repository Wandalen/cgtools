# Fix behaviour_tree hang/livelock risk

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** 2026-08-10
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/behaviour_tree
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Fix a condition in `behaviour_tree` identified during the workspace audit where a specific node
configuration or evaluation cycle can hang or livelock rather than terminate (P2 — remaining logic bugs,
Fix-in-place). **Carried forward from the audit triage plan — exact file/line and the precise triggering
condition are not re-verified in this filing pass; re-confirm against current
`module/helper/behaviour_tree/src/` before touching.** A regression test needs a bounded-time assertion
(e.g. a timeout-wrapped evaluation) to actually catch a hang rather than blocking the test suite itself.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P2 (remaining logic
  bugs) tier, Fix-in-place bucket.

- **[2026-08-10]** `IMPLEMENTED` — Re-verified against current source and fixed.

  **Root cause / triggering condition:** `module/helper/behaviour_tree/src/lib.rs`, `RepeatNode`'s
  `BehaviorNode::execute` impl (pre-fix: `impl BehaviorNode for RepeatNode` at line 604, unconditional
  `loop` at line 609). A `RepeatNode` built via `RepeatNode::infinite` / `repeat_forever`
  (`max_repeats == None`) wrapping a child that never returns `BehaviorStatus::Running` (e.g. an instant
  action/condition such as `SetBlackboardAction`/`BlackboardCondition`, or a `Sequence`/`Selector`
  composed entirely of such instant nodes) makes both loop-exit branches permanently unreachable — the
  child never yields `Running`, and `if let Some( max ) = self.max_repeats` is always `None` — so the
  `loop` spins forever inside a single `execute()` call, hanging the calling thread (livelock: full CPU
  use, zero progress toward returning). Confirmed to be the only unbounded-loop site in the crate:
  `SequenceNode`/`SelectorNode` bound their `while` loops by `children.len()`, `ParallelNode` uses a
  single bounded `for` over `children`, and `CooldownNode`/`InvertNode` contain no loops.

  **Fix:** `RepeatNode::execute` (now `module/helper/behaviour_tree/src/lib.rs:612-654`) bounds
  synchronous child re-invocations per call to a new associated const
  `RepeatNode::MAX_SYNC_ITERATIONS` (`u32 = 10_000`, defined at line 581); once the cap is hit without
  the child returning `Running` or `current_repeats` reaching `max_repeats`, the node yields
  `BehaviorStatus::Running` back to the caller instead of continuing to loop — consistent with the
  existing calling contract (callers already must treat `Running` as "call `execute()` again next
  tick"). 3-field source comment (`Fix(TASK-017)` / `Root cause` / `Pitfall`) placed directly above
  `execute` at lines 614-626.

  **Test (TDD):** `tests::test_repeat_node_infinite_livelock_guard`
  (`module/helper/behaviour_tree/src/lib.rs:1399`, 5-section doc comment above it), written *before*
  applying the fix. It runs the risky `RepeatNode::infinite( SetBlackboardAction::new( "tick", true ) )
  .execute( &mut context )` call on a spawned background thread — building the tree entirely inside that
  thread, since `Box< dyn BehaviorNode >` is not `Send` and cannot itself cross the thread boundary — and
  receives the resulting `BehaviorStatus` through an `mpsc` channel via
  `recv_timeout( Duration::from_secs( 2 ) )`, so a real hang fails the test after a bounded wait instead
  of blocking the suite.

  **Verification** (package-scoped to `module/helper/behaviour_tree`; every command launched via
  `longrun .launch dir::... --`, never raw foreground):
  - Pre-fix (RED): `cargo test test_repeat_node_infinite_livelock_guard` reproducibly FAILED after
    exactly 2.00s wall time with `RepeatNode::infinite over a non-Running child hung past the
    bounded-time guard: Timeout` — direct empirical proof of the hang, safely bounded.
  - Post-fix (GREEN): same command → `test result: ok. 1 passed; ... finished in 0.01s` — the
    bounded-time test itself completes in ~10ms, confirming it observed real termination
    (`BehaviorStatus::Running` from the new cap), not a silently-swallowed hang.
  - `cargo nextest run --all-features` → `Summary [0.176s] 14 tests run: 14 passed, 0 skipped` (13
    pre-existing tests + the 1 new regression test; `test_repeat_node_infinite_livelock_guard` itself
    passed in 0.029s).
  - `cargo clippy --all-targets --all-features -- -D warnings` → exit 0, no warnings.
  - `cargo test --doc --all-features` → `1 passed; 0 failed`.
  - Mandated final check `will .test l::3` → `Summary: 4/4 commands passed, 0 failed`, exit 0.

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
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| B1 | Rulebook Compliance | — | 🟢 | — | — |
| B2 | Test-First Requirement | — | 🟢 | Confirmed via diff: RED timing (2.00s timeout) documented before fix | — |
| B3 | Evidence of Failure | — | 🟢 | Concrete timeout message quoted in History | — |
| B4 | Proper Fix Only | — | 🟢 | Bounded loop + yield `Running`, not a suppressed/papered-over symptom | — |
| B5 | Fix Verification | — | 🟢 | Independently re-ran `will .test l::3` (not the dispatched implementer) → 4/4 passed, exit 0 | — |
| B6 | Knowledge Preservation | — | 🟢 | Confirmed by direct diff read: 3-field `Fix(TASK-017)` comment + 5-section test doc comment both present | — |
| B7 | Code Cleanliness | — | 🟢 | `git status --porcelain` confirms only `src/lib.rs` modified; no stray files | — |
| **Total** | | 🔴 | 🟢 | 0 | 0/0 |

**Aggregate verdict:** PASS — all 15 dimensions clean on both the confirming and adversarial pass, zero Blocking Findings. Verification independently re-executed (`will .test l::3` via `longrun`, package-scoped) rather than solely trusted from the implementing subagent's own prose, per this session's Stale Evidence Trust discipline.
