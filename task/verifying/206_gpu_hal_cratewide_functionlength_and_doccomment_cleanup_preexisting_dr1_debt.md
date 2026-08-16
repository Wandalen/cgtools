# 206: gpu_hal crate-wide function-length and doc-comment cleanup (pre-existing DR1 debt)

## Execution State

- **id:** 206
- **title:** gpu_hal crate-wide function-length and doc-comment cleanup (pre-existing DR1 debt)
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-16 21:43:30
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/gpu_hal
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-16 21:45:51
- **expires_at:** 2026-08-16 23:45:51
- **unverified_at:** 2026-08-16 21:45:43
- **unverified_by:** unknown
- **verifying_at:** 2026-08-16 21:45:51
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## Goal

Task 202's DR1 acceptance gate ("no function exceeds 50 lines") was checked with a
full, non-sampled crate sweep three times as the vulkan backend landed: round 1 found
7 over-limit functions in the new `vulkan.rs` (fixed under task 358); a follow-up
sweep of `device.rs`/`pass.rs` found 2 more (`submit`, `buffer_write`, both
task-202-caused, fixed in place); a final crate-wide sweep found 11 more functions
already over 50 lines, independently confirmed via `git show HEAD:<path>` to
pre-date task 202 entirely — task 202's own Vulkan-arm insertions never tipped any
of them from compliant to non-compliant (the smallest pre-202 span found was 55
lines, already 5 over on its own before task 202 touched it), so they are out of
scope for task 202 and recorded here as pre-existing crate debt instead. One
`pub fn` (`depth_range`, `device.rs:345`) is also missing its `///` doc comment,
confirmed pre-existing by the same method. Notably, `gpu_hal` already went through
one crate-wide function-length cleanup before (task 269, "11 violations" — the same
count, coincidentally), so this task's own investigation should start by checking
whether these 11 functions existed at that time and were missed, or were
added/lengthened by later work (tasks 088/089/090 wrote to `device.rs`/`pass.rs`
after 269) without a re-sweep. Resolve by splitting each over-limit function into
named helpers (same pattern task 358 used in `vulkan.rs`: extract the WebGl/Native
arm bodies into free functions placed after the `impl` block, before `mod
private`'s closing brace) and adding the missing `///` doc comment. Testable: a
full-crate `fn`-to-closing-brace line-span sweep of `module/helper/gpu_hal/src/`
reports zero functions over 50 lines, and `grep -B1 "pub fn depth_range"
device.rs` shows a `///` line immediately above the signature.

## In Scope

- `module/helper/gpu_hal/src/device.rs`: split `texture_create` (100 lines),
  `sampler_create` (60), `bind_group_layout_create` (77), `bind_group_create` (111,
  currently carries a `#[allow(clippy::too_many_lines, reason = "...")]` that should
  be removed once split), `render_pipeline_create` (82), `texture_write` (123),
  `native_render_pipeline_create` (99)
- `module/helper/gpu_hal/src/pass.rs`: split `render_pass_begin` (97),
  `bind_group_set` (66), `webgl_texture_pass_begin` (58)
- `module/helper/gpu_hal/src/native.rs`: split `texture_rgba8_read` (74 —
  already-accepted task-087 precedent for the "soft limit" reading of DR1; splitting
  it now brings the crate to a hard zero-violations state instead of relying on
  precedent to excuse it)
- `module/helper/gpu_hal/src/device.rs:345`: add a `///` doc comment to
  `depth_range` (currently only a `//` comment)
- Investigate (informational, not a Delivery Requirement) whether these 11 functions
  pre-date task 269's cleanup or were introduced after it, and note the finding in
  this task's Journal/History for whoever reviews the gap

## Out of Scope

- Any function in `vulkan.rs` — already fully compliant (task 358)
- `device.rs::submit`/`device.rs::buffer_write` — already fixed under task 202
  round 2, already compliant
- Any behavior change — this is a pure structural extraction, same as task 358's
  own precedent; no test assertions should need to change
- Any other crate's function-length debt — this task is `gpu_hal`-scoped only

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any), not
by this section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   No function in `module/helper/gpu_hal/src/` exceeds 50 lines (`fn` keyword to
    closing brace, inclusive) after this task, including the 4 new/split functions
    task 202 itself already added
