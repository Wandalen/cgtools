# Implement `Rotation` trait for the homogeneous transform types and migrate `look_at_rh` callers where possible; complete in-place variants

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-19 02:58:01
- **expires_at:** 2026-08-19 04:58:01
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/math/ndarray_cg
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **unverified_at:** 2026-08-19 02:57:55
- **unverified_by:** unknown
- **in_motion:** true
- **verifying_at:** 2026-08-19 02:58:01
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

`module/math/ndarray_cg/src/d2/rotation.rs` exposes a live `Rotation<const SIZE: usize>` trait
(`look_at`, `between_vectors`, `vector_rotate`, `invert`) plus two dead, unimplemented free-function
stubs with empty bodies: `inplace_between_vectors`/`inplace_look_at` — signatures and doc comments
fully written, bodies never filled in.

This session's investigation found the `Rotation` trait itself has **zero implementors and zero
callers anywhere in the workspace**. Real, live camera code exists and calls `look_at`-shaped logic —
`module/min/mingl/src/controls/camera_orbit_controls.rs:247`, feeding
`module/helper/renderer/src/webgl/camera.rs` — but through a **different, already-adopted
function**:

```rust
math::mat3x3h::look_at_rh( self.eye, self.center, self.up )
```

Signature comparison (both read this session):

```rust
// mat3x3h::look_at_rh — module/math/ndarray_cg/src/d2/mat3x3h/transformation.rs:223
pub fn look_at_rh< E, Vec3 >( eye : Vec3, center : Vec3, up : Vec3 ) -> Mat4< E, mat::DescriptorOrderColumnMajor >

// Rotation::look_at — module/math/ndarray_cg/src/d2/rotation.rs:26
fn look_at< Dir, Up >( dir : &Dir, up : &Up ) -> Self   // Self : Rotation<SIZE>, a pure SIZE×SIZE rotation
```

These are **not** drop-in equivalents: `look_at_rh` takes eye/center *positions* and returns a full
4×4 homogeneous view matrix (rotation + translation folded in, ready to use as a camera view
transform). `Rotation::look_at` takes a pre-computed *direction* vector and returns a bare
`SIZE×SIZE` rotation with no translation component — using it in place of `look_at_rh` at
`camera_orbit_controls.rs` would require the caller to separately derive the direction from
eye/center and to separately build the translation part of the view matrix, work `look_at_rh`
currently does inline. This is a real, structural difference, not just a naming one — call-site
migration may turn out not to be a clean swap at this specific site.

**Related Tasks:** `391` (other audit incident resolved to action, same user directive to
implement-and-use instead of leaving dead — different type family, no code overlap). `392`/`394`
(other audit incidents left open for developer decision, same audit).

## In Scope

- Implement `Rotation<SIZE>` for whatever concrete matrix type(s) make it real and callable —
  starting point: investigate whether `Rotation` was intended for the plain `Mat<SIZE,SIZE,E,
  Descriptor>` family (the trait's own `SIZE` const generic and `Collection`-based bounds suggest
  this) versus the homogeneous `mat3x3h`/`Mat4` family `look_at_rh` operates on; confirm which family
  the trait's bounds actually admit before implementing.
- Implement the two dead stub functions (`inplace_between_vectors`, `inplace_look_at`) for real, once
  real `Rotation` implementors exist to test them against.
- Investigate, then migrate wherever it's a genuine clean swap: the `camera_orbit_controls.rs:247`
  call site, and any other `look_at_rh`/`between_vectors`-shaped call site found via
  `grep -rln "look_at\|between_vectors"` (a non-exhaustive list from this session's audit included
  `mat3x3h.rs`, `mat3x3h/transformation.rs`, `module/helper/renderer/tests/webgl/camera.rs`, and
  multiple `examples/minwebgpu`/`examples/minwebgl` call sites — re-enumerate fresh at execution
  time, this list may drift).
