# BUG-080: `mingl::geometry::BoundingBox`'s `#[non_exhaustive]` breaks 7 struct-literal construction sites across 2 crates, including the default `cargo check --workspace` build

- **Severity:** High
- **state:** Completed
- **Affects:** `module/helper/primitive_generation` (`src/text/ufo.rs`, 2 sites, under `--features font-processing`/`full`/`--all-features`) and `examples/minwebgl/text_rendering` (`src/text.rs`, 5 sites, unconditionally — no feature gate); combined, breaks the plain `cargo check --workspace` / `cargo build --workspace` default-features command from the repo root
- **Component:** `module/min/mingl` (defect origin: `src/geometry.rs`'s `BoundingBox` struct) × 2 downstream consumer crates (breakage sites)
- **repo_identity:** self
- **Filed:** 2026-08-11
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-11
- **Fixed:** 2026-08-11
- **Accepted By:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self — same-session Tier 2 Dual-Role Self-Check, no separate PROC16 acceptance actor)

## Symptom

```bash
$ cargo check --workspace
...
error[E0639]: cannot create non-exhaustive struct using struct expression
   --> module/helper/primitive_generation/src/text/ufo.rs:368:20
    |
368 |           max_size : BoundingBox
    |  ____________________^
369 | |         {
370 | |           min,
371 | |           max
372 | |         }
    | |_________^

error[E0639]: cannot create non-exhaustive struct using struct expression
  --> module/helper/primitive_generation/src/text/ufo.rs:83:26
   |
83 |         let bounding_box = BoundingBox
   |  __________________________^
84 | |       {
85 | |         min : [ ( x1 + offsetx ) as f32, ( y1 + offsety ) as f32, 0.0 ].into(),
86 | |         max : [ ( x2 + offsetx ) as f32, ( y2 + offsety ) as f32, 0.0 ].into()
87 | |       };
   | |_______^

error: could not compile `primitive_generation` (lib) due to 2 previous errors
```

Independently, `examples/minwebgl/text_rendering` fails the same way, unconditionally (no special
flags needed):

```bash
$ cargo check -p text_rendering
...
error[E0639]: cannot create non-exhaustive struct using struct expression
   --> examples/minwebgl/text_rendering/src/text.rs:337:21
error[E0639]: cannot create non-exhaustive struct using struct expression
   --> examples/minwebgl/text_rendering/src/text.rs:967:20
error[E0639]: cannot create non-exhaustive struct using struct expression
  --> examples/minwebgl/text_rendering/src/text.rs:72:26
error[E0639]: cannot create non-exhaustive struct using struct expression
   --> examples/minwebgl/text_rendering/src/text.rs:384:26
error[E0639]: cannot create non-exhaustive struct using struct expression
   --> examples/minwebgl/text_rendering/src/text.rs:690:20
error: could not compile `text_rendering` (bin "text_rendering") due to 5 previous errors
```

Both reproduced fresh this session (2026-08-11).

## Impact

**Who is affected:** Anyone running the single most basic full-workspace command from the repo
root — `cargo check --workspace` or `cargo build --workspace`, default features, no special
flags — plus anyone building `examples/minwebgl/text_rendering` standalone, plus anyone building
`primitive_generation` with `--features font-processing`/`full`/`--all-features`.

