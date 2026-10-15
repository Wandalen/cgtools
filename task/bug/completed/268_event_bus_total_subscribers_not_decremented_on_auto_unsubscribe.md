# BUG-268: `EventBus.statistics.total_subscribers` never decrements when a listener self-unsubscribes via `EventResult::Unsubscribe`

- **Severity:** Medium (no panic or wrong-answer in core event delivery, but a public,
  documented statistic silently drifts upward from reality on a common, intended usage pattern)
- **state:** Completed
- **Affects:** `tiles_tools::events::EventBus::{events_process, events_for_type_process}` and
  `EventChannel::events_process` (`src/events.rs`)
- **Component:** `module/helper/tiles_tools` (`src/events.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`EventBus.statistics.total_subscribers` is incremented on every `subscribe()` call and
decremented in exactly one place: the explicit `EventBus::unsubscribe(id)` path
(`self.statistics.total_subscribers.saturating_sub(1)`). A listener can also leave a channel by
returning `EventResult::Unsubscribe` from its own callback during event processing --
`EventChannel::events_process` correctly removes it from the channel's internal listener list (so
`EventChannel::listener_count()` / `EventBus::subscriber_count::<T>()` are accurate), but that
removal never reached `EventBus.statistics.total_subscribers`, because `events_process` returned
`()` and gave its caller no way to learn how many listeners it had just auto-removed.

## Impact

**Who is affected:** any caller relying on `EventBus.statistics.total_subscribers` (or
`EventStatistics` as a whole, e.g. via a debug/monitoring overlay) alongside listeners that use the
self-unsubscribe pattern (`EventResult::Unsubscribe`) -- a first-class, documented way to
unsubscribe, not a misuse.

**What breaks:** `total_subscribers` silently drifts upward relative to the true number of active
listeners with every auto-unsubscribe, and never recovers (there is no periodic reconciliation).
`EventChannel`-level counts (`subscriber_count`) stay correct throughout, so the drift is
invisible unless a caller specifically cross-checks the bus-level statistic against per-channel
counts.

**Entity Scope:** `None` -- source-level statistics-bookkeeping defect, not entity directory
instances.

## How Discovered

During this session's Group J review of `tiles_tools/src/events.rs`, comparing every site that
mutates `self.statistics.total_subscribers` against every site that removes a listener showed the
explicit `unsubscribe()` path decrementing it correctly, while `EventChannel::events_process`'s
own listener-removal loop (triggered by a returned `EventResult::Unsubscribe`) had no path back to
`EventBus`'s statistics at all -- `events_process`'s `-> ()` signature structurally prevented it.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p tiles_tools --all-features --test events_test test_auto_unsubscribe_decrements_total_subscribers_statistic
```
**Expected** (fixed): 1 passed.
**Actual** (pre-fix, confirmed via temporary direct-source-edit revert of the fix, real run
alongside this session's other 2 then-reverted bugs, `--no-fail-fast`): 1 failed --
`assertion left == right failed: total_subscribers should track a listener that unsubscribed
itself via EventResult::Unsubscribe, not only the explicit unsubscribe() path` (`left: 1, right:
0`) (`events_test` target: 12 passed, 1 failed).

## Root Cause

`EventChannel::events_process` (pre-fix), abbreviated:
```rust
fn events_process(&mut self) {
  while let Some(event) = self.pending_events.pop_front() {
    let mut listeners_to_remove = Vec::new();
    // .. dispatch to listeners, collecting EventResult::Unsubscribe ids ..
    for id in listeners_to_remove {
      self.listener_remove(id);
    }
  }
}
```
and `EventBus::events_process` (pre-fix), abbreviated:
```rust
pub fn events_process(&mut self) {
  for channel in self.channels.values_mut() {
    channel.events_process();  // return value discarded -- there was none
  }
  self.statistics.process_cycles += 1;
  // total_subscribers never touched here
}
```
`self.listener_remove(id)` inside `EventChannel::events_process` correctly shrinks the channel's
own listener list, but that method's `-> ()` return type gave `EventBus` (the only place holding
`total_subscribers`) no signal that any removal had happened. The private `AnyEventChannel` trait
(the type-erased interface `EventBus` actually calls through) mirrored the same `-> ()` signature,
so the gap existed at every layer between the removal site and the statistic.

## Why Not Caught

Every existing statistics-related test (`test_statistics`, the BUG-137 regression test) exercises
`total_subscribers` only through the explicit `unsubscribe()` path or checks unrelated counters
(`events_processed`, `process_cycles`); none paired a self-unsubscribing listener
(`EventResult::Unsubscribe`) with an assertion on `total_subscribers` specifically, so the two
code paths' divergent bookkeeping was never cross-checked end to end.

## Fix Applied (2026-08-17)

**`src/events.rs`:** changed `EventChannel::events_process(&mut self)` from `-> ()` to `->
usize`, accumulating `unsubscribed += listeners_to_remove.len()` per pending-event iteration and
returning the total. Propagated the new return type through the private `AnyEventChannel` trait's
`events_process` method and its `impl<T: Event> AnyEventChannel for EventChannel<T>` forwarding
call. `EventBus::events_process` and `EventBus::events_for_type_process` now capture that
`usize` per channel and apply `self.statistics.total_subscribers =
self.statistics.total_subscribers.saturating_sub(unsubscribed as u64)` after processing.

**`tests/events_test.rs`** (new test):
`test_auto_unsubscribe_decrements_total_subscribers_statistic` subscribes a listener whose
callback returns `EventResult::Unsubscribe`, asserts `total_subscribers == 1` before processing,
publishes an event and processes it, then asserts both `subscriber_count::<TestEvent>() == 0` and
`total_subscribers == 0`.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p tiles_tools --all-features --test events_test
  test_auto_unsubscribe_decrements_total_subscribers_statistic` -- pre-fix (temporary
  direct-source-edit revert, real run): fails, `left: 1, right: 0`. Post-fix (restored): 1 passed.
- `cargo test -p tiles_tools --all-features --no-fail-fast` (full scoped suite, this session's
  other 3 bugs simultaneously reverted): `events_test` target 12 passed, 1 failed -- exactly and
  only the new test, with all 12 other pre-existing cases (including the BUG-137 regression test)
  still passing. Post-fix (all 4 restored): full suite green across all 10 test binaries
  (`events_test`: 13/13) plus 40 doctests, 0 failed.
- `cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings`: clean, exit 0.

## Generalized Version

**Broken assumption:** a "current count" statistic maintained alongside a collection is safe to
update from only the collection's single most-visible public removal API, on the assumption that
it is the only way items leave the collection. Any internal removal path -- an auto-cleanup loop,
a self-unsubscribe callback result, a capacity-triggered eviction -- must also report what it
removed back to every statistic that claims to track the collection's size, or that path silently
becomes an unaccounted-for leak (here, of the *count* only, not memory) the moment it's the one
actually exercised.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during Group J review of `tiles_tools/src/events.rs`. Root cause: `EventChannel::events_process`'s `-> ()` signature gave `EventBus` no way to learn how many listeners a self-unsubscribe (`EventResult::Unsubscribe`) removal had auto-removed, so `total_subscribers` was only ever decremented by the explicit `unsubscribe()` path, silently drifting upward on the self-unsubscribe path. Fixed by changing the signature to `-> usize` (propagated through the private `AnyEventChannel` trait) and applying the returned count via `saturating_sub` in both `EventBus::events_process` and `EventBus::events_for_type_process`. Verified via 1 new native unit test (confirmed fail pre-fix via a combined `--no-fail-fast` run with this session's other 2 then-reverted bugs -- real failure, exact expected assertion message -- and pass post-fix), the full scoped suite (13/13 in `events_test`, all 10 binaries + 40 doctests green), and clean clippy. Filed as BUG-268 after a fresh on-disk scan immediately before filing found 267 (this session's own field_of_view.rs bug) as the highest existing ID. |