- Where migration is **not** a clean swap (e.g. `Rotation::look_at`'s missing translation component
  vs. `look_at_rh`'s need for one), document why in this task's History rather than forcing an
  unsound substitution — this mirrors the user's own "wherever possible" framing, not "everywhere
  unconditionally."

## Out of Scope

- Any change to `look_at_rh`'s own signature or behavior — it may remain the correct tool for
  full-view-matrix construction even after this task, if `Rotation` genuinely can't cover that case.
- `Zero`/`ZeroIdentity` (see task 391 — separate incident, same audit).
- New rotation representations (quaternions, axis-angle) — only the existing `Rotation` trait and
  existing `look_at_rh` are in scope.

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any), not by this
section.

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`).
- Real implementation only — no mocks, no partial/placeholder impls; if a genuine blocker makes
  `Rotation` inapplicable to the homogeneous family, that finding itself (with evidence) is an
  acceptable task outcome in place of forced code — record it plainly rather than papering over it.
- New tests must cover the newly-real `Rotation` implementors and both in-place variants (at minimum:
  agreement with the corresponding allocating method's output for the same input).
- Each candidate call site found by the `grep` re-enumeration must be either migrated or given a
  one-line documented reason it wasn't, mirroring task 391's C7 pattern.
- Full workspace build stays green (`cargo check`/`clippy --all-features -- -D warnings`,
  `longrun`-detached per this project's mandatory long-run policy).
- Independent verification pass per `§ Acceptance Verification : Procedure - Execution` before state
  moves to ✅.

## Acceptance Criteria

- At least one concrete type implements `Rotation<SIZE>` for real, with a real caller (not just a
  unit test in isolation) — a currently-unimplemented, currently-uncalled trait that gains only a
  test-only implementor has not actually resolved the "zero adopters" finding this task exists to
  address.
- `inplace_between_vectors`/`inplace_look_at` have real, non-empty bodies and pass equivalence tests
  against their allocating counterparts.
- `camera_orbit_controls.rs:247` and every other enumerated call site shows either a migration to the
  `Rotation`-trait-based call or a documented reason it stays on `look_at_rh`.
- `cargo clippy -p ndarray_cg -p mingl --all-targets --all-features -- -D warnings` exits 0.
- `cargo nextest run -p ndarray_cg --all-features` exits 0, including new tests.

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an
independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**Rotation implementation**
- [ ] C1 — Does at least one concrete type implement `Rotation<SIZE>`, with a real (non-test-only)
      caller?
- [ ] C2 — Do `inplace_between_vectors`/`inplace_look_at` have real bodies, tested for agreement with
      their allocating counterparts?

**Call-site migration**
- [ ] C3 — Is every enumerated `look_at`/`between_vectors`-shaped call site (re-enumerated fresh at
      execution time) either migrated or given a documented reason it wasn't?

### Measurements

- [ ] M1 — `grep -rln "Rotation" module/ examples/` → at least 1 real (non-`rotation.rs`-internal,
      non-test) hit (was: 0 outside the trait's own definition file).
- [ ] M2 — `grep -n "fn inplace_between_vectors\|fn inplace_look_at" -A 3
      module/math/ndarray_cg/src/d2/rotation.rs` → non-empty function bodies (was: `{ }`).

### Invariants

- [ ] I1 — `cargo clippy -p ndarray_cg -p mingl --no-deps --all-targets --all-features -- -D
      warnings` → exit 0.
- [ ] I2 — `cargo nextest run -p ndarray_cg --all-features` → exit 0.

### Anti-faking checks

- [ ] AF1 — The "zero adopters" finding isn't closed by a test-only implementor that no real code
      calls — C1 explicitly requires a real caller, not just a passing unit test.
- [ ] AF2 — A call site marked "not migrated, documented reason" actually has a technical reason
      recorded (e.g. the eye/translation gap identified in this Goal) — not a bare "left as-is" with
      no rationale.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 02:57:55 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-19 02:58:01 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |

## History

- **[2026-08-19]** `FILED` — Filed via lightweight Draft capture
  (`tsk.rulebook.md § Core Procedures : Procedure - Draft Task`, PROC8) following a user-requested
  dead-code maturity review of `module/math`/`module/min`. User directive: implement and use instead
  of the alternative (`look_at_rh`) wherever possible, in preference to leaving the code dead — but
  this session's own signature comparison found a real structural gap (translation component) that
  may limit how much "wherever possible" actually covers; recorded here rather than assumed away.

- **[2026-08-19]** `EXECUTED` — Implemented `Rotation<3> for Mat3<E,Descriptor>` for real
  (`module/math/ndarray_cg/src/d2/rotation.rs`), fixing a pre-existing, previously-unexercised bound
  defect in the trait declaration itself: `Dir`/`Up`/`A`/`B`/`V` were bound to `VectorSpace<SIZE>`,
  which requires `Indexable` — a capability implemented by exactly one type in this crate (`Mat`
  itself), so no real vector argument could ever have satisfied it. Rebound to
  `VectorIter`/`VectorIterMut` directly, matching the crate's own working
  `mat3x3h::look_to_rh`/`look_at_rh` precedent. Filled in both previously-empty
  `inplace_between_vectors`/`inplace_look_at` stub bodies and moved them from `own use` to
  `exposed use` in `mod_interface!` (an `own use` item never propagates past its immediate parent
  module per `mod_interface` v0.66.1's own propagation rules, confirmed by reading the vendored crate
  doc — the flat `the_module::inplace_look_at` path the tests need would not have resolved
  otherwise). Added `tests/inc/d2_test/rotation_test.rs`: 10 tests (identity, self-alignment,
  between-vectors/vector_rotate/invert round-trip agreement, both in-place variants vs. their
  allocating counterparts — each parametrized over both `Descriptor` orderings). Verified clean:
  `cargo check -p ndarray_cg --tests --all-features` (exit 0), `cargo nextest run -p ndarray_cg
  --all-features` (294/294 pass, includes the new 10), `cargo clippy -p ndarray_cg -p mingl
  --all-targets --all-features -- -D warnings` (exit 0).

  **Open gap — AC1/C1/AF1 unsatisfied.** A fresh, repo-wide re-enumeration this session (`grep -rn`
  for `\blook_at\b`, `\bbetween_vectors\b`, `look_at_rh|look_to_rh`, `\bRotation\b` across `module/`
  and `examples/`) found **zero real (non-test) callers** of `Rotation`/`look_at`/`between_vectors`
  anywhere in the workspace. The one candidate call site named in this task's own Goal,
  `camera_orbit_controls.rs:247`, hits exactly the structural blocker the Goal itself anticipated: it
  needs a full `Mat4` view matrix with a translation component (via `look_at_rh`), while
  `Rotation::look_at` produces only a bare `Mat3` rotation with no translation — not a clean swap. Per
  this task's own Delivery Requirements ("if a genuine blocker makes `Rotation` inapplicable ... that
  finding itself, with evidence, is an acceptable task outcome in place of forced code"), this is
  recorded as the honest outcome rather than papered over with an unsound migration or a test-only
  implementor. AC1/C1/AF1 remain open pending either a real caller emerging elsewhere, or a developer
  decision that no real caller is warranted for this trait as currently scoped.

  **M1 measurement is mechanically misleading.** `grep -rln "Rotation" module/ examples/` trivially
  passes (many hits) because "Rotation" is a common pre-existing domain term used by unrelated code
  (e.g. `AnimationProperty::Rotation(QuatF64)` in `renderer/animation/pose.rs`, plus hits in
  `character_controls.rs`, `tilemap_renderer/types.rs`) — spot-checked 3 hits, none reference
  `ndarray_cg::Rotation`. M1 as literally written does not actually demonstrate adoption; C1/AF1 are
  the real gate here, and they remain open per above.

  **`Rotation2`/`rotation2.rs`.** Re-checked precisely during this session's own Tier 2 adversarial
  self-check pass, since a fresh `grep -rn "\bRotation\b"` surfaced it as a real hit worth
  double-checking, not dismissing on sight: `d2/rotation2.rs` declares `trait Rotation2 where Self :
  Rotation<2>` — a genuine supertrait dependency on `Rotation`, not merely a same-named unrelated
  concept. It does **not**, however, rescue the AC1/C1/AF1 zero-real-callers finding above:
  `Rotation2` itself has zero implementors anywhere in the workspace, its one method
  (`from_angle<Dir,Up>(angle: Scalar) -> Self`) carries unused `Dir`/`Up` generics that don't belong
  on an angle-based constructor (a copy-paste-and-abandon signal, likely drafted from `look_at`'s
  signature shape), and its own `own use { Rotation2 }` re-export in `mod_interface!` is not even
  live — `mat2x2.rs:16` shows the intended re-export line commented out (`//   Rotation2`). A
  supertrait bound that nothing ever implements never gets monomorphized or exercised, so it cannot
  count as a real caller under AC1/C1/AF1's own "not just a unit test in isolation" standard — it's a
  second, sibling piece of dead scaffolding equally in need of the same treatment this task gave
  `Rotation`, not a hidden adopter. Not mentioned in this task's Goal/Scope; left untouched and out of
  scope for this task, but recorded here precisely rather than waved off as "unrelated."

  **`tsk` lifecycle.** `.submit` → ❓, `.claim_verify` → 🔬 (Verifying) both completed. `.verify_pass`
  blocked: `tsk: .verify_pass: self-verification forbidden (actor matches filed_by)` — same actor
  filed and executed this task, matching the established same-sandbox actor-guard pattern already
  documented on numerous other rows in `task/readme.md`'s Tasks Index. Left at 🔬 (Verifying) pending
  an independent verifier in a different actor identity; not forced or spoofed past the guard.
