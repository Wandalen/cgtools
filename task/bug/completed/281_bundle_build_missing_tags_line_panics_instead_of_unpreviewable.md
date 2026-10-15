# BUG-281: `shader_chunks_preview_core::bundle_build` panics instead of returning `Unpreviewable` when a value chunk is missing its `//@ tags:` line

- **Severity:** Medium (a real, reachable process panic — not a corrupted-data or
  security defect; bounded to hand-authored/in-progress chunks passed via the CLI's
  `preview file::<path>` mode, since every bundled `CHUNKS` entry already carries a
  `//@ tags:` line)
- **state:** Completed
- **Affects:** `shader_chunks_preview_core::bundle_build`'s upfront manifest-completeness
  guard (`module/shader/shader_chunks_preview_core/src/lib.rs`)
- **Component:** `module/shader/shader_chunks_preview_core` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor (same-sandbox tsk actor-guard blocks .acceptance_pass in this environment)

## Symptom

`bundle_build`'s own doc comment documents `PreviewError::Unpreviewable` as the outcome
for "missing manifest lines" in general:

```rust
/// # Errors
///
/// - [`PreviewError::Unpreviewable`] — missing manifest lines, no
///   previewable export, or a fragment chunk without `fs_main` /
///   without at least one `//@ param:` uniform.
```

In reality, `bundle_build`'s own upfront check only guards two of the manifest fields
its own downstream call graph actually depends on:

```rust
for required in [ "name", "depends_on" ]
{
  let prefix = format!( "//@ {required}:" );
  if !target_wgsl.lines().any( | line | line.starts_with( prefix.as_str() ) )
  {
    return Err( PreviewError::Unpreviewable { .. } );
  }
}
```

For a value chunk (any non-`fragment`-stage target), `value_chunk_harness_and_parameters`
unconditionally calls `shader_chunks_core::tags_parse` once a previewable export has been
chosen, to detect the `category:sdf` tag:

```rust
let is_sdf = tags_parse( target_wgsl ).iter().any( | &( group, tag ) | group == "category" && tag == "sdf" );
```

`tags_parse` panics via `shader_chunks_core`'s private `manifest_field` helper when no
`//@ tags:` line exists at all:

```rust
fn manifest_field<'a>( wgsl : &'a str, key : &str ) -> &'a str
{
  let prefix = format!( "//@ {key}:" );
  wgsl.lines()
  .find_map( | line | line.strip_prefix( prefix.as_str() ) )
  .unwrap_or_else( || panic!( "chunk missing required `//@ {key}:` header line:\n{wgsl}" ) )
  .trim()
}
```

A value-chunk-shaped target with a valid previewable export ( `name`, `depends_on`, and
`export` all present and well-formed ) but no `//@ tags:` line therefore crashes the whole
process with a Rust panic instead of returning the documented, gracefully-handleable
`PreviewError::Unpreviewable`.

## Impact

**Who is affected:** any caller of `bundle_build` — directly, or via
`shader_chunks_preview::bundle_prepare` / the `shader_chunks_preview preview
file::<path>` CLI command — on a value-chunk-shaped local `.wgsl` file that is missing
its `//@ tags:` manifest line. This is a real, ordinary authoring-workflow scenario:
`file::` mode's whole purpose (per this crate's own doc comment and
`file_target_prepares_the_same_bundle_as_the_bundled_name`'s test contract) is previewing
a chunk before it is bundled into the registry — exactly the point in a chunk's life
cycle where its manifest header is most likely to still be incomplete.

**What breaks:** `bundle_build` (and therefore the `shader_chunks_preview` binary
process running it) aborts with an unhandled panic and a raw Rust backtrace instead of
the clean `unknown chunk` / `is not previewable: ...`-style message every other rejection
path in this crate produces. No bundled `CHUNKS` registry entry currently lacks a
`//@ tags:` line ( confirmed: every chunk in `shader/*/*.wgsl` already declares one, since
`tags_parse` is called elsewhere across the wider `shader_chunks` CLI too ), so
`PreviewTarget::Name` mode never reaches this panic in practice — only `PreviewTarget::File`
mode on a still-incomplete local chunk does.

