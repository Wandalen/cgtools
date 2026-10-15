# BUG-283: `shader_chunks_cli_core`'s `arg_string`/`arg_bool` silently drop a duplicated named argument instead of erroring, letting `shader_chunks_compose`'s `out::`/`transitive::` silently fall back to their defaults

- **Severity:** Medium (no data corruption on disk, but a silent behavior divergence from an
  explicit user request -- a `compose ... out::a out::b` invocation reports success (exit 0)
  while never writing the file the user asked for, with zero error indication)
- **state:** Completed
- **Affects:** `shader_chunks_cli_core::arg_string`/`arg_bool` (`src/lib.rs`); `shader_chunks_compose`'s
  `cmd_compose` routine (`src/lib.rs`) via its `out`/`transitive` parameters
- **Component:** `module/shader/shader_chunks_cli_core` + `module/shader/shader_chunks_compose`
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`shader_chunks_compose compose <names...> out::<path>` is documented (`docs/cli/param/01_out.md`)
to always write the composed WGSL to `<path>` and print only a summary line to stdout instead.
But if `out::` is supplied more than once on argv -- e.g. `compose hash21 out::a.wgsl out::b.wgsl`
(a plausible copy-paste/typo mistake, not a contrived construction) -- the command silently
reverts to its own "no `out::` given" behavior: it prints the composed WGSL text to stdout and
creates **neither** file, exiting 0 as if nothing were wrong. The same silent-default happens for
`transitive::`: `compose fbm3 transitive::1 transitive::1` silently composes with
`transitive=false`, immediately surfacing an unrelated `MissingDependency` error that reads like a
legitimate dependency-resolution failure rather than a swallowed duplicate-argument mistake.

## Impact

**Who is affected:** any user or script that accidentally repeats a named `key::value` argument on
a `shader_chunks_compose` invocation -- most plausibly through shell history editing, copy-paste,
or a generated command line that appends a default `out::` before a user-supplied override.

**What breaks:** the CLI's own documented "loud errors, no silent defaults" contract (this repo's
own CLI convention, and `docs/cli/param/01_out.md`'s explicit claim that supplying `out::` always
changes the output destination). A `compose ... out::a out::b` invocation exits 0, produces zero
files, and gives no indication that `out::` was ignored -- indistinguishable from a successful
"no `out::`" run except by noticing the file never appeared.

**Entity Scope:** `None` -- CLI argument-handling defect in library code, not entity directory
instances.

## How Discovered

