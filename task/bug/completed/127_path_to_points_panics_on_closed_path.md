# BUG-127: `path_to_points` panics via `unreachable!()` on any closed path

- **Severity:** High (unconditional panic — not a corrupted-value defect — reachable through a
  real, already-wired production call chain with no user action required beyond rendering a
  Lottie `Rect`/`Ellipse` shape)
- **state:** Completed
- **Affects:** Any caller of `primitive_generation::path_to_points` with a `Vec<PathEl>` that
  closes a subpath — including the crate's own real `lottie_surface_rendering` example, via
  `velato::Geometry::evaluate` on a `Rect`/`Ellipse` shape
- **Component:** `module/helper/primitive_generation` (`src/primitive.rs::path_to_points`)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — independent root cause from BUG-128/129, filed under the same task #63
  targeted `primitive_generation` review

## Symptom

```bash
# path = [MoveTo(0,0), LineTo(10,0), LineTo(10,10), LineTo(0,10), ClosePath]

# Wrong (pre-fix) -- panics instead of returning points:
path_to_points(path);
# thread 'main' panicked: internal error: entered unreachable code:
# kurbo::flatten can only return MoveTo and LineTo PathEls

# Correct (post-fix):
path_to_points(path);  // == [[0.0,0.0],[10.0,0.0],[10.0,10.0],[0.0,10.0]]
```

## Impact

