# BUG-155: `shader_chunks_render`'s "shared unknown chunk text" test asserts the wrong (unprefixed) string

- **Severity:** Low (test-only correctness defect -- the actual CLI behavior was always correct,
  confirmed by the sibling `shader_chunks_preview` test already passing against the identical
  error value; this only broke the `verb/test` full-suite gate, never a real user)
- **state:** Completed
- **Affects:** `shader_chunks_render`'s own test suite (`render_cli_test.rs`) only -- no runtime
  behavior for any binary is affected
- **Component:** `module/shader/shader_chunks_render` (tests/, docs/) + `module/shader/shader_chunks_preview` (docs/)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None -- independent of BUG-103/108/115/116/117 (same crate family, different
  code paths and root causes).

## Symptom

```
thread 'unknown_name_is_rejected_with_the_shared_unknown_chunk_text' panicked at
module/shader/shader_chunks_render/tests/render_cli_test.rs:74:3:
assertion `left == right` failed
  left: "unknown chunk: `bogus_chunk` (see `shader_chunks list` for valid names)"
 right: "unknown chunk: `bogus_chunk` (see `list` for valid names)"
```

`left` is the real return value of `render_to_png(...)`'s error `Display`; `right` is the test's
own hardcoded expectation -- the expectation was wrong, not the behavior.

## Impact

**Who is affected:** Nobody at runtime -- `shader_chunks_render`'s actual CLI error text was
already correct and already covered by a passing test (`shader_chunks_preview`'s own
`unknown_name_is_rejected_with_the_shared_unknown_chunk_text`, asserting the identical, correctly
prefixed string against the same shared `PreviewCliError::UnknownChunk` value). The only real
impact is CI-facing: `cargo test`/`verb/test` fails workspace-wide, blocking any task or release
gate with a "zero test failures" invariant, e.g. task 117's own `## Verification` § I1.

**What breaks:** `render_to_png` builds its target via `shader_chunks_preview::bundle_prepare`,
which returns `PreviewCliError::UnknownChunk(name.clone())` directly when a chunk name isn't
found (`shader_chunks_preview/src/lib.rs:122`) -- `render`'s own `RenderCliError::Preview(err)`
variant then delegates `Display` to that inner error verbatim (`render/src/lib.rs`'s own doc
comment: "see `shader_chunks_preview::PreviewCliError`, reused verbatim"). That inner error's
`Display` impl hardcodes `"...(see \`shader_chunks list\` for valid names)"`
(`shader_chunks_preview/src/lib.rs:59`) -- deliberately fully-qualified, because neither the
`shader_chunks_render` nor `shader_chunks_preview` standalone binary defines a local `list`
command of its own (each binary's own `commands()` returns only its one command --
`[cmd_render]` / `[cmd_preview]` respectively); only the `shader_chunks`/`sch` aggregator and the
`shader_chunks_query` standalone binary have a `list` command a user could actually run bare.
`render_cli_test.rs`'s own hardcoded expectation, however, asserted the bare, unprefixed form.

**Magnitude:** Blocks `verb/test` workspace-wide (1 of 1827 tests), discovered while completing
an unrelated task (117) whose own Delivery Requirement depends on a clean full-suite run.

**Entity Scope:** None -- a code-level test defect, not an operational-entity concern.

## How Discovered

