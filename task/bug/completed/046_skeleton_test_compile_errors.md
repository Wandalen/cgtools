# BUG-046: `skeleton_tests.rs`'s shared setup helper fails to compile — missing `Node` import and nonexistent `gltf.scene` field

- **Severity:** High
- **state:** Completed
- **Affects:** All 10 tests nested inside `module/helper/renderer/tests/skeleton_tests.rs`'s `#[ cfg( target_arch = "wasm32" ) ] mod tests` block — 4 `wasm_bindgen_test(async)` tests that directly call the broken helper, plus 6 plain `#[ test ]` unit tests bundled in the same cfg-gated module (blocked transitively, see `## Impact`)
- **Component:** `module/helper/renderer` — `tests/skeleton_tests.rs::tests::init_skeleton_test`
- **repo_identity:** self
- **Filed:** 2026-08-09
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-09
- **Fixed:** 2026-08-09
- **Accepted By:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self — same-session Tier 2 Dual-Role Self-Check, no separate PROC16 acceptance actor)

## Symptom

```bash
# terminal output — synthetic MRE reproducing both defects in isolation
# (the real crate cannot be live-compiled for wasm32 in this environment — see ## Why Not Caught)
$ rustc --edition 2021 /tmp/mre046/repro.rs -o /tmp/mre046/repro
error[E0425]: cannot find type `Node` in this scope
  --> /tmp/mre046/repro.rs:10:26
   |
10 | fn get_skeleton( _node : Node ) {}   // error: `Node` not imported into this scope
   |                          ^^^^ not found in this scope
   |
help: consider importing this struct
   |
 8 + use crate::renderer_webgl::Node;
   |

error[E0609]: no field `scene` on type `&GLTF`
  --> /tmp/mre046/repro.rs:14:22
   |
14 |   let _scene = &gltf.scene[ 0 ];   // error: no field `scene` on `GLTF` — the real field is `scenes`
   |                      ^^^^^ unknown field
   |
help: a field with a similar name exists
   |
14 |   let _scene = &gltf.scenes[ 0 ];   // error: no field `scene` on `GLTF` — the real field is `scenes`
   |                           +

error: aborting due to 2 previous errors
$ echo "exit: $?"
exit: 1
```

Two independent, additive compile errors live in the same helper function,
`init_skeleton_test` (`module/helper/renderer/tests/skeleton_tests.rs:31-58`):
`Node` is used as a type annotation (line 43) but never imported, and `gltf.scene[ 0 ]`
(line 55) references a field that does not exist on `GLTF` — the real field is `scenes`
(plural).

## Impact

**Who is affected:** Any invocation of this crate's wasm32 test target with the
`animation` feature enabled — specifically all 10 tests declared inside
`skeleton_tests.rs`'s `#[ cfg( target_arch = "wasm32" ) ] #[ cfg( test ) ] mod tests`
block: the 4 `wasm_bindgen_test(async)` tests that call `init_skeleton_test` directly
(`set_displacement_another_new_displacement_size_test`, `skeleton_clone_test`,
`skeleton_load_displacement_test`, `skeleton_load_transform_test`), **plus** 6 plain
`#[ test ]` unit tests that never touch GLTF/Node at all
(`pack_displacements_data_test` and the 5 tests inside the nested
`calculate_data_texture_size_tests` module) — these are compiled inside the *same*
cfg-gated `mod tests` block, so a compile error anywhere in the module (including in a
function none of them call) blocks the whole module from building. One broken helper
function takes down all 10 tests in the file, not just the 4 that use it.

**What breaks:** Loud — `rustc` refuses to compile the test binary at all
(`error[E0425]`, `error[E0609]`), so no test in the file can ever reach runtime. This is
not a silent wrong-value bug; it is a total, immediate build failure of the intended
target.

**Why High, not Critical/Medium:** No production/runtime code path is affected — both
defects are confined to test-only code, so nothing shipped to end users is broken, which
rules out Critical. But unlike a dormant bug that only manifests for some *future*
caller that doesn't exist yet, this defect manifests *immediately* and *unconditionally*
the moment anyone runs the one command that exists specifically to exercise this file
(`cargo test`/`wasm-bindgen-test` against `--target wasm32-unknown-unknown` with
`--features animation`) — it is not a question of reachability, only of whether that
command has ever actually been run. That makes the entire skeleton/GLTF-animation test
suite (a major verification capability for this crate) completely non-functional today,
which is why this is High rather than Medium.

**Entity Scope:** `None` — the affected file is an ordinary integration test file
(`tests/skeleton_tests.rs`), not an entity directory instance;
`## Affected Entity Collections` does not apply.

