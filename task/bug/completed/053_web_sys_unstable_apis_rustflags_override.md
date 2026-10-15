# BUG-053: An explicit `RUSTFLAGS` override silently disables `web_sys_unstable_apis`, flipping `get_image_data` and `MouseEvent` coordinate accessors between two incompatible web-sys signatures

- **Severity:** High
- **state:** Completed
- **Affects:** `module/min/minwebgl::texture::d2::upload_sprite` (core library — sprite-sheet texture loading, the `get_image_data` call); `module/helper/browser_input::input::{CLIENT, PAGE, SCREEN}` (pointer-coordinate accessor statics — `client_x`/`client_y`, `page_x`/`page_y`, `screen_x`/`screen_y`); and mouse-coordinate handling in 3 downstream example binaries — `examples/minwebgl/hexagonal_map`, `examples/minwebgl/object_picking`, `examples/minwebgl/filter`
- **Component:** `module/min/minwebgl` + `module/helper/browser_input` + `examples/minwebgl/{hexagonal_map,object_picking,filter}`
- **repo_identity:** self
- **Filed:** 2026-08-10
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-10
- **Fixed:** 2026-08-10
- **Accepted By:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self — same-session Tier 2 Dual-Role Self-Check, no separate PROC16 acceptance actor)

## Symptom

```bash
# real output captured this session, mid-investigation (module/min/minwebgl/src/texture/d2.rs
# temporarily held a single-branch `f64` fix at this point — see ## How Discovered)
$ RUSTFLAGS="-D warnings" cargo nextest run --all-features
   Compiling minwebgl v0.3.0 (/home/user1/pro/lib/yrd_gamedev/cgtools/module/min/minwebgl)
error[E0308]: arguments to this method are incorrect
    --> module/min/minwebgl/src/texture/d2.rs:290:20
     |
 290 |     let data = ctx.get_image_data( 0.0, 0.0, img_width as f64, img_height as f64 ).unwrap().data().to_vec();
     |                    ^^^^^^^^^^^^^^  ---  ---  ----------------  ----------------- expected `i32`, found `f64`
     |                                    |    |    |
     |                                    |    |    expected `i32`, found `f64`
     |                                    |    expected `i32`, found floating-point number
     |                                    expected `i32`, found floating-point number
     |
note: method defined here
    --> .../web-sys-0.3.104/src/features/gen_CanvasRenderingContext2d.rs:1496:12
     |
1496 |     pub fn get_image_data(
     |            ^^^^^^^^^^^^^^

error: could not compile `minwebgl` (lib) due to 1 previous error
```

Reverting the same line to `i32` arguments instead makes this exact command pass — but then a
plain `cargo check -p minwebgl` (no `RUSTFLAGS` override) fails with the mirror-image error
("expected `f64`, found `i32`") against the same line. Neither form is simply "correct" —
each is correct only under one of two mutually exclusive build configurations.

## Impact

**Who is affected:** Any contributor building this workspace. The set of argument/return types
that type-check at these 5 files' call sites depends entirely on whether `web_sys_unstable_apis`
is active, and two individually ordinary invocation styles land on opposite sides of that flag:

- `cargo check -p <crate>` / `cargo build` / an IDE's `rust-analyzer` background check (no
  `RUSTFLAGS` override) — `web_sys_unstable_apis` stays **ON**, via `.cargo/config.toml`.
- `RUSTFLAGS="-D warnings" cargo nextest run --all-features` — **this project's own documented
  Level 1 final-verification command** — silently turns `web_sys_unstable_apis` **OFF** (see
  `## Root Cause`).

**What breaks:** A total, immediate compile failure (`error[E0308]`) of the affected crate the
moment anyone runs whichever of the two invocation styles the code does not currently match.
Not a subtle runtime defect — a full build failure, loud and immediate.

**Why High, not Critical/Medium:** No end-user runtime behavior is affected by the choice of
signature itself (both branches compute the same thing once compiling), which rules out
Critical. But unlike a bug that only manifests for a caller that doesn't exist yet, this defect
manifests *immediately* and *unconditionally* the instant a contributor runs either of two
completely ordinary commands — and one of those two commands is this project's own mandated
Level 1 verification gate. The 8-distinct-commits regression history (`## History`) shows this
has already caused real, repeated engineering time loss across roughly five months, which is
why this is High rather than Medium (matching the severity precedent set by BUG-046, another
compile-breaking defect).

