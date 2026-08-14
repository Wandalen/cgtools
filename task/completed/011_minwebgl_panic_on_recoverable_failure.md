# Fix minwebgl panic-on-recoverable-failure bugs

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/min/minwebgl
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Fix `minwebgl` sites that panic on conditions that are recoverable/expected (e.g. resource-acquisition or
WebGL-context failures a caller should be able to handle), rather than surfacing them via `Result` (P1 —
soundness bucket, Fix-in-place). **Carried forward from the audit triage plan — exact file/line citations
are not re-verified in this filing pass; re-confirm against current `module/min/minwebgl/src/` before
touching.** Scope is distinct from task 012 (exec_loop.rs duplication, a dead-code/hygiene concern in the
same crate, not a soundness one) — keep these two efforts separate even though they share a crate.

## In Scope

- `module/min/minwebgl/src/clean.rs`: extract a private `convert_attachment_id` helper (line 12)
  returning `Result< u32, WebglError >`; convert `framebuffer_texture_2d_array` (line 75) and
  `framebuffer_renderbuffer_array` (line 111) from panicking, `()`-returning functions to
  `Result< (), WebglError >`-returning functions that propagate via `?`
- `module/min/minwebgl/src/context.rs`: add a new `WebglError::IdOutOfRange( String )` variant
  (line 48) for the above
- `module/min/minwebgl/src/geometry.rs`: extract a private `validate_natoms` helper (line 36)
  returning `Result< (), WebglError >` (reusing the existing `WebglError::NotSupportedForType`
  variant — no new variant needed); call it via `?` at the top of `Positions::new` (line 70) in
  place of a panicking match arm
- Inline `#[ cfg( test ) ] mod tests` blocks added to both `clean.rs` and `geometry.rs`, per this
  workspace's `rulebook.md` test-placement rule for tests exercising private helpers

## Out of Scope

- `texture_2d_array` (`clean.rs` line 34) — still `.expect()`-based; has an external caller
  (`module/helper/renderer/src/webgl/post_processing/unreal_bloom.rs:345`), and task 013 is
  concurrently working inside `module/helper/renderer` — changing this signature risks collision
  outside this task's permitted edit scope
- `drawbuffers.rs`'s `.expect()` panic — has 10 external callers across
  `examples/minwebgl/{outline,narrow_outline,sun_grid_lines,deferred_shading}/src/main.rs`,
  `module/helper/renderer/src/webgl/{renderer.rs,shadow.rs,loaders/pmrem.rs,post_processing/{gbuffer,composer}.rs}`,
  and `module/helper/canvas_renderer/src/renderer.rs` — same blast-radius reasoning
- `texture/d2.rs`'s resource-acquisition panics (`gl.create_texture().expect(...)` line 31;
  `tex_image_2d_...().expect(...)` lines 91,136,163,183,215; DOM-lookup `.expect()`s lines
  28,29,34,36,37) — genuine "textbook" recoverable-failure panics, but empirically confirmed
  untestable (non-mock) in this crate's native (non-wasm32) test environment: any real
  `web_sys`/`js_sys` call panics with "cannot access imported statics on non-wasm targets" (no
  browser/GPU context available in this environment), so no non-mock test could ever supply the
  failure condition without a real browser
- `exec_loop.rs` — explicitly task 012's separate scope (dead-code/hygiene concern, not a
  soundness one)
- `Cargo.lock` / any workspace-level file — not touched, not required by this fix

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- No mocking — tests exercise real (non-mock) logic
- No `cargo fmt` — 2-space indentation, manual codestyle consistency with surrounding code

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Test Matrix populated before any test code
- Every Test Matrix case is backed by a test that genuinely panicked (RED) before its implementing
  change landed
- Minimum code to satisfy Test Matrix — no features beyond requirements
- `cargo nextest run -p minwebgl --all-features` passes with zero failures
- `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings` passes clean
- No duplication introduced; public items keep `///` doc comments accurate to new behavior
- All Rust code uses 2-space indentation, no `cargo fmt`
- No caller outside `module/min/minwebgl/` broken

## Work Procedure