## How Discovered

Found during the same unrelated `todo.md` investigation (2026-08-09 session, also
responsible for `task/bug/verified/043_vector_w_wrong_index.md`) — specifically while
checking whether the GLTF loader's node/scene graph already has test coverage for
bounding-box and world-matrix computation. Reading
`module/helper/renderer/tests/skeleton_tests.rs` end-to-end surfaced two independent,
unrelated compile errors in its shared test-setup helper:

```bash
# original (pre-annotation) state, captured via `git show HEAD` since backreference
# comments were added to the live file once this report started (see ## Refs: tests/)
$ git show HEAD:module/helper/renderer/tests/skeleton_tests.rs | sed -n '12,19p'
  use renderer::webgl::
  {
    Object3D,
    calculate_data_texture_size,
    load_texture_data_4f,
    loaders::gltf::{ GLTF, load },
    skeleton::{ DisplacementsData, Skeleton, TransformsData }
  };
# ^ Node is not in this import list

$ git show HEAD:module/helper/renderer/tests/skeleton_tests.rs | sed -n '40,53p'
    let mut get_skeleton =
    |
      node : Rc< RefCell< Node > >
    | -> Result< (), gl::WebglError >
    {
      if let Object3D::Mesh( ref mesh ) = node.borrow().object
      {
        skeleton = mesh.borrow().skeleton.clone();
      }

      Ok( () )
    };

    gltf.scene[ 0 ].borrow().traverse( &mut get_skeleton );
# ^ Node used above (originally line 42) but never imported; gltf.scene (singular, originally line 53)

$ grep -n "pub scenes" module/helper/renderer/src/webgl/loaders/gltf.rs
57:    pub scenes : Vec< Rc< RefCell< Scene > > >,
# ^ the real field is plural "scenes" — no singular "scene" field or alias exists
```

Current line numbers (post-annotation, after `## Refs: tests/`'s backreference comments were
added): import block `13-20`, `Node` usage `43`, `gltf.scene[ 0 ]` `55` — see
`## Hypothesis Table` / `## Fix Location` for citations against the current file state.

## Minimum Reproducible Example

Fully self-contained — plain `rustc`, no cargo project, no external crates, no cgtools
paths. `renderer::webgl::{ Node, GLTF }` are private, unpublished workspace types not
reachable from outside this repo, so the script below reproduces the exact defect
*pattern* instead: a minimal stand-in module exposing `Node`/`GLTF` shaped like the real
ones, with the same two errors at the same two independent call sites.

```bash
mkdir -p /tmp/mre046
cat > /tmp/mre046/repro.rs <<'EOF'
mod renderer_webgl
{
  pub struct Node;
  pub struct Scene;
  pub struct GLTF { pub scenes : Vec<Scene> }
}

use renderer_webgl::GLTF;   // Node deliberately NOT imported, mirroring the real bug

fn get_skeleton( _node : Node ) {}   // error: `Node` not imported into this scope

fn init_skeleton_test( gltf : &GLTF )
{
  let _scene = &gltf.scene[ 0 ];   // error: no field `scene` on `GLTF` — the real field is `scenes`
}

fn main() {}
EOF
rustc --edition 2021 /tmp/mre046/repro.rs -o /tmp/mre046/repro 2>&1
echo "compile exit: $?"
```

**Expected** (once both defects are fixed — i.e. `Node` imported and `.scenes` used):
```
compile exit: 0
```

**Actual:**
```
error[E0425]: cannot find type `Node` in this scope
  --> /tmp/mre046/repro.rs:10:26
   |
10 | fn get_skeleton( _node : Node ) {}   // error: `Node` not imported into this scope
   |                          ^^^^ not found in this scope
   |
help: consider importing this struct
   |
 8 + use crate::renderer_webgl::Node;
   |

error[E0609]: no field `scene` on type `&GLTF`
  --> /tmp/mre046/repro.rs:14:22
   |
14 |   let _scene = &gltf.scene[ 0 ];   // error: no field `scene` on `GLTF` — the real field is `scenes`
   |                      ^^^^^ unknown field
   |
help: a field with a similar name exists
   |
14 |   let _scene = &gltf.scenes[ 0 ];   // error: no field `scene` on `GLTF` — the real field is `scenes`
   |                           +

error: aborting due to 2 previous errors
compile exit: 1
```

