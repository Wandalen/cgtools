# BUG-205: `value_chunk_harness_and_parameters`'s candidate selection prefers a chunk's own primitive over its dedicated `_preview` wrapper

- **Severity:** Medium (wrong preview rendered -- not a crash/panic, but a silently misleading visual: `sch preview <name>` shows the wrong export's output with no error, which is exactly the tool someone would trust to validate a chunk during authoring)
- **state:** Completed
- **Affects:** `shader_chunks_preview_core::bundle_build`'s value-chunk branch, for any bundled chunk declaring both a raw primitive export and a dedicated `NAME_preview` wrapper export that share at least one trailing argument name -- concretely `domain_warp` (visible: rendered `domain_warp`'s raw `vec2f` warp displacement as a blue-padded 2-channel swatch instead of `domain_warp_preview`'s intended `f32` grayscale noise value) plus `d2_sdf_circle`/`d2_sdf_ring` (latent: their `_preview` wrappers are trivial same-signature passthroughs, so the wrong selection produced byte-identical output either way -- confirmed directly, see Evidence E4)
- **Component:** `module/shader/shader_chunks_preview_core` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** None -- first defect filed against this crate's candidate-selection logic. Discovered as the sole confirmed finding of task #122's bounded review of the 8 remaining `shader_chunks_*` crates; that review's dispatched agent had already written and left uncommitted a proving regression test (`value_chunk_prefers_dedicated_preview_wrapper_over_same_named_primitive_sharing_an_argument_name`), which this fix makes pass.

## Symptom

```rust
// pre-fix -- shader_chunks_preview_core/src/lib.rs, value_chunk_harness_and_parameters
let ( value_fn, kind, extra_args ) = candidates.iter()
.filter( | ( _, _, extra_args ) | is_viable( extra_args ) )
.find( | ( found, _, _ ) | *found == name )                                          // always matches the primitive
.or_else( || candidates.iter().find( | ( _, _, extra_args ) | is_viable( extra_args ) ) )
.cloned()
.ok_or_else( || PreviewError::Unpreviewable { /* ... */ } )?;
```

`domain_warp.wgsl` exports both `fn domain_warp(p: vec2f, strength: f32) -> vec2f` (the real API,
called by dependents) and `fn domain_warp_preview(p: vec2f, strength: f32) -> f32` (a dedicated
preview wrapper). Both share the trailing argument name `strength`, and the chunk's single
`//@ param: strength argument f32 range(...)` line makes both viable. The tie-break above only
ever checks "is this candidate named like the chunk itself" -- which the primitive always
satisfies by construction and the `_preview`-suffixed wrapper never does -- so the primitive won
regardless of the dedicated wrapper's existence.

## Impact

**Who is affected:** Anyone running `sch preview domain_warp` (or `d2_sdf_circle`/`d2_sdf_ring`,
latently) to visually validate a chunk during authoring, plus the wasm browser runner
(`shader_chunks_preview_web`), which renders whatever `bundle_build` hands it with no independent
export-choice logic of its own.

**What breaks:** `domain_warp`'s preview rendered the raw `vec2f` displacement field as a
blue-padded 2-channel swatch (`Vec2` write mode: `vec3f(value, 0.5)`) instead of
`domain_warp_preview`'s intended scalar noise value (`F32` write mode: `vec3f(value)` grayscale)
-- a visibly different image, silently wrong with no error surfaced anywhere in the pipeline.

**Magnitude:** Single candidate-selection chokepoint inside one function, but every chunk that
follows the "primitive + dedicated `_preview` wrapper sharing an argument name" authoring pattern
hits it -- confirmed 3 of 50 bundled chunks match this shape today (`domain_warp`,
`d2_sdf_circle`, `d2_sdf_ring`); any future chunk added the same way would too.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Task #122 dispatched a bounded, read-only `Explore` agent to review the 8 `shader_chunks_*`
crates not yet covered by this session's earlier passes. It found this candidate-selection defect
and wrote a proving regression test directly against a real bundled chunk (`domain_warp`), left
uncommitted in the working tree (`git status` showed it as a modified, uncommitted file at the
start of this fix). Confirmed genuinely mine (not a concurrent-actor artifact) by reading the
diff directly: the test's own doc comment already states the exact root cause this report
independently re-derives from the source.

## Minimum Reproducible Example

```rust
// shader_chunks_preview_core/tests/preview_bundle_test.rs -- pre-fix, this test fails
let target = shader_chunks_core::chunk_get( "domain_warp" ).unwrap();
let bundle = bundle_build( target.wgsl ).unwrap();
assert_eq!( bundle.target, "domain_warp" );
// pre-fix: bundle.wgsl contains `let value = domain_warp( p, params.strength );` and
// `let color = vec3f( value, 0.5 );` -- the primitive, Vec2-written -- not the wrapper.
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/shader/shader_chunks_preview_core && cargo nextest run -E 'test(value_chunk_prefers_dedicated_preview_wrapper_over_same_named_primitive_sharing_an_argument_name)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | Candidate selection's same-name tie-break can never reach a `NAME_preview` wrapper, because the tie-break only checks equality against the chunk's own `name`, which the primitive always satisfies and the wrapper never does. | ✅ Root Cause | Confirmed by direct reading of `value_chunk_harness_and_parameters`'s pre-fix selection chain, corroborated by the pre-existing failing test's own observed output. | E1, E2, E3 |
| H2 | The same defect class silently affects other bundled chunks beyond `domain_warp`, not just theoretically. | ✅ Confirmed (2 more, both latent) | `d2_sdf_circle`/`d2_sdf_ring` each declare a primitive + `_preview` wrapper sharing a trailing `radius` argument name, structurally identical to `domain_warp`'s case -- but each wrapper is a trivial same-signature passthrough (`return d2_sdf_circle(p, radius);`), so the wrong selection produced byte-identical rendered output either way, making the defect invisible there. | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `shader/domain_warp/domain_warp.wgsl` | Two exports, `domain_warp(p: vec2f, strength: f32) -> vec2f` and `domain_warp_preview(p: vec2f, strength: f32) -> f32`, plus a single `//@ param: strength argument f32 range(...)` line with no way to scope it to one export. | H1 ✅ |
| E2 | `shader_chunks_preview_core/src/lib.rs`, pre-fix `value_chunk_harness_and_parameters` | The tie-break `.find(\|(found,_,_)\| *found == name)` matches only against the chunk's own manifest `name` ("domain_warp"), which the primitive shares by construction; the wrapper's name ("domain_warp_preview") can never equal it. | H1 ✅ |
| E3 | `value_chunk_prefers_dedicated_preview_wrapper_over_same_named_primitive_sharing_an_argument_name`'s pre-fix run | Failing assertion confirms the composed harness called `domain_warp(p, params.strength)` and wrote `vec3f(value, 0.5)` (Vec2 mode) -- the primitive, not the wrapper. | H1 ✅ |
| E4 | `shader/d2_sdf_circle/d2_sdf_circle.wgsl` (read directly this session) | `d2_sdf_circle_preview(p: vec2f, radius: f32) -> f32 { return d2_sdf_circle(p, radius); }` -- same signature shape, same `radius` argument name as the primitive it wraps, and a single `//@ param: radius ...` line. Selecting either candidate calls the same function with the same argument, producing byte-identical output -- confirms the defect is present but visually silent here, not absent. | H2 ✅ |

## Root Cause

```rust
// before -- shader_chunks_preview_core/src/lib.rs
let ( value_fn, kind, extra_args ) = candidates.iter()
.filter( | ( _, _, extra_args ) | is_viable( extra_args ) )
.find( | ( found, _, _ ) | *found == name )
.or_else( || candidates.iter().find( | ( _, _, extra_args ) | is_viable( extra_args ) ) )
.cloned()
.ok_or_else( || PreviewError::Unpreviewable { /* ... */ } )?;
```

A `//@ param:` declaration is scoped to an argument *name*, not to one specific export -- when a
primitive and its dedicated `NAME_preview` wrapper share a trailing argument name (a natural
pattern, since both conceptually take the same parameter), both pass the `is_viable` filter. The
tie-break that follows only ever checks "is this candidate named exactly like the chunk itself",
which the primitive satisfies by construction and the wrapper structurally never can -- so a
dedicated preview wrapper, when one exists specifically to be the correct preview target, was
unreachable by this selection logic no matter what.

## Why Not Caught

No existing test exercised a chunk with both a primitive and a dedicated `_preview` wrapper
sharing a trailing argument name before task #122's review wrote one directly against
`domain_warp`; the crate's other value-chunk tests use fixture chunks with only one viable
candidate, where the tie-break's blind spot never triggers.

## Fix Location

`module/shader/shader_chunks_preview_core/src/lib.rs`, `value_chunk_harness_and_parameters`:
inserted a new preference tier -- a viable candidate named `"{name}_preview"` -- checked *before*
the existing exact-name tie-break, which itself now only applies as the second fallback, ahead of
the original first-viable-in-file-order fallback as the third and last.

## Prevention

The regression test discovered during task #122's review, already written against a real bundled
chunk (`domain_warp`) rather than a synthetic fixture, is retained as-is (no new test needed --
this fix makes it pass). Additionally corrected the crate's own algorithm documentation
(`docs/algorithm/001_value_function_shape_detection.md`), whose Stage 0/Stage 1 tables and
Abstract predated both the trailing-`f32`-argument (`own_params`) feature and this fix's new
preference tier -- left accurate for the single-`vec2f`-argument shape only, silently wrong about
the multi-argument candidate-selection rule this bug lived in.

## Pitfall

A manifest declaration scoped by *name* (`//@ param: strength ...`) silently applies to every
export sharing that name, not just the one it was conceptually written for -- when two exports
share an argument name (a natural pattern for a primitive and its own preview wrapper), viability
alone cannot disambiguate them; the tie-break that runs after viability filtering is what actually
decides, and every one of its rules needs to be checked against the *wrapper* case explicitly, not
just the common "one obvious candidate" case most fixture tests exercise.

## Generalized Version

**Broken assumption:** "if a candidate is viable and its name matches an obvious rule, it must be
the right one."

**Confirmed general rule:** When a selection rule's tie-break checks only one specific name
pattern (here: "equals the chunk's own name"), any correct candidate that structurally can never
match that pattern (here: a `NAME_preview`-suffixed wrapper) is unreachable regardless of how
"viable" it is -- viability and preference are separate concerns, and a preference chain must
include an explicit tier for every intentionally-named candidate shape the authoring convention
actually produces, not just the most common one.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Surfaced as task #122's sole confirmed finding (Explore agent review of the 8 remaining `shader_chunks_*` crates); regression test already written and left uncommitted by that review; confirmed as a real defect via direct source reading and reserved as BUG-205 (fresh grep confirmed unused; `highest_id` was 204). |
| 2026-08-16 | fixed | Added a `"{name}_preview"`-named viable-candidate preference tier ahead of the existing exact-name tie-break in `value_chunk_harness_and_parameters`; corrected the crate's algorithm doc (`docs/algorithm/001_value_function_shape_detection.md`) to match both this fix and the pre-existing, independently-stale trailing-argument (`own_params`) feature. |
| 2026-08-16 | verified | `cargo nextest run -p shader_chunks_preview_core` (via `longrun`): 21/21 passed, including the target regression test by name. `cargo clippy -p shader_chunks_preview_core --all-targets --all-features -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass ran the pre-existing regression test post-fix and confirmed it passes by name in the full log (not just an aggregate "N passed" count). Adversarial pass specifically distrusted the "only `domain_warp` is affected" framing before writing Impact/Affects -- grepped all 50 chunks for `_preview(` exports and directly read `d2_sdf_circle`'s own wrapper source to confirm H2 (2 more chunks affected, latently) rather than accepting the earlier review's unverified claim at face value. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | First bug filed against this crate's candidate-selection logic — correctly identified as not a duplicate of any prior BUG-NNN; correctly attributed the pre-existing regression test to this session's own earlier task #122 dispatch (verified via diff content matching the review's documented finding) rather than misattributing it to the concurrent actor's "revert" pattern, which an inherited memory note had incorrectly done for this exact file. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct reading of the pre-fix selection chain and the manifest grammar's name-scoping (not argument-scoping) shown in `domain_warp.wgsl` itself; H2's blast-radius claim independently re-verified against `d2_sdf_circle.wgsl`'s actual current source rather than trusted from an earlier, unverified finding. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is one additional `.find()` tier plus one `format!`-built name in a single function; the crate's algorithm doc was also corrected, in scope because it directly documents the exact function and rule this fix changed, and was independently confirmed stale (not speculative doc churn). No changes to the separate, actively-in-progress "make every chunk previewable" plan's own remaining scope (per-chunk readme/preview.png work), which was investigated and confirmed to be the concurrent actor's own active territory, not touched. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Fix and doc correction both live entirely in `shader_chunks_preview_core`; no changes to any other `shader_chunks_*` crate or to any individual chunk's `.wgsl` source. | — |

**Reproduced:** YES -- pre-fix, `value_chunk_prefers_dedicated_preview_wrapper_over_same_named_primitive_sharing_an_argument_name`
failed (candidate selection chose `domain_warp` over `domain_warp_preview`); post-fix, the same
test passes with no other changes to the test file. 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_preview_core/src/lib.rs` | `value_chunk_harness_and_parameters`: added a `"{name}_preview"`-named viable-candidate preference tier before the existing exact-name tie-break (full `Fix(BUG-205)` comment block); updated the adjacent leading comment to describe all 3 tiers. |
| `module/shader/shader_chunks_preview_core/docs/algorithm/001_value_function_shape_detection.md` | Corrected Abstract, Stage 0's table (now reflects trailing-`f32`-argument support), and Stage 1's table (now reflects the `is_viable` filter and the new 3-tier preference order); added a Tests-table reference to the regression test covering this bug. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_preview_core/tests/preview_bundle_test.rs` | No new test added -- `value_chunk_prefers_dedicated_preview_wrapper_over_same_named_primitive_sharing_an_argument_name`, already written (uncommitted) by task #122's review agent, now passes unmodified. |