**Who is affected:** Any caller passing a `Vec<PathEl>` that closes a subpath — which is the
*normal* case for any filled/closed shape, not an edge case. The crate's own real example,
`examples/minwebgl/lottie_surface_rendering`, reaches this: `animation.rs`'s
`geometry_to_primitive` calls `geometry.evaluate(0.0, &mut path); primitive_generation::path_to_points(path);`
for the non-`Spline` branch, and `velato::Geometry::evaluate` (confirmed directly in
`velato` 0.11.0's source, `src/runtime/model/mod.rs`) calls `.path_elements(0.1)` on `Rect`/
`Ellipse` variants — `kurbo::Rect::path_elements`/`kurbo::Ellipse` both always terminate their
path with `PathEl::ClosePath` (confirmed directly in `kurbo` 0.13.1's `src/rect.rs`).

**What breaks:** `path_to_points`'s `kurbo::flatten` callback matched only
`PathEl::MoveTo`/`PathEl::LineTo` and treated every other variant — including `ClosePath` — as
`unreachable!()`. `kurbo::flatten` always re-emits a `PathEl::ClosePath` element (not just the
line/curve segments) whenever the input closes a subpath, so the `unreachable!()` arm was in fact
always reachable for closed input, panicking the entire call instead of returning points.

**Magnitude:** Not a partial/degraded result — a hard panic that aborts whatever thread called it.
Any Lottie animation containing a `Rect` or `Ellipse` shape layer (extremely common — these are
two of the most basic Lottie primitive shape types) reaches this path through the crate's own
documented example pipeline.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #63, a targeted code review of `primitive_generation` dispatched under the standing bug-hunt
mandate. The reviewing agent flagged that `path_to_points`'s `unreachable!()` arm's premise (only
`MoveTo`/`LineTo` are ever emitted) is contradicted by `kurbo::flatten`'s own documented and actual
behavior. Independently re-verified before filing by direct source reads of both the crate and the
exact pinned `kurbo`/`velato` dependency versions:

```bash
$ sed -n '390,429p' module/helper/primitive_generation/src/primitive.rs
# confirms the pre-fix closure: `_ => unreachable!(...)` with no ClosePath arm

$ sed -n '622,707p' ~/.cargo/registry/src/index.crates.io-*/kurbo-0.13.1/src/bezpath.rs
# confirms flatten's own match: `PathEl::ClosePath => { last_pt = None; callback(PathEl::ClosePath); }`
# -- ClosePath IS explicitly re-emitted to the callback, not swallowed internally

$ sed -n '717,792p' ~/.cargo/registry/src/index.crates.io-*/kurbo-0.13.1/src/rect.rs
# confirms RectPathIter::next() always emits [MoveTo, LineTo, LineTo, LineTo, ClosePath]

$ sed -n '135,159p' ~/.cargo/registry/src/index.crates.io-*/velato-0.11.0/src/runtime/model/mod.rs
# confirms Geometry::evaluate calls .path_elements(0.1) on Rect/Ellipse variants

$ grep -n "geometry_to_primitive\|path_to_points\|\.evaluate(" examples/minwebgl/lottie_surface_rendering/src/animation.rs
# confirms the real call chain: evaluate(0.0, &mut path) -> primitive_generation::path_to_points(path)

$ grep -n "primitive_generation" examples/minwebgl/lottie_surface_rendering/Cargo.toml
# confirms primitive_generation = { workspace = true, features = [ "font-processing" ] } (implies "text")
```

## Minimum Reproducible Example

```bash
rm -rf /tmp/mre127 && mkdir -p /tmp/mre127/src
cat > /tmp/mre127/Cargo.toml <<'EOF'
[package]
name = "mre127"
version = "0.1.0"
edition = "2021"

[dependencies]
primitive_generation = { path = "/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/primitive_generation", default-features = false, features = [ "text" ] }
kurbo = "=0.13.1"
EOF
cat > /tmp/mre127/src/main.rs <<'EOF'
use primitive_generation::path_to_points;
use kurbo::{ PathEl, Point };

fn main()
{
  let path = vec!
  [
    PathEl::MoveTo( Point::new( 0.0, 0.0 ) ),
    PathEl::LineTo( Point::new( 10.0, 0.0 ) ),
    PathEl::LineTo( Point::new( 10.0, 10.0 ) ),
    PathEl::LineTo( Point::new( 0.0, 10.0 ) ),
    PathEl::ClosePath,
  ];
  println!( "{:?}", path_to_points( path ) );
}
EOF
cd /tmp/mre127 && cargo run 2>&1 | tail -1
```

**Expected** (post-fix — returns the 4 corner points, no panic):
```
[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
```

**Actual** (pre-fix — confirmed by independently reproducing the exact pre-fix closure body,
byte-for-byte, in an isolated scratch crate against the same pinned `kurbo` 0.13.1, since the real
crate source is already fixed):
```
thread 'main' panicked at src/main.rs:18:14:
internal error: entered unreachable code: kurbo::flatten can only return MoveTo and LineTo PathEls
```

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre127 && cargo run 2>&1 | tail -1
# a 4-point list = fixed; a panic = bug present
```

**Known MRE limitation (check 205):** `primitive_generation` is this workspace's own crate; the
MRE path-depends on it locally rather than a registry version, mirroring BUG-116/118-126's own
documented exception. The pre-fix panic was independently confirmed by reproducing the exact old
closure body against the real pinned `kurbo` dependency in a separate scratch crate (see How
Discovered), not by reverting the actual crate source.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `kurbo::flatten` emits `PathEl::ClosePath` to its callback for any closed subpath, so the pre-fix `unreachable!()` arm was always reachable for closed input. | ✅ Root Cause | Direct read of `kurbo` 0.13.1's `flatten` source confirms `PathEl::ClosePath => { ...; callback(PathEl::ClosePath); }`. Independent scratch-crate reproduction of the exact pre-fix closure panics exactly as predicted. | E1, E2 |
| H2 | This is unreachable in practice because no real caller passes a path that closes a subpath — `path_to_points` is only ever used for already-open point sequences. | ❌ Falsified | The crate's own `lottie_surface_rendering` example reaches this via `velato::Geometry::evaluate` on `Rect`/`Ellipse` shapes, both of which always terminate in `ClosePath` (confirmed in `kurbo`'s own `RectPathIter`). | E3, E4 |
| H3 | `kurbo::BezPath::from_vec` normalizes/drops a trailing `ClosePath` before `flatten` ever sees it. | ❌ Falsified | `flatten`'s own match arm for `PathEl::ClosePath` is reached and calls `callback(PathEl::ClosePath)` directly — nothing upstream filters it out; the scratch-crate reproduction observed the panic, proving the element reaches the callback. | E1, E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `~/.cargo/registry/.../kurbo-0.13.1/src/bezpath.rs`, `flatten`'s match (lines ~622-707) | `PathEl::ClosePath => { last_pt = None; callback(PathEl::ClosePath); }` — `ClosePath` is explicitly re-emitted to the caller's callback, not consumed internally. | H1 ✅, H3 ❌ |
| E2 | Isolated scratch-crate reproduction of the exact pre-fix closure, run against real `kurbo` 0.13.1 | `thread 'main' panicked ...: internal error: entered unreachable code: kurbo::flatten can only return MoveTo and LineTo PathEls` — panics exactly as predicted from E1. | H1 ✅, H3 ❌ |
| E3 | `~/.cargo/registry/.../kurbo-0.13.1/src/rect.rs`, `RectPathIter::next()` (lines ~717-792) | Emits exactly `[MoveTo, LineTo, LineTo, LineTo, ClosePath]` — every `Rect::path_elements()` call ends in `ClosePath`, unconditionally. | H2 ❌ |
| E4 | `examples/minwebgl/lottie_surface_rendering/src/animation.rs` (lines ~165-190) and `~/.cargo/registry/.../velato-0.11.0/src/runtime/model/mod.rs` (lines ~135-159) | `geometry_to_primitive`'s non-`Spline` branch calls `geometry.evaluate(0.0, &mut path); primitive_generation::path_to_points(path)`; `Geometry::evaluate` calls `.path_elements(0.1)` on `Rect`/`Ellipse` — a real, already-wired production call chain, not hypothetical misuse. | H2 ❌ |

## Root Cause

```
path_to_points( path: Vec<PathEl> ) -> Vec<[f32; 2]>
  kurbo::flatten( BezPath::from_vec(path), 0.25, |el| {
    match el {
      MoveTo(p) | LineTo(p) => push [p.x, p.y]
      _ => unreachable!(...)          <- ClosePath lands here; NOT actually unreachable
    }
  })
```

The closure's premise — that `kurbo::flatten` "can only return MoveTo and LineTo PathEls" — was
never verified against `kurbo`'s own source. `flatten` also re-emits `ClosePath` for any subpath
that closes, which is the normal case for any filled/closed shape (rectangles, ellipses, closed
Bezier outlines).

## Why Not Caught

No existing test called `path_to_points` with a `Vec<PathEl>` that includes a `ClosePath` element
— every prior exercised input was an open, unclosed point sequence, so the always-reachable panic
path was never triggered in the test suite. The crate's own doc comment states the function
"converts a `Vec<PathEl>` into a flattened vector of 2D points" with no restriction to open paths,
and the function is `pub`, so nothing in its own contract warned against closed input.

## Fix Location

`module/helper/primitive_generation/src/primitive.rs`, `pub fn path_to_points`. Restructured the
flatten closure to explicitly handle `ClosePath` as a no-op instead of falling into the catch-all:

```rust
// before
let point = match el
{
  PathEl::MoveTo( p ) | PathEl::LineTo( p ) => [ p.x as f32, p.y as f32 ],
  _ => unreachable!( "kurbo::flatten can only return MoveTo and LineTo PathEls" )
};
points.push( point );

// after
match el
{
  PathEl::MoveTo( p ) | PathEl::LineTo( p ) => { points.push( [ p.x as f32, p.y as f32 ] ); },
  PathEl::ClosePath => {}
  _ => unreachable!( "kurbo::flatten can only return MoveTo, LineTo, and ClosePath PathEls" )
}
```

`ClosePath` carries no coordinate of its own and the function's flat `Vec<[f32; 2]>` output has no
subpath-boundary marker to attach it to (a pre-existing, separate design limitation shared by every
`path_to_points` caller, which already treats one call's output as a single implicitly-closed
contour — not something this fix changes or needs to change). Every previously-working open-path
input is bit-for-bit unchanged; only the previously-panicking closed-path input now succeeds.

## Prevention

Added `path_to_points_does_not_panic_on_a_closed_path` and
`path_to_points_accepts_an_open_path` to the new
`tests/path_to_points_test.rs` (gated `required-features = ["text"]` in `Cargo.toml`, matching the
crate's existing `required-features` pattern for feature-gated test binaries).

**Pitfall:** a local comment or assumption about what a dependency "can only return" is not proof
— verify against the dependency's own source before writing an `unreachable!()` arm over its
output, especially when the arm's own message states the (wrong) assumption as fact.

## Generalized Version

**Broken assumption:** "I've enumerated every variant this callback can receive, based on what I
expect the API to produce" — false when the assumption was never checked against the dependency's
actual source, only inferred from the dependency's typical/documented use case.

**Confirmed general rule:** before writing an `unreachable!()` (or any other non-recoverable
assertion) over a dependency's callback/output enum, read the dependency's own implementation of
whatever produces that enum. A closed/filled shape is the *common* case for path data, not a rare
edge case — any path-processing function that only tests open input has not tested its own default
usage pattern.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Discovered via task #63's targeted code review of `primitive_generation`; confirmed reachable by tracing the full `lottie_surface_rendering` -> `velato` -> `kurbo` call chain before filing. |
| 2026-08-15 | fixed | Restructured `path_to_points`'s flatten closure to treat `ClosePath` as a no-op instead of `unreachable!()`. |
| 2026-08-15 | verified | Added 2 tests to `tests/path_to_points_test.rs`; scoped test run (`cargo nextest run --all-features` via `longrun`) passed 9/9 alongside the pre-existing suite. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-15, this verifier 2026-08-16). Independently re-read `path_to_points`'s flatten closure (confirmed `PathEl::ClosePath => {}` genuinely present ahead of the `unreachable!()` catch-all, 3-field comment intact) and `path_to_points_does_not_panic_on_a_closed_path` (non-tautological: asserts the exact 4-point output, not just absence of panic). Fresh `cargo nextest run -p primitive_generation --all-features` via `longrun`: 9/9 passed. `cargo clippy -p primitive_generation --all-features --all-targets -- -D warnings`: clean. `**Related Bugs:** None` confirmed accurate — no overlap with BUG-128/129. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-127/128/129 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs: src/`/`Refs: tests/` present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass accepted the post-fix `cargo run` output at face value; adversarial pass independently reproduced the pre-fix closure body byte-for-byte in an isolated scratch crate against the real pinned `kurbo` dependency, confirming the exact predicted panic rather than trusting the source-read derivation alone. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed no `**Related Bugs:**` overlap with BUG-128/129 — distinct function (`primitive.rs` vs `text/ufo.rs`), distinct root cause. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass independently re-read `kurbo::Rect::path_elements`'s iterator (not just trusted the confirming pass's paraphrase) to confirm `ClosePath` is unconditional, not shape-dependent. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass checked whether any OTHER catch-all `unreachable!()` exists elsewhere in `primitive.rs`/`text/ufo.rs` with the same flaw — none found; this was the only such arm. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `primitive_generation`'s own `src/`/`tests/`/`Cargo.toml` and this bug-tracking file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to `path_to_points`'s own closure; callers (`lottie_surface_rendering`) are unmodified — the fix corrects the callee instead of requiring every caller to pre-filter `ClosePath`. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix does not add any new responsibility — it makes `path_to_points` honor its own doc comment ("Converts a `Vec<PathEl>` into a flattened vector of 2D points") for the input shape it was already documented to accept. | — |

**Reproduced:** YES — isolated reproduction of the exact pre-fix closure body panicked with
`internal error: entered unreachable code: kurbo::flatten can only return MoveTo and LineTo
PathEls` against real `kurbo` 0.13.1, for a 4-corner closed rectangle path, 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/src/primitive.rs` | `path_to_points`: restructured the flatten closure to treat `PathEl::ClosePath` as a no-op instead of falling into `unreachable!()`. `Fix(BUG-127)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/tests/path_to_points_test.rs` | New file: `path_to_points_does_not_panic_on_a_closed_path` (`bug_reproducer(BUG-127)`, 5-section doc comment) and `path_to_points_accepts_an_open_path` (regression guard for the pre-existing open-path case). |
| `module/helper/primitive_generation/Cargo.toml` | Added `[[test]] name = "path_to_points_test" required-features = ["text"]`. |
