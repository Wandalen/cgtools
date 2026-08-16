# BUG-131: `RectangularGrid<Parity, Flat>::center()` guards on the wrong axis

- **Severity:** Medium (silently wrong pixel-space center for any multi-column `Flat`-oriented
  hex grid bounds — no panic, no compile error, just an incorrect numeric result)
- **state:** Completed
- **Affects:** Any caller of `RectangularGrid<Parity, Flat>::center()` whose bounds span more
  than one `q` (column) value
- **Component:** `module/helper/tiles_tools` (`src/layout.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — first bug filed for this crate this session; independent of BUG-130
  (different crate)

## Symptom

```rust
let bounds =
[
  Coordinate::< Offset< Odd >, Flat >::new( 0, 0 ),
  Coordinate::< Offset< Odd >, Flat >::new( 2, 0 ),
];
let grid = RectangularGrid::< Odd, Flat >::new( bounds );
let center = grid.center();

// Wrong (pre-fix):
center[ 1 ] == 0.0        // ignores column q=1's shifted y entirely

// Correct (post-fix):
center[ 1 ] == 0.4330127  // true midpoint across the q range's two parity levels
```

## Impact

**Who is affected:** Any caller of the `Flat`-orientation `RectangularGrid::center()` whose
bounds span more than one column (`min.q != max.q`) — e.g. placing a camera or UI anchor at the
center of a flat-topped hex map region.

**What breaks:** The computed center's y-coordinate silently ignores the shifted y-level that
`Flat` orientation's odd/even column parity introduces, instead reporting only the min/max
corner's own y — the exact wrong-behavior scenario a caller would see is a center point offset
from the map region's true visual center by up to roughly a quarter of a hex's height per
additional column spanned.

**Magnitude:** Not a crash — a silently wrong `f32` value consumed directly by rendering/camera
code with no error signal.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #66, a targeted code review of `tiles_tools` under the standing bug-hunt mandate. The
reviewing agent flagged that `RectangularGrid<Parity, Flat>::center()`'s min/max guard tests
`min.r < max.r` / `max.r > min.r`, the same field the sibling `Pointy` impl (immediately above
it in the same file) uses for its own guard — despite `Flat`'s pixel-y depending on `q`'s parity,
not `r`'s. Independently confirmed by direct comparison of the two impl blocks in `src/layout.rs`
and by deriving the crate's own `Offset<Parity,Flat>→Axial,Flat→Pixel` conversion chain
(`src/coordinates/hexagonal.rs` lines 257-277, `src/coordinates/pixel.rs` lines 84-95) by hand.

## Minimum Reproducible Example

```bash
cd module/helper/tiles_tools && cargo test --test layout_test --features enabled 2>&1 | tail -10
```

**Expected** (post-fix):
```
test flat_center_accounts_for_the_shifted_middle_column ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting both guard conditions in the real crate
source back to `min.r < max.r` / `max.r > min.r`, then restoring the fix immediately after
capturing the failure):
```
thread 'flat_center_accounts_for_the_shifted_middle_column' panicked at
module/helper/tiles_tools/tests/layout_test.rs:50:3:
column q=1 (odd, between the two even-parity corners q=0/q=2) sits at a shifted y (~0.866) that
the true bounding range must account for -- the center's y must be their true midpoint (~0.433),
not q=0's own y alone (0.0, what the buggy r-based guard would produce since min.r==max.r here
and never notices the q range spans a different-parity column) -- got 0
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo test --test layout_test --features enabled
# 1 passed = fixed; 1 failed (got 0, expected ~0.433) = bug present
```