During this fork's assigned review of `shader_chunks_cli_core` and `shader_chunks_compose` (5
files: `shader_chunks_cli_core/src/lib.rs`, its test file, `shader_chunks_compose/src/lib.rs`, its
`src/bin/` entry point, and its test file). After an exhaustive static trace of the help-routing
logic and argument-extraction helpers found no defect through reading alone, empirically probed
the compiled `shader_chunks_compose` binary against several unilang argument-parsing edge cases
named in this fork's own domain hints ("duplicate keys ... does it error loudly or silently
default?"). `compose hash21 out::/tmp/dupA.wgsl out::/tmp/dupB.wgsl` printed the composed WGSL to
stdout and created neither file. Root-caused by reading `unilang`'s own semantic analyzer source
(`argument_binding.rs`'s `bind_argument_values`), which confirmed a repeated named key is *always*
bound to `Value::List` regardless of the argument's declared `multiple` attribute -- a documented
`unilang` behavior ("TASK 024 FIX ... requirement R1"), not a `unilang` bug itself.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
./target/debug/shader_chunks_compose compose hash21 out::/tmp/dupA.wgsl out::/tmp/dupB.wgsl; echo "exit=$?"
ls /tmp/dupA.wgsl /tmp/dupB.wgsl 2>&1
```
**Expected** (fixed): exits 1, stderr names the offending `out` parameter, neither file exists.
**Actual** (pre-fix):
```
//@ name: hash21
...
fn hash21( p : vec2f ) -> f32
{ ... }
exit=0
ls: cannot access '/tmp/dupA.wgsl': No such file or directory
ls: cannot access '/tmp/dupB.wgsl': No such file or directory
```
Composed WGSL silently printed to stdout; neither requested file was written; exit 0.

## Root Cause

`shader_chunks_cli_core/src/lib.rs` (pre-fix):
```rust
pub fn arg_string( cmd : &VerifiedCommand, key : &str ) -> Option< String >
{
  match cmd.arguments.get( key )
  {
    Some( Value::String( s ) | Value::Enum( s ) ) => Some( s.clone() ),
    _ => None,
  }
}

pub fn arg_bool( cmd : &VerifiedCommand, key : &str, default : bool ) -> bool
{
  match cmd.arguments.get( key )
  {
    Some( Value::Boolean( flag ) ) => *flag,
    _ => default,
  }
}
```
`unilang`'s own semantic analyzer (`unilang::semantic::argument_binding::bind_argument_values`,
confirmed via the locally available source tree) collects *any* repeated named argument into
`Value::List` unconditionally:
```rust
// TASK 024 FIX: Automatic Multiple Parameter Collection
// Always collect multiple values into a list, regardless of the `multiple` attribute
if parser_args.len() > 1
{
  // Multiple values detected - always collect into a list
  ...
  bound_arguments.insert( arg_def.name.clone(), Value::List( values ) );
}
```
This runs *before* consulting `arg_def.attributes.multiple` at all -- so even a named argument
explicitly declared single-value (`out`, `transitive` -- both built via `named_arg`, which never
sets `multiple: true`) becomes a `Value::List` the moment its key is repeated on argv.
`arg_string`'s and `arg_bool`'s catch-all `_` arms treat any non-matching `Value` variant --
including this `Value::List` -- identically to "the key was never supplied," so
`shader_chunks_compose`'s `cmd_compose` routine (`match arg_string( &cmd, "out" ) { Some(out) =>
..write.., None => ..print to stdout.. }`) silently takes its own well-tested "absent" branch.

## Why Not Caught

Every existing test for `out::`/`transitive::` (in both `shader_chunks_cli_core_test.rs` and
`shader_chunks_compose_test.rs`) supplies each named key exactly once; nothing in either suite
repeated a named key. The silent fallback routes into a *legitimate, independently well-tested*
code path (`compose`'s own documented "no `out::` given" stdout-printing behavior), so nothing
about the resulting output looks structurally wrong in isolation -- only a side-by-side comparison
against the actually-requested destination reveals the mismatch.

## Fix Applied (2026-08-17)

**`shader_chunks_cli_core/src/lib.rs`:** added `arg_string_checked`/`arg_bool_checked` -- new,
purely additive helpers (existing `arg_string`/`arg_bool` untouched, so the 3 sibling utility
crates that still call them -- `shader_chunks_preview`, `shader_chunks_query`, `shader_chunks_render`,
all outside this fork's assigned scope -- are unaffected) that explicitly match `Value::List` and
return a loud `ErrorData` (exit 1, `ErrorCode::ValidationRuleFailed`) naming the duplicated key and
how many times it was given, instead of falling through a bare `_` catch-all. Exported via the
crate's `mod_interface!` block alongside the existing `arg_string`/`arg_bool`.

**`shader_chunks_compose/src/lib.rs`:** `cmd_compose`'s routine now calls
`arg_bool_checked`/`arg_string_checked` (via `?`) for its `transitive`/`out` parameters instead of
the unchecked originals.

**`shader_chunks_compose/tests/shader_chunks_compose_test.rs`** (2 new tests):
`subprocess_compose_out_given_twice_fails_loudly_instead_of_silently_printing_to_stdout` and
`subprocess_compose_transitive_given_twice_fails_loudly_instead_of_silently_defaulting_to_false`,
each asserting exit 1, a stderr message naming the offending parameter, and (for `out::`) that
neither candidate file is created and the composed text never reaches stdout.

## Verification

`longrun`-detached, from repo root:
- **Pre-fix (RED):** wrote both new regression tests first (true TDD ordering, no revert needed)
  and ran them against the as-found source: `cargo test -p shader_chunks_compose --all-features
  given_twice` -- both failed (`0 passed; 2 failed`), confirming the tests actually detect the bug
  before any fix existed.
- **Post-fix (GREEN):** same filtered run after applying the fix: `2 passed; 0 failed`.
- `cargo test -p shader_chunks_cli_core -p shader_chunks_compose --all-features`: 16 passed / 0
  failed (14 pre-existing + 2 new), across both crates' unit/integration/doc-test binaries.
- `cargo clippy -p shader_chunks_cli_core -p shader_chunks_compose --all-targets --all-features --
  -D warnings`: clean, exit 0.

## Generalized Version

**Broken assumption:** a `Value` match's catch-all `_` arm reads as "the argument is absent," but
`unilang` can bind *any* named argument -- regardless of its own declared `multiple` attribute --
to `Value::List` the instant its key appears more than once on argv. Any single-value extractor
that doesn't explicitly handle `Value::List` will silently conflate "never supplied" with "supplied
redundantly," routing into whatever behavior the "absent" case already has -- which is doubly
dangerous when that fallback behavior is itself legitimate and well-tested in its own right, since
nothing about the resulting output looks wrong in isolation. `shader_chunks_cli_core::arg_usize`
shares the identical pattern (`_ => Ok(0)`) but is not currently called by any routine in either
crate this fork owns, so it was left unfixed here -- any future caller of `arg_usize` (in this
crate or a sibling) should either use a similarly-checked variant or be aware of this gap.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during this fork's assigned review of `shader_chunks_cli_core`/`shader_chunks_compose` (5 files), via empirical probing of unilang argument-parsing edge cases (duplicate named keys) after exhaustive static review found no defect through reading alone. Root cause: `unilang`'s semantic analyzer binds any repeated named argument to `Value::List` regardless of its own `multiple` attribute; `arg_string`/`arg_bool`'s catch-all arms silently treated that as "argument absent," so a duplicated `out::`/`transitive::` on `shader_chunks_compose compose` silently fell back to each parameter's own default instead of erroring. Fixed by adding additive `arg_string_checked`/`arg_bool_checked` helpers to `shader_chunks_cli_core` and switching `shader_chunks_compose`'s `cmd_compose` routine to use them. Verified via 2 new regression tests (written and confirmed failing against the unfixed source first -- true TDD ordering, no `git stash`/revert needed) plus the full `--all-features` suite (16/16) and clean clippy. Filed as BUG-283 after a fresh on-disk scan immediately before filing found BUG-282 had, in the interim, already been independently claimed and closed by a different concurrent fork (`shader_chunks_params` docs staleness, unrelated); `task/readme.md`'s `highest_id` stood at 282 at filing time. |