**Verify Command:** `rustc --edition 2021 /tmp/mre046/repro.rs -o /tmp/mre046/repro; test $? -eq 1` —
**What:** demonstrates that a type used without being imported and a singular/plural
field-name mismatch each independently abort compilation, reproducing the exact pair of
invariant violations present in `init_skeleton_test` at
`module/helper/renderer/tests/skeleton_tests.rs:43` and `:55`.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Node` is used at line 43 but was never added to the `use renderer::webgl::{...}` import block (lines 13-20) — a plain missing-import omission | ✅ Root Cause | The import list at lines 13-20 enumerates `Object3D, calculate_data_texture_size, load_texture_data_4f, loaders::gltf::{...}, skeleton::{...}` — no `Node` | E1, E2, E5, E6 |
| H2 | `gltf.scene[ 0 ]` at line 55 uses a nonexistent singular field name; the real `GLTF` field is `scenes` (plural) — a singular/plural naming slip | ✅ Root Cause | `GLTF`'s struct definition declares only `pub scenes : Vec<...>` | E3, E4 |
| H3 | `GLTF` exposes `scene` as a deprecated/alternate alias field kept for backward compatibility alongside `scenes` | ❌ Disproved | Full read of the `GLTF` struct definition (`gltf.rs:53-58` and surrounding fields) shows exactly one scenes-related field, `pub scenes`; no `scene` alias or deprecated field exists anywhere in the struct | E4 |
| H4 | `skeleton_tests.rs`'s `mod tests` block is dead/abandoned code never meant to compile, so the errors are harmless | ❌ Disproved | The same module contains 9 other live, well-formed tests exercising real `Skeleton`/`DisplacementsData`/`TransformsData` behavior (e.g. `pack_displacements_data_test` has full multi-step assertions); the sibling file `blender_tests.rs` in the same `tests/` directory uses the identical `#![ cfg( feature = "animation" ) ]` gate and correctly imports `Node` — proving this test file and its gating pattern are live, intentional, and meant to compile | E6 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/tests/skeleton_tests.rs:13-20` | The `use renderer::webgl::{...}` import block — `Node` is absent from the list | H1 ✅ |
| E2 | `module/helper/renderer/tests/skeleton_tests.rs:41-44` | `get_skeleton` closure signature: `node : Rc< RefCell< Node > >` — `Node` referenced as a type but unresolved given E1 | H1 ✅ |
| E3 | `module/helper/renderer/tests/skeleton_tests.rs:55` | `gltf.scene[ 0 ].borrow().traverse( &mut get_skeleton );` — accesses `.scene` (singular) | H2 ✅ |
| E4 | `module/helper/renderer/src/webgl/loaders/gltf.rs:53-58` | `pub struct GLTF { pub scenes : Vec< Rc< RefCell< Scene > > >, pub nodes : ..., ... }` — the only scenes-related field is `scenes` (plural); no `scene` field or alias exists | H2 ✅, H3 ❌ |
| E5 | `module/helper/renderer/src/webgl/node.rs:533-539` | `crate::mod_interface! { orphan use { Node, Object3D }; }` — proves `Node` is a real, exported type reachable at `renderer::webgl::Node` | H1 ✅ |
| E6 | `module/helper/renderer/tests/blender_tests.rs:4-8` | Sibling test file in the same `tests/` directory, same `#![ cfg( feature = "animation" ) ]` gate, correctly imports `use renderer::webgl::{ Node, animation::{...} };` | H1 ✅, H4 ❌ |

## Root Cause

Two independent, additive defects inside the same helper function,
`init_skeleton_test` (`module/helper/renderer/tests/skeleton_tests.rs:31-58`):

```
1. Missing import  — Node used at line 43, never imported at lines 13-20    (H1, ✅ Root Cause)
2. Wrong field name — .scene[0] at line 55, real field is .scenes (plural)  (H2, ✅ Root Cause)
```

Comparing against `blender_tests.rs`'s working import (E6) confirms defect 1 is a plain
omission, not a deliberate exclusion — `Node` is exported and importable exactly the way
that sibling file already does it. Comparing against the `GLTF` struct definition (E4)
confirms defect 2 is a singular/plural naming slip — no `scene` field or alias exists to
justify the singular form. Both defects sit in the same function, both independently
abort compilation on their own (as the MRE demonstrates: removing either error line still
leaves the other), and both must be fixed for the function to compile — this is a
compound root cause (**H1 ✅** and **H2 ✅** jointly, per this project's Hypothesis Table
convention for bugs with more than one independently-sufficient cause), filed as one
report rather than two because they share the same file, the same function, the same
`## Affects`/`## Why Not Caught` context, and the same fix commit naturally touches both.

## Why Not Caught