-   No duplication introduced by the extraction (each helper called from exactly one
    call site, matching task 358's established placement convention)
-   `depth_range` carries a `///` doc comment
-   Zero behavior change: `cargo nextest run -p gpu_hal --all-features` passes with
    the same test count and zero failures, before and after
-   `cargo clippy -p gpu_hal --all-targets --all-features -- -D warnings` exits 0
-   `wasm32-unknown-unknown` check + clippy clean for the touched wasm-gated
    functions (`webgl_texture_pass_begin`, `bind_group_set`, `render_pass_begin`)
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

*(Pure refactor, zero behavior change — rows are structural checks, not new
`cargo test` cases; the existing `vulkan_backend_test.rs`/`native_backend_test.rs`
suites are the behavior-preservation regression net.)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Full-crate line-span sweep of `module/helper/gpu_hal/src/*.rs` | Every `fn`/`pub fn` | Zero functions exceed 50 lines |
| T02 | `grep -B1 "pub fn depth_range" device.rs` | Doc comment line | Starts with `///`, not `//` |
| T03 | `cargo nextest run -p gpu_hal --all-features` | Full existing test suite | Same pass count as before this task, 0 failures |
| T04 | `cargo clippy -p gpu_hal --all-targets --all-features -- -D warnings` | Lint | Exit 0, 0 warnings |
| T05 | `cargo check -p gpu_hal --target wasm32-unknown-unknown --features webgl,webgpu` | Wasm build | Exit 0 |

## Acceptance Criteria

-   Every function in `module/helper/gpu_hal/src/` is ≤50 lines
-   `depth_range` has a `///` doc comment
-   No test assertions changed; full suite still passes
-   `clippy -D warnings` clean natively and for `wasm32-unknown-unknown`
-   Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Structural**
- [ ] C1 — Are all 8 named `device.rs`/`native.rs` functions ≤50 lines after the split?
- [ ] C2 — Are both named `pass.rs` functions ≤50 lines after the split?
- [ ] C3 — Does `depth_range` carry a `///` doc comment?
- [ ] C4 — Are the new helper functions placed following task 358's established
      convention (free functions after the `impl` block, before `mod private`'s
      closing brace), with no duplication?

**Out of Scope confirmation**
- [ ] C5 — Is `vulkan.rs` untouched (zero diff)?
- [ ] C6 — Are `submit`/`buffer_write` untouched (zero diff beyond task 202 round 2's
      already-accepted state)?

### Measurements

- [ ] M1 — Full-crate sweep: zero functions >50 lines
- [ ] M2 — `cargo nextest run -p gpu_hal --all-features` pass count unchanged vs. pre-task baseline

### Invariants

- [ ] I1 — `cargo nextest run -p gpu_hal --all-features` — 0 failures
- [ ] I2 — `cargo clippy -p gpu_hal --all-targets --all-features -- -D warnings` — 0 warnings
- [ ] I3 — `cargo check -p gpu_hal --target wasm32-unknown-unknown --features webgl,webgpu` — exit 0

### Anti-faking checks

- [ ] AF1 — Confirms the split didn't silently drop a match arm or code path (line
      count of extracted helper + remaining call site ≈ original function's line
      count, not less)

## Related Documentation

- `task/accepting/202_gpu_hal_vulkan_backend.md` — DR1's originating gate; its
  round-3 acceptance sweep found these 11 functions and this task's evidence base
  (exact pre/post `git show HEAD` line spans) was gathered while closing that gate
- `task/completed/` task 358 (folded into 202's own history) — established the
  split-into-named-helpers pattern this task reuses
- `task/completed/269...` (`gpu_hal` crate, 11 violations) — the prior crate-wide
  `gpu_hal` function-length cleanup; this task should confirm whether these 11
  findings pre-date or post-date that sweep

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-16 21:43:30 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | FILED | task created |
| 2026-08-16 21:45:43 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-16 21:45:51 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_VERIFY | verification claimed |

## History

- **[2026-08-16]** `FILED` — Filed while closing task 202's DR1 gate. Task 202's
  round-3 acceptance sweep found 11 functions in `device.rs`/`pass.rs`/`native.rs`
  already over 50 lines plus 1 missing doc comment; each was independently confirmed
  via `git show HEAD:<path>` line-span comparison to pre-date task 202 (smallest
  pre-202 span: 55 lines, already over the limit on its own) — out of scope for 202,
  filed here as separately-tracked pre-existing debt instead.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value/YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | — | — |

Pass 1 (Confirming): In Scope names all 11 exact functions by name/file/current
line count plus the 1 doc-comment gap; Out of Scope names 4 explicit exclusions
(`vulkan.rs`, `submit`/`buffer_write`, behavior changes, other crates). Goal states
the exact discovery history (3 DR1 sweeps), cites the exact evidence (pre-202 line
spans via `git show HEAD`) and a concrete success test (full-crate sweep + grep).
Delivery Requirements are directly testable (line-span sweep, doc-comment grep,
existing test-suite pass-count parity). Single crate (`gpu_hal`), same unit as task
202, no new module/crate needed — pure internal refactor.

Pass 2 (Adversarial): attempted to disprove this is gold-plating — checked whether
DR1's literal text ("no function exceeds 50 lines") is actually still active/binding
now that task 202 itself is closing; it is, verbatim, in this task's own inherited
Delivery Requirements and is not scoped to "only code task 202 touched," so leaving
these 11 pre-existing violations unfixed would leave the crate in a state task 202's
own governing requirement calls non-compliant. Attempted to find scope creep beyond
the 11 functions + 1 doc gap already enumerated by name — none found; the task text
explicitly excludes `vulkan.rs` and the already-fixed `submit`/`buffer_write`.
Attempted to find a duplication/locality violation — task 358's own precedent
(already accepted under task 202) establishes the exact extraction pattern this task
reuses, so this isn't introducing a new pattern needing separate justification. All 8
hold.

**Tier-cap note:** this Gate Check, and the DR1 causal-attribution self-check
recorded in task 202's own Outcomes section, were both run at Tier 2 per this
project's standing instruction to cap verification at Tier 2
(`feedback_maav_tier_cap.md`). Earlier rounds of task 202's own DR1 acceptance
checking used dispatched Tier 3 verifiers (one formally recorded in task 202's
Outcomes section, two more ad hoc during this same round) — inconsistent with that
standing cap, noted transparently in task 202's own record rather than continued
here or silently carried forward.