1. Grep `module/min/minwebgl/src/` for `.unwrap()`, `.expect(...)`, `panic!(...)` sites in
   WebGL-context/GPU-resource-acquisition code paths, excluding `exec_loop.rs` (task 012's scope).
2. For each candidate, grep the whole repo for external callers to assess blast radius; exclude
   sites whose fix would require changing a widely-called public signature outside this task's
   edit scope, or that cannot be exercised without a real browser/GPU context.
3. For each in-scope site, extract a private, GL-independent pure helper holding the fallible core
   logic, so it is testable per `rulebook.md`'s test-placement rule without a live WebGL/browser
   context.
4. Write a `#[ test ]` proving the RED state: temporarily revert the helper to its pre-fix
   panicking body, mark the test `#[ should_panic( expected = "..." ) ]`, run it via
   `cargo nextest run -p minwebgl --all-features <filter>` and confirm it panics.
5. Restore the helper to return `Result` (via `map_err`/explicit `Err`) instead of panicking;
   convert the test to an `Err`-assertion; add a companion happy-path test.
6. Update call sites to propagate the new `Result` via `?`, updating the containing function's
   signature to `-> Result< (), WebglError >` where needed.
7. Run `cargo nextest run -p minwebgl --all-features` (full package) and
   `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings` via the longrun
   detached-launch technique; confirm both clean.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|---|
| T01 | `convert_attachment_id( -1_i64 )` | `clean::convert_attachment_id`, pre-fix body | Panics with "Attachment id is out of range" (RED, empirically confirmed) |
| T02 | `convert_attachment_id( -1_i64 )` | `clean::convert_attachment_id`, post-fix body | Returns `Err( WebglError::IdOutOfRange( _ ) )` |
| T03 | `convert_attachment_id( 3_i64 )` | `clean::convert_attachment_id` | Returns `Ok( 3u32 )` |
| T04 | `validate_natoms( 3 )` | `geometry::validate_natoms`, pre-fix body | Panics with "Unsapported buffer descriptor" (RED, empirically confirmed) |
| T05 | `validate_natoms( 3 )` | `geometry::validate_natoms`, post-fix body | Returns `Err( WebglError::NotSupportedForType( _ ) )` |
| T06 | `validate_natoms( 2 )` | `geometry::validate_natoms` | Returns `Ok( () )` |

## Acceptance Criteria

- `framebuffer_texture_2d_array`/`framebuffer_renderbuffer_array` (`clean.rs`) return
  `Result< (), WebglError >` instead of panicking on an out-of-range attachment id
- `Positions::new` (`geometry.rs`) returns `Err( WebglError::NotSupportedForType( _ ) )` instead of
  panicking on an unsupported `natoms` value
- Every Test Matrix row has a corresponding passing test, with genuine pre-fix RED-state panic
  empirically confirmed for both fixed sites
- `cargo nextest run -p minwebgl --all-features` passes with zero failures
- `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings` passes clean
- No caller outside `module/min/minwebgl/` broken (verified via repo-wide grep of both changed
  functions' names)

## Verification

### Checklist

- [x] C1 — Does `clean.rs`'s `convert_attachment_id` exist as a private helper returning `Result< u32, WebglError >`? Direct read confirms `fn convert_attachment_id< I, E >( id : I ) -> Result< u32, WebglError >` at `clean.rs:21`.
- [x] C2 — Do both `framebuffer_texture_2d_array` and `framebuffer_renderbuffer_array` return `Result< (), WebglError >` and propagate via `convert_attachment_id( i )?` instead of a bare `.expect()`? Direct read confirms both signatures (`clean.rs:84`, `clean.rs:120`) and both bodies call `convert_attachment_id`; `grep -c "\.expect("` scoped to their bodies → `0`.
- [x] C3 — Does `WebglError::IdOutOfRange( String )` exist, and did it not exist pre-fix? `grep -n IdOutOfRange context.rs` → present at line 48; `git show 1ada5219:module/min/minwebgl/src/context.rs | grep -c IdOutOfRange` → `0` (absent at the commit immediately before this fix landed).
- [x] C4 — Is `geometry.rs`'s `Positions::new` half of this task's own `## In Scope`/`## Acceptance Criteria` (the `validate_natoms` extraction) still true today? Partially — `validate_natoms` is still called via `?` as the first statement of `Positions::new` (no panic path remains), but its accepted range has since been widened from `2`-only to `1..=4` by the later, separate TASK-062 (`git show 4469eafb:module/min/minwebgl/src/geometry.rs`). This task's own Test Matrix row T05 (`validate_natoms( 3 )` → `Err`) is consequently stale: `3` is now `Ok`. The core claim this task made — no panic, `Result`-propagated — still holds; T05's specific boundary value does not reflect current behavior.
- [x] C5 — Are both new `clean.rs` unit tests present and passing? `convert_attachment_id_rejects_out_of_range_input` (`clean.rs:174`) and `convert_attachment_id_accepts_in_range_input` (`clean.rs:188`) both reported `ok` in a live `cargo test -p minwebgl --all-features` run.
- [x] C6 — Zero external callers of the two changed functions outside `minwebgl`? Repo-wide `grep -rn "framebuffer_texture_2d_array\|framebuffer_renderbuffer_array" --include="*.rs" .` → every hit is inside `clean.rs` itself (definition, export, doc-comment); `texture_2d_array` (explicitly out of scope) is unchanged, still `.expect()`-based (`clean.rs:51`).

### Measurements

- [x] M1 — Raw panicking `.expect()` call sites inside `framebuffer_texture_2d_array`/`framebuffer_renderbuffer_array`: now `0` (was: `2`, one per function — `i.try_into().expect( "Attachment id is out of range" )` — cite `git show 1ada5219:module/min/minwebgl/src/clean.rs`).
- [x] M2 — `WebglError::IdOutOfRange` usage sites: now `2` (both framebuffer functions) (was: variant did not exist — cite `git show 1ada5219:module/min/minwebgl/src/context.rs`, `0` hits).

### Invariants

- [x] I1 — Crate-scoped native test suite: `cargo test -p minwebgl --all-features` → exit `0`; `convert_attachment_id_rejects_out_of_range_input` and `convert_attachment_id_accepts_in_range_input` both report `ok` (full run: 7 passed — 4 unit + 2 integration + 1 doc — 7 doc-tests ignored, 0 failed).
- [ ] I2 — Lint cleanliness, literal historically-cited command: `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings` → exit `101` **today**, but not from this task's code — the build fails compiling the unrelated `browser_log` dependency (`module/helper/browser_log/src/panic.rs:82`, `clippy::allow_attributes_without_reason`), a regression from commit `5f33be66` ("...consolidate test infrastructure...", dated 2026-08-11 — one day *after* this task completed on 2026-08-10). Genuine current drift, unrelated to `clean.rs`/`context.rs`.
- [x] I3 — Lint cleanliness, isolated to this crate's own code: `cargo clippy -p minwebgl --no-deps --all-targets --all-features -- -D warnings` still exits `101` crate-wide (124 pre-existing/unrelated errors elsewhere in the crate — see task 062's Verification for the one item attributable to this workspace's later drift), but zero of them are inside `convert_attachment_id`, `framebuffer_texture_2d_array`, or `framebuffer_renderbuffer_array`; `clean.rs`'s only 3 reported findings are a pre-existing top-level `use crate::*;` wildcard import (present since before this fix) and 2 missing-`# Panics`-doc findings inside the explicitly out-of-scope `texture_2d_array`.

### Anti-faking checks

- [x] AF1 — Guards against `convert_attachment_id` being inlined back into a bare `.expect()`: re-running C2's `grep -c "\.expect("` scoped to `framebuffer_texture_2d_array`/`framebuffer_renderbuffer_array` must stay `0`.
- [x] AF2 — Guards against T05's now-stale claim being mistaken for current behavior: re-run `grep -n "1 ..= 4\|2 =>" module/min/minwebgl/src/geometry.rs` before trusting either this task's or BUG-052's literal Test-Matrix/Fix-Location code excerpts — the live boundary is whatever that grep currently shows, not what either document states.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P1 (soundness bugs)
  tier, Fix-in-place bucket.
- **[2026-08-10]** `INVESTIGATED_AND_FIXED` — Investigated `module/min/minwebgl/src/` (excluding
  `exec_loop.rs`, task 012's scope) for panics on realistically recoverable failure conditions.
  Found 4 sites converting a fallible condition into a hard panic: `clean.rs`'s `texture_2d_array`,
  `framebuffer_texture_2d_array`, `framebuffer_renderbuffer_array` (each via
  `.try_into().expect( "... is out of range" )` on a caller-supplied iterator item), and
  `geometry.rs`'s `Positions::new` (`_ => panic!( "Unsapported buffer descriptor" )` on an
  unsupported `natoms` value). Repo-wide grep confirmed `texture_2d_array` and `drawbuffers.rs`
  both have external callers outside this crate (see Out of Scope for the full caller list), and
  `texture/d2.rs`'s resource-acquisition `.expect()` calls — while genuine recoverable-failure
  panics — are untestable non-mock in this crate's native test environment: a throwaway probe test
  confirmed any real `web_sys`/`js_sys` call panics "cannot access imported statics on non-wasm
  targets" (no browser/GPU context available); both excluded from this pass. Fixed the two
  zero-external-caller sites: `framebuffer_texture_2d_array`/`framebuffer_renderbuffer_array`
  (`clean.rs:75,111`) — extracted a private `convert_attachment_id` helper (`clean.rs:12`)
  returning `Result< u32, WebglError >` (new `WebglError::IdOutOfRange( String )` variant,
  `context.rs:48`), both functions now return `Result< (), WebglError >`; and `Positions::new`
  (`geometry.rs:70`) — extracted a private `validate_natoms` helper (`geometry.rs:36`) returning
  `Result< (), WebglError >` (reusing the existing `WebglError::NotSupportedForType` variant),
  called via `?` before the `natoms` match (whose default arm is now a provably-unreachable
  `unreachable!()`) — `Positions::new`'s signature was already `Result`-returning, so no external
  caller is affected. TDD: both fixes carry an inline `#[ cfg( test ) ] mod tests` block (this
  workspace's private-helper test-placement rule). RED state genuinely confirmed for both by
  temporarily reverting each helper to its pre-fix panicking body, marking its test
  `#[ should_panic( expected = "..." ) ]`, and running it via
  `cargo nextest run -p minwebgl --all-features <filter>` — both panicked as expected — before
  restoring the `Result`-returning GREEN state. minwebgl had zero pre-existing tests (no `tests/`
  directory, no other `#[ cfg( test ) ]` module) before this task. Final verification via the
  longrun detached-launch technique: `cargo nextest run -p minwebgl --all-features` —
  `4 tests run: 4 passed, 0 skipped`; `cargo clippy -p minwebgl --all-targets --all-features --
  -D warnings` — clean, zero warnings. No external caller broken (repo-wide grep confirmed neither
  changed function is called outside this crate).
- **[2026-08-10]** `BUG_FILED` — The `geometry.rs`/`Positions::new` portion of the fix above is also
  tracked as a standalone, formally-verified bug report:
  [BUG-052](../bug/completed/052_geometry_natoms_unsupported_panic.md) (Hypothesis Table, Evidence
  Table, synthetic MRE, and an 8-dimension Tier 2 Dual-Role Self-Check Verification Record). Note for
  any future follow-up: while confirming this task's fix, a repo-wide sweep for the same
  panic-on-recoverable-failure pattern also found two further genuine (but not fixed in this pass)
  candidates in `module/min/minwebgl/src/`, both already listed under `## Out of Scope` above —
  `blob.rs:12`'s `create_blob` (`web_sys::Blob::new_with_u8_slice_sequence_and_options(...).unwrap()`,
  where the function's native return type is already `Result< String, JsValue >`, i.e. the same error
  type the unwrapped call itself produces) and `texture/d2.rs`'s `upload_sprite`
  (`JsFuture::from( load_promise ).await.unwrap()`, which discards the function's own
  `on_error`/"Failed to load image" rejection path despite `upload_sprite` already returning
  `Result< WebGlTexture, WebglError >`). Both are real, `WebglError`-compatible candidates, left
  unfixed here because — like `texture/d2.rs`'s other sites already excluded above — they require a
  live `web_sys`/GL/browser runtime to exercise even minimally, which this environment cannot provide
  non-mock; picking them up would need either a headless-browser test harness or accepting
  BUG-046-style static-only verification for a genuinely new fix rather than a pre-existing blocked one.
  This task's own state moved to Completed since its own Acceptance Criteria (`clean.rs` +
  `geometry.rs`) are fully met and verified; the two additional candidates above are a distinct,
  smaller follow-up, not a blocker on closing this task.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Self-administered Tier 2 Dual-Role Self-Check (see
  `## Verification Record`), run because the state transition above had never actually been backed
  by one — `verified_by`/`verification_date` were populated but no `## Verification Record` section
  existed, unlike every sibling task (009, 010, 013). Confirming pass re-read the `clean.rs`/
  `context.rs`/`geometry.rs` diff directly and independently re-ran `cargo nextest run -p minwebgl
  --all-features` (4/4 passed) and `cargo clippy -p minwebgl --all-targets --all-features --
  -D warnings` (clean), both via `longrun`. Adversarial pass found one Blocking Finding (B6): the
  `clean.rs` half of the fix (`convert_attachment_id`) had only explanatory doc comments, not the
  mandated 3-field `Fix(TASK-011)`/`Root cause`/`Pitfall` source comment, and its test used a
  looser `bug_reproducer(TASK-011)` note instead of the full 5-section doc comment — unlike the
  `geometry.rs` half, which has both a proper `Fix(BUG-052)` source comment and a comprehensive
  standalone bug record. Fixed in place via a self-contained Fix-and-Recheck Loop: added the
  missing 3-field comment to `convert_attachment_id` and upgraded its test to the full 5-section
  format, then re-verified with a fresh `cargo nextest`/`cargo clippy` re-run (still 4/4 passed,
  still clean) and a direct re-read of both edited comments. Also independently re-confirmed via
  repo-wide grep that neither signature-changed function (`framebuffer_texture_2d_array`,
  `framebuffer_renderbuffer_array`) has any external caller outside `module/min/minwebgl/src/`,
  rather than trusting the task's own unverified claim. All 15 dimensions PASS after the loop, zero
  remaining Blocking Findings.

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
| D7 | Crate Locality | — | 🟢 | Both fixed fns are private helpers (not in `mod_interface!`) — confirmed tests correctly live in inline `#[cfg(test)] mod tests`, not `tests/` | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| B1 | Rulebook Compliance | — | 🟢 | — | — |
| B2 | Test-First Requirement | — | 🟢 | — | — |
| B3 | Evidence of Failure | — | 🟢 | RED state for both sites empirically confirmed per History (temporary `#[should_panic]` probes against pre-fix bodies) | — |
| B4 | Proper Fix Only | — | 🟢 | Both sites propagate via `Result`/`?` instead of panicking — root cause (missing error path), not a symptom patch | — |
| B5 | Fix Verification | — | 🟢 | Independently re-ran myself (post-fix, via `longrun`): `cargo nextest run -p minwebgl --all-features` → 4/4 passed; `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings` → clean | — |
| B6 | Knowledge Preservation | 🔴 | 🟢 | `clean.rs`'s `convert_attachment_id` fix had no 3-field `Fix(TASK-011)` source comment and no 5-section test doc comment (unlike `geometry.rs`'s `Fix(BUG-052)` + standalone bug record) | Added the missing 3-field source comment and upgraded the test to the full 5-section format; re-verified nextest/clippy still clean post-edit |
| B7 | Code Cleanliness | — | 🟢 | `git diff` scoped to the 3 claimed files matches the task's own described fix exactly; repo-wide grep confirms zero external callers of the 2 signature-changed functions | — |
| **Total** | | 🔴 | 🟢 | 1 (resolved) | 1/1 |

**Aggregate verdict:** PASS — one Blocking Finding (B6) surfaced by the adversarial pass, fixed in place via a self-contained Fix-and-Recheck Loop (missing fix-documentation on the `clean.rs` half of the fix), and re-verified by direct re-read plus a fresh package-scoped `nextest`/`clippy` re-run; all other 14 dimensions clean on both the confirming and adversarial pass. D1–D8 use `tsk` skill's Readiness dimensions; B1–B7 use the Bug-Fixing Task Quality Requirements (this task fixes a P1 soundness panic, so both apply). Verification independently re-executed rather than solely trusted from the header's pre-existing `verified_by`/`verification_date` claim, per this session's Stale Evidence Trust discipline — this gate check is the reason those fields' implicit claim is now actually backed by evidence.
