# BUG-440: `IBL` leaks its 3 environment-map textures -- no `gl` field, no `Drop`, and `Clone` risked aliased double-free

- **Severity:** Low (no crash, no visual corruption -- an unbounded GPU-memory leak that grows
  with every environment-map swap, since `IBL` had no teardown path of any kind)
- **state:** Completed
- **Affects:** Every consumer of `renderer::webgl::IBL` -- notably `Renderer::ibl_set`, which
  replaces an already-set `self.ibl` at runtime (e.g. an application swapping environment maps),
  silently leaking the previous `IBL`'s three textures every time; also any consumer dropping a
  `Renderer` that still holds an `IBL`.
- **Component:** `module/helper/renderer` (`src/webgl/ibl.rs`, `src/webgl/loaders/ibl.rs`,
  `tests/pmrem_tests.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect class as BUG-432/433/436/437/438, found in the same sweep.
  Distinct additional wrinkle: `IBL` previously derived `Clone`, which needed its own analysis
  (see Pitfall) before a `Drop` impl could be added safely.

## Symptom

`IBL` allocates three textures (`diffuse_texture`/`specular_1_texture`/`specular_2_texture`, or
similarly named fields populated by its loaders) but had no `gl` field and no `impl Drop` --
it was a plain data bag with `pub` texture fields, so nothing in the type itself was ever
responsible for cleanup. It also derived `Clone`, which would copy the texture handles by
reference (aliasing the same GPU textures across instances) with no reallocation-on-clone
mechanism.

## Impact

**Who is affected:** Any consumer swapping environment maps at runtime via `Renderer::ibl_set`,
or dropping a `Renderer`/`IBL` at the end of its lifetime.

**What breaks:** No immediate visual/functional symptom -- cumulative GPU memory pressure that
grows with every environment-map swap; three textures leaked per swap.

**Magnitude:** 3 textures (diffuse + 2 specular mip chains) per `IBL` construct/drop cycle.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during the same repo-wide bug/UX-defect discovery sweep as BUG-432/433 -- cross-referencing
every GPU-resource-owning struct against whether it has a matching `gl.delete_*`/`Drop` path.
`IBL` had neither, and additionally had a `Clone` derive that would need explicit handling before
a naive `Drop` impl could be added safely (a blanket `impl Drop` on a `Clone`-deriving struct
with shared handles risks a double-free the moment either copy is dropped).

## Minimum Reproducible Example

```rust
// module/helper/renderer/src/webgl/ibl.rs, mod tests (inline)
// IBL has no public constructor that populates a real gl context (the real path is via
// loaders::ibl/loaders::pmrem) -- the test constructs it directly via struct literal.
let gl = gl_init();
let ibl = ibl_with_real_textures( &gl ); // helper: struct-literal construction with real textures
let ( diffuse, spec1, spec2 ) = ( ibl.diffuse_texture.clone(), ibl.specular_1_texture.clone(), ibl.specular_2_texture.clone() );
drop( ibl );
// pre-fix: all three textures still live GL objects after drop.
assert!( !gl.is_texture( diffuse.as_ref() ) );
assert!( !gl.is_texture( spec1.as_ref() ) );
assert!( !gl.is_texture( spec2.as_ref() ) );

// Clone-aliasing safety: a dropped clone must never free the original's still-live textures.
let ibl2 = ibl_with_real_textures( &gl );
let clone = ibl2.clone();
drop( clone );
assert!( gl.is_texture( ibl2.diffuse_texture.as_ref() ) ); // still live -- clone's gl was None
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo test -p renderer --lib --target wasm32-unknown-unknown -- ibl_drop_frees_all_three_textures_when_gl_populated ibl_clone_does_not_double_free_original_textures
```

## Root Cause

`IBL` had no `gl` field and no `impl Drop` -- it was a plain data bag with `pub` texture fields,
so nothing in the type itself was ever responsible for cleanup. Separately, `IBL`'s `gl` field
(added by this fix) was initially bare-private, which broke compilation: `loaders::ibl`/
`loaders::pmrem` -- the only two call sites that ever populate the field with a real context --
are sibling modules of `webgl::ibl`, not descendants of its `mod private`, so a bare-private
field was invisible to them (`E0451`). Rust module privacy is scoped to the defining module and
its descendants only, not "the whole crate."

## Why Not Caught

`IBL` had no prior test coverage of its construction/destruction lifecycle -- existing tests
(`tests/pmrem_tests.rs`) exercise the prefiltering pipeline's numerical output, not resource
teardown.

## Fix Location

- `module/helper/renderer/src/webgl/ibl.rs`: added a `pub(crate) gl: Option<WebGl2RenderingContext>`
  field (crate-visible, not merely descendant-visible, so `loaders::ibl`/`loaders::pmrem` can
  populate it); added `impl Drop for IBL`, deleting all three textures only when `gl` is `Some`;
  replaced the derived `Clone` with a manual `impl Clone` that keeps the same field-for-field
  behavior but always resets `gl` to `None` on the copy, so only the original loader-populated
  instance ever frees -- every `Clone` is a non-owning view for as long as it exists.
- `module/helper/renderer/tests/pmrem_tests.rs`: line 203 changed
  `ibl.specular_1_texture.expect(...)` to `ibl.specular_1_texture.clone().expect(...)` -- see
  Pitfall for why the bare `impl Drop` addition made the original code a compile error.

## Prevention

Two new inline tests in `ibl.rs`'s `mod tests` block (inline because `IBL` has no public
constructor populating a real `gl`, and because the fields under test are private -- see
`rulebook.md § Test placement`):
- `ibl_drop_frees_all_three_textures_when_gl_populated` -- constructs an `IBL` directly via
  struct literal with real textures and a populated `gl`, captures all three handles, drops, and
  asserts all three are no longer live GL objects.
- `ibl_clone_does_not_double_free_original_textures` -- constructs an `IBL`, clones it, drops
  only the clone, and asserts the *original*'s textures are still live GL objects (via
  `gl.is_texture`, not an `assert_eq!` identity comparison -- no precedent exists in this
  codebase for `PartialEq` on `web_sys` texture handle types).

## Pitfall

`IBL` previously derived `Clone`, which would have copied the texture handles (aliasing the same
GPU textures across instances, with no reallocation-on-clone mechanism like
`TransformsData`/`DisplacementsData` have). Adding `Drop` on top of that derive would let either
copy free textures the other still relies on. The manual `impl Clone` added by this fix keeps
that field-for-field behavior but always resets `gl` to `None` on the copy, so only the original
loader-populated instance ever frees; no caller in this workspace currently clones an `IBL`
(verified by grep across `module/` and `examples/`), so this is a documented safety margin, not a
fix for an observed bug.

Separately: adding `impl Drop for IBL` retroactively made a pre-existing move-out field access
(`pmrem_tests.rs`'s `ibl.specular_1_texture.expect(...)`) a compile error (`E0509`) -- Rust's
partial-move-out-of-a-`Drop`-type restriction is unconditional. Adding `Drop` to a previously-
`Drop`-free struct is not purely additive -- every existing call site that moves a field out of
the type (`.expect(`/`.unwrap(`/`.take(` on each field name) must be audited across the
workspace before landing the change. A full grep confirmed `pmrem_tests.rs:203` was the only
real affected call site.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during the repo-wide bug/UX-defect discovery sweep of `module/helper/renderer/`/`module/helper/tilemap_renderer/`. |
| 2026-08-20 | fixed | Added `pub(crate) gl` field, `impl Drop for IBL`, and a manual non-aliasing `impl Clone`; fixed the one broken pre-existing move-out call site in `pmrem_tests.rs`; added `Fix(BUG-440)`/`Root cause`/`Pitfall` source comments (3 sites) and two inline reproducer tests. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 4/4

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `cargo check --target wasm32-unknown-unknown --tests -p renderer` compiles clean, including `pmrem_tests.rs`'s fixed call site. Adversarial pass: confirmed by direct inspection that pre-fix `IBL` had no `Drop`/free path, and separately confirmed via full-workspace grep that `pmrem_tests.rs:203` was the *only* pre-existing move-out call site affected by the new `Drop` impl (no other `.expect(`/`.unwrap(`/`.take(` on any `IBL` field elsewhere). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-440)`/`Root cause`/`Pitfall` 3-field source comments at all 3 fix sites (field visibility, `Drop`+`Clone`, and the `pmrem_tests.rs` regression fix); 5-section test doc comments on both reproducers. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `ibl.rs`'s `IBL` struct/impl block, its own inline test module, and the one regression call site in `tests/pmrem_tests.rs`. | — |
| D4 | Clone-aliasing safety | — | 🟢 | Confirming pass: manual `Clone` impl resets `gl` to `None` on every copy, so only the gl-populated original ever frees. Adversarial pass: attempted to find a workspace call site that clones an `IBL` and then relies on the clone's `gl` being populated (which would silently break under this fix) -- grepped `module/` and `examples/` for `.clone()` on any `IBL`-typed value; none found. | — |

**Reproduced:** YES -- direct code inspection confirms pre-fix `IBL` had no delete path for any
of its three textures on any code path; the new tests' post-drop existence checks (both the
plain-drop case and the clone-then-drop-clone case) are the direct, deterministic checks for the
fix and its non-aliasing safety property, respectively. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/ibl.rs` | Added `pub(crate) gl` field, `impl Drop for IBL`, and a manual non-aliasing `impl Clone`, each with `Fix(BUG-440)`/`Root cause`/`Pitfall` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/ibl.rs` | Added inline `mod tests::ibl_drop_frees_all_three_textures_when_gl_populated` and `mod tests::ibl_clone_does_not_double_free_original_textures`. |
| `module/helper/renderer/tests/pmrem_tests.rs` | Fixed pre-existing move-out compile error at line 203 (`.clone()` before `.expect(...)`), with `Fix(BUG-440)`/`Root cause`/`Pitfall` comment. |