**Entity Scope:** None — a pure-function parsing/validation defect, not entity directory
instances.

## How Discovered

Systematic line-by-line review of every function in `shader_chunks_preview_core::lib.rs`
and `shader_chunks_preview::lib.rs`, per this session's assigned bug hunt of the
`shader_chunks_preview`/`shader_chunks_preview_core` crate pair. `bundle_build`'s own doc
comment explicitly promises `Unpreviewable` for "missing manifest lines" (plural, general)
as one of its three documented error causes; tracing every manifest-parser call reachable
from `bundle_build` against the upfront guard's own two-item field list (`name`,
`depends_on`) surfaced the gap — `tags_parse` is called in
`value_chunk_harness_and_parameters` with no equivalent upfront guard for `"tags"`, and
`shader_chunks_core::manifest_field` (which backs it) is confirmed, by reading its source
directly, to panic rather than return an `Option`/`Result` on a missing line — unlike
`stage_parse`/`exports_parse`, which use the non-panicking `manifest_field_opt`/
`manifest_field_all` variants and were confirmed safe by the same read.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p shader_chunks_preview_core --test preview_bundle_test value_chunk_missing_tags_line_is_rejected_not_panicked -- --nocapture
```
**Expected** (fixed): `test value_chunk_missing_tags_line_is_rejected_not_panicked ... ok`
— `bundle_build` returns `Err( PreviewError::Unpreviewable { .. } )`.

**Actual** (pre-fix, confirmed via the test-first RED/GREEN sequence below — no `git
stash` used, the file carried no other in-flight changes to protect):
```
thread 'value_chunk_missing_tags_line_is_rejected_not_panicked' panicked at module/shader/shader_chunks_core/src/lib.rs:258:25:
chunk missing required `//@ tags:` header line:
//@ name: local_probe
//@ description: Probe.
//@ depends_on:
//@ export: fn local_probe(p: vec2f) -> f32

fn local_probe( p : vec2f ) -> f32 { return 0.0; }
```

## Root Cause

`bundle_build`'s upfront required-manifest-fields loop (pre-fix) only ever validated
`"name"` and `"depends_on"` before calling any `shader_chunks_core` manifest parser:

```rust
for required in [ "name", "depends_on" ]
```

But the value-chunk branch's own call graph — `value_chunk_harness_and_parameters` →
`tags_parse( target_wgsl )`, used to detect `category:sdf` for the SDF-specific harness
treatment — depends on a THIRD manifest field ( `"tags"` ) that the upfront guard never
checked. `tags_parse` is only reached AFTER a previewable export has already been
successfully chosen (candidate selection succeeds first, since `discover`/`value_fn_of`
never require a `tags:` line themselves), so the panic is specifically gated behind
"otherwise-valid value chunk, missing only `//@ tags:`" — a narrow but entirely real input
shape, not a hypothetical one.

## Why Not Caught

Every existing test's inline WGSL fixture across `preview_bundle_test.rs` always includes
a `//@ tags: category:test` (or similar) line, matching the real manifest convention out
of habit — none of the 21 pre-existing tests happened to omit it. Every bundled `CHUNKS`
registry entry also already carries a `//@ tags:` line (required independently elsewhere
across the wider `shader_chunks` CLI, e.g. `sch tags`), so `PreviewTarget::Name` mode can
never exercise this path against real bundled data. The gap is only visible through
`PreviewTarget::File` mode against a hand-authored, still-incomplete local chunk — a real
but narrower slice of this crate's own input space that the existing test suite never
constructed.

## Fix Applied (2026-08-17)

**`module/shader/shader_chunks_preview_core/src/lib.rs`:** added `"tags"` to
`bundle_build`'s upfront required-manifest-fields loop, alongside the pre-existing
`"name"`/`"depends_on"` checks:

```rust
for required in [ "name", "depends_on", "tags" ]
```

with a 3-field `Fix(BUG-281)` / `Root cause` / `Pitfall` comment directly above it. This
closes the gap using the exact same graceful-rejection mechanism already used for
`name`/`depends_on` — no change to `tags_parse` itself, no change to any other call site,
and no behavior change for any chunk that already declares `//@ tags:` (which is every
chunk in the bundled registry and every existing test fixture).

