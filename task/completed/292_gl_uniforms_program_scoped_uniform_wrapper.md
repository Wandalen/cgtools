# Create `gl_uniforms`: program-scoped WebGL uniform upload wrapper, relocated from `codename_space_sandbox`

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** module/helper/gl_uniforms
- **repo_identity:** self
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Three WebGL2 renderer crates in the sibling `codename_space_sandbox` repository
(`game_client`, `slingshot_lab`, `verlet_demo`) each repeated the same
boilerplate at every uniform upload call site: `gl.get_uniform_location(
program, name )`, then `gl::uniform::(matrix_)upload( ... )`, then
`.expect( "uniform upload should not fail" )`. This crate was first extracted
in-repo within `codename_space_sandbox` (its original home), then relocated
into this workspace as `module/helper/gl_uniforms` once it became clear the
wrapper is a thin ergonomic layer directly over `minwebgl`'s own uniform
primitives — general-purpose WebGL rendering infrastructure, not
game-specific logic — and therefore belongs beside this workspace's other
`minwebgl`-adjacent helpers rather than living in a downstream game repo.
`ProgramUniforms` binds a `GL` context and a linked `WebGlProgram` once, so
each call site only needs `.upload( name, &value )` /
`.matrix_upload( name, &value, column_major )`. Success is observable as: a
published-shape crate at `module/helper/gl_uniforms` with `ProgramUniforms`
as its sole public type, registered in the workspace `Cargo.toml`, covered by
live-WebGL2-context tests proving both `upload` and `matrix_upload` complete
without panicking against a real linked program — run via
`cargo check -p gl_uniforms` (native) and the crate's `wasm-bindgen-test`
suite (browser-only, per its `#[ cfg( target_arch = "wasm32" ) ]` gate).

## In Scope

- `module/helper/gl_uniforms/src/lib.rs` — `ProgramUniforms< 'a >` struct
  wrapping a `&GL` + `&WebGlProgram` pair, with `.upload()` and
  `.matrix_upload()` methods generic over `minwebgl::UniformUpload` /
  `UniformMatrixUpload`, plus a `Debug` impl
- `module/helper/gl_uniforms/Cargo.toml` — packaged crate manifest (`license`,
  `repository`, `homepage`, `documentation`, `keywords`, `categories`,
  `include`), depending on `minwebgl` (workspace) and
  `wasm-bindgen-test` (dev-dependency)
- `module/helper/gl_uniforms/readme.md` — user-facing entry point with a
  Responsibility Table
- `module/helper/gl_uniforms/tests/program_uniforms_test.rs` +
  `tests/readme.md` — live-WebGL2-context tests (browser-only)
- Workspace root `Cargo.toml` registration: member list entry plus
  `[workspace.dependencies.gl_uniforms]` block
- `rulebook.md`'s Rendering layer placement table — `gl_uniforms` listed
  beside the L0–L5 ladder (not occupying a rung), with rationale

## Out of Scope

- Migrating `codename_space_sandbox`'s three renderer crates' call sites onto
  this relocated path — that is per-crate work in the other repository (see
  Related Documentation)
- Any new uniform-upload capability beyond what the three original call sites
  already needed (e.g. uniform buffer objects, array-uniform batch upload) —
  no concrete caller needs this today
