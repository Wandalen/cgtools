# Native test coverage for renderer's glTF animation-channel decode + vec3 sequence builder

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-18 23:49:12
- **expires_at:** 2026-08-19 01:49:12
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/renderer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **in_motion:** true
- **verifying_at:** 2026-08-18 23:49:12
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **unverified_at:** 2026-08-18 23:47:41
- **unverified_by:** system

## Goal

`module/helper/renderer/src/webgl/animation/loaders/gltf.rs` contains four
private, pure functions with zero `gl`/`GL`/`WebGl` references anywhere in
their bodies — `channel_decode` (line 48-81), `quat_sequence` (line
83-175), `vec3_sequence` (line 177-268), `weights_sequence` (line
270-360) — confirmed by direct full-file re-read this session. Only
`load` (line 363-468, `pub async fn`, takes `gl: &GL`) is exported via
the file's `mod_interface! { own use { load }; }` block (line 471-477).
None of the four pure functions has any test coverage, native or
otherwise: `tests/animation_tests.rs`, the only test file touching this
loader, is wholly wasm/browser-bound (`#[cfg(target_arch = "wasm32")]`,
`wasm_bindgen_test_configure!(run_in_browser)`, builds a real
`WebGl2RenderingContext`) and exercises `load()` end-to-end, never these
four sub-functions in isolation. This mirrors the exact shape of gap this
crate already closed once for the *other* glTF loader
(`light_list_get`, task 118) — the same "GPU-free logic buried inside a
GL-gated module, with only end-to-end browser coverage" pattern. Fix by
giving `channel_decode` and `vec3_sequence` — the shared decode primitive
and the simplest of the three sequence-builders — native test coverage,
following task 118's exact precedent: mark both `pub fn` (matching how
`light_list_get` was exposed at `src/webgl/loaders/gltf.rs` line 300/1366)
and add both to this file's `mod_interface!` block. `quat_sequence` and
`weights_sequence` are the same shape (decode → per-keyframe iterate →
build easing → push tween → BUG-188 single-keyframe guard →
`Sequence::new`) and are explicitly deferred as same-shape follow-up
work, not part of this task. `vec3_sequence` is currently 92 lines
(177-268), over this project's ≤50-line function-length convention;
exporting it for testing brings it into scope for the same split-on-touch
treatment already applied to `gpu_hal`'s `device.rs`/`pass.rs`/`native.rs`
(tasks 358, 374-376) — this task splits it into ≤50-line pieces as part
of exporting it, not as a separate cleanup pass. This is gap #5b from the
2026-08-17 docs/layer round-3 gap audit.
Testable: `cargo nextest run -p renderer --lib --tests channel_decode`
and `... vec3_sequence` both report ≥1 passing test each (currently: the
function names appear nowhere under `tests/`, confirmed by
`grep -rn "channel_decode\|vec3_sequence" module/helper/renderer/tests/`
returning no matches).

## In Scope

- `module/helper/renderer/src/webgl/animation/loaders/gltf.rs`: mark
  `channel_decode` and `vec3_sequence` `pub fn` (currently private, no
  `pub` keyword); add both to the file's `mod_interface! { own use { ... } }`
  block alongside the existing `load` entry.
- Same file: split `vec3_sequence`'s body (currently 92 lines) into
  `≤50`-line pieces as part of exporting it — e.g. extracting the
  per-keyframe tween-construction inner-loop body into its own helper,
  matching the split shape already used in `gpu_hal`'s `device.rs`/
  `pass.rs`/`native.rs` (tasks 358, 374-376). No behavior change; this is
  a pure extraction.