The entire `mod tests` block is double-gated: `#![ cfg( feature = "animation" ) ]`
(file-level, line 2) **and** `#[ cfg( target_arch = "wasm32" ) ] #[ cfg( test ) ]`
(module-level, lines 4-6) — this code only compiles when building for the `wasm32`
target with `cfg(test)` **and** the `animation` feature enabled simultaneously. An
ordinary native `cargo test` / `cargo nextest run --all-features` invocation compiles
for the host target, not `wasm32`, so this entire module — including its compile
errors — is silently skipped; the errors are invisible to the workspace's standard
verification path. Two further gaps compound this:

```bash
$ find . -iname "*.yml" -o -iname "*.yaml" | grep -v target | xargs grep -l "wasm" 2>/dev/null
# (no output — no CI workflow file references wasm/wasm32 anywhere in the repo)

$ grep -n "^[a-zA-Z_-]*:" Makefile
40:help:
74:env-install:
86:env-check:
105:cwa:
# (no wasm-test target of any kind in the root Makefile)
```

There is no automated CI job and no documented manual command that ever actually
compiles this file for its intended target — a live compile attempt against the real
crate in this environment (`cargo check --target wasm32-unknown-unknown --features
animation --tests`) is itself blocked by an unrelated, pre-existing workspace dependency
gap (`getrandom v0.2.17` lacks `wasm32-unknown-unknown` support without its `"js"`
feature enabled). That gap is out of scope for this bug — it is not part of these two
defects and is noted here only as further, independent evidence that the wasm32 target
is not exercised anywhere in this environment today.

## Fix Location

Two edits, both in `module/helper/renderer/tests/skeleton_tests.rs:13-20` (import list)
and `module/helper/renderer/tests/skeleton_tests.rs:55` (field access):

```rust
// Before (lines 13-20):
  use renderer::webgl::
  {
    Object3D,
    calculate_data_texture_size,
    load_texture_data_4f,
    loaders::gltf::{ GLTF, load },
    skeleton::{ DisplacementsData, Skeleton, TransformsData }
  };

// After:
  use renderer::webgl::
  {
    Node,
    Object3D,
    calculate_data_texture_size,
    load_texture_data_4f,
    loaders::gltf::{ GLTF, load },
    skeleton::{ DisplacementsData, Skeleton, TransformsData }
  };
```

```rust
// Before (line 55):
    gltf.scene[ 0 ].borrow().traverse( &mut get_skeleton );

// After:
    gltf.scenes[ 0 ].borrow().traverse( &mut get_skeleton );
```

Both edits land in the same file; no other locations are affected.

## Fix Applied

Applied exactly as documented above, with both fix-time comments upgraded from the filing-time
backreference to the standard 3-field form (`Fix(BUG-046)` / `Root cause` / `Pitfall`,
`skeleton_tests.rs:12-16` and `skeleton_tests.rs:59-64`).