**Entity Scope:** `None` — ordinary source files, not entity directory instances.

## How Discovered

During this session's final full-workspace verification of five unrelated, already-completed
items (BUG-043, BUG-046, Task 044, Task 047, Task 048 — see their own `task/*/completed/`
reports) — re-running the project's mandated Level 1 command
(`RUSTFLAGS="-D warnings" cargo nextest run --all-features`) surfaced a fresh compile failure in
`module/min/minwebgl/src/texture/d2.rs`, a file none of those five items had touched. A
single-branch fix matching the immediate error (switching the literal arguments to plain `f64`)
broke in the exact opposite direction the moment the *same* command was re-run again — proving
the required argument types flip based on environment state, not on any source change.

A workspace-wide grep sweep for the same class of API
(`client_x\(\)|client_y\(\)|screen_x\(\)|screen_y\(\)|page_x\(\)|page_y\(\)|movement_x\(\)|movement_y\(\)|get_image_data\(`)
found 6 further call sites beyond `d2.rs`. Three were confirmed genuinely safe by pre-existing
idiom (`module/min/mingl/src/controls/camera_orbit_controls.rs` casts `screen_x()`/`screen_y()`
to `f32`, `module/min/mingl/src/controls/character_controls.rs` casts the *unconditionally*
`i32`-typed `movement_x()`/`movement_y()` to `f64` — see `## Evidence Table` E7 — neither cast
is ever an identity conversion regardless of the cfg branch). Three more —
`examples/minwebgl/hexagonal_map`, `examples/minwebgl/object_picking`,
`examples/minwebgl/filter` — were confirmed broken by direct compiler error and fixed alongside
`d2.rs`.

A **4th, initially-missed** call site surfaced only after those first 4 files were fixed: a
scoped clippy re-verification (`cargo clippy -p minwebgl -p hexagonal_map -p object_picking
-p filter --all-targets --all-features -- -D warnings`, run to confirm the fix under the
opposite cfg direction) surfaced 6 new `clippy::unnecessary_cast` errors in
`module/helper/browser_input/src/input.rs`. The original sweep had only checked "does this
compile" (true in both directions for this file, since `as i32` on either an `i32` or an `f64`
source is always valid Rust) — it had not checked *clippy* in both directions, where `i32 as
i32` is flagged as a redundant identity cast under `-D warnings`. This expanded the fix from 4
files to 5 (`## Fix Location`).

## Minimum Reproducible Example

Fully self-contained — plain `rustc`, no cargo project, no external crates. `web_sys`'s real
`CanvasRenderingContext2d`/`MouseEvent` types require a `wasm32` + browser target unreachable
from a plain `rustc` invocation, so this MRE reproduces the exact defect *pattern* with a
minimal stand-in mirroring web-sys's own two `#[cfg(web_sys_unstable_apis)]`-gated overloads
(see `## Evidence Table` E2 for the real signatures this stand-in mirrors):

```bash
mkdir -p /tmp/mre053
cat > /tmp/mre053/repro.rs <<'EOF'
mod websys_stub
{
  #[ cfg( not( web_sys_unstable_apis ) ) ]
  pub fn get_image_data( sx : f64, sy : f64 ) -> f64 { sx + sy }

  #[ cfg( web_sys_unstable_apis ) ]
  pub fn get_image_data( sx : i32, sy : i32 ) -> i32 { sx + sy }
}

fn upload_sprite() -> f64
{
  // A single, non-cfg-branched call site can only ever match ONE of the two signatures.
  websys_stub::get_image_data( 0.0, 0.0 )
}

fn main() { println!( "{}", upload_sprite() ); }
EOF
rustc --edition 2021 /tmp/mre053/repro.rs -o /tmp/mre053/repro_off 2>&1
echo "cfg OFF exit: $?"
rustc --edition 2021 --cfg web_sys_unstable_apis /tmp/mre053/repro.rs -o /tmp/mre053/repro_on 2>&1
echo "cfg ON exit: $?"
```

**Expected** (once fixed — i.e. the call site branches on the same cfg the callee does):
```
cfg OFF exit: 0
cfg ON exit: 0
```