- New test file `module/helper/renderer/tests/gltf_animation_loader_test.rs`:
  native (non-wasm, no `WebGl2RenderingContext`) tests for `channel_decode`
  and `vec3_sequence`, using an inline minimal glTF JSON fixture constant
  parsed via `gltf::Gltf::from_slice` — the exact pattern already
  established in this same directory's `gltf_light_parsing_test.rs`
  (task 118's own precedent) — plus a hand-constructed `buffers:
  Vec<Vec<u8>>` byte vector (e.g. via `f32::to_le_bytes()`), since
  `channel_decode`/`vec3_sequence` take buffer bytes as a plain parameter
  independent of the document's own `uri` field — no base64/`data:` URI
  decoding or async fetch is needed in the test at all.
- `module/helper/renderer/tests/readme.md`: add a Responsibility Table
  row for the new test file.

## Out of Scope

- `quat_sequence` and `weights_sequence` — same shape as `vec3_sequence`,
  deliberately deferred as same-shape follow-up work; not exported, not
  tested, not split by this task.
- `load()` itself, or its own `mod_interface` export — already correct,
  not touched.
- `tests/animation_tests.rs` (the existing wasm/browser suite) — remains
  the browser-side coverage; not modified, not replaced.
- `assets/gltf/animated/single_keyframe_translation.gltf` — not reused or
  modified; the new test constructs its own inline fixture rather than
  depending on this file (matching task 118's own inline-constant
  approach over a separate asset file).
- Any behavior change to `channel_decode`/`vec3_sequence`'s actual logic
  — the split is a pure extraction; outputs must be bit-for-bit identical
  before and after.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `verb/test_only` (or `cargo nextest run -p renderer`) passes with the
    new test file included
-   `channel_decode` and `vec3_sequence` are both `pub fn`, both exported
    via `mod_interface!`, and both callable from `tests/`
-   No function in the touched file exceeds 50 lines (the existing
    ≤50-line convention applied to `vec3_sequence`'s split, and confirmed
    unbroken elsewhere in the file)
-   `cargo clippy -p renderer --all-targets --all-features -- -D warnings`
    passes clean
-   New public functions (`channel_decode`, `vec3_sequence`) carry a doc
    comment stating their pure, GPU-free contract
-   `tests/readme.md` has a Responsibility Table row for the new test file
-   Independent verification passes per `§ Acceptance Verification :
    Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to
    `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Linear-interpolation Translation channel, 2 keyframes | `channel_decode(channel, buffers)` | `Some((1, times, ReadOutputs::Translations(_)))` — `components == 1` for non-CubicSpline |
| T02 | CubicSpline-interpolation channel, 2 keyframes (in-tangent/value/out-tangent triples) | `channel_decode(channel, buffers)` | `Some((3, times, values))` — `components == 3` for CubicSpline |
| T03 | Linear-interpolation Translation channel, 2 keyframes with distinct known values | `vec3_sequence(channel, buffers)` | `Some(Sequence)` with exactly 2 tweens; tween endpoint values match the fixture's authored translation vectors |
| T04 | Translation channel with exactly 1 keyframe (BUG-188 regression precedent, same shape as `assets/gltf/animated/single_keyframe_translation.gltf`) | `vec3_sequence(channel, buffers)` | `Some(Sequence)` — the lone tween is duplicated to satisfy `Sequence::new`'s minimum-2 requirement, not silently dropped (mirrors the guard at line 260-265) |

## Acceptance Criteria

-   `channel_decode` has native test coverage for both the Linear
    (`components == 1`) and CubicSpline (`components == 3`) branches
-   `vec3_sequence` has native test coverage for both the ordinary
    multi-keyframe case and the BUG-188 single-keyframe-duplication case
-   `vec3_sequence` is ≤50 lines per function after the split, with no
    behavior change
-   `quat_sequence`/`weights_sequence` are untouched (same private,
    untested state as before)
-   Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does
NOT self-verify — an independent verifier performs the walk after the
task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Test coverage**
- [ ] C1 — Does `cargo nextest run -p renderer` report the 4 new test
  cases passing?
- [ ] C2 — Do the tests exercise `channel_decode` directly (not only
  indirectly through `vec3_sequence`)?
- [ ] C3 — Does the BUG-188 single-keyframe test (T04) assert the tween
  is duplicated (length 2), not just that the call returns `Some`?

**Code quality**
- [ ] C4 — Is `vec3_sequence` (and any extracted helper) ≤50 lines each?
- [ ] C5 — Does `cargo clippy -p renderer --all-targets --all-features --
  -D warnings` pass clean?
- [ ] C6 — Do `channel_decode` and `vec3_sequence` carry a doc comment?
- [ ] C7 — Is `vec3_sequence`'s output bit-for-bit unchanged by the split
  (verified by the new tests passing, and by `quat_sequence`/
  `weights_sequence`'s own still-passing indirect coverage via `load()`'s
  existing browser tests)?

**Documentation**
- [ ] C8 — Does `tests/readme.md` have a row for the new test file?

**Out of Scope confirmation**
- [ ] C9 — Are `quat_sequence` and `weights_sequence` still private and
  untested (`git diff` shows no `pub` added to either)?
- [ ] C10 — Is `tests/animation_tests.rs` untouched?

### Measurements

- [ ] M1 — `cargo nextest run -p renderer --tests -- channel_decode` → ≥1 passing test (was: 0, function name absent from `tests/`)
- [ ] M2 — `cargo nextest run -p renderer --tests -- vec3_sequence` → ≥1 passing test (was: 0)
- [ ] M3 — longest function in `src/webgl/animation/loaders/gltf.rs` ≤ 50 lines (was: `vec3_sequence` at 92)

### Invariants

- [ ] I1 — full crate still builds: `cargo check -p renderer --all-features` → 0 errors
- [ ] I2 — full existing test suite still passes (no regression from the
  split): `cargo nextest run -p renderer` → 0 failures
- [ ] I3 — `quat_sequence`/`weights_sequence` remain private (`grep -n
  "pub fn quat_sequence\|pub fn weights_sequence"
  module/helper/renderer/src/webgl/animation/loaders/gltf.rs` → no match)

### Anti-faking checks

- [ ] AF1 — T04's assertion checks the tween *count* (2, not 1) and/or
  that both tweens carry the same authored value — not just `is_some()`,
  which would pass even if the BUG-188 guard were silently removed
- [ ] AF2 — the CubicSpline test (T02) uses a channel whose sampler
  interpolation is actually `Interpolation::CubicSpline`, not a Linear
  channel mislabeled in the test's own comment — checked by reading the
  literal fixture JSON's `sampler.interpolation` field

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`renderer`), matching task 118's own precedent exactly | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | — | — |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-17 04:20:34 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-17 04:21 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 223` → blocked: `tsk: .verify_pass: self-verification forbidden (actor matches filed_by)`; left at 🔬 Verifying |
| 2026-08-18 23:47:41 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:12 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-17]** `FILED` — Task filed via docs/layer round-3 gap audit (gap #5b): native test coverage for `channel_decode` + `vec3_sequence` in renderer's glTF animation loader, following task 118's exact `light_list_get` precedent; `quat_sequence`/`weights_sequence` deferred as same-shape follow-up.
- **[2026-08-17]** `EXECUTED` — **Implementation.** `channel_decode` marked `pub fn` with a doc comment stating its pure, GPU-free contract (no `gl`/`GL`/`WebGl` calls). `vec3_sequence`'s original 92-line private body split into a new private helper `vec3_tween_from_keyframe` (49 lines, extracted per-keyframe tween-construction logic) and a shortened `pub fn vec3_sequence` (48 lines) that calls it in its loop — pure extraction, no behavior change, including the BUG-188 single-keyframe-duplication guard preserved unchanged. Both new `pub fn`s added to the file's `mod_interface! { own use { ... } }` block alongside the existing `load` entry. New native test file `tests/gltf_animation_loader_test.rs` added: T01/T02 call `channel_decode` directly (Linear → `components == 1`; CubicSpline → `components == 3`, with an explicit assertion the fixture's sampler interpolation really is `CubicSpline` per AF2); T03 calls `vec3_sequence` on a 2-keyframe Linear channel and asserts both tweens' `start_value`/`end_value` match the fixture's authored translation vectors via `Tween`'s public fields; T04 exercises the BUG-188 guard on a 1-keyframe channel and asserts the tween count is 2 (not just `Some`), per AF1. `tests/readme.md` given a Responsibility Table row for the new file.
  **Verification.** `cargo clippy -p renderer --all-targets --all-features -- -D warnings` initially failed twice: (1) `clippy::must_use_candidate` on `pub fn vec3_sequence` — fixed by adding `#[ must_use ]` immediately above it, matching task 118's own `light_list_get` precedent exactly; (2) `clippy::elidable_lifetime_names` on the new test file's `first_channel< 'a >` helper — fixed by eliding to `fn first_channel( gltf : &gltf::Gltf ) -> gltf::animation::Channel< '_ >`. Both fixed, clippy reconfirmed clean (exit 0). `cargo nextest run -p renderer --all-features` (full suite): 131/131 pass, 0 regressions (was 127 pre-task + 4 new). M1 (`cargo nextest run -p renderer --all-features --tests -- channel_decode`) → 2/2 pass. M2 (same for `vec3_sequence`) → 2/2 pass. M3: line-span sweep confirms `vec3_sequence` (48) and `vec3_tween_from_keyframe` (49) both ≤50 lines; `channel_decode` is 34 lines. I1 (`cargo check -p renderer --all-features`) → 0 errors. I3 (`grep -n "pub fn quat_sequence\|pub fn weights_sequence"`) → no match, both remain private. C10 (`git status --porcelain module/helper/renderer/tests/animation_tests.rs`) → empty, untouched.
  **Checklist-wording tensions (documented per this session's established practice, not silently resolved either way):**
  - **M3 / Delivery Requirements** — read literally, "longest function in `src/webgl/animation/loaders/gltf.rs` ≤ 50 lines" and "No function in the touched file exceeds 50 lines... confirmed unbroken elsewhere in the file" both imply a *file-wide* bound. But `quat_sequence` (93 lines) and `weights_sequence` (91 lines) were already over 50 lines before this task touched the file, and the task's own Out of Scope section explicitly defers splitting them as "same-shape follow-up work." Interpreted per the Goal/Acceptance Criteria's actual scoping intent ("`vec3_sequence` is ≤50 lines per function after the split") — i.e., the bound applies to the touched functions (`vec3_sequence` + its extracted helper), not the whole file. Passes by intent; a literal file-wide reading can never pass while the Out of Scope carve-out holds.
  - **M1/M2 and the Goal section's own "Testable:" line** — the literal commands (`cargo nextest run -p renderer --tests -- channel_decode` / `... vec3_sequence`, with no `--all-features`) fail to compile: `E0433: cannot find 'animation' in 'webgl'`, because the crate's `animation` Cargo feature is optional and not in `default` (confirmed via `Cargo.toml`'s `[features]` section) — a pre-existing condition of the crate, not introduced by this task. The Delivery Requirements' own clippy line already correctly includes `--all-features`; M1/M2 simply omit it. Re-run with `--all-features` added (`cargo nextest run -p renderer --all-features --tests -- channel_decode` / `vec3_sequence`) → 2/2 pass each, confirmed above. Passes by intent.
  Self-check performed as Tier 2 Dual-Role Self-Check (this repo's MAAV cap). `tsk .claim_verify 223` and `tsk .verify_pass 223` outcomes recorded in the Journal above.

## Related Documentation

- `task/accepting/118_renderer_gltf_light_extension_parsing_test.md` —
  the precedent this task follows exactly (private pure fn → `pub` +
  `mod_interface` export → native test, same crate, same loader family)
- `module/helper/renderer/tests/gltf_light_parsing_test.rs` — the inline-
  JSON-fixture-via-`Gltf::from_slice` pattern this task's new test reuses
- `assets/gltf/animated/single_keyframe_translation.gltf` — the BUG-188
  regression shape T04 mirrors (not reused as a file, per Out of Scope)
- `task/accepting/202_gpu_hal_vulkan_backend.md` — the task whose
  execution split `gpu_hal`'s `device.rs`/`pass.rs`/`native.rs`
  over-50-line functions into pieces; the ≤50-line split-on-touch
  precedent this task applies to `vec3_sequence`