Task 117 (removing `tiles_tools`'s unused `animation` dependency) required `verb/test` to pass
with zero failures per its own `## Verification` § I1. The full-workspace run
(`-0027_longrun.log`) reported exactly one failure, in an entirely unrelated crate
(`shader_chunks_render`, no dependency relationship to `tiles_tools`). Re-ran the single test in
isolation (`-0028_longrun.log`) to confirm the failure reproduces reliably (0.00s, not a
parallel-execution/ordering artifact) before investigating root cause.

## Minimum Reproducible Example

```bash
cd module/shader/shader_chunks_render && cargo test --test render_cli_test unknown_name_is_rejected_with_the_shared_unknown_chunk_text
```

**Expected** (post-fix):
```
test unknown_name_is_rejected_with_the_shared_unknown_chunk_text ... ok
```

**Actual** (pre-fix -- captured in `-0028_longrun.log`):
```
assertion `left == right` failed
  left: "unknown chunk: `bogus_chunk` (see `shader_chunks list` for valid names)"
 right: "unknown chunk: `bogus_chunk` (see `list` for valid names)"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 18 filtered out
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/shader/shader_chunks_render && cargo test --test render_cli_test unknown_name_is_rejected_with_the_shared_unknown_chunk_text
# ok = fixed; assertion left/right mismatch = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `render_cli_test.rs`'s expected string is stale -- missing the `shader_chunks ` prefix the actual, correct, shared error text carries. | ✅ Root Cause | The sibling `shader_chunks_preview` test with the exact same name already asserts the prefixed text and passes; `render_to_png` returns that same error value verbatim by construction. | E3, E4 |
| H2 | The CLI's actual error text is wrong and should drop the prefix -- i.e. fix the source, not the test. | ❌ Rejected | Neither `shader_chunks_render` nor `shader_chunks_preview`'s standalone binary defines a local `list` command (`commands()` returns only their own one command) -- bare `list` is not an independently runnable instruction from either binary, so the fully-qualified `shader_chunks list` is the only form that's always correct regardless of which binary the error surfaces from. | E5 |
| H3 | This is a fresh regression introduced by the `f3fde26a` "split shader_chunks into specialized CLI crates" refactor, not a long-standing latent bug. | Plausible, not load-bearing for the fix | `shader_chunks_render` is a newly-specialized crate from that refactor; the fix is identical either way (correct the stale literal), so this wasn't pursued further. | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `-0027_longrun.log` (full-workspace `verb/test`) | Exactly 1 of 1827 tests failed, isolated to `shader_chunks_render::render_cli_test`, unrelated to task 117's own `tiles_tools` change. | H3 |
| E2 | `-0028_longrun.log` (isolated single-test re-run) | Reproduces reliably in 0.00s -- rules out parallel-execution/ordering flakiness as the cause. | H1 ✅ |
| E3 | `module/shader/shader_chunks_preview/tests/preview_cli_test.rs:28` | Sibling test, identical name (`unknown_name_is_rejected_with_the_shared_unknown_chunk_text`), asserts `"...see \`shader_chunks list\`..."` against the same `PreviewCliError::UnknownChunk` value -- currently passing. | H1 ✅ |
| E4 | `module/shader/shader_chunks_preview/src/lib.rs:59,115-122` + `module/shader/shader_chunks_render/src/lib.rs`'s own doc comment | `bundle_prepare` returns `PreviewCliError::UnknownChunk(name.clone())` directly (line 122); `PreviewCliError`'s `Display` hardcodes the prefixed text (line 59); `render`'s `RenderCliError::Preview` reuses it "verbatim" per its own doc comment -- confirms `render_to_png`'s actual output must match E3's already-correct text exactly. | H1 ✅ |
| E5 | `module/shader/shader_chunks_render/src/lib.rs`'s `commands()` (`vec![ cmd_render( binary ) ]`) and `module/shader/shader_chunks_preview/src/lib.rs`'s `commands()` (`vec![ cmd_preview( binary ) ]`) | Neither standalone binary has a local `list` command -- confirms bare `list` guidance would be unrunnable from either binary, so the prefixed form is the deliberately correct one. | H2 ❌ |

## Root Cause

```
render_cli_test.rs (pre-fix)
  assert_eq!( err.to_string(), "unknown chunk: `bogus_chunk` (see `list` for valid names)" );
                                                              ^^^^^^ missing "shader_chunks " prefix
                                                              the real Display impl carries
```

The test's hardcoded expected string dropped the `shader_chunks ` prefix that
`PreviewCliError::UnknownChunk`'s `Display` impl actually produces -- the same error value the
sibling `shader_chunks_preview` test already correctly asserts against.

## Why Not Caught

No mechanism links the two "shared" tests' hardcoded literals -- their identical names document
an intent (assert the same text) but nothing enforces it. `render_cli_test.rs`'s copy of the
expected string went stale (or was authored independently and never matched) without any
compiler or test-runner signal until a full-workspace `verb/test` run actually exercised both.

## Fix Location

`module/shader/shader_chunks_render/tests/render_cli_test.rs` (test-only; no source-code change
-- the actual behavior was already correct):

```rust
// before
assert_eq!( err.to_string(), "unknown chunk: `bogus_chunk` (see `list` for valid names)" );

// after
assert_eq!( err.to_string(), "unknown chunk: `bogus_chunk` (see `shader_chunks list` for valid names)" );
```

Plus 2 CLI doc examples showing the same stale, unprefixed text for the identical error path,
corrected to match:
- `module/shader/shader_chunks_render/docs/cli/command/01_render.md`
- `module/shader/shader_chunks_preview/docs/cli/command/01_preview.md`

## Prevention

Corrected the assertion to match the already-correct, already-tested value -- no new test
needed, since a passing test now exists on both sides of the "shared" text (this test and its
`shader_chunks_preview` sibling), and both now genuinely agree.

## Pitfall

A test name that says "shared" with a sibling test in another crate is a documentation
convention, not an enforced invariant -- two independently hardcoded string literals can still
silently drift apart when one crate's error text is authored (or a wrapping crate is introduced
that reuses it "verbatim") without updating the other's copy.

## Generalized Version

**Broken assumption:** "if two tests across sibling crates share a name, their hardcoded expected
values must already agree." False -- nothing links them mechanically; only a full-workspace test
run surfaces the drift, and only after the fact.

**Confirmed general rule:** when an error type is deliberately reused "verbatim" across crates
(per an explicit doc-comment contract, as `shader_chunks_render` states for
`shader_chunks_preview::PreviewCliError`), any test asserting that error's `Display` text is
implicitly asserting a cross-crate invariant, not a local one -- worth a comment pointing at the
canonical source/sibling test, even without extracting a shared constant.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered while completing task 117 (`verb/test` full-suite invariant); confirmed unrelated to task 117's own change and reliably reproducible in isolation before filing. |
| 2026-08-16 | fixed | Corrected `render_cli_test.rs`'s expected string to the actual, already-correctly-tested `shader_chunks list`-prefixed text; corrected the same stale, unprefixed text in 2 CLI doc examples (`shader_chunks_render`'s and `shader_chunks_preview`'s own `01_render.md`/`01_preview.md`). |
| 2026-08-16 | verified | Isolated test re-run confirmed `ok`; full-workspace `verb/test` re-run confirmed 0 failures. |
| 2026-08-16 | completed | Filed, fixed, and verified in one continuous session (compressed lifecycle) -- no interruption, so closed directly without a separate later-session confirmation pass. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass captured the real pre-fix assertion mismatch from a real isolated run (`-0028_longrun.log`); adversarial pass specifically checked whether the fix could be "fix the source instead" by reading `commands()` for both standalone binaries to rule out H2 with direct evidence, not assumption. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-103/108/115/116/117 (same crate family, different code paths); `shader_chunks_preview`'s sibling test cited directly as the corroborating source, not assumed. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause traced to the exact literal and confirmed against the exact source line (`shader_chunks_preview/src/lib.rs:59`) producing the real value. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Checked both `shader_chunks_render`'s and `shader_chunks_preview`'s own doc examples for the same staleness (found and fixed both) rather than fixing only the failing test in isolation; did not touch `shader_chunks_compose`/`shader_chunks_params`/`shader_chunks_query`'s own independently-defined `UnknownChunk` variants -- those have no failing test and are a separate, untested design question (do they also warrant the aggregator-qualified form?), out of this bug's confirmed scope. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Touches 2 crates' docs (`shader_chunks_render`, `shader_chunks_preview`) plus the render crate's test -- precedented for a bug whose blast radius genuinely spans crates from one root cause (BUG-053, BUG-080, BUG-109 all touched multiple crates similarly). | — |
| D7 | Crate Locality | 🟢 | 🟢 | Each fix is a single-line literal correction; no signature/behavior change anywhere. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; corrects existing assertions/docs to match already-correct, already-established behavior. | — |

**Reproduced:** YES -- `-0028_longrun.log` captured the exact pre-fix assertion mismatch from a
real isolated run; applying the fix and re-running returned the test to passing, and the
full-workspace `verb/test` re-run confirmed 0 failures, 2026-08-16.

## Refs: src/

None -- the actual application behavior was already correct; no source-code file changed.

## Refs: tests/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_render/tests/render_cli_test.rs` | `unknown_name_is_rejected_with_the_shared_unknown_chunk_text`: corrected the expected string to include the `shader_chunks ` prefix. `test_kind: bug_reproducer(BUG-155)` + 5-section doc comment added. |

## Refs: docs/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_render/docs/cli/command/01_render.md` | Corrected the `shader_chunks render bogus_chunk` example's error text to include the `shader_chunks ` prefix. |
| `module/shader/shader_chunks_preview/docs/cli/command/01_preview.md` | Corrected the `shader_chunks preview bogus_chunk` example's error text to include the `shader_chunks ` prefix. |