A live `cargo check -p renderer --target wasm32-unknown-unknown --features animation --tests`
remains blocked in this environment by the same pre-existing, out-of-scope `getrandom v0.2.17`
gap this bug's own `## Why Not Caught` already flagged (`renderer`'s `Cargo.toml` has no
`getrandom = { workspace = true, features = ["wasm_js"] }` override, unlike several sibling
crates in this workspace that need one — e.g. `module/helper/scene_script/Cargo.toml`,
`module/helper/primitive_generation/Cargo.toml`). Confirmed this is unrelated to the fix: the
build fails while compiling `getrandom` itself, before `renderer` or its tests are ever reached
(`error: could not compile getrandom (lib) due to 1 previous error`, exit 101) — identical
failure point regardless of whether the two fixed lines are present. Fixing that gap is a
separate, out-of-scope concern (not part of this bug's `## Fix Location`).

In its place, both edits were verified by direct cross-check against their target definitions:
- `Node` — confirmed re-exported at `renderer::webgl::Node` via `orphan use { Node, Object3D };`
  in `src/webgl/node.rs`, surfaced through `layer node;` in `src/webgl.rs`'s `mod_interface!`.
- `gltf.scenes[ 0 ].borrow().traverse( &mut get_skeleton )` — confirmed type-correct against
  `GLTF::scenes : Vec< Rc< RefCell< Scene > > >` (`src/webgl/loaders/gltf.rs:57`) and
  `Scene::traverse< F >( &self, callback : &mut F ) -> Result< (), gl::WebglError > where
  F : FnMut( Rc< RefCell< Node > > ) -> Result< (), gl::WebglError >`
  (`src/webgl/scene.rs:204-205`), which matches `get_skeleton`'s own signature exactly.
- Workspace-wide sweep (`grep -rn "gltf\.scene\["`) confirmed no other call site carries the
  same singular-field typo.

## Prevention

Wire up an actual wasm32 test execution path for this crate — a CI job or a documented
`make`/`just` target that runs `wasm-pack test` (or the workspace's equivalent) with
`--features animation` against this crate — so a compile error in a
`#[ cfg( target_arch = "wasm32" ) ]`-gated test module is caught before merge instead of
sitting undetected indefinitely. Detection once wired up:

```bash
cargo check -p renderer --target wasm32-unknown-unknown --features animation --tests
```

should exit 0.

**Pitfall:** A test module gated behind `#[ cfg( target_arch = "wasm32" ) ]` (or any cfg
that an ordinary native `cargo test`/`cargo nextest run` sweep never satisfies) is
invisible to standard full-workspace verification — such modules can carry arbitrarily
broken code indefinitely with zero signal. Never assume `will .test level::N` or a
green `cargo nextest run --all-features` says anything about the compileability of
target-gated test modules; they require their own explicit, separately-invoked
verification step.

## Generalized Version

**Broken assumption:** "If `cargo nextest run --all-features` passes, every test file in
the workspace compiles." False whenever a test module is gated behind a `cfg` that
native test runs never satisfy (`target_arch = "wasm32"`, a target-specific feature,
etc.) — such modules can carry arbitrarily broken code indefinitely without any signal
from ordinary verification, because the compiler never even attempts to build them in
that configuration.

**Detection invariant:**
```
for every #[cfg(...)]-gated test module M in the workspace,
there exists at least one CI job or documented command that builds the workspace
with the exact cfg configuration M requires, and that job/command runs on a
recurring cadence (not "exists but has never been executed").
```

## History

| Date       | Event  | Notes                                                                                                     |
|------------|--------|-------------------------------------------------------------------------------------------------------------|
| 2026-08-09 | filed  | Found during unrelated `todo.md` investigation (same session as BUG-043); both defects confirmed via source read + `GLTF`/`Node` definition cross-check + synthetic MRE before filing |
| 2026-08-09 | confirmed | VERIFY_PASS — Tier 2 Dual-Role Self-Check, 8/8 dimensions PASS after one Fix-and-Recheck Loop round; MRE re-executed fresh and reproduces (exit 1) |
| 2026-08-09 | completed | Both edits applied (`Node` import added, `gltf.scene`→`gltf.scenes`), fix comments upgraded to 3-field form. Live wasm32 compile blocked by pre-existing out-of-scope `getrandom` gap (confirmed unrelated — fails during `getrandom`'s own build, before `renderer` is reached); verified instead via direct cross-check against `Node`'s export path and `GLTF::scenes`/`Scene::traverse`'s actual signatures, plus a workspace-wide sweep confirming no other `gltf.scene[` call site. Same-session, self-administered (filer = fixer = verifier) — Tier 2 Dual-Role Self-Check per governance/maav.rulebook.md's default, not an independent PROC16-style acceptance pass. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Adversarial pass: re-executing the MRE fresh showed rustc's suggestion line retains the original line's trailing `// error: ...` comment, but the documented `## Symptom`/`## Minimum Reproducible Example` **Actual** blocks had trimmed that comment — a verbatim-fidelity gap against the real captured output (check 206) | Restored the verbatim trailing comment in both **Actual** blocks to byte-match the freshly re-executed rustc output |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | — | — |
| D4 | Root Cause Quality | 🟡 | 🟢 | Adversarial pass: `## Fix Location`'s opening line named only the file, leaving the two `file:line` citations implicit inside sub-headers rather than stated explicitly up front (check 402) | Added an explicit `skeleton_tests.rs:13-20` / `skeleton_tests.rs:55` citation to the section's opening line |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 2 fixed | 2/2 |

**Reproduced:** YES — exit 1, 2026-08-09 (`/tmp/mre046/repro`, verbatim output captured and matched into `## Symptom` and `## Minimum Reproducible Example`).

**Aggregate verdict:** PASS — all 8 dimensions 🟢 after one Fix-and-Recheck Loop round (`governance/maav.rulebook.md § MAAV : Fix-and-Recheck Loop`); self-administered, no subagent dispatch (Verification Delegation would be forbidden per `file.rulebook.md § Report New Bug : Step 9 - VERIFY Gate`).

## Refs: tests/

- `module/helper/renderer/tests/skeleton_tests.rs` — add missing `Node` import (lines 13-20, post-annotation) and fix `.scene[ 0 ]` → `.scenes[ 0 ]` (line 55, post-annotation); backreference comments already added at lines 12 and 54