**Actual** (unfixed — a single, non-cfg-branched call site):
```
cfg OFF exit: 0
error[E0308]: arguments to this function are incorrect
  --> /tmp/mre053/repro.rs:13:3
   |
13 |   websys_stub::get_image_data( 0.0, 0.0 )
   |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^  ---  --- expected `i32`, found floating-point number
   |                                |
   |                                expected `i32`, found floating-point number
   |
note: function defined here
  --> /tmp/mre053/repro.rs:7:10
   |
 7 |   pub fn get_image_data( sx : i32, sy : i32 ) -> i32 { sx + sy }
   |          ^^^^^^^^^^^^^^  --------  --------

error[E0308]: mismatched types
  --> /tmp/mre053/repro.rs:13:3
   |
10 | fn upload_sprite() -> f64
   |                       --- expected `f64` because of return type
...
13 |   websys_stub::get_image_data( 0.0, 0.0 )
   |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `f64`, found `i32`
error: aborting due to 2 previous errors
cfg ON exit: 1
```

**Verify Command:** `rustc --edition 2021 --cfg web_sys_unstable_apis /tmp/mre053/repro.rs -o /tmp/mre053/repro; test $? -eq 1` —
**What:** demonstrates that a single call site with literal argument types can only ever satisfy
one of two `#[cfg(...)]`-gated, mutually exclusive signatures, reproducing the exact invariant
violation present at all 5 real call sites (`## Fix Location`).

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The literal argument/cast types at these call sites are simply wrong against a single, fixed web-sys signature — an ordinary typo | ❌ Disproved | Both the `i32` and the `f64` forms compile successfully — just under different, mutually exclusive build configurations; neither is unconditionally wrong | E2, E3, E4 |
| H2 | `web_sys_unstable_apis` is unconditionally active in this workspace, since `.cargo/config.toml`'s `[build] rustflags` sets it | ❌ Disproved | `.cargo/config.toml` does set it, but an explicit `RUSTFLAGS` env var (as set by this project's own Level 1 command) replaces — not merges with — `[build] rustflags`, silently dropping the cfg | E1, E4 |
| H3 | An explicit `RUSTFLAGS` environment variable completely replaces (never merges with) `.cargo/config.toml`'s `[build] rustflags`, silently turning `web_sys_unstable_apis` off whenever a caller sets `RUSTFLAGS` directly — flipping which of web-sys's two `#[cfg(web_sys_unstable_apis)]`-gated signatures applies for `get_image_data`, `client_x`/`client_y`, `page_x`/`page_y`, and `screen_x`/`screen_y` | ✅ Root Cause | Direct reproduction this session: the same source line fails in opposite directions depending solely on whether `RUSTFLAGS` was set on the invoking command, never on any code change | E1, E2, E3, E4, E5 |
| H4 | `module/min/minwebgpu/build.rs`'s unconditional `println!("cargo:rustc-cfg=web_sys_unstable_apis")` is what actually controls the cfg workspace-wide, making this a build-script propagation bug rather than a `RUSTFLAGS`/`config.toml` conflict | ❌ Disproved | Cargo scopes a build script's `cargo:rustc-cfg` output to the emitting crate only; `minwebgl`, `browser_input`, and the 3 example crates do not depend on `minwebgpu` and are unaffected by its build script | E6 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `.cargo/config.toml:4-5` and `:7-8` | `[build] rustflags = ["--cfg", "web_sys_unstable_apis"]` (and again under `[target.wasm32-unknown-unknown]`) — intends the cfg always-on | H2 ❌ (contradicted by E4), H3 ✅ |
| E2 | `web-sys-0.3.104/src/features/gen_CanvasRenderingContext2d.rs:1460-1498` | Two `#[cfg(...)]`-gated `get_image_data` overloads on the *same* method name: `#[cfg(not(web_sys_unstable_apis))]` → `fn get_image_data(sx: f64, sy: f64, sw: f64, sh: f64)` at line 1473; `#[cfg(web_sys_unstable_apis)]` → `fn get_image_data(sx: i32, sy: i32, sw: i32, sh: i32)` at line 1496 | H1 ❌, H3 ✅ |
| E3 | `web-sys-0.3.104/src/features/gen_MouseEvent.rs:37-52` and `:192-213` | Same dual-cfg pattern for `MouseEvent::client_x`/`client_y`: `i32` when off (lines 44, 52), `f64` when on (lines 202, 213); `page_x`/`page_y` (lines 90, 98 / 246, 257) and `screen_x`/`screen_y` (lines confirmed via the same file) follow the identical pattern. `movement_x`/`movement_y` (lines 162, 169) carry **no** `#[cfg(...)]` gate at all — always `i32` — which is why `character_controls.rs`'s cast to `f64` needed no fix (H1 disproof, by contrast) | H1 ❌, H3 ✅ |
| E4 | Direct reproduction this session (captured verbatim in `## Symptom`) | `RUSTFLAGS="-D warnings" cargo nextest run --all-features` fails `error[E0308]` at `d2.rs:290` — "expected `i32`, found `f64`" — while a `cargo check -p minwebgl` with no override accepts that exact `f64` form and instead rejects the `i32` form | H2 ❌, H3 ✅ |
| E5 | `module/min/mingl/src/controls/camera_orbit_controls.rs` (pre-existing code comment, predates this bug) | "screen_x/y return f64 under web_sys_unstable_apis (web-sys ≥ 0.3.94)" — independent, prior confirmation of the identical dual-signature phenomenon on a sibling call site, by a different author, already worked around there via a cast to `f32` that is never an identity conversion in either direction | H3 ✅ |
| E6 | `module/min/minwebgpu/build.rs` | `println!("cargo:rustc-cfg=web_sys_unstable_apis")` — unconditional, but scoped by Cargo to the emitting crate (`minwebgpu`) only; `minwebgl`/`browser_input`/the 3 examples do not depend on it | H4 ❌ |
| E7 | Workspace grep sweep (`## How Discovered`) | Exactly 7 call sites workspace-wide beyond `d2.rs`; 3 safe by construction (`camera_orbit_controls.rs`'s `as f32`, `character_controls.rs`'s `movement_x/y` — never cfg-gated per E3 — cast to `f64`, and an equivalent always-`i32` idiom), 4 confirmed broken and fixed (`hexagonal_map`, `object_picking`, `filter`, `browser_input`) | Defines full `## Fix Location` scope |

