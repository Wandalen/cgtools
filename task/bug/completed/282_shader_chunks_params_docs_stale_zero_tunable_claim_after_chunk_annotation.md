# BUG-282: `shader_chunks_params`/`shader_chunks_params_core` docs claimed bundled chunks (`fbm3`, `value_noise` among them) declare zero `//@ param:` tunables — 46 of 50 now do

- **Severity:** Low (no runtime defect — a documentation-only staleness gap; the underlying
  `discover`/`range_infer`/`chunk_discover`/`tunables_of_chunk` code was already, and remains,
  fully correct)
- **state:** Completed
- **Affects:** `module/shader/shader_chunks_params/docs/cli/command/01_tunables.md` (Description,
  Examples, Notes), `module/shader/shader_chunks_params/readme.md` (Usage section), and
  `module/shader/shader_chunks_params_core/readme.md` (`## Chunk annotations` section) —
  documentation only, no source code path affected
- **Component:** `module/shader/shader_chunks_params`, `module/shader/shader_chunks_params_core`
  (docs/readme only — source in both crates was already correct and needed no change)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox tsk actor-guard blocks .acceptance_pass in this environment)

## Symptom

Three documentation files in the assigned crate pair claimed real bundled `shader/*.wgsl` chunks
declare zero `//@ param:` tunable-parameter lines — contradicted by the actual, current, correctly
parsed WGSL source:

1. `shader_chunks_params/docs/cli/command/01_tunables.md` — Description said a chunk declaring no
   tunables was "true for every bundled chunk today"; the `Examples` block showed
   `shader_chunks tunables fbm3` rendering `# chunk 'fbm3' declares no tunable parameters`; Notes
   repeated "Every bundled chunk today declares zero `//@ param:` lines".
2. `shader_chunks_params/readme.md` — claimed "none of the 4 bundled chunks (`hash21`,
   `value_noise`, `fbm3`, `fullscreen_triangle`) do today".
3. `shader_chunks_params_core/readme.md`'s `## Chunk annotations` section named the same 4 chunks
   — `hash21`, `value_noise`, `fbm3`, `fullscreen_triangle` — as the ones that "still carry none".

In reality, `shader/fbm3/fbm3.wgsl` declares 2 tunables (`lacunarity`, `gain`) and
`shader/value_noise/value_noise.wgsl` declares 1 (`seed`), both with explicit declared ranges, and
`shader_chunks_params::tunables("fbm3")`/`tunables("value_noise")` correctly render populated
tables — not the empty message the docs described. 46 of the repo's 50 bundled `shader/*.wgsl`
chunks now declare one or more `//@ param:` lines; only `hash21`, `hash22`, `srgb`, and
`fullscreen_triangle` remain param-less.

## Impact

**Who is affected:** anyone reading these 3 files to learn the `tunables` command's current
behavior, or looking for a real bundled chunk to exercise the populated-table path against.

**What breaks:** nothing at runtime — a pure documentation defect; `discover`, `range_infer`,
`chunk_discover`, and `tunables_of_chunk` were already correct and needed no change (confirmed by
a clean 28/28-test, zero-clippy-warning baseline run before any edit). The risk was to a reader
trusting the doc's copy-pasteable `fbm3` example (whose commented "expected" output was false) or
copy the false premise into new work, or a future contributor "fixing" the code to match the stale
"every chunk has zero tunables" assumption.

**Entity Scope:** None — a documentation-level defect, not an operational-entity concern.

## How Discovered

