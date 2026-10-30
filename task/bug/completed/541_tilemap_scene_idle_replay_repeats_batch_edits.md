# BUG-541: An idle replay repeated the previous frame's batch edits

- **Severity:** High (a submitted replay drew ghost sprites, leaked GPU batches, or failed the
  frame on a bounds check; it happened on the first idle frame after any batch change)
- **state:** Completed
- **Affects:** every `tilemap_scene` consumer that submits each `Renderer::render` result to its
  backend, as the method's documentation asks.
- **Component:** `module/helper/tilemap_scene` (`src/renderer.rs`)
- **repo_identity:** self
- **Filed:** 2026-10-22
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-10-22
- **Related Bugs:** None.

## Symptom

On a cache hit, `Renderer::render` returned the command buffer of the last miss unchanged. That
buffer holds the frame's draws and its batch edits: `CreateSpriteBatch`, `BindBatch`, the
`SetSpriteInstance` / `RemoveInstance` / `AddSpriteInstance` diff, `UnbindBatch`, `DeleteBatch`.
The caller submits it again, so the backend applies the edits a second time:

- `AddSpriteInstance` adds another instance, and the added sprite is drawn twice;
- `RemoveInstance` removes one more instance, or fails the WebGL backend's bounds check
  (`"RemoveInstance: index … out of bounds"`), which aborts the frame;
- `CreateSpriteBatch` allocates a new GPU buffer and VAO over the live batch and leaks the old
  ones.

## Impact

**Who is affected:** Any game that redraws continuously (the idle replay exists for exactly
that loop) and changes the scene now and then.

**What breaks:** The first idle frame after a spawn, despawn or batch creation draws wrongly or
errors. Later idle frames keep drawing the corrupted batch.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Round 5 of the PR #207 review (2026-10-22). The replay path predates the PR (it is on `master`).

## Minimum Reproducible Example

```rust
let mut backend = /* any backend that keeps batches */;
backend.submit( renderer.render( &scene, &camera )? )?;   // creates the batch
let h = scene.spawn( knight, Placement::Hex { q : 1, r : 0 } );
backend.submit( renderer.render( &scene, &camera )? )?;   // miss: AddSpriteInstance
backend.submit( renderer.render( &scene, &camera )? )?;   // hit: pre-fix adds the knight again
```

**Verify Command:**
```bash
cargo nextest run -p tilemap_scene --test renderer_cache_test
```

## Root Cause

The idle replay was written as "return the same commands", which holds for draws but not for
the batch edits that `render` started emitting into the same buffer when batching arrived. The
tests compared the replay to the miss by length and by stateless flattening, and so required
the bug.

## Why Not Caught

No test submitted a miss and the hit after it to one backend that keeps batches. Each replay test
looked at the returned commands on their own.

## Fix Location

- `module/helper/tilemap_scene/src/renderer.rs`: the first hit after a miss drops every batch
  edit from `cmd_buf` (`command_draws`), so a replay only draws. Later hits return the filtered
  buffer as it is. `Fix(BUG-541)` comment in `render`.

## Prevention

`replay_after_a_spawn_and_a_despawn_draws_each_sprite_once` in
`module/helper/tilemap_scene/tests/renderer_cache_test.rs` submits every frame to one
`BatchFlattener`, which keeps the batch table as a backend does and panics on an edit that does
not apply. The replay tests there and in `sorted_batching_test.rs` now compare what one backend
draws for the miss and for the replay.

## Pitfall

Everything `render` returns is submitted once per call, so a cached stream must be safe to apply
again: only draws are.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-10-22 | filed | Raised by round 5 of the PR #207 review. |
| 2026-10-22 | fixed | The replay drops the miss's batch edits. |
| 2026-10-22 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `tilemap_scene` passes 235/235. Adversarial pass: with the filter removed, the reproducer fails (the idle frame draws three sprites) and so does `idle_render_returns_cached_slice`. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-541)` / `Root cause` / `Pitfall` comment in `render`; 5-section doc comment and `bug_reproducer(BUG-541)` marker on the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to the idle-replay branch of `render` and its new `command_draws` filter. | — |

**Reproduced:** YES -- the reverted run above. 2026-10-22.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_scene/src/renderer.rs` | The first hit after a miss keeps only draw commands; `Fix(BUG-541)` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_scene/tests/renderer_cache_test.rs` | `replay_after_a_spawn_and_a_despawn_draws_each_sprite_once`, marked `bug_reproducer(BUG-541)`; the replay tests compare what one backend draws. |
| `module/helper/tilemap_scene/tests/sorted_batching_test.rs` | `idle_replay_draws_the_priming_frame_for_sorted_batch` replaces the length comparison. |
