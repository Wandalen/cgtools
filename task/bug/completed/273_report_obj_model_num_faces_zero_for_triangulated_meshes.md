# BUG-273: `ReportObjModel::num_faces` always reports `0` for any triangulated OBJ mesh

- **Severity:** Medium (silently wrong diagnostic data, not a crash or corruption of the actual
  model geometry -- but wrong on the single most common real load configuration)
- **state:** Completed
- **Affects:** `mingl::model::obj::ReportObjModel::new`'s `num_faces` field computation
- **Component:** `module/min/mingl` (`src/model/obj.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`ReportObjModel::new` computed `num_faces` as `mesh.face_arities.len()` directly. Per `tobj`
4.0.5's own doc comments on `Mesh::face_arities`: *"Empty if loaded with `triangulate` set to
`true` or if the mesh consists only of triangles."* -- in both of those cases `Mesh::indices`
still holds the full, non-empty triangle data (3 indices per face), but `face_arities.len()` is
always `0`. The report's `num_faces` field -- documented as "The total number of faces (polygons)
in the model" -- therefore reads `0` for any mesh that is natively all-triangles or was loaded
with `triangulate: true`, regardless of how many faces it actually has.

## Impact

**Who is affected:** any consumer of `mingl::model::obj::ReportObjModel` /
`reports_make`, and the only real one so far is `examples/minwebgl/obj_load`, which loads its
model via `tobj::GPU_LOAD_OPTIONS` -- a `tobj`-provided constant with `triangulate: true` --
guaranteeing `face_arities` is empty on every run of that example, and then logs
`gl::diagnostics::obj::reports_make( &models, &materials )` for each model via
`gl::log::info!( "{report}" )`. Every such log line silently under-reports the face count as `0`
no matter how many triangles the loaded `.obj` file actually has (the example loads
`suzanne.obj`, which has thousands).

**What breaks:** no crash, no corruption of the model's actual render data (`positions`,
`indices`, etc. are untouched) -- only the derived diagnostic statistic is wrong. Any tooling or
person relying on this report to sanity-check a loaded model (the exact purpose the `diagnostics`
feature and this report type exist for) sees a nonsensical "0 faces" for a visibly non-empty mesh.

**Entity Scope:** `None` -- source-code computation defect, not entity directory instances.

## How Discovered

Static review of `module/min/mingl/src/model/obj.rs` (this session's assigned bug-scouting file
list) while cross-checking `ReportObjModel::new`'s field computations against their own doc
comments one by one. The `num_faces = mesh.face_arities.len()` line, combined with the very next
`if num_faces == 0 { num_of_arities.insert( 3 ); }` branch's own comment ("...the face_arities
array is going to be empty, implying the amount of arities per face equal to 3"), was internally
inconsistent: the code already knew `face_arities` being empty means "assume triangles", but
still let `num_faces` read as `0` instead of deriving the real triangle count. Confirmed against
`tobj` 4.0.5's actual source (`~/.cargo/registry/.../tobj-4.0.5/src/lib.rs`, `Mesh::face_arities`
and `Mesh::indices` doc comments) and traced to a live call site
(`examples/minwebgl/obj_load/src/main.rs:22`, using `tobj::GPU_LOAD_OPTIONS` which hardcodes
`triangulate: true`) to confirm this is reachable through real, already-existing production code,
not a hypothetical configuration.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p mingl --all-features model_obj_test
```
**Expected** (fixed): 3 passed, 0 failed.
**Actual** (pre-fix, confirmed via temporary `git checkout stash@{1} --` / `git stash push --`
revert of only `src/model/obj.rs`, new test file left in place, real run):
```
error[E0432]: unresolved import `the_module::model::obj::num_faces_compute`
 --> module/min/mingl/tests/tests/model_obj_test.rs:8:5
  |
8 | use the_module::model::obj::num_faces_compute;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `num_faces_compute` in `model::obj`
```
(The extracted helper is itself part of the fix -- reverting the fix removes it entirely, so the
pre-fix state fails to compile rather than merely failing an assertion; the underlying defect is
the same one the assertion would otherwise catch: `mesh.face_arities.len()` reads `0` in the
exact state the new test constructs.)

Minimal direct reproduction of the pre-fix formula, for illustration:
```rust
// pre-fix: mesh loaded with tobj::GPU_LOAD_OPTIONS ( triangulate: true )
let face_arities : Vec< u32 > = vec![];       // always empty when triangulated
let indices : Vec< u32 > = vec![ 0, 1, 2, 0, 2, 3 ]; // 2 real triangles
let num_faces = face_arities.len();           // == 0, wrong -- should be 2
```

## Root Cause

```rust
// pre-fix
let num_faces = mesh.face_arities.len();
let mut num_of_arities = HashSet::new();

// The defualt amount of arities is three, so when the object either containes only triangles,
// Or "triangulate" option is chosen when loading with tobj crate, then the face_arities array is going
// to be empty, implying the amount of arities per face equal to 3
if num_faces == 0
{
  num_of_arities.insert( 3 );
}
else
{
  mesh.face_arities.iter().for_each( | &a | { num_of_arities.insert( a ); } );
}
```
`tobj::Mesh::face_arities` is documented (tobj 4.0.5) to be *empty* -- not a per-face `3` entry
each -- whenever every face is a triangle (native or via the `triangulate` load option). The code
already accounted for this when populating `num_of_arities` (inserting `3` as the implied,
unstated arity), but reused the same "`face_arities` is empty" signal as if it were also a valid
face *count* for the `num_faces` field, when it is actually a "no per-face data was stored, look
elsewhere" sentinel. The real count in that state has to come from `mesh.indices.len() / 3`
(3 indices per triangular face), which the pre-fix code never consulted.

## Why Not Caught

No test in this crate exercised `model::obj`'s report-building logic at all --
`ReportObjModel`/`reports_make` require a live `tobj::Model`/`tobj::Mesh`, which the previous
test suite (`bounding_box.rs`, `bounding_sphere.rs`, `camera_orbit_controls.rs`,
`data_type_test.rs`, `nd_test.rs`, `web_file_test.rs`) never constructed. The only place this
code path runs for real is `examples/minwebgl/obj_load`, which merely logs the report string via
`gl::log::info!` in a browser console -- nothing asserts on its contents, so a `0` face count for
a model with thousands of visible triangles never surfaced as a build or test failure, only as a
wrong number a human would have to notice by eye in a browser console log.

## Fix Applied

**`module/min/mingl/src/model/obj.rs`:** extracted the face-count logic into a new, independently
testable, `pub` helper:
```rust
pub fn num_faces_compute( face_arities : &[ u32 ], indices : &[ u32 ] ) -> usize
{
  if face_arities.is_empty()
  {
    indices.len() / 3
  }
  else
  {
    face_arities.len()
  }
}
```
`ReportObjModel::new` now calls `num_faces_compute( &mesh.face_arities, &mesh.indices )` instead
of reading `mesh.face_arities.len()` directly. The `num_of_arities` branch, which used to key off
the now-decoupled `num_faces == 0`, was updated to key off `mesh.face_arities.is_empty()`
directly, so its own "assume arity 3 when empty" behavior is unchanged by this fix (previously
`num_faces == 0` and `face_arities.is_empty()` were equivalent by construction; they are not
after this fix, since `num_faces` can now be non-zero while `face_arities` is still empty). The
new helper was added to the module's public surface (`orphan use { ..., num_faces_compute };`)
so it is testable as an ordinary integration test per this workspace's own
`rulebook.md § Test placement` rule (private-helper tests belong inline; public-API tests belong
in `tests/`).

**`module/min/mingl/tests/tests/model_obj_test.rs`** (new): 3 tests --
`empty_face_arities_derives_count_from_indices` (the bug reproducer: empty `face_arities` + 6
real indices must report 2 faces, not 0), `nonuniform_face_arities_are_counted_directly`
(non-empty `face_arities` behavior is unchanged), `empty_mesh_reports_zero_faces` (a genuinely
empty mesh must still correctly report 0, not a spurious non-zero count). Registered in
`tests/tests.rs` under `#[ cfg( feature = "model_obj" ) ]`, matching the file's own feature gate.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p mingl --all-features model_obj_test` -- pre-fix (temporary revert of only
  `src/model/obj.rs` via `git stash push --`/targeted `git checkout stash@{1} --` restore, new
  test file left live): fails to compile, `error[E0432]` referencing the now-absent
  `num_faces_compute`, exactly as diagnosed above. Post-fix (file restored from the stash entry):
  3 passed, 0 failed.
