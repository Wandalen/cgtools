# BUG-137: `EventResult::Unsubscribe` removes the listener but doesn't stop the event

- **Severity:** Medium (silently over-delivers events to lower-priority listeners — no panic, no
  compile error, just a listener contract violation that only manifests with 2+ listeners)
- **state:** Completed
- **Affects:** Any `EventChannel`/`EventBus` consumer relying on `EventResult::Unsubscribe`'s doc
  comment ("Stop processing and remove this listener") to also halt propagation to
  lower-priority listeners for the current event, not just future ones
- **Component:** `module/helper/tiles_tools` (`src/events.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — seventh bug filed for this crate this session; independent of
  BUG-131/132/133/134/135/136 (different module, different mechanism)

## Symptom

```rust
bus.subscribe_with_priority(|_: &TestEvent| EventResult::Unsubscribe, EventPriority::High);
bus.subscribe_with_priority(|_: &TestEvent| { /* records that it ran */ EventResult::Continue }, EventPriority::Low);

bus.publish(TestEvent { id: 1, message: "test".to_string() });
bus.events_process();

// Wrong (pre-fix): the Low-priority listener still runs for THIS event, even though the
// High-priority listener ahead of it returned Unsubscribe.
// Correct (post-fix): the Low-priority listener is never invoked for this event.
```

## Impact

**Who is affected:** Any caller with 2+ listeners on the same event type at different priorities,
where a higher-priority listener returns `EventResult::Unsubscribe`.

**What breaks:** `EventChannel::events_process`'s `for listener in &self.listeners` loop treats
`Unsubscribe` as "queue for removal, then keep iterating" instead of "stop processing this event."
The removal itself is correct (the listener is gone by the next event), but the *current* event
keeps propagating to every lower-priority listener after it — contradicting the variant's own doc
comment ("Stop processing and remove this listener") and behaving inconsistently with the sibling
`Consume` variant, which does `break` immediately.

**Magnitude:** Not a crash — a silent extra invocation of every listener below the unsubscribing
one, for exactly the one event that triggered the unsubscribe. A gameplay handler relying on
"a listener that unsubscribes also swallows the event" (e.g. a modal UI consuming its last input
and detaching) would incorrectly let that same input reach handlers underneath it.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Continuation of task #66's targeted code review of `tiles_tools` under the standing bug-hunt
mandate, reading `src/events.rs` after closing BUG-135/BUG-136. Direct read of
`EventChannel::events_process`'s match arms showed `Consume => break` immediately adjacent to
`Unsubscribe => { listeners_to_remove.push(listener.id); }` with no `break` — an asymmetry between
two variants whose doc comments both promise "stop processing."

## Minimum Reproducible Example

```bash
cd module/helper/tiles_tools && cargo test --test events_test --features enabled test_unsubscribe_halts_propagation_to_lower_priority_listeners 2>&1 | tail -10
```

**Expected** (post-fix):
```
test test_unsubscribe_halts_propagation_to_lower_priority_listeners ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting just the added `break;`, restoring the
exact pre-fix behavior, then restoring the fix immediately after capturing the failure):
```
thread 'test_unsubscribe_halts_propagation_to_lower_priority_listeners' panicked at module/helper/tiles_tools/tests/events_test.rs:205:3:
lower-priority listener was invoked after a higher-priority listener returned EventResult::Unsubscribe
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo test --test events_test --features enabled test_unsubscribe_halts_propagation_to_lower_priority_listeners
# 1 passed = fixed; 1 failed (panic above) = bug present
```

**Known MRE limitation (check 205):** none — `EventBus` is pure, synchronous, dependency-free
state; runs as an ordinary native `cargo test` against the real crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `EventChannel::events_process`'s `Unsubscribe` arm never `break`s, so the same event keeps reaching lower-priority listeners after one unsubscribes. | ✅ Root Cause | Direct read of `src/events.rs`: `Consume => break,` immediately followed by `Unsubscribe => { listeners_to_remove.push(listener.id); }` — no `break` in the latter. | E1 |
| H2 | The gap is invisible with only one listener subscribed, which is why it went uncaught. | ✅ Confirmed | `test_auto_unsubscribe` uses a single listener and only checks `subscriber_count` after later events — it can never observe cross-listener propagation because there is no second listener to observe it with. | E2 |
| H3 | `unsubscribe()` (the manual API) shares the same gap. | ❌ Falsified — out of scope | `EventBus::unsubscribe` removes a listener by ID outside of `events_process` entirely; it doesn't process an in-flight event at all, so there's no "current event" for it to fail to stop. `test_unsubscribe` already covers this path correctly. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/events.rs`, pre-fix `events_process` | `EventResult::Consume => break,` vs `EventResult::Unsubscribe => { listeners_to_remove.push(listener.id); }` — the only two variants whose doc comments say "stop processing," but only one actually does. | H1 ✅ |
| E2 | `tests/events_test.rs`, pre-fix | `test_unsubscribe` (manual API only) and `test_auto_unsubscribe` (single listener) — neither test configuration can observe a second, lower-priority listener still being invoked for the same event. | H2 ✅ |
| E3 | `src/events.rs`, `EventBus::unsubscribe` | Looks up the channel by `TypeId` and calls `listener_remove` directly — no interaction with `pending_events` or the `events_process` loop at all. | H3 ❌ (out of scope, not a shared defect) |

## Root Cause

```
events_process():
  for listener in self.listeners:            // priority-ordered
    match listener(event):
      Continue    => {}                        // keep going -- correct
      Consume     => break                      // stop this event -- correct
      Unsubscribe => listeners_to_remove.push(id)   // queue removal only -- MISSING break
  for id in listeners_to_remove: self.listener_remove(id)   // deferred, correct