## Root Cause

`.cargo/config.toml` sets `[build] rustflags = ["--cfg", "web_sys_unstable_apis"]`, intending
that cfg to be active for every build in this workspace (E1). Cargo, however, treats an
explicit `RUSTFLAGS` environment variable as a **complete replacement** for `[build] rustflags`,
never a merge. This project's own documented Level 1 final-verification command,
`RUSTFLAGS="-D warnings" cargo nextest run --all-features`, sets `RUSTFLAGS` directly — which
silently discards `.cargo/config.toml`'s `--cfg web_sys_unstable_apis` for the whole invocation,
without any warning that the substitution happened (E4).

`web_sys_unstable_apis` gates two *independent* pairs of mutually exclusive signatures inside
web-sys itself (E2, E3):

```
get_image_data( sx, sy, sw, sh )     f64×4  when web_sys_unstable_apis is OFF
                                      i32×4  when web_sys_unstable_apis is ON

client_x() / client_y()              i32    when OFF
page_x()   / page_y()                f64    when ON
screen_x() / screen_y()
```

A call site with literal-typed arguments (or a cast to a literal target type) can only ever
satisfy one branch. Since ordinary, everyday commands (`cargo check`, an IDE's background
check) leave `RUSTFLAGS` unset and thus land on the `.cargo/config.toml` default (ON), while
this project's own Level 1 command silently flips it OFF, a call site "fixed" under one
invocation style immediately breaks under the other — with no code change involved, only a
change in which command happened to run last (**H3 ✅**, confirmed by elimination of H1, H2,
H4).

## Why Not Caught

Two gaps compounded to let this recur at least 8 times over roughly five months (`## History`):