During this session's assigned bug-hunt of `shader_chunks_params`/`shader_chunks_params_core`
(hunting for range-inference/type-confusion/whitespace-parsing/zero-tunables-fallback logic
defects per the session's domain hints). An exhaustive line-by-line audit of `discover`,
`range_infer`, `range_by_name_infer`, `range_by_type_infer`, `param_line_parse`, and
`range_clause_parse` against `docs/algorithm/001_range_inference_heuristic.md` and
`docs/api/001_tunable_parameter_taxonomy.md` found the implementation a faithful, correct match of
its own documented contract in every case checked (boundary values, declared-vs-inferred
precedence, all 5 kinds/14 types, malformed-input panics) — confirmed objectively by a
`longrun`-detached `cargo test`/`cargo clippy` baseline run: 28/28 tests passing, zero clippy
warnings, before any edit. With no source-level defect found, cross-referenced the crate's own
docs against the real bundled `shader/*.wgsl` chunk content (`grep -h "^//@ param:" shader/*/*.wgsl`)
and found the 3-file contradiction above; independently confirmed by actually running
`cargo run -p shader_chunks_params -- tunables fbm3` / `tunables value_noise` /  `tunables hash22`
/ `tunables srgb` against the real bundled registry rather than trusting the docs' claim at face
value.

## Minimum Reproducible Example

**Verify Command** (≤3 lines, standalone):
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -c "^//@ param:" shader/fbm3/fbm3.wgsl shader/value_noise/value_noise.wgsl
cargo run -p shader_chunks_params --quiet -- tunables fbm3
```
**Expected** (matches the fixed docs, and was already true of the code pre-fix):
`shader/fbm3/fbm3.wgsl:2`, `shader/value_noise/value_noise.wgsl:1`, and `tunables fbm3` prints a
populated 2-row table:
```
name        kind      type  range  source
----------  --------  ----  -----  --------
lacunarity  Argument  F32   1..3   Declared
gain        Argument  F32   0..1   Declared
```
**Actual** (what the pre-fix docs claimed, contradicted by the above): `01_tunables.md` showed
`shader_chunks tunables fbm3` rendering `# chunk 'fbm3' declares no tunable parameters` — false,
and had been false since `fbm3.wgsl` was annotated with its 2 `//@ param:` lines.

## Root Cause

`fbm3.wgsl`/`value_noise.wgsl` (and 44 other bundled chunks — 46 of 50 total) were annotated with
real `//@ param:` lines by later chunk-preview work, turning each `_preview` wrapper's former
hardcoded literals into real `argument`-kind tunables driving a live browser slider (see
`shader_chunks_preview_core`'s own readme, whose "Chunk annotations" section already correctly
says "most... carry `//@ param:` lines today"). `shader_chunks_params`'s and
`shader_chunks_params_core`'s own docs/readme, however, were never revisited after that
annotation work landed — their "which bundled chunks currently have zero params" examples and
claims had been perfectly accurate when originally written (0 of 50 bundled chunks carried any
`//@ param:` line at the time), and nothing forced a re-check as the underlying `shader/*.wgsl`
collection kept evolving independently of this crate pair's own commit history.

## Why Not Caught

No test asserts documentation prose against real bundled-chunk content — by explicit design, this
crate's own Rust tests deliberately avoid depending on any particular bundled chunk's own
annotation state (`tunables_test.rs` uses a self-contained `LOCAL_GLOW` fixture for the
populated-table case, and `hash21` — which happens to remain genuinely param-less — for the empty
case), so nothing in the test suite could regress when the *docs'* specific worked example (`fbm3`)
drifted out of sync with the evolving `shader/*.wgsl` collection. This is the same failure class as
BUG-249 (`docs/cli/param/21_width.md`): prose stating a general, system-wide "every/no bundled
chunk currently has property P" fact silently rots as unrelated work changes the underlying data,
and no grep-for-library-symbol-names sweep would catch it either, since the stale claim is phrased
in end-user vocabulary ("declares zero tunable parameters"), not internal API terms.

## Fix Applied (2026-08-17)

Corrected all 3 files to state the current, verified fact (46 of 50 bundled chunks now declare
`//@ param:` lines; only `hash21`, `hash22`, `srgb`, and `fullscreen_triangle` remain param-less)
and replaced `01_tunables.md`'s `fbm3` example with its real, verified `tunables fbm3` output (a
2-row table: `lacunarity`, `gain`) instead of a fabricated empty-message comment — this also
upgrades the example from a previously-unreachable hypothetical (when every bundled chunk was
param-less, `01_tunables.md` could only demonstrate the empty-message path via subprocess) to a
genuinely subprocess-reachable worked example of the populated-table path. No source code
changed — `discover`, `range_infer`, `chunk_discover`, and `tunables_of_chunk` were already
correct; confirmed directly by running the actual CLI against the real registry (see Verification).

## Verification

