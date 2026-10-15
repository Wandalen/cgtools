# BUG-437: `TransformsData`/`DisplacementsData` leak their skinning/morph-target textures -- no `Drop`, no free method

- **Severity:** Low (no crash, no visual corruption -- an unbounded GPU-memory leak across
  repeated construct/drop cycles, e.g. discarding and rebuilding a skinned `Skeleton`/`Mesh`)
- **state:** Completed
- **Affects:** Every consumer of `renderer::webgl::skeleton::{TransformsData, DisplacementsData}`
  that constructs and drops more than one instance over the application's lifetime -- e.g.
  loading/unloading skinned or morph-target-animated meshes at runtime.
- **Component:** `module/helper/renderer` (`src/webgl/skeleton.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect class as BUG-432/433/436/438/440, found in the same sweep --
  `TransformsData`/`DisplacementsData` bundled into one report since they share the identical
  root cause, fix shape, and `Clone`/`need_clone_inner` caveat, applied independently to two
  sibling structs in the same file.

## Symptom

`TransformsData::upload` allocates `global_texture`/`inverse_texture` via `gl.create_texture()`;
`DisplacementsData::upload` allocates `displacements_texture` the same way. Neither struct had an
`impl Drop` or any manual free method -- dropping either (e.g. when its owning
`Skeleton`/`Mesh`/`Node` is discarded) silently leaked every texture it had allocated.

## Impact

**Who is affected:** Any consumer that constructs and drops more than one skinned or
morph-target-animated `Skeleton`/`Mesh` over the application's lifetime -- e.g. a scene that
loads and unloads character models at runtime.

**What breaks:** No immediate visual/functional symptom -- cumulative GPU memory pressure that
only shows up after enough load/unload cycles.

**Magnitude:** `TransformsData`: 2 textures (`global_texture`, `inverse_texture`) per
construct/drop cycle. `DisplacementsData`: 1 texture (`displacements_texture`) per cycle.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during the same repo-wide bug/UX-defect discovery sweep as BUG-432/433 -- cross-referencing
every GPU-resource-owning struct in `skeleton.rs` against whether it has a matching
`gl.delete_texture`/`Drop` path. Neither struct had one.

## Minimum Reproducible Example

```rust
// module/helper/renderer/src/webgl/skeleton.rs, mod tests (inline, wasm32-gated)
// TransformsData and DisplacementsData have no public constructor that populates a real `gl`
// context (their real path is via the private, heavyweight `upload()`), so the test constructs
// each directly via struct literal -- legitimate white-box testing, since the inline test has
// full private-field access anyway.
let gl = gl_init();
let global_texture = gl.create_texture();
let inverse_texture = gl.create_texture();
let data = TransformsData { gl : gl.clone(), global_texture, inverse_texture, /* .. */ };
let ( g, i ) = ( data.global_texture.clone(), data.inverse_texture.clone() );
drop( data );
// pre-fix: both textures still live GL objects after drop.
assert!( !gl.is_texture( g.as_ref() ) );
assert!( !gl.is_texture( i.as_ref() ) );
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo test -p renderer --lib --target wasm32-unknown-unknown -- transforms_data_drop_frees_global_and_inverse_textures displacements_data_drop_frees_displacements_texture
```

## Root Cause

Neither struct had an `impl Drop` and neither had a manual `gl_resources_free`-style method --
nothing in either type ever called `gl.delete_texture` on any of their texture fields.

## Why Not Caught

Neither struct had prior test coverage of its construction/destruction lifecycle -- existing
skinning/morph-target tests exercise the upload/render path, not resource teardown.

## Fix Location

`module/helper/renderer/src/webgl/skeleton.rs`: added `impl Drop for TransformsData` (deleting
`global_texture`/`inverse_texture`) and `impl Drop for DisplacementsData` (deleting
`displacements_texture`).

## Prevention

Two new inline tests in `skeleton.rs`'s `#[cfg(all(test, target_arch = "wasm32"))] mod tests`
block (inline because both need private-field access, and because construction bypasses the
normal, panic-risky `upload()` path entirely via direct struct-literal construction -- see
`rulebook.md § Test placement`):
- `transforms_data_drop_frees_global_and_inverse_textures`
- `displacements_data_drop_frees_displacements_texture`

Both capture texture handle clones before drop, then assert `gl.is_texture` flips to `false`
afterward -- the same deterministic existence-check pattern used by this crate's other
GPU-teardown reproducer tests.

## Pitfall

`Clone` on both structs copies the texture fields by handle (the same underlying GPU texture,
not a deep copy), relying on `need_clone_inner = true` to force `upload()` to allocate the clone
its *own* fresh texture(s) before ever binding/uploading through them -- freeing unconditionally
in `Drop` is safe only because that reallocation happens before any GL call that would actually
use the field. If a future edit ever read `global_texture`/`inverse_texture`/
`displacements_texture` for a GL call *before* the `need_clone_inner` reallocation in `upload()`,
dropping the original ahead of the clone's first `upload()` would leave the clone pointing at an
already-deleted texture.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during the repo-wide bug/UX-defect discovery sweep of `module/helper/renderer/`/`module/helper/tilemap_renderer/`. |
| 2026-08-20 | fixed | Added `impl Drop for TransformsData` and `impl Drop for DisplacementsData`; added `Fix(BUG-437)`/`Root cause`/`Pitfall` source comments (one per struct) and two inline reproducer tests. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `cargo check --target wasm32-unknown-unknown --tests -p renderer` compiles clean; both tests use direct struct-literal construction, verified against each struct's actual private field list. Adversarial pass: confirmed by direct inspection that pre-fix neither struct had any delete call for any texture field -- the post-drop existence checks would have failed against that code. Also checked that the `need_clone_inner` reallocation-before-use ordering documented in Pitfall still holds post-fix (it does; `Drop` was added, `upload()`'s reallocation logic was not touched). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-437)`/`Root cause`/`Pitfall` 3-field source comments on both `impl Drop` blocks; 5-section test doc comments on both reproducers. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `skeleton.rs`'s `TransformsData`/`DisplacementsData` impl blocks plus their own inline test module. | — |

**Reproduced:** YES -- direct code inspection confirms pre-fix neither struct had a delete path
for any texture field on any code path; the new tests' post-drop `gl.is_texture` checks are the
direct, deterministic check for exactly that absence. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/skeleton.rs` | Added `impl Drop for TransformsData` and `impl Drop for DisplacementsData`, each with its own `Fix(BUG-437)`/`Root cause`/`Pitfall` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/skeleton.rs` | Added inline `mod tests::transforms_data_drop_frees_global_and_inverse_textures` and `mod tests::displacements_data_drop_frees_displacements_texture` (wasm32-gated). |