1. **No single invocation style is authoritative.** `cargo check -p <crate>` (fast local
   iteration, IDE background checks — cfg ON) and `RUSTFLAGS="-D warnings" cargo nextest run
   --all-features` (this project's own Level 1 command — cfg OFF) are both completely ordinary,
   both individually "correct" ways to build this workspace, and neither is inherently more
   authoritative than the other — yet they silently disagree on which of two mutually exclusive
   signatures is valid. A contributor who only ever runs one style has no way to discover the
   other style would reject their change.
2. **The failure mode looks like an ordinary,易-to-"fix" type error**, not a systemic
   configuration conflict. `error[E0308]: expected i32, found f64` reads exactly like a
   plain argument-type typo, inviting a single-branch literal fix — which is precisely what
   happened at least 8 times in `d2.rs`'s git history (`## History`), each time "fixing" the
   error for whichever invocation style the fixer happened to be using, at the cost of
   immediately breaking the other.

**Pitfall:** `cargo check -p <crate>` (no `RUSTFLAGS` override) and `RUSTFLAGS="-D warnings"
cargo nextest run --all-features` (this project's own Level 1 command) can resolve the *same*
web-sys method call to *opposite*, mutually exclusive overloads in the *same* workspace. Never
assume one invocation style's success says anything about the other's — for any call site
gated by `web_sys_unstable_apis` (or any other raw `--cfg` flag set via `.cargo/config.toml`'s
`[build] rustflags]`), verify both directions explicitly before considering a fix complete.

## Fix Location

Five files, one root cause. `d2.rs` needs a genuine dual-cfg branch (the argument *values*
differ between branches); the other four need only a targeted lint suppression (the cast is
inherent to satisfying both branches, not a design choice):

```
module/min/minwebgl/src/texture/d2.rs:301-304        — get_image_data dual-cfg branch
examples/minwebgl/hexagonal_map/src/main.rs:262-263   — client_x()/client_y() via .into()
examples/minwebgl/object_picking/src/main.rs:120-123  — client_x()/client_y() via .into()
examples/minwebgl/filter/src/main.rs:77-80            — client_x()/client_y() via as f64
module/helper/browser_input/src/input.rs:180-217      — client_x/y, page_x/y, screen_x/y via as i32
```

## Fix Applied

**`d2.rs`** — replaced the single-branch call with two `#[cfg(web_sys_unstable_apis)]` /
`#[cfg(not(web_sys_unstable_apis))]`-gated `let` statements, one per signature, plus a
function-level `#[allow(unexpected_cfgs)]` on the enclosing `upload_sprite` (item-level
placement per `rulebook.md § #![allow] and #[allow] attributes` — **note:** a per-statement
placement directly on the `#[cfg(...)]`-gated `let` was tried first and empirically did **not**
suppress the error under `-D warnings`; only function-level placement worked):

```rust
#[ cfg( web_sys_unstable_apis ) ]
let data = ctx.get_image_data( 0, 0, img_width as i32, img_height as i32 ).unwrap().data().to_vec();
#[ cfg( not( web_sys_unstable_apis ) ) ]
let data = ctx.get_image_data( 0.0, 0.0, img_width as f64, img_height as f64 ).unwrap().data().to_vec();
```

**`hexagonal_map/src/main.rs`** and **`object_picking/src/main.rs`** — widened `client_x()`/
`client_y()` through `.into()` (valid in both directions: `i32: Into<f64>` widens, `f64:
Into<f64>` is identity), with an item-level `#[allow(clippy::useless_conversion)]` on each
affected `let` statement for the direction where the conversion is a no-op:

```rust
#[ allow( clippy::useless_conversion ) ]
let coord = gl::F64x2::new( e.client_x().into(), e.client_y().into() ) * dpr;
```

**`filter/src/main.rs`** — same widening via an explicit `as f64` (this file's existing style
subtracts `rect.left()`/`rect.top()`, both always `f64`, before a final `as f32`), with an
item-level `#[allow(clippy::unnecessary_cast)]` per statement:

```rust
#[ allow( clippy::unnecessary_cast ) ]
let x = ( e.client_x() as f64 - rect.left() ) as f32;
```

**`browser_input/src/input.rs`** — `PointerEvent` derefs to `MouseEvent` (`extends =
"MouseEvent"` in web-sys's own macro invocation), so its `CLIENT`/`PAGE`/`SCREEN` statics'
existing `as i32` casts inherit the same dual-signature issue. Added a second, item-level
`#[allow(clippy::unnecessary_cast)]` beside each static's existing
`#[allow(clippy::cast_possible_truncation)]`:

```rust
#[ allow( clippy::cast_possible_truncation ) ]
#[ allow( clippy::unnecessary_cast ) ]
pub static SCREEN : fn( &PointerEvent ) -> I32x2 = | event |
{
  I32x2::from_array( [ event.screen_x() as i32, event.screen_y() as i32 ] )
};
```

All five `#[allow]` placements follow `rulebook.md § #![allow] and #[allow] attributes in
source files`'s preferred suppression order (narrowest scope first) — each is scoped to the
single item/statement carrying the false-positive lint, not a file-level block, since none of
these is a whole-file macro-expansion concern.

**Verification (both cfg directions, `## Verification Record` has the full table):**

```bash
RUSTFLAGS="-D warnings" cargo nextest run --all-features   # cfg OFF direction — exit 0, 1150/1150
cargo check --workspace --all-features                     # cfg ON direction (no override) — exit 0
RUSTFLAGS="-D warnings" cargo clippy -p minwebgl -p hexagonal_map -p object_picking \
  -p filter -p browser_input --all-targets --all-features -- -D warnings   # cfg OFF — clean
cargo clippy -p minwebgl -p hexagonal_map -p object_picking \
  -p filter -p browser_input --all-targets --all-features -- -D warnings  # cfg ON — clean
```

## Prevention

Any final verification of this workspace should exercise **both** cfg directions, not just the
documented Level 1 command — since `.cargo/config.toml`'s `[build] rustflags` is silently
defeated by that command's own `RUSTFLAGS` override:

```bash
RUSTFLAGS="-D warnings" cargo nextest run --all-features   # exercises web_sys_unstable_apis OFF
cargo check --workspace --all-features                     # exercises web_sys_unstable_apis ON (no override)
```

Both should exit 0. This is a recommendation for how future verification passes catch this
class of defect early — not a scope or tooling change made as part of this fix.

**Pitfall:** A raw `--cfg` flag set only via `.cargo/config.toml`'s `[build] rustflags` (as
opposed to a Cargo *feature*) is invisible to, and silently overridden by, any command that
sets the `RUSTFLAGS` environment variable directly — including this project's own Level 1
command. A green run of exactly one of the two directions says nothing about the other.

## Generalized Version

**Broken assumption:** "If `.cargo/config.toml` sets a `[build] rustflags` cfg, that cfg is
active for every build in this workspace." False whenever any invocation sets the `RUSTFLAGS`
environment variable directly — Cargo replaces, never merges, `[build] rustflags` with an
explicit `RUSTFLAGS` override. Any call site whose valid argument/return types depend on such a
cfg must branch on that same cfg explicitly; a literal-typed call site can never be correct
under both configurations at once.

**Detection invariant:**
```
for every raw `--cfg NAME` flag declared in `.cargo/config.toml`'s `[build] rustflags`,
there exists at least one documented verification command that runs with `RUSTFLAGS` set
directly (turning the cfg off) AND at least one that runs with no `RUSTFLAGS` override
(leaving the cfg on) — and both are exercised on a recurring cadence, not "assumed
equivalent because one of them passed."
```

## Verification

### Checklist

- [x] C1 — Are the claimed dual `#[allow(...)]` pairs (`cast_possible_truncation` + `unnecessary_cast`) genuinely present on all 3 of `CLIENT`/`PAGE`/`SCREEN` in `input.rs`, each backed by a `Fix(BUG-053)` comment? Read in full: lines 201/206 (`CLIENT`), 214/219 (`PAGE`), 227/232 (`SCREEN`) — all 3 statics carry both allows plus a 4-line `Fix(BUG-053)` comment explaining the dual-signature mechanism.
- [x] C2 — Does clippy pass clean under BOTH `web_sys_unstable_apis` cfg directions for `browser_input` — the exact mechanism this bug is about? `cargo clippy -p browser_input --all-targets --all-features -- -D warnings` (no `RUSTFLAGS` override — cfg ON via `.cargo/config.toml`) → exit 0; `RUSTFLAGS="-D warnings" cargo clippy -p browser_input --all-targets --all-features -- -D warnings` (cfg OFF — the override that silently drops `.cargo/config.toml`'s cfg) → exit 0. Both clean.
- [x] C3 — Is the underlying `as i32` cast itself unchanged (the fix is the lint suppression, not a behavior change)? Confirmed present verbatim: `event.client_x() as i32`, `event.page_x() as i32`, `event.screen_x() as i32` (and their `_y` counterparts) in the current `CLIENT`/`PAGE`/`SCREEN` bodies.
- [x] C4 — Does the fix comment correctly cite the dual-signature root cause rather than a generic suppression note? Confirmed: each comment states "`PointerEvent` derefs to `MouseEvent`, whose `client_x`/`client_y` return `i32` or `f64` depending on `web_sys_unstable_apis` ... `as i32` is a real truncating cast in the `f64` case and a same-type identity cast clippy calls 'unnecessary' in the `i32` case — both are the same source line" — matches `## Root Cause`'s explanation exactly.

### Measurements

- [x] M1 — `#[allow]` attribute count across `CLIENT`/`PAGE`/`SCREEN` combined: `6` now (2 each × 3 statics) vs `3` before the fix commit (`git show 9b71cf39^:module/helper/browser_input/src/input.rs` shows only `#[ allow( clippy::cast_possible_truncation ) ]` on each of the 3, no `unnecessary_cast`).

### Invariants

- [x] I1 — Test suite (crate-scoped): `cargo test -p browser_input --all-features` → exit 0; unittests 0/0, `active_pointers_test` 7/7, `pointer_type_test` 6/6, doc-tests 0/0.
- [x] I2 — Compiler/lints clean, cfg ON direction (default — no override, `.cargo/config.toml`'s `web_sys_unstable_apis` active): `cargo clippy -p browser_input --all-targets --all-features -- -D warnings` → exit 0, zero warnings.
- [x] I3 — Compiler/lints clean, cfg OFF direction (this bug's own mechanism — an explicit `RUSTFLAGS` replaces, not merges with, `.cargo/config.toml`'s cfg): `RUSTFLAGS="-D warnings" cargo clippy -p browser_input --all-targets --all-features -- -D warnings` → exit 0, zero warnings.

### Anti-faking checks

- [x] AF1 — Guards against a future edit "cleaning up" one of the two `#[allow]` attributes as apparently-redundant (a very plausible-looking wrong simplification, since each is dead code under exactly one of the two directions and live under the other): re-running I2 AND I3 together after any edit to `CLIENT`/`PAGE`/`SCREEN` — a single-direction clippy pass cannot detect this regression by definition, which is precisely the failure mode behind the original 8-commit flip-flop history (`## History`).
- [x] AF2 — Guards against this same defect class recurring at a different, not-yet-fixed call site: the `## Prevention` section's detection invariant (any `--cfg NAME` set via `.cargo/config.toml`'s `[build] rustflags` needs verification exercising both the flag-on and flag-off direction) is a workspace-wide policy, not a one-time fix — a new `browser_input` call site added against a `web_sys_unstable_apis`-gated web-sys method without a dual-cfg branch or a widening cast reintroduces this exact bug under a new location.

## History

| Date       | Event  | Notes                                                                                                     |
|------------|--------|-------------------------------------------------------------------------------------------------------------|
| 2026-03-06 to 2026-08-09 | pre-existing regression | `d2.rs`'s `get_image_data` call flip-flopped between bare `f64` and `i32` argument literals across at least 8 distinct commits, each "fixing" the immediately preceding break for one invocation style while reintroducing it for the other: `fbd8b89b` (2026-03-06, f64→i32), `009b1b01` (2026-03-26, i32→f64), `36fa3013` (2026-03-27, f64→i32), `d1b56927` (2026-03-31, f64→i32), `4beba0de` (2026-08-08, i32→f64::from), `67cea248` (2026-08-09, f64::from→i32), `77cc9b9a` (2026-08-09, i32→f64), `573ce63f` (2026-08-09, f64→i32, the state at the start of this bug's investigation) |
| 2026-08-10 | filed  | Discovered mid-session while running final full-workspace verification of 5 unrelated, already-completed items (BUG-043, BUG-046, Task 044, Task 047, Task 048); root cause confirmed via direct reproduction plus first-party web-sys source citations |
| 2026-08-10 | confirmed | VERIFY_PASS — Tier 2 Dual-Role Self-Check, 8/8 dimensions PASS after one Fix-and-Recheck Loop round; synthetic MRE re-executed fresh and reproduces in both directions (exit 0 / exit 1) |
| 2026-08-10 | scope expanded | A scoped clippy re-verification of the first 4 fixed files (run to confirm the opposite cfg direction) surfaced a 5th affected call site, `module/helper/browser_input/src/input.rs`, missed by the original compile-only sweep — same root cause (`PointerEvent` derefs to `MouseEvent`), fixed the same session before completion |
| 2026-08-10 | completed | All 5 files fixed: `d2.rs` (genuine dual-cfg branch), `hexagonal_map`/`object_picking` (`.into()` + `clippy::useless_conversion` allow), `filter` (`as f64` + `clippy::unnecessary_cast` allow), `browser_input` (`as i32` + `clippy::unnecessary_cast` allow on all 3 statics). Verified both cfg directions: `RUSTFLAGS="-D warnings" cargo nextest run --all-features` (1150/1150 tests, exit 0), `cargo check --workspace --all-features` with no override (exit 0), and a scoped `cargo clippy` sweep of all 5 fixed packages clean in both directions. Same-session, self-administered (filer = fixer = verifier) — Tier 2 Dual-Role Self-Check per `governance/maav.rulebook.md`'s default, not an independent PROC16-style acceptance pass. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness (all 12 required sections + established project extensions present) | 🟢 | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Adversarial pass: the first MRE draft used real `web_sys` type names in the stand-in module, which could mislead a reader into thinking it compiles against the real crate; also the MRE was asserted but not actually executed before drafting | Renamed the stand-in module to `websys_stub` to avoid implying it's the real crate; executed both directions for real via `rustc` before writing the **Actual** blocks — output above is verbatim captured, not reconstructed |
| D3 | Cross-Reference Integrity (web-sys source citations, commit hashes, line numbers all independently re-verified this session, not carried over from memory) | 🟡 | 🟢 | Adversarial pass: the initial commit-hash/date list was carried over from a pre-compaction summary; re-running `git log -p --follow` fresh against the live repository was required to confirm it before citing it as evidence | Re-ran `git log --format="COMMIT %h %ad %s" --date=short -p --follow` fresh this session and matched every cited hash/date/message against its live output before writing `## History` |
| D4 | Root Cause Quality (explains the mechanism, not just the symptom; traces to first-party evidence) | 🟢 | 🟢 | — | — |
| D5 | Execution Scope (every fixed path resolves inside this repository; no external changes) | 🟢 | 🟢 | — | — |
| D6 | Fix Locality (each fix lands in the crate that owns its own call site) | 🟢 | 🟢 | This bug's root cause is a workspace-level `RUSTFLAGS`/`.cargo/config.toml` interaction, so its blast radius genuinely spans 5 crates by nature, not by scope creep — each individual fix is still locally scoped to the crate whose call site it corrects | — |
| D7 | Verification Coverage (both cfg directions checked, for both compilation and clippy) | 🟡 | 🟢 | Adversarial pass: the first re-verification attempt after tightening `#[allow]` scope from file-level to item-level hit an unrelated, transiently-broken `mdmath_core` (external, concurrent edit — confirmed via file mtimes newer than the last clean run, not caused by this bug's fix) when run at full-workspace breadth, which would have produced a false-negative reading of this fix's own correctness | Re-scoped verification to the 5 owning packages only (`-p minwebgl -p hexagonal_map -p object_picking -p filter -p browser_input`) across both cfg directions, isolating this fix's correctness from unrelated, concurrently-changing workspace state |
| D8 | Scope Honesty (report accurately discloses the mid-investigation 4→5 file scope expansion and its cause, rather than presenting the final 5-file scope as if it were found complete on the first sweep) | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 4 fixed | 4/4 |

**Reproduced:** YES — MRE exit 0 (cfg OFF) / exit 1 (cfg ON), 2026-08-10, verbatim rustc output
captured and matched into `## Symptom` and `## Minimum Reproducible Example`. Real call sites
confirmed via direct compiler/clippy output, not the synthetic MRE alone.

**Aggregate verdict:** PASS — all 8 dimensions 🟢 after one Fix-and-Recheck Loop round
(`governance/maav.rulebook.md § MAAV : Fix-and-Recheck Loop`); self-administered, no subagent
dispatch (Verification Delegation would be forbidden per `file.rulebook.md § Report New Bug :
Step 9 - VERIFY Gate`).