`longrun`-detached, from repo root, both before and after the doc edits (docs-only change, so no
behavior could regress, but the full gate was still run per this session's standing verification
requirement):
- **Baseline (pre-edit):** `cargo test -p shader_chunks_params -p shader_chunks_params_core
  --all-features`: 28/28 passed (3 `tunables_test.rs` + 12 `discovery_test.rs` + 13
  `range_inference_test.rs`), 0 failed, plus 0/0 doc tests (none declared) — clean.
  `cargo clippy -p shader_chunks_params -p shader_chunks_params_core --all-targets --all-features
  -- -D warnings`: clean, exit 0.
- **Real-data proof (RED, i.e. docs vs. reality contradiction confirmed before editing):**
  `grep -n "declares zero" .../01_tunables.md`, `grep -n "none of the 4 bundled" .../readme.md`,
  `grep -n "hash21.*value_noise.*fbm3" .../shader_chunks_params_core/readme.md` — all matched
  (stale claims present); `cargo run -p shader_chunks_params -- tunables fbm3` /
  `tunables value_noise` — both printed real, populated tables, directly contradicting the docs'
  claims.
- **Post-fix (GREEN):** re-ran the same 3 greps for the stale claim text — 0 matches each (all
  removed); `grep -c` for the corrected `fbm3` example's exact table row — 1 match (present and
  correct). Re-ran `cargo test -p shader_chunks_params -p shader_chunks_params_core --all-features`
  and `cargo clippy -p shader_chunks_params -p shader_chunks_params_core --all-targets
  --all-features -- -D warnings` after the doc edits: still 28/28 passed, still clippy-clean (doc
  files are outside any code path either tool inspects, so this reconfirms no regression rather
  than proving the fix itself — the grep checks above are this bug's actual regression proof).

## Generalized Version

**Broken assumption:** "A crate's own docs describing a system-wide 'every/no bundled chunk
currently has property P' fact stay accurate indefinitely, since the crate's own commit history is
the only thing that could invalidate them." False when P is a fact about a companion data
collection (`shader/*.wgsl`) that evolves independently, under different tasks/crates' own commits,
outside this crate's own version-control boundary — and doubly so when, by deliberate test-design
choice, the crate's own regression suite is built specifically to *not* depend on that companion
collection's current state (for good reason: test isolation from an evolving fixture set), which
means nothing in CI can ever catch this specific class of drift. Any doc claim of the shape "every
X in collection Y currently has/lacks property P," where Y is not owned by the same crate/commit
history as the doc, needs a periodic real-data re-check (e.g. a `grep`/count sweep against the
live collection) — it cannot be caught by the crate's own test suite by construction.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during this session's assigned bug-hunt of `shader_chunks_params`/`shader_chunks_params_core`, after an exhaustive source-level audit (against `docs/algorithm/001_range_inference_heuristic.md`, `docs/api/001_tunable_parameter_taxonomy.md`, and a clean 28/28-test + zero-clippy baseline) found zero logic defects in the actual Rust source. Cross-referencing the crate's own docs/readme against the real bundled `shader/*.wgsl` content surfaced a 3-file stale-documentation contradiction: `fbm3`/`value_noise` were named as zero-tunable-parameter chunks in `shader_chunks_params/docs/cli/command/01_tunables.md`, `shader_chunks_params/readme.md`, and `shader_chunks_params_core/readme.md`, but both chunks were annotated with real `//@ param:` lines by later, unrelated chunk-preview work (46 of 50 bundled chunks now declare params; only `hash21`/`hash22`/`srgb`/`fullscreen_triangle` remain param-less) — confirmed directly via `cargo run -p shader_chunks_params -- tunables fbm3`/`tunables value_noise` against the real registry. Same failure class as BUG-249. Fixed by correcting all 3 files' claims/examples to the current, verified fact, including replacing `01_tunables.md`'s fabricated `fbm3` empty-message example with its real, verified 2-row populated-table output. No source code changed. Verified via grep-based before/after proof of the stale-claim text plus a full `cargo test`/`cargo clippy` re-run (28/28 passed, clean). Tier 2 Dual-Role Self-Check (per this session's standing MAAV Tier Cap). |
