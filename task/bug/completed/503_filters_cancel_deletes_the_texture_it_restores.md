# BUG-503: `filters`' Cancel button deletes the very GL texture it is restoring `image_texture` to

- **Severity:** High (reachable through entirely ordinary interactive use -- upload an image, click
  Apply once on any filter, select a second filter, click Cancel -- and produces an immediately
  visible corrupted/blank canvas with no recovery short of re-uploading the image)
- **state:** Completed
- **Affects:** `examples/minwebgl/filters`'s `Renderer::image_texture_set` (and, by the same call
  path, every UI action that routes through `previous_texture_restore` -- the Cancel button, and
  every filter-card click that restores an in-progress preview before starting a new one).
- **Component:** `examples/minwebgl/filters` (`src/renderer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** [BUG-463](../verified/463_filters_gpu_texture_framebuffer_leak.md) -- BUG-463
  added the `gl.delete_texture` call this bug's dangling handle comes from. Before BUG-463's fix,
  `image_texture_set` never deleted anything at all, so this exact scenario could not have existed
  yet; this is a completeness gap in BUG-463's own aliasing guard (it only ever compared the
  outgoing texture against `original_texture`, never against the incoming replacement value
  itself), not a re-opening of BUG-463 or a duplicate of it. BUG-463's own verification hand-trace
  explicitly claims to have traced `cancel_button_setup`'s call path and found no over-deletion --
  that trace only covered a fresh-upload session (where `image_texture` still aliases
  `original_texture`, so the existing guard already happens to prevent deletion), not the
  "Apply once, then filter again, then Cancel" sequence where the two fields have since decoupled
  and this bug's self-assignment case becomes live.

## Symptom

```rust
// pre-fix -- src/renderer.rs
pub fn image_texture_set( &mut self, image_texture : Option< WebGlTexture > )
{
  if let Some( old ) = self.image_texture.take()
  {
    if self.original_texture.as_ref() != Some( &old )
    {
      self.gl.delete_texture( Some( &old ) );
    }
  }
  self.image_texture = image_texture;
}
```

The task that led to this bug's discovery originally suspected the Cancel button was a no-op
(`previous_texture_save` computed/stored but never read back) -- that premise is false:
`cancel_button_setup` (`main.rs`) does call `previous_texture_restore()`. The real defect is more
subtle: `previous_texture_restore` always calls `image_texture_set` with a clone of whatever
`previous_texture_save` cloned out of `image_texture` earlier, and nothing in between ever mutates
`image_texture` -- every `Filter::draw` takes `&impl FilterRenderer`, an immutable borrow with no
field-mutating access, so a filter preview only draws to the canvas/framebuffer, it never replaces
this field. So on every real Cancel, `old` (the outgoing value `image_texture_set` is about to
delete) and its own `image_texture` parameter (the incoming replacement) are the *same* handle --
and the guard above only ever checks that handle against `original_texture`, never against the
value it is simultaneously being reassigned to. The moment `original_texture` no longer happens to
alias that same handle too (true as soon as at least one "Apply" click has already baked a new base
texture earlier in the session), the guard's one check stops protecting it, `gl.delete_texture` runs
on the handle, and `self.image_texture` is then reassigned to that just-deleted handle.

## Impact

**Who is affected:** Any user of the `filters` demo who clicks Apply on one filter, then selects a
second filter and clicks Cancel (or simply selects a third filter card without an intervening
Apply/Revert -- every filter-card click routes through the same `previous_texture_restore` call
before starting its own preview).

**What breaks:** `image_texture` ends up pointing at a GL texture object `gl.delete_texture` has
already freed. Per the WebGL spec, a deleted texture object's storage is released and any pending
binding reverts to the default texture -- so every subsequent draw sampling `image_texture`
(including the very `filter_apply( &filters::original::Original )` call `cancel_button_setup`
itself makes immediately after the restore) renders from an empty/default texture instead of the
image the user was expecting Cancel to bring back. The canvas goes blank/corrupted, with no way to
recover the original image short of re-uploading it from disk.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Investigating a task-supplied suspicion that the Cancel button was a no-op. Grep confirmed
`previous_texture_restore()` genuinely is called on Cancel, contradicting the suspected premise --
but tracing the full call graph (every filter-card setup file under `src/ui_setup/`, `renderer.rs`,
and `main.rs`'s `apply_button_setup`/`cancel_button_setup`) surfaced that `previous_texture_save`'s
clone is never mutated before the matching restore, making every real Cancel a self-assignment
through a setter whose entire job is "delete the thing being replaced" -- and that BUG-463's
aliasing guard, added earlier in the same file, only defends against one specific other aliasing
partner (`original_texture`), not this one.

## Root Cause

See Symptom above -- `image_texture_set`'s BUG-463 aliasing guard checked
`self.original_texture.as_ref() != Some( &old )` alone. It never checked whether the *incoming*
`image_texture` argument was itself `old` (i.e., whether this call is actually a no-op
reassignment rather than a genuine replacement) -- the simplest aliasing case of all, and the one
`previous_texture_restore` hits on every single real Cancel once `image_texture` has decoupled from
`original_texture`.

## Why Not Caught

BUG-463's own verification was a hand-trace, not an automated test (this crate has no native GL
test harness -- see Manual Reproduction / Verification below) -- and that hand-trace's Cancel
scenario started from a fresh upload, where `image_texture` still aliases `original_texture` and
the pre-existing guard already happens to prevent deletion by coincidence. It never traced the
"Apply once first, then filter-and-Cancel" sequence, the one where the two fields have since
decoupled and the missing self-assignment check becomes reachable. Visually, the symptom (a
blank/corrupted canvas right after Cancel) reads as "Cancel doesn't restore the image", which is
consistent with -- and easy to misattribute to -- the task's originally-suspected (and, on direct
code inspection, false) "never read back" premise, rather than the actual "reads it back after
having just deleted it" defect underneath.

## Manual Reproduction / Verification

Like BUG-463 (the fix this bug's regression sits on top of), GL texture identity/deletion cannot be
observed without a real browser `WebGlTexture`, which this crate has no native test scaffolding
for (`minwebgl`'s own default target is `wasm32-unknown-unknown`; `web_sys::WebGlTexture` is a
JS-interop handle not constructible on a native host). Verified instead, matching this crate's own
established regression-test idiom (`tests/hsl_wraparound_test.rs`, `tests/blur_kernel_test.rs` --
`include_str!` the real source plus a hand-ported pure-Rust mirror of the fixed logic):

1. A hand-ported pure-Rust mirror of `Renderer`'s texture-handle bookkeeping
   (`tests/cancel_dangling_texture_test.rs`), using comparable `u32` IDs standing in for opaque
   `WebGlTexture` handles, exercising the exact reachable sequence (upload → filter → Apply →
   filter → Cancel) and asserting the restored handle is never recorded as "deleted". Against the
   pre-fix single-condition guard, this sequence deletes handle `2` at the Cancel step (traced by
   hand: `old = 2`, `original_texture = Some(1)`, `Some(1) != Some(2)` is true, so the pre-fix guard
   alone deletes it) -- exactly the defect being reproduced.
2. A source-text assertion (same file) that `renderer.rs` carries the fixed self-assignment check,
   catching a regression back to the single-condition guard.
3. `cargo nextest run -p filters` (native) -- 3/3 tests pass, including the new reproducer.
4. `cargo clippy -p filters --all-features --all-targets -- -D warnings` (native) and
   `cargo clippy -p filters --target wasm32-unknown-unknown --all-features -- -D warnings` (the
   real production target) -- both clean.

**Verify Command:**
```bash
cd module && cargo nextest run -p filters --all-features
cargo clippy -p filters --all-features --all-targets -- -D warnings
cargo clippy -p filters --target wasm32-unknown-unknown --all-features -- -D warnings
```

## Fix Location

`examples/minwebgl/filters/src/renderer.rs`: `image_texture_set` now also skips the delete when the
incoming `image_texture` argument is the same handle as `old` (self-assignment), alongside the
pre-existing `original_texture` aliasing check. `original_texture_set` mirrors the same guard
defensively (no currently-reachable call site drives it into the self-assignment case, but it is
documented as mirroring `image_texture_set` field-for-field, so its guard completeness is kept in
lockstep). `previous_texture_restore`'s doc comment now explains why it depends on this guard.

## Prevention

`tests/cancel_dangling_texture_test.rs` (new) asserts both the fixed source text and the fixed
algorithm's behavior across the exact reachable sequence that made this bug live, so a regression
back to the single-condition guard fails immediately rather than silently reintroducing a
use-after-delete.

## Pitfall

An aliasing guard that special-cases one *other* field can still miss the simplest alias of all --
the incoming value aliasing the very value it's replacing. When adding a guard against deleting a
resource some other field still needs, always check self-assignment first, independent of whatever
sibling-field reasoning the guard also needs -- and re-verify any *existing* hand-traced guard
against every call site that reaches it with the new field/condition in play, not just the call
site's most obvious (e.g. freshest-state) scenario.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Investigated a task-supplied suspicion that the Cancel button was a no-op; confirmed that premise false by direct grep/read, but surfaced this distinct dangling-handle defect while tracing the full call graph. |
| 2026-08-20 | fixed | Added a self-assignment check to `image_texture_set`'s (and, mirrored, `original_texture_set`'s) BUG-463 aliasing guard. Documented with `Fix(BUG-503)`/`Root cause`/`Pitfall` at 3 sites in `renderer.rs`. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Fix correctness (self-assignment guard closes the traced dangling-handle path, no new over-/under-deletion) | — | 🟢 | Confirming pass: hand-traced the fixed guard across all 5 real call-site sequences BUG-463's own report enumerates (`image_handler_create`, `apply_button_setup`, `cancel_button_setup`, `revert_button_setup`, `bg_removal_image_handler_create`), confirming the added `is_self_assign` check only ever suppresses a delete when the incoming and outgoing handles are identical (a true no-op-value case), never suppressing a delete BUG-463 still needs. Adversarial pass: specifically hunted for a case where `is_self_assign` could be true while a *different* texture genuinely needs freeing (which would reintroduce a leak) -- found none, since `is_self_assign` requires literal handle equality, not mere non-aliasing; also checked `original_texture_set`'s mirrored guard has no reachable call site that could make its own defensive check wrongly suppress a real delete (confirmed: its only call site, `main.rs`'s `image_handler_create`, always passes a freshly-created texture, never the current `original_texture` value). | Split the guard into two named booleans (`aliases_original`/`is_self_assign`) instead of a single compound condition, for the adversarial pass to audit each independently. |
| D2 | Fix documentation + test compliance | — | 🟢 | `Fix(BUG-503)`/`Root cause`/`Pitfall` 3-field format applied at 3 sites in `renderer.rs`; 5-section test doc (`Root Cause`/`Why Not Caught`/`Fix Applied`/`Prevention`/`Pitfall`) applied to the new reproducer test. `cargo nextest run -p filters` 3/3 pass; native and wasm32 clippy both clean with `-D warnings`. | — |
| D3 | Scope containment (no unrelated files touched) | — | 🟢 | `git diff --stat` confirms only `examples/minwebgl/filters/src/renderer.rs` (edited) and `examples/minwebgl/filters/tests/cancel_dangling_texture_test.rs` (new) changed for this item. | — |

**Reproduced:** Confirmed via hand-trace of the pre-fix single-condition guard against the exact
reachable "Apply once, then filter-and-Cancel" sequence (see Manual Reproduction / Verification),
and via the new test's pure-Rust mirror, which deletes handle `2` at the Cancel step when run
against the pre-fix algorithm. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/filters/src/renderer.rs` | `image_texture_set`/`original_texture_set`: added a self-assignment check (`is_self_assign`) alongside the existing sibling-aliasing check, so neither setter deletes a handle it is simultaneously reassigning to itself. `previous_texture_restore`: doc comment explains the dependency on this guard. All 3 sites carry `Fix(BUG-503)` comments. |

## Refs: tests/

| File | Change |
|------|--------|
| `examples/minwebgl/filters/tests/cancel_dangling_texture_test.rs` (new) | `bug_reproducer_bug_503_cancel_does_not_delete_the_texture_it_restores`: source-text assertion plus a pure-Rust mirror of `Renderer`'s texture-handle bookkeeping (comparable `u32` IDs standing in for `WebGlTexture`, per this crate's own `hsl_wraparound_test.rs`/`blur_kernel_test.rs` precedent), exercising the exact reachable sequence that makes the dangling-handle case live. |