**What breaks:** A hard `E0639` compile failure (not a lint, not a warning) at 7 total call sites
across 2 independent crates:
- `module/helper/primitive_generation/src/text/ufo.rs` — 2 sites (lines 83, 368) — only reachable
  when `font-processing` (or `full`, or `--all-features`) is active. `cargo check -p
  primitive_generation` alone (bare default features) is clean (confirmed fresh this session: `cargo
  check -p primitive_generation` → `Finished` in 12.00s, zero errors). But `cargo check --workspace`
  activates the feature anyway via Cargo's workspace-wide feature unification: 4 other example
  crates (`lottie_surface_rendering`, `animation_surface_rendering`, `curve_surface_rendering`,
  `character_control` — per `task/completed/055`'s own consumer survey) request
  `primitive_generation` with `font-processing`/`full`, and Cargo unifies that across the whole
  `--workspace` graph even for what looks like a "default features" build from the caller's point of
  view.
- `examples/minwebgl/text_rendering/src/text.rs` — 5 sites (lines 72, 337, 384, 690, 967) —
  unconditional, no feature gate at all. This crate depends on `mingl` directly (it has its own
  independent UFO/text-rendering implementation, not routed through `primitive_generation`) and
  breaks on a bare `cargo check -p text_rendering`, every time, regardless of features.

Combined, these two independently-broken crates mean **`cargo check --workspace` (the plain,
default, most-basic full-workspace command) currently fails** — confirmed directly this session.

**Why High, not Critical:** unlike BUG-007 (Critical — broke literally every cargo invocation
workspace-wide via `Cargo.lock` resolution, no possible workaround), the vast majority of this
~100+-crate workspace's individual `cargo check -p <crate>` invocations for crates outside these two
are completely unaffected — this is a severe, workspace-visible break (the default aggregate
`--workspace` command fails) but not a total blockage of all possible commands.

**Entity Scope:** `None` — struct API surface break, not entity directory instances.

## How Discovered

Independently self-documented as a Non-Blocking, out-of-scope finding in the `## Verification`
sections of **four** already-completed tasks that each re-ran a build command touching one of the
affected files and hit this regression fresh, none of which filed a dedicated tracker for it:
`task/completed/036` (§ I4 — first to characterize the `--workspace` default-feature-unification
mechanism), `task/completed/018` (§ I2), `task/completed/021` (§ I3), and `task/completed/055`
(§ I2/I4) — all four independently trace the same root cause (commit `5f33be66`) and the same 2
`ufo.rs` sites. `task/completed/039`'s health.md capstone additionally names the `text_rendering`
5-site manifestation in its own "known open issues" list, but likewise files no tracker. Surfaced
for filing during this session's TA106 (`## Verification` section) task-normalization sweep, whose
own out-of-scope-findings triage is what prompted checking whether any of these mentions had ever
been promoted to a trackable bug — confirmed via `grep -rl "E0639" task/` plus `grep -rl
"non_exhaustive" task/` that none had.

## Minimum Reproducible Example

No synthetic MRE needed — the real workspace reproduces it directly and deterministically:

```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo check --workspace
# or, isolated to one crate each:
cargo check -p primitive_generation --features font-processing
cargo check -p text_rendering
```

**Verify Command:** any of the three above; **Expected** (once fixed): exit 0; **Actual:** exit 101
for all three, `error[E0639]: cannot create non-exhaustive struct using struct expression` (verbatim
output in `## Symptom`).

## Root Cause

Commit `5f33be66` ("feat: consolidate test infrastructure and refactor module architecture",
2026-08-11) added `#[ non_exhaustive ]` to `mingl::geometry::BoundingBox`:

```rust
#[ derive( Debug, Clone, Copy ) ]
#[ non_exhaustive ]
pub struct BoundingBox
{
  pub min : F32x3,
  pub max : F32x3
}
```

(`module/min/mingl/src/geometry.rs:14-20`; git-blamed directly this session — `git log -L
14,15:module/min/mingl/src/geometry.rs` → `5f33be66954da3ee478cb9ccb2d473e2378f92f6 2026-08-11`.)

`#[non_exhaustive]` on a struct with 100%-public fields forbids direct struct-literal construction
(`BoundingBox { min, max }`) from outside the defining crate — by design, this is what
`#[non_exhaustive]` means for structs (only relevant when the type may later gain a private field;
here every field is already `pub`, so the attribute buys no future flexibility that a public
constructor doesn't already provide). `mingl` already ships the replacement — a public constructor
at `src/geometry.rs:45`, `pub fn new< T : Into< F32x3 > >( min : T, max : T ) -> Self` — and this
constructor is already the established pattern used correctly elsewhere in the workspace, confirmed
this session: `module/helper/renderer/src/webgl/loaders/gltf.rs:857` and all of `mingl`'s own
`tests/tests/bounding_box.rs` already call `BoundingBox::new(min, max)`, not the struct-literal
form.

The commit added `#[non_exhaustive]` without auditing or updating the two crates whose code still
used the pre-existing struct-literal form: `primitive_generation/src/text/ufo.rs` (2 sites) and
`text_rendering/src/text.rs` (5 sites) — neither file was touched by `5f33be66` itself (confirmed:
neither path appears in `git show 5f33be66 --stat`).

## Why Not Caught

This workspace's standard verification commands (`will .test level::3`, `cargo nextest`, `cargo
clippy --all-features`) were not run against a `--workspace`-scoped default-feature build
specifically at the moment `5f33be66` landed — and once other, unrelated per-crate work later in the
session started reaching `--all-features`/`--workspace` builds that actually compile
`ufo.rs`/`text.rs` for the first time since that commit, the same regression surfaced
independently, four separate times, in four unrelated tasks' Verification sections (036, 018, 021,
055) — each one confirming it's real, root-causing it identically, and each correctly judging it out
of that specific task's own scope to fix. None of the four promoted it to a dedicated,
independently-trackable bug report; each treated it as a one-off aside within an unrelated task's
own file.

**Pitfall:** a defect independently rediscovered by multiple unrelated verification passes, each of
which correctly disclaims it as "not my scope," is not the same as a defect that is actually being
tracked — if no single one of those passes also files a dedicated tracker, the defect can be
rediscovered indefinitely without ever being scheduled for an actual fix. Treat "this is the Nth
time I've independently found the same root-caused regression in an unrelated task's Verification
section" as a specific trigger to check for (and, if absent, file) a dedicated tracker — not just to
note it again in the Nth+1 place.

## Fix Applied

Option 1 below is what landed — all 7 call sites now construct via `BoundingBox::new( min, max )`:

- `examples/minwebgl/text_rendering/src/text.rs` — all 5 sites switched by the task-058
  all-warnings sweep lane (2026-08-11), as part of that sweep's text_rendering pass.
- `module/helper/primitive_generation/src/text/ufo.rs` — both sites observed already switched to
  `BoundingBox::new` as of commit `96bb2aef` ("feat: consolidate GPU HAL adoption, modernize
  examples, and expand test infrastructure", 2026-08-11); fixed upstream of this closure, not by
  the sweep lane.

**Verification (2026-08-11):**
- `cargo check -p text_rendering` → exit 0, `Finished` (previously exit 101, 5×E0639).
- `cargo check -p primitive_generation --features font-processing` → exit 0, `Finished` (previously exit 101, 2×E0639).
- The session's 4-phase workspace gate (host clippy `--workspace --all-features -D warnings`, with
  only the live-claim `tilemap_renderer` cone excluded; wasm32 clippy over 20 examples including
  `text_rendering`; `--workspace` nextest 1220/1220; `--workspace` doc tests) → all green.

The original candidate approaches, kept for the record:

1. **Switch all 7 call sites to `BoundingBox::new(min, max)`** — the already-available,
   already-idiomatic-elsewhere constructor. Cheapest, lowest-risk option: no API change needed
   anywhere, purely a caller-side mechanical edit. `primitive_generation/src/text/ufo.rs:83-87` and
   `:368-372`; `text_rendering/src/text.rs:72, 337, 384, 690, 967` (exact per-site shape needs a
   quick read at pickup — some may be direct constructions, others may be reachable via a small
   local helper).
2. **Revert `#[non_exhaustive]`** on `BoundingBox` — viable since every field is already `pub` (the
   attribute currently buys no forward-compatibility benefit that isn't already lost by the fields
   being public), but this is a judgment call on `mingl`'s own API-evolution intent, not purely
   mechanical — worth checking whether `5f33be66`'s own commit message or any doc explains why
   `non_exhaustive` was added before choosing this over option 1.

Option 1 is very likely correct given the precedent already established elsewhere in the codebase
(`gltf.rs`, `mingl`'s own tests) — but recorded as a choice, not pre-decided, since it touches `src/`
outside this session's scope.

## Generalized Version

**Broken assumption:** "adding `#[non_exhaustive]` to a struct is a purely additive,
backward-compatible change." False whenever the struct has 100%-public fields and existing callers
construct it via struct-literal syntax — `#[non_exhaustive]` specifically forbids that construction
form from outside the defining crate, regardless of field visibility. A workspace-wide `grep` for
`StructName\s*{` (struct-literal construction, not just type mentions) across all downstream crates
is the concrete check to run before landing `#[non_exhaustive]` on any existing public struct.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-11 | filed | Discovered as an already-four-times-independently-rediscovered, never-tracked regression (tasks 036/018/021/055's own Verification sections, plus health.md's known-issues list) during this session's TA106 out-of-scope-findings triage. Re-verified fresh directly against source (not trusted from the prior mentions alone): confirmed 7 total call sites across 2 crates (not the 2 originally assumed), confirmed `cargo check --workspace` itself currently fails (not just an opt-in-feature edge case), confirmed the replacement constructor (`BoundingBox::new`) already exists and is already the established pattern elsewhere. Left in Draft/unfixed state — fixing requires `src/` edits in 2 downstream crates, outside this filing session's own edit scope. |
| 2026-08-11 | fixed + completed | Option 1 landed at all 7 sites: `text_rendering/src/text.rs` ×5 by the task-058 all-warnings sweep lane; `primitive_generation/src/text/ufo.rs` ×2 observed already fixed in commit `96bb2aef` (upstream of this closure). Both per-crate Verify Commands re-run fresh → exit 0; workspace gate green (see Fix Applied). Closed same-session, Round 0, self-accepted per BUG-079 precedent. |