- A `docs/` collection for this crate — the crate's entire public surface is
  one struct with two methods, fully covered by its own doc comments and
  `readme.md`; no invariant, feature contract, or API surface warrants a
  separate typed doc instance per this workspace's own Documentation
  Necessity Test (`rulebook.md § Documentation layout`)

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code (code tasks)
-   Every Test Matrix case is backed by a test that failed before its implementing change landed (code tasks)
-   Minimum code to satisfy Test Matrix — no features beyond requirements (code tasks)
-   `cargo check -p gl_uniforms` passes with zero warnings under `RUSTFLAGS="-D warnings"`
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments (code tasks)
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution` (all non-admin tasks)
-   Task state updated to ✅ on verification pass; file moved to `task/completed/` (final)

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Scalar (`f32`) uniform present in the linked program | `.upload()` | Completes without panicking |
| T02 | Vector (`[f32;3]`) uniform present in the linked program | `.upload()` | Completes without panicking |
| T03 | Matrix (`[f32;16]`, identity) uniform present in the linked program | `.matrix_upload()` with `column_major: true` | Completes without panicking |
| T04 | Uniform name absent from the linked program | `.upload()` | Completes without panicking (WebGL silently no-ops a `None` location) |

## Acceptance Criteria

-   `ProgramUniforms` is the crate's sole public type, generic over `minwebgl`'s `UniformUpload`/`UniformMatrixUpload` traits
-   Crate builds cleanly as a standalone workspace member (`cargo check -p gl_uniforms`)
-   Every Test Matrix row has a corresponding passing test in `tests/program_uniforms_test.rs`
-   Workspace `Cargo.toml` and `rulebook.md`'s layer table both reference the crate accurately

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting (via EXEC_COMPLETE, ⚙️ → 📦, followed by an acceptance CLAIM, 📦 → 🔎).

### Checklist

Desired answer for every question is YES.

**Crate surface (src/)**
- [ ] C1 — Is `ProgramUniforms`'s only public API `new`/`upload`/`matrix_upload` (plus the `Debug` impl), with no extra surface beyond what the three call-site migrations needed?
- [ ] C2 — Does `.upload()`/`.matrix_upload()` delegate to `minwebgl::uniform::upload`/`matrix_upload` rather than reimplementing GL calls directly?

**Packaging**
- [ ] C3 — Does `Cargo.toml` declare `minwebgl` as a workspace dependency and `wasm-bindgen-test` as a dev-dependency only?
- [ ] C4 — Is the crate registered in the workspace root `Cargo.toml`'s member list and `[workspace.dependencies.gl_uniforms]`?

**Out of Scope confirmation**
- [ ] C5 — Is there no `docs/` directory under this crate?
- [ ] C6 — Does the crate add no uniform-buffer-object or array-batch-upload capability beyond scalar/vector/matrix single-uniform upload?

### Measurements

- [ ] M1 — Public API surface: `grep -n "pub fn\|pub struct" module/helper/gl_uniforms/src/lib.rs` → exactly `ProgramUniforms` (struct) and `new`/`upload`/`matrix_upload` (fns)
- [ ] M2 — Test count: `cargo nextest run -p gl_uniforms 2>&1 | tail -5` or equivalent wasm test runner → T01-T04 all present

### Invariants

- [ ] I1 — compiler clean: `RUSTFLAGS="-D warnings" cargo check -p gl_uniforms` → 0 warnings
- [ ] I2 — workspace member list and `rulebook.md`'s layer table both reference `gl_uniforms` consistently

### Anti-faking checks

- [ ] AF1 — No raw `gl.get_uniform_location(...).expect(...)` boilerplate reintroduced inside `gl_uniforms` itself (the crate's whole purpose is collapsing that pattern for callers): `grep -n "get_uniform_location" module/helper/gl_uniforms/src/lib.rs` → exactly 0 direct calls outside the two documented delegation lines to `minwebgl::uniform::*`

## Outcomes

*(Added by Procedure - Closure when the task transitions to ✅ Completed. Absent from task files prior to completion.)*

This task documents work already completed earlier in the same session,
before this retroactive task record was filed — the crate was built, then
relocated into this workspace, and live-verified against all three
downstream consumers, before this task file existed. See
`## Verification Record` and the independent Acceptance Verification below
for the evidence gathered against the current repository state.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` — Task filed retroactively by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/, documenting already-completed work: `gl_uniforms` crate creation (originally extracted within `codename_space_sandbox`, then relocated to this workspace as its permanent home). Goal: give this workspace's rendering-layer table a canonical entry for the program-scoped uniform-upload wrapper now used by three renderer crates in the sibling repository.

## Related Documentation

- `module/helper/gl_uniforms/readme.md` — crate's own user-facing entry point
- `rulebook.md § Rendering layer placement` — this task's placement rationale ("beside L0" — thin ergonomic layer, not a portability seam)
- Sibling repository `codename_space_sandbox`'s `task/completed/011_*.md`, `012_*.md`, `013_*.md` — the three per-crate adoption tasks documenting each consumer's migration onto this crate (filed in the same retroactive pass as this task)