```

`Unsubscribe`'s doc comment ("Stop processing and remove this listener") bundles two distinct
obligations: (1) stop the *current* event's propagation, and (2) remove the listener for *future*
events. The implementation only ever did (2) — removal is correctly deferred until after the loop
(so the loop's own iteration isn't invalidated by `Vec` mutation mid-iterate), but that deferral
was never paired with the `break` needed to satisfy (1) for the event currently being processed.

## Why Not Caught

Every existing test exercising `Unsubscribe` used exactly one listener (`test_auto_unsubscribe`)
or bypassed `events_process` entirely (`test_unsubscribe`, via the manual API) — neither
configuration has a second, lower-priority listener whose invocation could reveal that
propagation wasn't actually halted.

## Fix Location

`module/helper/tiles_tools/src/events.rs`, `EventChannel::events_process`:

```rust
// before
EventResult::Unsubscribe => {
  listeners_to_remove.push(listener.id);
}

// after
EventResult::Unsubscribe => {
  listeners_to_remove.push(listener.id);
  break;
}
```

No signature change — this is a pure internal-logic fix, no downstream callers affected.

## Prevention

Added `test_unsubscribe_halts_propagation_to_lower_priority_listeners` to
`tests/events_test.rs`, using two listeners at different priorities (modeled on the existing
`test_event_priorities` multi-listener pattern) so a lower-priority listener's invocation is
actually observable.

**Pitfall:** invisible with a single listener — `subscriber_count` alone (as `test_auto_unsubscribe`
checks) can confirm removal happened but says nothing about whether the *current* event's
propagation was halted; that requires a second listener positioned to observe it.

## Generalized Version

**Broken assumption:** "an enum variant whose doc comment groups two obligations together (stop
+ remove) will have both implemented together, since they're described in the same sentence."
False here — the two obligations have genuinely different timing requirements (immediate loop
`break` vs. deferred post-loop removal), and it's easy to implement the deferred one correctly
while silently dropping the immediate one, especially when copy-adjacent code (`Consume`) already
solved the immediate half and the new arm only needs to *also* queue a removal.

**Confirmed general rule:** when a match arm's doc comment implies multiple distinct effects,
verify each effect is independently tested — a test that only checks the easier-to-observe effect
(e.g. `subscriber_count` after removal) can pass while the harder-to-observe effect (propagation
timing within the same event) is silently broken.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via continued review of `tiles_tools/src/events.rs` following BUG-135/BUG-136; confirmed by direct read of `events_process`'s `Consume`/`Unsubscribe` match arm asymmetry. |
| 2026-08-16 | fixed | Added `break;` to the `EventResult::Unsubscribe` arm in `EventChannel::events_process`, matching the existing `Consume` arm. |
| 2026-08-16 | verified | Added `test_unsubscribe_halts_propagation_to_lower_priority_listeners`; confirmed it fails against the reverted pre-fix logic with the exact predicted panic text and passes against the fix; full crate suite (242 tests incl. 39 doctests) + `cargo clippy --all-targets --features enabled,integration -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-16 earlier same day, this verifier 2026-08-16). Independently re-read `EventChannel::events_process`'s `Unsubscribe` arm (confirmed the added `break;` genuinely present immediately after `listeners_to_remove.push(listener.id)`, 5-line `Fix(BUG-137)`/`Root cause`/`Pitfall` comment intact, matching the sibling `Consume => break` arm's shape) and `test_unsubscribe_halts_propagation_to_lower_priority_listeners` (non-tautological: two listeners at different priorities, asserts the lower-priority one is never invoked and `subscriber_count` drops to 1). Fresh `cargo nextest run --all-features` via `longrun` (crate-wide, covering BUG-131 through BUG-137 together): 251/251 passed. `cargo clippy --all-features --all-targets -- -D warnings`: clean. `**Related Bugs:** None` confirmed accurate. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-131 through BUG-137 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass reasoned the arm's missing `break` would let propagation continue; adversarial pass required actually observing the FAIL against reverted code, not trusting the reasoning — closed via revert-test-restore, captured panic text matched exactly. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Seventh bug for `tiles_tools` this session; independent of BUG-131 through BUG-136 — no cross-ref needed. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass challenged whether the manual `unsubscribe()` API path shared the same gap (H3, falsified — it never touches `events_process`'s loop at all) and whether the gap was single-listener-only (H2, confirmed via both existing tests' configurations). | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped `EventResult::` usage across the crate and workspace — `events.rs` (definition + match site) and `events_test.rs` (consumers) are the only sites; no other match on `EventResult` variants exists anywhere else to audit for the same asymmetry. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `tiles_tools` `src/events.rs` + `tests/events_test.rs` + this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to one match arm; no other function touched. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface added — existing "stop processing and remove" contract now actually honored in full. | — |

**Reproduced:** YES — temporarily removing the added `break;` (restoring the exact pre-fix
behavior with no signature change needed) and running
`cargo test --test events_test --features enabled test_unsubscribe_halts_propagation_to_lower_priority_listeners`
produced the exact predicted panic (`lower-priority listener was invoked after a higher-priority
listener returned EventResult::Unsubscribe`); restoring the fix returned the full suite to
242/242 passing (plus 39 doctests) and a clean
`cargo clippy --all-targets --features enabled,integration -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/events.rs` | `EventChannel::events_process`: added `break;` to the `EventResult::Unsubscribe` match arm. `Fix(BUG-137)`/`Root cause`/`Pitfall` comment added. No signature change. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/events_test.rs` | New test (`bug_reproducer(BUG-137)`, 5-section doc comment) — `test_unsubscribe_halts_propagation_to_lower_priority_listeners`. |