**Known MRE limitation (check 205):** none — `RectangularGrid::center()` is pure, synchronous,
dependency-free arithmetic; runs as an ordinary native `cargo test` against the real crate
directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The `Flat` impl's guard was copy-pasted from the `Pointy` impl without updating which field it compares, since `Pointy`'s x-dependency is on `r`'s parity but `Flat`'s y-dependency is on `q`'s parity. | ✅ Root Cause | Direct side-by-side read of both impl blocks: `Pointy` guards `min.r<max.r`/candidate varies `r`; `Flat` guards `min.r<max.r` (same field) but its own candidate already varies `q` (`min.q+1`) — the guard axis and the candidate axis disagree only in the `Flat` block. | E1, E2 |
| H2 | The candidate-point construction (`min.q + 1`, `max.q - 1`) is itself also wrong and needs to change. | ❌ Falsified | The candidate already correctly varies `q` (matching `Flat`'s true q-parity dependency, confirmed via the `Offset<Parity,Flat>→Axial` conversion formulas) — only the guard's comparison field was stale. | E3 |
| H3 | This is invisible/inert in practice because no real caller ever spans more than one column. | ⚠️ Partially true, doesn't excuse the defect | `examples/minwebgl/hexagonal_grid` only instantiates `Pointy`, so no *current* caller hits this — but the API is public or a future `Flat` caller with multi-column bounds would silently get a wrong center with no error signal. | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/layout.rs` `Pointy` impl (lines 68-106) vs `Flat` impl (lines 108-146), pre-fix | Both guards read `min.r < max.r` / `max.r > min.r` verbatim — but `Pointy`'s candidate varies `r` (`min.r + 1`) while `Flat`'s candidate varies `q` (`min.q + 1`), confirming the `Flat` guard is the stale copy. | H1 ✅ |
| E2 | `src/coordinates/hexagonal.rs:257-277` (`Offset<Odd/Even>,Flat → Axial,Flat`) | Both conversion formulas compute `r_axial` using `value.q & 1` (q's parity) — confirms `Flat`'s pixel-y genuinely depends on `q`'s parity, not `r`'s. | H1 ✅ |
| E3 | `src/coordinates/pixel.rs:84-95` (`Axial,Flat → Pixel`) | `y = sqrt(3)/2*q_axial + sqrt(3)*r_axial` — hand-computing bounds `[(q=0,r=0),(q=2,r=0)]` (Odd) gives y(q=0)=0.0, y(q=1)=0.8660254, y(q=2)=0.0 — confirming the candidate's own `q`-varying construction is correct and the shift is real. | H2 ❌ |
| E4 | `examples/minwebgl/hexagonal_grid/src/main.rs` | Only instantiates `RectangularGrid<Odd, Pointy>` — no current caller reaches the `Flat` impl at all. | H3 |

## Root Cause

```
Pointy::center()  ->  guard: min.r < max.r   candidate varies: r   (x depends on r's parity)
Flat::center()    ->  guard: min.r < max.r   candidate varies: q   (y depends on q's parity)
                            ^^^^^^^^^^^^^^^ stale copy from Pointy -- should be min.q < max.q
```

The `Flat` impl's structure (compute a corner's own pixel value, then check whether the bounds
span more than one value of the parity-relevant coordinate, and if so also check an
adjacent-parity candidate point and take the min/max) was correctly adapted for `Flat`'s own
candidate construction, but the guard condition gating that check was left referencing `r` — the
axis relevant to the sibling `Pointy` impl, not `Flat`.

## Why Not Caught

`RectangularGrid` had no dedicated test file at all before this fix — `tests/readme.md`'s
Responsibility Table had no entry for it. The one real workspace caller
(`examples/minwebgl/hexagonal_grid`) only ever instantiates the `Pointy` orientation.

## Fix Location

`module/helper/tiles_tools/src/layout.rs`, `impl<Parity> RectangularGrid<Parity, Flat>::center()`:

```rust
// before
let min_y = if min.r < max.r { ... };
...
let max_y = if max.r > min.r { ... };

// after
let min_y = if min.q < max.q { ... };
...
let max_y = if max.q > min.q { ... };
```

Only the two guard conditions changed; the candidate-point construction and the rest of the
function are untouched.

## Prevention

Added `tests/layout_test.rs` with `flat_center_accounts_for_the_shifted_middle_column`, covering
a 3-column `Flat` bounds range whose middle column has the opposite parity from both corners.

**Pitfall:** when two sibling impls share a near-identical structure differing only in which axis
is parity-dependent, copy-pasting one into the other silently leaves stale references to the
wrong axis in guard conditions that never fail to compile — verify every field reference against
the orientation's own parity rule, not just the shape of the surrounding code.

## Generalized Version

**Broken assumption:** "this guard condition, copied from a structurally similar sibling
implementation, still refers to the axis relevant to *this* implementation." Silently false
whenever two sibling type-parameterized impls (here, two `Orientation` variants of the same
generic type) differ in which field drives their core computation — the compiler cannot catch a
guard that references a real, valid field on the wrong axis.

**Confirmed general rule:** when adapting a block of logic from a sibling impl that differs by a
type parameter with axis-swapped semantics (row-major vs column-major, x-dependent vs
y-dependent), audit every field reference in the adapted block individually against the new
variant's own semantics — do not assume structural symmetry implies field-level correctness.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via task #66's targeted code review of `tiles_tools`; confirmed by direct side-by-side comparison of the `Pointy`/`Flat` impls and by hand-deriving the crate's own coordinate-conversion formulas. |
| 2026-08-16 | fixed | Both guard conditions in the `Flat` impl changed from `.r` to `.q` comparisons. |
| 2026-08-16 | verified | Added `tests/layout_test.rs`; confirmed the test fails against the reverted pre-fix guards with the exact hand-predicted wrong value (`got 0`, expected ~0.433) and passes against the fix; full crate suite (39 tests incl. doctests) + `cargo clippy --all-targets --features enabled,integration -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-16 earlier same day, this verifier 2026-08-16). Independently re-read `RectangularGrid<Parity,Flat>::center()` (confirmed both guards genuinely read `min.q < max.q`/`max.q > min.q`, 5-line `Fix(BUG-131)`/`Root cause`/`Pitfall` comment intact) and `flat_center_accounts_for_the_shifted_middle_column` (non-tautological: asserts the exact `0.433012_7` midpoint, not just non-zero). Fresh `cargo nextest run --all-features` via `longrun` (crate-wide, covering BUG-131 through BUG-137 together): 251/251 passed. `cargo clippy --all-features --all-targets -- -D warnings`: clean. `**Related Bugs:** None` confirmed accurate. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-131 through BUG-137 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs:` sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass initially derived the expected value by hand only; adversarial pass required actually observing the test FAIL against the exact pre-fix guard, not just trusting the hand-derivation — closed via revert-test-restore, and the captured failure text matched the hand-derived prediction exactly (`got 0`, expected ~0.433). | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | First bug filed for `tiles_tools` this session — no cross-refs needed. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass challenged whether the candidate-point construction (not just the guard) was also wrong (H2) — falsified by independently re-deriving the coordinate-conversion formulas. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Checked the sibling `Pointy` impl for the same class of defect — confirmed its own guard correctly matches its own candidate axis (`r`), no equivalent bug there. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `tiles_tools`'s own `src/`/`tests/` + this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to two guard conditions in one function; no public API/signature change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface added — `center()`'s existing contract is now actually honored. | — |

**Reproduced:** YES — reverting both guard conditions to their exact pre-fix form and running
`cargo test --test layout_test --features enabled` produced the exact predicted wrong value
(`got 0`, expected ~0.433); restoring the fix returned the full suite to 39/39 passing plus a
clean `cargo clippy --all-targets --features enabled,integration -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/layout.rs` | `RectangularGrid<Parity, Flat>::center()`: both guard conditions changed from `.r` to `.q` comparisons. `Fix(BUG-131)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/layout_test.rs` | New file: 1 test (`bug_reproducer(BUG-131)`, 5-section doc comment) — `flat_center_accounts_for_the_shifted_middle_column`. |
| `module/helper/tiles_tools/tests/readme.md` | Added Responsibility Table row for the new `layout_test.rs` file. |
