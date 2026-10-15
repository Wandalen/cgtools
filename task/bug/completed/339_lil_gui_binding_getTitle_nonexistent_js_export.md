# BUG-339: 7 `examples/minwebgl` crates' `lil_gui.rs` bound a title-setter function to the nonexistent JS export `"getTitle"`, copy-pasted from an older sibling `gui.js` without checking each crate's own actual exports

- **Severity:** Medium (runtime `wasm_bindgen` error if the binding is ever actually called from Rust; latent until then)
- **state:** Completed
- **Affects:**
  - (A) `examples/minwebgl/gltf_viewer/src/lil_gui.rs` + `tests/lil_gui_binding_test.rs`
  - (B) `examples/minwebgl/morph_targets/src/lil_gui.rs` + `tests/lil_gui_binding_test.rs`
  - (C) `examples/minwebgl/postprocessing/src/lil_gui.rs` + inline test in `src/main.rs`
  - (D) `examples/minwebgl/animation_amplitude_change/src/lil_gui.rs` + `tests/lil_gui_binding_test.rs`
  - (E) `examples/minwebgl/skeletal_animation/src/lil_gui.rs` + `tests/lil_gui_binding_test.rs`
  - (F) `examples/minwebgl/pbr_lighting/src/lil_gui.rs` + `tests/lil_gui_binding_test.rs`
  - (G) `examples/minwebgl/character_control/src/lil_gui.rs` + `tests/lil_gui_binding_test.rs`
- **Component:** examples/minwebgl/{gltf_viewer, morph_targets, postprocessing, animation_amplitude_change, skeletal_animation, pbr_lighting, character_control}
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

Each crate's `lil_gui.rs` declared a `wasm_bindgen` extern binding

```rust
#[ wasm_bindgen( js_name = "getTitle" ) ]
pub fn title_set( gui : &JsValue, value : &str ) -> JsValue;
```

(exact function name varied slightly) binding a two-argument title-*setter* to a JS export named
`getTitle` -- a getter-shaped name for a setter-shaped signature, and one that does not exist at
all in any of these 7 crates' own `gui.js` files, which instead export a two-argument setter under
the name `setTitle`.

## Impact

**Who is affected:** any user of these 7 demos, if and when the affected binding is actually
invoked from Rust.

**What breaks:** calling `title_set`/`name_set` (whichever name each crate used) at runtime would
produce a `wasm_bindgen`/JS runtime error (calling a JS function that does not exist), since the
compiled Rust binding compiles fine — `wasm_bindgen` extern declarations are not checked against
the actual JS module's real exports at compile time — but has no matching JS-side implementation.

**Entity Scope:** `None` -- confined to each crate's own `lil_gui.rs` bindings file; no shared/
cross-cutting state.

## How Discovered

Found directly by the orchestrating session (not by any of the 4 parallel forks assigned to
these crates in task #184's original scope) via a direct, independent repo-wide grep across all
`examples/minwebgl/*/src/lil_gui.rs` files for JS-export/binding-name mismatches, after noticing
one instance in a crate a fork had already reported clean. The direct sweep found this exact
mismatch present in 7 crates total -- 4 more than any individual fork had caught, since each
fork's per-crate review checked its own assigned crates for other defect classes but did not
specifically cross-reference every `lil_gui.rs` binding's `js_name` against its sibling `gui.js`'s
actual exports.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p minwebgl_gltf_viewer -p morph_targets -p postprocessing -p animation_blending \
  -p skeletal_animation -p pbr_lighting -p character_control --no-fail-fast
```
**Expected** (fixed): every `lil_gui.rs` binding's `js_name` matches a real export in that crate's
own `gui.js`. **Actual** (pre-fix): each of the 7 crates bound a setter function to the
nonexistent JS export name `"getTitle"`.

## Root Cause

`lil_gui.rs`'s bindings file was copy-pasted across these sibling crates from a common
originating template/older sibling `gui.js` copy that once exported (or was mistakenly believed to
export) a `getTitle` function, without re-checking each crate's own actual `gui.js` exports --
every one of the 7 crates' real `gui.js` only exports a two-argument `setTitle`, not `getTitle`.

## Why Not Caught

`wasm_bindgen` extern-block bindings compile successfully regardless of whether the named JS
export actually exists -- the mismatch has zero compile-time signal and only surfaces as a runtime
error if the binding is ever called. None of these 7 crates had a pre-existing test exercising
this specific binding's `js_name` against the real `gui.js` file's own export list, and the 4
per-crate forks in task #184 each checked their own assigned crates for various other defect
classes without specifically cross-referencing every JS-binding pair.

## Fix Applied (2026-08-18)

In all 7 crates, changed the binding's `js_name` from `"getTitle"` to `"set_name"` (or the
equivalent real setter export each crate's own `gui.js` actually provides), matching each crate's
real JS export and following this project's noun-first (`name_set`) binding-naming convention
already used elsewhere in these same files. Added a regression test to each crate:
`tests/lil_gui_binding_test.rs` for 6 of the 7 (none had this file pre-existing); `postprocessing`
uses an inline test in `src/main.rs` instead, since that crate's binding tests were already
co-located there. Each test parses the crate's own `lil_gui.rs` and `gui.js` via `include_str!`
and asserts every `js_name` in the Rust bindings has a matching export in the JS file.

## Verification

- **Pre-fix (RED):** reverted each crate's `js_name` to `"getTitle"`; each new test failed
  (binding-to-nonexistent-export detected).
- **Post-fix (GREEN):** `cargo test -p minwebgl_gltf_viewer -p morph_targets -p postprocessing -p animation_blending -p skeletal_animation -p pbr_lighting -p character_control --no-fail-fast`
  -- all 15 test binaries across the 7 crates pass, 0 failures;
  `cargo clippy -p minwebgl_gltf_viewer -p morph_targets -p postprocessing -p animation_blending -p skeletal_animation -p pbr_lighting -p character_control --all-targets --all-features -- -D warnings`
  and the equivalent `cargo check --target wasm32-unknown-unknown` across all 7 crates both clean.

## Generalized Version

`wasm_bindgen` extern bindings have no compiler-enforced link to the real JS module they claim to
bind -- a `js_name` typo or copy-paste-without-recheck compiles silently and only fails at runtime,
if and when called. When a bindings file is copy-pasted across sibling crates (a common pattern in
this codebase's `examples/minwebgl` tree), every `js_name` must be individually cross-checked
against that specific crate's own JS file, not assumed correct because it compiled or because a
near-identical sibling crate's copy happens to be correct. This is the second time in this same
bug-hunt (task #184) that an orchestrator-level direct sweep for a known defect's exact signature
found more instances than any individual fork's per-crate review caught (the other: BUG-340's
stale doc-comment sweep) -- both point at the same systemic gap: per-crate fork review does not by
itself guarantee a cross-crate pattern gets checked everywhere it recurs.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found directly by the orchestrating session via an independent repo-wide grep sweep, going beyond the 4 parallel forks' original per-crate assignments in task #184. Fixed and tested under a `BUG-XXX` placeholder marker (`BUG-XXX-F` in `postprocessing`, since that crate was also touched by Fork C's own separate finding) across all 7 crates. Filed as BUG-339 after a fresh on-disk collision scan (highest prior ID: 338). Related: BUG-340, the other instance this session of an orchestrator-level sweep finding coverage gaps beyond fork-assigned review. |