- `cargo test -p mingl --all-features` (full crate suite): 62 passed, 0 failed, 10 doc-tests
  ignored (pre-existing, unrelated `#[ cfg_attr( doc, ... ) ]`-gated doc examples) -- full
  regression-free confirmation. (One transient run mid-session showed 2 unrelated failures in
  `tests::character_controls::*` -- traced via `git status`/mtime to a different concurrent
  fork's own in-flight edit to `src/controls/character_controls.rs`, resolved on its own by the
  next run; not caused by, or related to, this fix.)
- `cargo clippy -p mingl --all-targets --all-features -- -D warnings`: clean, exit 0.

## Generalized Version

**Broken assumption:** a struct field left *empty* by an external crate as a documented
"the real value lives elsewhere, apply the implied default instead" signal is not safe to read
`.len()` off directly as if emptiness meant "zero of this thing." `tobj::Mesh::face_arities`
communicates "every face has arity 3, go count something else" via emptiness, not "there are no
faces" -- and the surrounding code already understood this well enough to special-case it for one
field (`num_of_arities`) while still misreading it for the sibling field (`num_faces`) computed
two lines above. Whenever an external type's field has an "empty means look elsewhere" contract,
audit *every* local computation derived from that field's length, not just the one the original
author happened to special-case.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found via static review of `module/min/mingl/src/model/obj.rs`, one of 15 files assigned to this fork (one of 14 parallel forks scouting `module/min`'s 5 crates for bugs this session). Root cause: `num_faces = mesh.face_arities.len()` reads 0 for any mesh loaded with `tobj`'s `triangulate` option (or natively all-triangle), even though `mesh.indices` holds the real triangle data -- confirmed reachable via the live `examples/minwebgl/obj_load` example, which always loads with `tobj::GPU_LOAD_OPTIONS` (`triangulate: true`). Fixed by extracting a `num_faces_compute` helper that falls back to `indices.len() / 3` when `face_arities` is empty, and re-keying the sibling `num_of_arities` branch off `face_arities.is_empty()` directly instead of the now-decoupled `num_faces == 0`. Verified via 3 new native unit tests (confirmed fail-to-compile pre-fix / pass post-fix via a targeted `git stash` revert-and-restore of only `src/model/obj.rs`) plus the full `--all-features` suite (62/62) and clean clippy. Filed as BUG-273 after a fresh on-disk scan (both `task/` and `task/bug/` namespaces, plus `task/readme.md`'s `highest_id: 272`) found 272 as the highest existing ID in either namespace, immediately before writing this file. Mid-fix, `git stash pop` (intended to restore this fix from `stash@{1}`) instead attempted to pop a *different*, concurrently-running fork's own `stash@{0}` entry (`module/min/minwebgl/Cargo.toml`) -- git's own safety check aborted before applying anything; recovered by identifying the correct entry via `git stash show --name-only` on each index and restoring only this fix's file via a targeted `git checkout stash@{1} -- <path>` + `git stash drop stash@{1}`, leaving the other fork's `stash@{0}` entry completely untouched. |
| 2026-08-17 | post-filing collision observed (no action needed) | A separate concurrent fork independently filed a *different* bug (`module/min/minwebgpu` storage-texture binding layout) also as `273_storage_texture_binding_layout_default_format_not_storage_capable.md`, created after this report per its later mtime. That fork's own workflow self-detected the collision and is renumbering its file to BUG-275 rather than displacing this one -- consistent with this bug already being the first, on-disk-confirmed claimant of ID 273 at filing time. No renumbering or edit was required on this report as a result. |