**`module/shader/shader_chunks_preview_core/tests/preview_bundle_test.rs`** (new test):
`value_chunk_missing_tags_line_is_rejected_not_panicked` constructs a value-chunk-shaped
WGSL fixture with `name`/`description`/`depends_on`/`export` present and a structurally
valid previewable export, but no `//@ tags:` line, and asserts `bundle_build` returns
`Err( PreviewError::Unpreviewable { .. } )` rather than panicking. Doc comment carries the
mandatory 5 sections (`## Root Cause` / `## Why Not Caught` / `## Fix Applied` /
`## Prevention` / `## Pitfall`), mirroring the format already established in
`shader_chunks_render/tests/render_cli_test.rs`'s `unknown_name_is_rejected_with_the_shared_unknown_chunk_text`
test (this crate's own prior `BUG-205` regression test predates that convention and does
not use it).

## Verification

`longrun`-detached, from repo root:
- **Pre-fix (RED):** the regression test was written and run BEFORE the source fix existed
  (natural test-first ordering, not a revert) — `cargo test -p shader_chunks_preview_core
  --test preview_bundle_test value_chunk_missing_tags_line_is_rejected_not_panicked`:
  failed with exit 101, the process panicking at `shader_chunks_core/src/lib.rs:258:25`
  with `chunk missing required `//@ tags:` header line`, exactly as diagnosed. No `git
  stash` used; a full backup copy of both touched files was taken to the session scratchpad
  first, per this session's standing convention against concurrent-fork git-stash
  collisions (unneeded in the end, since the fix was applied strictly after the pre-fix
  test run, giving true RED before GREEN with nothing to revert).
- **Post-fix (GREEN):** same command: `test value_chunk_missing_tags_line_is_rejected_not_panicked
  ... ok` — 1 passed, 0 failed.
- `cargo test -p shader_chunks_preview -p shader_chunks_preview_core --all-features`:
  34 passed / 0 failed across both crates (12 in `shader_chunks_preview`'s
  `preview_cli_test.rs`, 22 in `shader_chunks_preview_core`'s `preview_bundle_test.rs`
  including the new test), 0 ignored.
- `cargo clippy -p shader_chunks_preview -p shader_chunks_preview_core --all-targets
  --all-features -- -D warnings`: clean, exit 0, no warnings.

## Generalized Version

**Broken assumption:** an upfront "reject missing manifest lines before any panicking
parser runs" guard is only as complete as the literal field list it checks — it does not
automatically track every manifest-parser call actually reachable from the function it
guards. When a later code path (here, `category:sdf` detection for the SDF-specific
harness) adds a new call to a panicking `shader_chunks_core`/`shader_chunks_params_core`
manifest parser, that call silently reopens the exact panic class the upfront guard exists
to close, unless the guard's own field list is extended in the same change. The fix
generalizes to: any future new dependency on a `manifest_field`-backed (panicking) parser
inside `bundle_build`'s reachable call graph must add its field name to this same upfront
loop.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during this session's assigned line-by-line review of `shader_chunks_preview`/`shader_chunks_preview_core` (5 files). `bundle_build`'s own doc comment promises `Unpreviewable` for "missing manifest lines" generally, but its upfront guard only checked `name`/`depends_on`, leaving `tags_parse` (called from `value_chunk_harness_and_parameters` for `category:sdf` detection) to panic on a value chunk missing `//@ tags:`. Fixed by adding `"tags"` to the upfront guard's field list. Verified via 1 new native test (true test-first RED before the fix existed, GREEN after — no revert or `git stash` needed) plus the full `--all-features` suite for both crates (34/34) and clean clippy. Originally scanned as BUG-280 (max ID 279 at scan time), but a fresh on-disk scan immediately before filing found BUG-280 had, in the interim, already been independently claimed by a different concurrent fork; re-scanned and filed as BUG-281 after confirming it was genuinely unclaimed (`task/readme.md`'s `highest_id` stood at 280 at filing time). |
