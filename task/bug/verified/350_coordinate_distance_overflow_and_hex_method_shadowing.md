# BUG-350: `distance()` overflows or silently corrupts for large-but-valid `i32` coordinates across every `tiles_tools::coordinates` type, compounded by inherent/trait method shadowing on hex coordinates

- **Severity:** Medium (every affected coordinate type's fields are `pub` and its constructor(s) accept the
  full `i32` range with no range validation, so the overflow is reachable via public API alone with no
  special setup; failure mode ranges from a debug-build panic to release-build silent data corruption)
- **state:** Verified
- **Affects:** `tiles_tools::coordinates::hexagonal::Coordinate<Axial,_>::distance` (BOTH the inherent
  `i32` method and the `Distance` trait `u32` method -- see method-shadowing note below),
  `tiles_tools::coordinates::square::Coordinate<FourConnected>::distance`,
  `tiles_tools::coordinates::square::Coordinate<EightConnected>::distance`,
  `tiles_tools::coordinates::isometric::Coordinate<Diamond>::distance`,
  `tiles_tools::coordinates::triangular::Coordinate<_>::distance`
- **Component:** `module/helper/tiles_tools` (`src/coordinates/hexagonal.rs`, `src/coordinates/square.rs`,
  `src/coordinates/isometric.rs`, `src/coordinates/triangular.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/

## Symptom

```bash
# Actual (pre-fix): 6 distance() implementations across 4 files, all reachable via public
# constructors, overflow or silently corrupt for large-but-valid i32 coordinates.
$ cargo test -p tiles_tools --test coordinates_distance_overflow_test
running 10 tests
test all_distance_impls_unchanged_for_ordinary_small_coordinates ... ok
test hex_inherent_distance_saturates_on_i32_min_negation ... FAILED
test hex_inherent_distance_saturates_on_extreme_subtraction ... FAILED
test hex_trait_distance_exact_on_i32_min_negation ... FAILED
test hex_trait_distance_exact_on_extreme_subtraction ... FAILED
test square_eight_connected_distance_exact_on_extreme_subtraction ... FAILED
test isometric_distance_exact_on_extreme_subtraction ... FAILED
test square_four_connected_distance_exact_on_extreme_subtraction ... FAILED
test square_four_connected_distance_saturates_beyond_u32_max ... FAILED
test triangular_distance_saturates_instead_of_wrapping ... FAILED

thread 'hex_inherent_distance_saturates_on_i32_min_negation' panicked at src/coordinates/hexagonal.rs:182:13:
attempt to negate with overflow
thread 'hex_inherent_distance_saturates_on_extreme_subtraction' panicked at src/coordinates/hexagonal.rs:184:13:
attempt to subtract with overflow
thread 'hex_trait_distance_exact_on_i32_min_negation' panicked at src/coordinates/hexagonal.rs:423:23:
attempt to negate with overflow
thread 'hex_trait_distance_exact_on_extreme_subtraction' panicked at src/coordinates/hexagonal.rs:428:5:
attempt to add with overflow
thread 'square_four_connected_distance_exact_on_extreme_subtraction' panicked at src/coordinates/square.rs:185:7:
attempt to subtract with overflow
thread 'square_eight_connected_distance_exact_on_extreme_subtraction' panicked at src/coordinates/square.rs:207:7:
attempt to subtract with overflow
thread 'isometric_distance_exact_on_extreme_subtraction' panicked at src/coordinates/isometric.rs:279:6:
attempt to subtract with overflow
thread 'triangular_distance_saturates_instead_of_wrapping' panicked at tests/coordinates_distance_overflow_test.rs:175:3:
assertion `left != right` failed: must not silently wrap back to the pre-fix corrupted value
  left: 3705032704
 right: 3705032704

test result: FAILED. 1 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

# Expected (fixed):
$ cargo test -p tiles_tools --test coordinates_distance_overflow_test
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Impact

**Who is affected:** any caller of `distance()` on any of the four coordinate systems
(`hexagonal`, `square`, `isometric`, `triangular`) where the two coordinates are on the order of
`2e9` apart, or where either has a component equal to `i32::MIN` -- e.g. a very large procedurally
generated map, an off-map/sentinel coordinate convention using extreme values, or a corrupted/
malicious save file (every field on every coordinate type here is `pub`, and none of the
constructors -- `new`, `new_uncheked` (hex), `new_unchecked` (triangular) -- validate the input
range beyond triangular's own `a + b + c` sum check).

**What breaks:**
- **Debug builds** (this workspace has no `[profile]` override disabling `overflow-checks`, so the
  default dev-profile `overflow-checks = true` applies): 5 of the 6 concrete `distance()` methods
  (hex inherent, hex trait, both square variants, isometric) panic outright -- `attempt to negate
  with overflow`, `attempt to subtract with overflow`, or `attempt to add with overflow` --
  crashing whatever system called `distance()` (e.g. a per-frame pathfinding heuristic or spatial
  query).
- **Release builds** (`overflow-checks` compiled out by default): the same 5 methods silently wrap
  instead of panicking, returning an arbitrary wrong distance with no error at all.
- **`triangular::Coordinate::distance()`** never panics in either profile (it already widened to
  `i64` before this fix) but its final `as u32` narrowing cast silently wraps once the true `i64`
  sum exceeds `u32::MAX` -- a genuine distance of `8_000_000_000` silently became `3_705_032_704`
  (`8e9 mod 2^32`) in every build profile, the worst failure mode of the six: no crash, no error,
  just a wrong number a caller might route pathfinding, damage falloff, or LOS logic on.

**Compounding factor (method shadowing):** `hexagonal::Coordinate<Axial,_>` carries TWO methods
named `distance` -- an inherent `i32` method (`hexagonal.rs`) and the `Distance` trait's `u32`
method (`hexagonal.rs`). Rust's inherent-shadows-trait method resolution means any concrete,
non-generic call written as `coord.distance(x)` on a hex coordinate ALWAYS resolves to the
inherent method, regardless of `x`'s reference-ness -- the trait method is only reachable via
explicit UFCS (`Distance::distance(&a, &b)`) or from code generic over `C: Distance`. Three
concrete call sites exist in this crate: `src/lib.rs:30` (doctest), `benches/coordinate_benchmarks.rs:24`,
and `tests/integration/coordinates_tests.rs:33` -- all three use owned-argument, method-call syntax,
so all three are (and always were) bound to the inherent method, not the trait method a reader
skimming `impl Distance for Coordinate<Axial,_>` might assume is in play.

**Entity Scope:** `None` -- source-level arithmetic defect, not entity directory instances.

## How Discovered

During a systematic bug-hunt pass across `tiles_tools`, comparing every coordinate type's
`distance()` implementation against what its own `pub` fields and unchecked constructor(s) actually
allow as input showed that none of the 6 concrete implementations bounded their internal arithmetic
against that domain -- each performed ordinary `i32` negation/subtraction (or, for the hex trait
method, narrowed each term to `u32` before summing; for triangular, narrowed the final `i64` sum to
`u32` with a bare `as`) directly on values that can legitimately be up to `2^31` apart. Grepping for
`fn distance` across `src/coordinates/*.rs` found exactly 6 implementations; grepping for
`.distance(` across the whole crate found the 3 concrete hex call sites that expose the
inherent/trait shadowing issue.

## Minimum Reproducible Example

**Verify Command**:
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p tiles_tools --test coordinates_distance_overflow_test
```
**What:** violates every affected `distance()`'s implicit contract (return the distance between two
coordinates the type's own constructor accepted) -- 5 methods panic before returning anything, and
`triangular`'s method returns a silently wrong value, for inputs no more exotic than "two
coordinates far enough apart to matter on a very large map."

**Expected** (fixed): `test result: ok. 10 passed; 0 failed`.

**Actual** (pre-fix, directly observed via temporary revert-and-rerun of this fix, log captured
verbatim in the Symptom section above): `test result: FAILED. 1 passed; 9 failed` -- 8 panics at
the precise source lines predicted by reading each implementation, plus 1 wrong-value assertion
failure for `triangular` confirming the exact corrupted value (`3_705_032_704`) predicted by manual
modular-arithmetic calculation.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | Hex inherent `distance()` overflows on both negation and subtraction | Verified | `-self.q` negates i32::MIN; `self.q - q` subtracts across ~2^31 | E1, E2 |
| H2 | Hex trait `distance()` overflows on negation (i64 upgrade started too late) AND on the final sum (narrowed to u32 per-term before summing) | Verified | Root Cause | `i64::from(-self.q)` still negates in i32 first; `q as u32 + r as u32 + s as u32` sums three already-narrowed u32 terms | E3, E4 |
| H3 | Both `square::Coordinate` variants overflow on raw i32 subtraction before `.abs()` | Verified | `self.x - other.x` on raw i32 fields, shared root cause, two separate impls | E5, E6 |
| H4 | `isometric::Coordinate<Diamond>::distance()` overflows identically to `square::FourConnected` | Verified | `((self.x - other.x).abs() + (self.y - other.y).abs()) as u32`, same raw-i32-subtraction shape | E7 |
| H5 | `triangular::Coordinate::distance()` does NOT panic but silently wraps | Verified | Root Cause | Already widened to i64 for the subtract/abs/sum chain, but the final `as u32` cast has no bounds check | E8 |
| H6 | The overflow is reachable via public API alone, no unsafe/internal-only construction needed | Verified | Root Cause | Every field on every affected coordinate struct is `pub`; `new`/`new_uncheked`/`new_unchecked` perform no range validation beyond triangular's `a+b+c` sum check | E9 |
| H7 | Hex's inherent and trait `distance` methods are genuinely two distinct, independently-reachable methods (not one shadowing the other into unreachability) | Verified | `a.distance(b)` method-call syntax always resolves to the inherent method (Rust: inherent shadows trait); the trait method is reachable only via UFCS or generic-over-`Distance` code | E10, E11 |
| H8 | Neither method can simply be deleted in favor of the other -- each is load-bearing for a different, out-of-scope consumer | Verified | The INHERENT method's owned-`Self` calling convention is load-bearing at `tests/integration/coordinates_tests.rs:33` (`coord1.distance(coord2)`, owned args, outside this bug's fix scope); the TRAIT method is load-bearing for any code generic over `C: Distance` (e.g. `src/pathfind.rs`'s `astar*` functions), which can only ever see the trait method, never the inherent one | E12 |
| H9 | No existing test/doctest/benchmark exercises `distance()` anywhere near the `i32` boundary | Verified | All 3 concrete call sites (`src/lib.rs:30`, `benches/coordinate_benchmarks.rs:24`, `tests/integration/coordinates_tests.rs:33`) and every `distance()` doctest use single/double-digit magnitudes | E13 |
| H10 | The fix does not change behavior for any ordinary (non-overflowing) input | Verified | `all_distance_impls_unchanged_for_ordinary_small_coordinates` passes both pre- and post-fix; full `cargo test -p tiles_tools --all-features` (all 4 fixed files + all pre-existing tests/doctests/benches) clean post-fix | E14, E15 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/coordinates/hexagonal.rs:182` (pre-fix) | `let s = -self.q - self.r;` -- panics `attempt to negate with overflow` when `self.q == i32::MIN` | H1 |
| E2 | `src/coordinates/hexagonal.rs:184` (pre-fix) | `let q = self.q - q;` -- panics `attempt to subtract with overflow` for coordinates ~2^31 apart | H1 |
| E3 | `src/coordinates/hexagonal.rs:423` (pre-fix) | `let s = i64::from(-self.q) - i64::from(self.r);` -- the negation `-self.q` still executes in `i32` BEFORE `i64::from`, panicking identically to E1 | H2 |
| E4 | `src/coordinates/hexagonal.rs:428` (pre-fix) | `(q as u32 + r as u32 + s as u32) / 2` -- each term individually narrowed to `u32` before the sum, so the sum itself can overflow `u32` even though each term fits alone | H2 |
| E5 | `src/coordinates/square.rs:185` (pre-fix) | `FourConnected::distance`: `(self.x - other.x).abs() as u32 + (self.y - other.y).abs() as u32` -- raw `i32` subtraction before `.abs()` | H3 |
| E6 | `src/coordinates/square.rs:207` (pre-fix) | `EightConnected::distance`: same raw-`i32`-subtraction shape, independent `impl` block | H3 |
| E7 | `src/coordinates/isometric.rs:279` (pre-fix) | `((self.x - other.x).abs() + (self.y - other.y).abs()) as u32` -- same shape as E5 | H4 |
| E8 | `src/coordinates/triangular.rs:213-221` (pre-fix, captured in Symptom) | Computation entirely in `i64` (no panic), but `... as u32` on the final sum wraps `8_000_000_000` to `3_705_032_704` | H5 |
| E9 | `src/coordinates/{hexagonal,square,isometric,triangular}.rs` struct definitions | All coordinate fields (`q`,`r`/`x`,`y`/`a`,`b`,`c`) are `pub`; `new`/`new_uncheked`/`new_unchecked` perform no range validation (triangular's `new` only checks `a+b+c ∈ {1,2}`, not individual field magnitude) | H6 |
| E10 | `src/coordinates/hexagonal.rs:176-197` vs `:436-462` | Two `impl` blocks each defining a method literally named `distance` for the same concrete type `Coordinate<Axial,Orientation>` -- one inherent (`impl<Orientation> Coordinate<Axial,Orientation>`), one via `impl<Orientation> Distance for Coordinate<Axial,Orientation>` | H7 |
| E11 | Rust method resolution rules (language semantics, not file-specific) | Inherent methods are always preferred over trait methods of the same name during method-call-syntax (`.`) resolution, independent of argument types/reference-ness | H7 |
| E12 | `src/pathfind.rs` (`astar*` functions generic over `C: Distance` or similar bounds) | Generic code bounded only by the `Distance` trait can only ever see the trait method -- the inherent method is invisible outside concrete, monomorphized call sites | H8 |
| E13 | `src/lib.rs:30`, `benches/coordinate_benchmarks.rs:24`, `tests/integration/coordinates_tests.rs:33`, and every pre-existing `distance()` doctest | All use coordinates in the 0-24 magnitude range | H9 |
| E14 | Terminal output (`cargo test -p tiles_tools --test coordinates_distance_overflow_test`, post-fix) | `test result: ok. 10 passed; 0 failed` | H10 |
| E15 | Terminal output (`cargo test -p tiles_tools --all-features`, post-fix) | Every test binary (`coordinates_distance_overflow_test`, `debug_test`, `events_test`, `field_of_view_test`, `flowfield_test`, `game_systems_test`, `integration_tests` -- 201 passed, includes `integration::coordinates_tests::test_coordinate_distance_axial`, the hex-inherent-method load-bearing test --, `layout_test`, `serialization_test`, `spatial_test`) and both doctest sweeps (39 default-features, 40 all-features) pass clean; `cargo check -p tiles_tools --benches --all-features` compiles clean | H10 |

## Root Cause

```
Every distance() implementation performed its arithmetic directly on the coordinate type's
raw field values, or narrowed intermediate/final results too early, without ever bounding
that arithmetic against the domain the type's OWN constructor actually allows:

  hex inherent:  -self.q  /  self.q - q       -- raw i32 negate/subtract, panics near i32 bounds
  hex trait:      i64::from(-self.q)           -- negation still happens in i32 BEFORE the i64
                                                   upgrade; separately, q/r/s each narrowed to u32
                                                   BEFORE the final sum, so the sum can still
                                                   overflow u32 even with every term individually
                                                   valid
  square (x2):   self.x - other.x              -- raw i32 subtract, panics near i32 bounds
  isometric:     self.x - other.x               -- identical shape to square
  triangular:    (i64 computation) as u32       -- the ONLY site that already widened correctly
                                                   for the subtract/abs/sum chain, but the FINAL
                                                   narrowing cast is a bare `as`, which never
                                                   panics and never saturates -- it just wraps

Compounding: hexagonal::Coordinate<Axial,_> defines `distance` twice -- once as an inherent
method, once via the `Distance` trait. Because inherent methods always shadow trait methods
of the same name in method-call resolution, every concrete `coord.distance(x)` call in this
crate was already, silently, calling the (buggier of the two) inherent method -- the `Distance`
trait impl existing right below it in the same file was, for all method-call-syntax purposes,
dead code for any concrete hex coordinate.
```
Four independent files, four superficially different symptoms (2 panic mechanisms + 1
narrow-before-sum overflow + 1 silent-wrap), but one shared root cause: `distance()` assumed a
"reasonable" sub-range of `i32` that none of these types' fully-public fields or unchecked
constructors ever actually enforce.

## Why Not Caught

Every pre-existing test, doctest, and benchmark exercising any `distance()` implementation in this
crate (`src/lib.rs:30`, `benches/coordinate_benchmarks.rs:24`, `tests/integration/coordinates_tests.rs:33`,
and every doctest inside `square.rs`/`isometric.rs`) used only small, ordinary tile-grid-scale
coordinates (single- and double-digit magnitudes). Nothing in the crate's prior coverage exercised
coordinates anywhere near the `i32` boundary or an `i32::MIN` field value, so the gap between "what
the constructors accept" and "what `distance()` can safely process" had no trigger. The hex
method-shadowing issue additionally had no test coverage gap of its own to reveal it -- it is not a
runtime defect but a latent maintenance hazard: a future edit to only the trait method (the
"obviously correct" one to touch, since it implements the named `Distance` trait) would silently
have zero effect on any of this crate's 3 concrete call sites.

## Fix Location

**Uniform fix pattern applied at all 6 sites**: widen every operand to `i64` BEFORE any arithmetic
(including negations, not only subtractions), perform the ENTIRE computation in `i64`, then narrow
the final result exactly once via `.clamp(0, i64::from(TARGET::MAX)) as TARGET` rather than a bare
`as` cast. Widening only part of a computation (e.g. the hex trait method's original partial
upgrade) reintroduces the exact overflow the wider type was meant to prevent.

1. **`src/coordinates/hexagonal.rs:176-197`** (inherent `Coordinate<Axial,Orientation>::distance`,
   fix comment at `:179-191`, method signature at `:193`) -- widened both negations (`s`/`other_s`)
   and all three subtractions to `i64`; final narrowing via `.clamp(0, i64::from(i32::MAX)) as i32`.
2. **`src/coordinates/hexagonal.rs:436-462`** (`impl Distance for Coordinate<Axial,Orientation>`,
   fix comment at `:439-452`, method signature at `:453`) -- same widen-everything treatment;
   critically, the final sum is now also computed in `i64` before the single `.clamp(0,
   i64::from(u32::MAX)) as u32` narrowing, instead of narrowing each term to `u32` first.
3. **`src/coordinates/square.rs:168-199`** (`impl Distance for Coordinate<FourConnected>`, fix
   comment at `:183-192`, method signature at `:193`) -- `dx`/`dy` computed in `i64`, summed, then
   `.clamp(0, i64::from(u32::MAX)) as u32`.
4. **`src/coordinates/square.rs:202-229`** (`impl Distance for Coordinate<EightConnected>`, fix
   comment at `:217-221`, method signature at `:223`) -- same `i64` `dx`/`dy`, then `dx.max(dy)`,
   then the same saturating narrowing.
5. **`src/coordinates/isometric.rs:261-290`** (`impl Distance for Coordinate<Diamond>`, fix comment
   at `:277-284`, method signature at `:285`) -- identical treatment to `square::FourConnected`.
6. **`src/coordinates/triangular.rs:211-235`** (`impl<Orientation> Distance for Coordinate<Orientation>`,
   fix comment at `:213-225`, method signature at `:227`) -- computation was already entirely `i64`;
   only the final narrowing changed, from a bare `as u32` to `.clamp(0, i64::from(u32::MAX)) as u32`.

**Hex duplicate-method decision: KEEP BOTH, fix both.** `tests/integration/coordinates_tests.rs:33`
(`coord1.distance(coord2)`, owned-argument method-call syntax on a concrete hex coordinate) is
outside this bug's editable scope and is load-bearing on the INHERENT method's exact signature
(`fn distance(&self, Self { .. }: Self) -> i32`, owned `Self` parameter) -- removing the inherent
method in favor of the trait method would break that call site's calling convention (the trait
method takes `&Self`, not `Self`) without ever touching that file. Since both methods are reachable
(inherent via method-call syntax, trait via UFCS or generic `C: Distance` bounds -- e.g.
`src/pathfind.rs`), both needed independent overflow fixes rather than deleting one in favor of the
other; source comments on both explicitly cross-reference the sibling method and this shadowing
hazard for future maintainers (see fix comment at `hexagonal.rs:179-191`, "widening only the
subtraction (as the sibling `Distance` trait impl below originally did) just moves the overflow
earlier instead of removing it").

Source comment format (`Fix(BUG-350)` / `Root cause` / `Pitfall`, 3 fields) added directly above
each of the 6 fixed methods.

## Prevention

Detection command for the general pattern (any `distance`-named method performing raw `i32`
arithmetic without first widening) across this module:
```bash
grep -n "fn distance" -A 3 module/helper/tiles_tools/src/coordinates/*.rs | grep -B 3 "self\.[a-z]* [+-]"
```
Run against the fixed files, this finds no remaining matches (every `distance()` body now widens to
`i64` via `i64::from(...)` before any arithmetic) -- a starting point for review, not a precise
detector; it would not catch a future `distance`-shaped method written with a differently-named
receiver or an already-widened-but-unsaturated final cast (the exact shape `triangular.rs` had
pre-fix).

**Pitfall:** (1) widening only PART of a computation -- e.g. the subtraction but not a negation
feeding it, or narrowing each term to the target type before a final summation -- silently
reintroduces the exact overflow the wider type was meant to prevent; the widened type must cover
every single operation in the computation, with narrowing happening exactly once, at the very end.
(2) `as` casts never panic regardless of `overflow-checks`, in any build profile -- a narrowing cast
on a value that can legitimately exceed the target type's range must saturate explicitly
(`.clamp(...)`), never rely on `as` alone to signal something went wrong. (3) When a type defines
the same method name both inherently and via a trait, method-call syntax always prefers the
inherent method regardless of argument types -- if both must be kept (e.g. because removing either
would change a load-bearing call site's calling convention), both need independent correctness
review; a fix to only the "obviously correct" one (the named trait) can silently leave the other,
actually-more-commonly-invoked method broken.

## Generalized Version

**Broken assumption:** a coordinate/vector-like type's arithmetic-heavy methods (here, `distance()`)
can safely operate on the type's own field type (`i32`) because "coordinates are usually small in
practice."

Fails whenever:
1. The type's fields are `pub` (or otherwise settable without going through a validating
   constructor), AND
2. At least one constructor accepts the field's full underlying-type range without a bounds check
   tighter than the arithmetic method can safely handle, AND
3. The arithmetic method performs its computation in the field's own (non-widened) type, or widens
   only part of the computation, or narrows the final result with a non-saturating cast

**Detection invariant:**
```
for every distance()-shaped method on a type whose fields are pub and whose constructor(s)
perform no matching range validation:
  the ENTIRE computation (every negation, subtraction, and summation) must run in a strictly
  wider type than the fields' own type, and the final narrowing back to the return type must
  saturate (never a bare `as` cast)
```
6 confirmed instances in this crate, spanning 4 files, all sharing this exact root cause (grep
swept every `fn distance` in `src/coordinates/*.rs`; all 6 matched). Not a duplicate of any prior
bug in this repo's `task/bug/` history (dedup search:
`grep -rli "distance.*overflow\|coordinate.*distance\|hex.*distance\|distance.*panic" task/bug/`
found BUG-343 (`movement::calculate_raw_distance` ignoring a cost policy -- a different defect
class, not overflow) and five `completed/` reports about unrelated geometry bugs (rectangle-vs-circle
query, JFA outline buffer selection, tangent NaN, false line-of-sight, epsilon mismatch) -- none
reference `tiles_tools::coordinates`'s `distance()` overflow specifically).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found during a systematic bug-hunt pass over `tiles_tools`'s coordinate systems; root-caused by comparing each `distance()` implementation's arithmetic against what its own type's `pub` fields and unchecked constructor(s) actually allow. Reproducer test (`tests/coordinates_distance_overflow_test.rs`, 10 tests) confirmed 9 FAIL pre-fix (8 panics at precise predicted source lines, 1 wrong-value assertion for `triangular`'s silent wraparound) and 10 PASS post-fix. Full scoped verification (`cargo test -p tiles_tools` and `--all-features`, plus `--doc` both variants, plus `cargo check --benches --all-features`) clean post-fix across all pre-existing test/doctest/benchmark suites. |
| 2026-08-18 | VERIFY Gate | Reproducer suite `coordinates_distance_overflow_test` confirmed all passing against current source (`cargo test -p tiles_tools --test coordinates_distance_overflow_test`: 10 passed; 0 failed); all 6 fix sites confirmed present: hexagonal.rs inherent distance (:176-206, `.clamp(0, i64::from(i32::MAX)) as i32`), hexagonal.rs trait distance (:436-467, `.clamp(0, i64::from(u32::MAX)) as u32`), square.rs FourConnected (:168-199) and EightConnected (:202-229), isometric.rs Diamond (:261-291), triangular.rs (:211-235, final `.clamp(...) as u32`). state: Unverified -> Verified |

## Refs: src/

- `src/coordinates/hexagonal.rs` -- fixed BOTH `distance` methods (inherent `i32` at `:176-197`,
  `Distance` trait `u32` at `:436-462`); kept both (see Fix Location's method-shadowing decision)
- `src/coordinates/square.rs` -- fixed `Distance for Coordinate<FourConnected>` (`:168-199`) and
  `Distance for Coordinate<EightConnected>` (`:202-229`)
- `src/coordinates/isometric.rs` -- fixed `Distance for Coordinate<Diamond>` (`:261-290`)
- `src/coordinates/triangular.rs` -- fixed `Distance for Coordinate<Orientation>` (`:211-235`)

## Refs: tests/

- `tests/coordinates_distance_overflow_test.rs` -- new reproducer file (`test_kind:
  bug_reproducer(BUG-350)`), 10 tests: 9 targeting each of the 6 fix sites (2 tests each for hex
  inherent/trait covering both the negation and subtraction overflow mechanisms; 1 each for square
  FourConnected/EightConnected, isometric, triangular) plus 1 regression-sanity test confirming
  ordinary small-magnitude coordinates are byte-for-byte unchanged across all 4 coordinate types

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | Adversarial pass confirmed `**Affects:**` enumerates all 5 coordinate-type entries (6 methods, hex counted twice for inherent+trait) matching the 6 actual fix sites; no placeholder sections found | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Confirming pass executed the crate's full test suite fresh (`cargo nextest run -p tiles_tools --all-features`, detached via longrun): 272/272 passed, including all 10 dedicated `coordinates_distance_overflow_test` tests, plus `--doc --all-features`: 40/40 passed. Adversarial pass independently traced the arithmetic in all 6 fixed method bodies (not just the `Fix(BUG-350)` comment text) — confirmed every site widens to `i64` for the ENTIRE computation (including hex's negations) before a single saturating `.clamp(0, i64::from(TARGET::MAX)) as TARGET` narrow, closing the specific per-term-narrow-before-sum defect the hex trait method had | — |
| D3 | Cross-Reference Integrity | — | 🟢 | 10 Hypothesis rows (H2/H5/H6 marked Root Cause), all H/E rows cross-cited and bidirectional (spot-checked); `grep -n "Fix(BUG-350)"` across all 4 source files returns exactly 6 matches, matching `## Refs: src/`; reproducer file's `test_kind: bug_reproducer(BUG-350)` tag confirmed. Adversarial pass independently re-verified the two method-shadowing load-bearing claims (H7/H8) by reading the actual call sites directly: `tests/integration/coordinates_tests.rs:33` uses owned-argument method-call syntax (binds only to the inherent method), `src/pathfind.rs`'s `astar*` functions are generic over `C: Distance` (can only ever reach the trait method) — both confirmed genuinely load-bearing, not merely asserted | — |
| D4 | Root Cause Quality | — | 🟢 | Root Cause section accurately synthesizes all 6 sub-causes under one shared pattern; `## Fix Location` enumerates all 6 sites with file:line spans independently re-verified against current source. Adversarial pass grepped `fn distance` across `src/coordinates/*.rs` to hunt for a possible 7th unfixed site — found exactly 6, matching the report's own claimed count exactly, no site missed | — |
| D5 | Execution Scope | — | 🟢 | `repo_identity: self`; all 4 `## Fix Location` files resolve inside this same repo | — |
| D6 | Crate Scope Unity | — | 🟢 | `**Component:**` (`module/helper/tiles_tools`) matches the crate all 4 touched files resolve to | — |
| D7 | Crate Locality | — | 🟢 | `tiles_tools` directly owns `src/coordinates/*.rs` — not a pushed-up aggregator | — |
| D8 | Crate Single Responsibility | — | 🟢 | Fix stays within `tiles_tools`'s existing tile-logic-library responsibility; no scope expansion | — |
| **Total** | | — | 🟢 | 0 open | 0/0 |

**Reproduced:** YES — `cargo nextest run -p tiles_tools --all-features` exit 0, 272/272 passed (includes all 10 `coordinates_distance_overflow_test` tests); `cargo test -p tiles_tools --doc --all-features` exit 0, 40/40 passed, 2026-08-18.
