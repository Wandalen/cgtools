# BUG-051: `BindGroupLayoutEntry`'s conversion to `web_sys` panics on `BindingType::Other`, its own documented default

- **Severity:** High
- **state:** Completed
- **Affects:** Any caller of `minwebgpu::BindGroupLayoutEntry`'s conversion to
  `web_sys::GpuBindGroupLayoutEntry` (directly via `.into()`/`.try_into()`, or indirectly via
  `BindGroupLayoutDescriptor::entry`/`entry_from_ty`) for an entry whose `.ty(..)` was never
  called — currently zero live panic-triggering callers (both real call sites confirmed to
  always set `.ty(..)` first; see `## Impact`)
- **Component:** `module/min/minwebgpu` — `descriptor::bind_group_layout_entry::BindGroupLayoutEntry`'s `From`/`TryFrom` impl for `web_sys::GpuBindGroupLayoutEntry`
- **repo_identity:** self
- **Filed:** 2026-08-10
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-10
- **Fixed:** 2026-08-10
- **Accepted By:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self — same-session Tier 2 Dual-Role Self-Check, no separate PROC16 acceptance actor)

## Symptom

```bash
# terminal output — synthetic MRE reproducing the exact defect pattern
# (the real crate's real conversion needs a wasm32 + JS-host environment to run; see ## Why Not Caught)
$ /tmp/mre051/repro
thread 'main' (2607660) panicked at /tmp/mre051/repro.rs:25:29:
The type of the binding entry was not set
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
$ echo "exit: $?"
exit: 101

# terminal output — correct, expected behavior once fixed
$ /tmp/mre051/repro_fixed
ok: Other -> Err(TypeNotSetError), Buffer(7) -> 7
$ echo "exit: $?"
exit: 0
```

`BindGroupLayoutEntry::new()` defaults `ty` to `BindingType::Other`, whose own doc comment
(`binding_type.rs:21`, pre-fix) reads "A placeholder for other or unhandled binding types" —
an explicitly documented, expected, reachable value. Yet the `impl From< BindGroupLayoutEntry >
for web_sys::GpuBindGroupLayoutEntry` (`descriptor/bind_group_layout_entry.rs:99`, pre-fix)
panicked with `"The type of the binding entry was not set"` the moment that placeholder reached
conversion, instead of returning an error per the type's own documented contract.

## Impact

**Who is affected:** Any code that constructs a `minwebgpu::BindGroupLayoutEntry` and converts
it to `web_sys::GpuBindGroupLayoutEntry` (directly, or via `BindGroupLayoutDescriptor::entry` /
`entry_from_ty`) without first calling `.ty(..)` — an easy omission, since nothing in the type
system requires `.ty(..)` to be called before conversion; the builder happily compiles and only
panics at the FFI boundary, at runtime, deep inside a browser-only code path.

**What breaks:** Loud — a Rust panic, not a silent wrong value. But it is a **soundness/contract
violation**, not merely a crash: the type's own documentation states `Other` is an expected
placeholder, not an invariant violation, so a caller reading only the enum's doc comment has no
way to know that reaching conversion with `Other` still set is fatal.

**Magnitude — currently zero live callers, confirmed by exhaustive search:** a workspace-wide
grep for every `BindGroupLayoutEntry` usage (`## Evidence Table`) found exactly two real call
chains that ever convert one: `gpu_hal::Device::create_bind_group_layout`
(`module/blank/gpu_hal/src/device.rs:381-404`) and the `deffered_rendering` example
(`examples/minwebgpu/deffered_rendering/src/main.rs`) — both always call `.ty(..)` (or
`entry_from_ty`, which always supplies a concrete type) on every entry before conversion, so
neither currently triggers the panic. The defect is real and confirmed, but dormant in the same
sense as BUG-043: it will panic the first future caller that forwards a `BindGroupLayoutEntry`
to conversion without setting `.ty(..)` — e.g. any caller building entries in a loop that can
leave one unconfigured. Severity is High rather than Critical because no live call path panics
today; it is High rather than Medium (unlike BUG-043's silent-wrong-value case) because the
failure mode is a hard crash in a public API's ordinary (non-`unsafe`) usage, directly
contradicting that API's own documented contract.

**Entity Scope:** `None` — the affected code is an ordinary source file
(`src/descriptor/bind_group_layout_entry.rs`), not an entity directory instance;
`## Affected Entity Collections` does not apply.

## How Discovered

Filed against `task/draft/010_minwebgpu_invariant_violating_panics.md`, which named 3 alleged
sites in `module/min/minwebgpu/src/` where code panics on a condition its own doc comments
document as recoverable, but explicitly flagged that the original file/line citations were not
preserved and had to be re-confirmed fresh. Re-investigation against current source (2026-08-10)
found:

```bash
$ grep -rn "\.unwrap()\|\.expect(\|panic!\|unreachable!\|todo!\|unimplemented!" \
    module/min/minwebgpu/src --include="*.rs"
# 8 panic-capable call sites across 4 files, individually checked against each site's own
# local doc comments for a documented-recoverable contract:
```

Only one of the 8 matched the strict criterion (the panicking branch's own documented contract
states the input is expected/recoverable, not merely that a general WebGPU-spec reader might
expect it to be fallible):

- `layout/vertex_buffer.rs:105` (`value.array_stride.unwrap()`) — **ruled out**: the field's own
  doc says "if not specified, will be computed automatically", and the code immediately before
  the unwrap (`if value.array_stride.is_none() { value.array_stride = Some( offset ); }`)
  provably guarantees `Some` by the time `.unwrap()` runs.
- `context.rs`'s 3 unwraps (`navigator`/`request_adapter`/`request_device`) — **ruled out**: no
  local doc comment on these functions documents recoverability; general WebGPU-spec knowledge
  that adapter/device requests can fail does not meet this task's "own doc comments" criterion,
  and the crate's `readme.md` examples using `?` do not match these functions' actual (panicking)
  signatures, so were not used as justification.
- `layout/vertex_attribute.rs:125` (`_ => panic!( "Unexpected vertex format")`) — **ruled out**:
  no local doc comment justifies the wildcard arm as an expected/documented case.
- `descriptor/bind_group_layout_entry.rs:99` (`BindingType::Other => panic!(..)`) —
  **confirmed**: `BindingType::Other`'s own doc comment (`binding_type.rs:21`, pre-fix) reads "A
  placeholder for other or unhandled binding types" — explicitly documenting it as the type's
  expected default, not an invariant violation, while the conversion panicked on it regardless.

This is the **only** genuine match; the true count for `task/draft/010` is **1**, not 3 (see
`## Refs:` and that task's own updated History).

## Minimum Reproducible Example

Fully self-contained — plain `rustc`, no cargo project, no external crates, no cgtools paths.
`web_sys::GpuBindGroupLayoutEntry` is a wasm-bindgen extern type that only resolves against a
real JS host (browser or wasm32 test runner), unreachable from a plain native `rustc` script; the
scripts below reproduce the exact defect *pattern* instead — an enum with a documented
placeholder/default variant, and a conversion that panics on it instead of returning an error —
structurally identical to the real bug at
`module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs:86-104` (pre-fix).

```bash
mkdir -p /tmp/mre051
cat > /tmp/mre051/repro.rs <<'EOF'
enum BindingType
{
  Buffer( u32 ),
  /// A placeholder for other or unhandled binding types.
  Other,
}

struct Entry { ty : BindingType }

// BEFORE (real code, bind_group_layout_entry.rs:86-104): infallible `From`, panics on `Other`
// even though `Other`'s own doc comment documents it as the type's expected default/placeholder.
impl From< Entry > for u32
{
  fn from( value : Entry ) -> Self
  {
    match value.ty
    {
      BindingType::Buffer( n ) => n,
      BindingType::Other => panic!( "The type of the binding entry was not set" ),
    }
  }
}

fn main()
{
  let default_entry = Entry { ty : BindingType::Other };
  let _ : u32 = default_entry.into();
}
EOF
rustc --edition 2021 -O /tmp/mre051/repro.rs -o /tmp/mre051/repro 2>&1
/tmp/mre051/repro
echo "exit: $?"
```

**Expected** (once fixed — `Other` yields `Err`, not a panic):
```
exit: 0
```

**Actual:**
```
warning: variant `Buffer` is never constructed
 --> /tmp/mre051/repro.rs:9:3
  |
7 | enum BindingType
  |      ----------- variant in this enum
8 | {
9 |   Buffer( u32 ),
  |   ^^^^^^
  |
  = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: 1 warning emitted

thread 'main' (2607660) panicked at /tmp/mre051/repro.rs:25:29:
The type of the binding entry was not set
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
exit: 101
```

**Verify Command:** `/tmp/mre051/repro; test $? -eq 101` — **What:** demonstrates that an enum's
own documented placeholder/default variant, reachable via ordinary construction, causes an
infallible `From` conversion to panic instead of returning an error, reproducing the exact
invariant violated by `BindGroupLayoutEntry`'s `From` impl at
`module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs:99` (pre-fix).

A second script proves the fixed pattern resolves cleanly:

```bash
cat > /tmp/mre051/repro_fixed.rs <<'EOF'
#[ derive( Debug ) ]
enum BindingType
{
  Buffer( u32 ),
  /// A placeholder for other or unhandled binding types.
  Other,
}

struct Entry { ty : BindingType }

#[ derive( Debug ) ]
struct TypeNotSetError;

impl TryFrom< Entry > for u32
{
  type Error = TypeNotSetError;

  fn try_from( value : Entry ) -> Result< Self, Self::Error >
  {
    match value.ty
    {
      BindingType::Buffer( n ) => Ok( n ),
      BindingType::Other => Err( TypeNotSetError ),
    }
  }
}

fn main()
{
  let default_entry = Entry { ty : BindingType::Other };
  let result : Result< u32, _ > = default_entry.try_into();
  assert!( matches!( result, Err( TypeNotSetError ) ), "documented placeholder must yield Err, not panic" );

  let real_entry = Entry { ty : BindingType::Buffer( 7 ) };
  let ok : u32 = real_entry.try_into().unwrap();
  assert_eq!( ok, 7 );

  println!( "ok: {:?} -> {:?}, Buffer(7) -> {}", "Other", result, ok );
}
EOF
rustc --edition 2021 -O /tmp/mre051/repro_fixed.rs -o /tmp/mre051/repro_fixed 2>&1
/tmp/mre051/repro_fixed
echo "exit: $?"
```

**Actual (post-fix pattern):**
```
ok: "Other" -> Err(TypeNotSetError), Buffer(7) -> 7
exit: 0
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `BindingType::Other` is documented as an expected/default value, but its conversion treats reaching it as an unrecoverable invariant violation (`panic!`) instead of a documented, foreseeable input | ✅ Root Cause | `binding_type.rs:21` (pre-fix) doc: "A placeholder for other or unhandled binding types"; `bind_group_layout_entry.rs:99` (pre-fix): `BindingType::Other => panic!(..)` | E1, E2, E3 |
| H2 | The panic is intentional defensive programming — `Other` is only ever reached via a caller bug (never expected to occur), so panicking is the correct "fail fast" response | ❌ Disproved | `BindGroupLayoutEntry::new()` (`bind_group_layout_entry.rs:30-42`, pre-fix) sets `ty = BindingType::Other` unconditionally as the *default* — `Other` is reached by ordinary construction (simply never calling `.ty(..)`), not by a defect; nothing in the type system prevents converting a freshly-constructed entry | E1, E4 |
| H3 | Every real call site already guards against this, so the panic is unreachable in practice and effectively harmless | ❌ Disproved (as a reason not to fix) | Confirmed true for the *current* 2 call sites (`gpu_hal::Device::create_bind_group_layout`, the `deffered_rendering` example — both always call `.ty(..)`/`entry_from_ty` first), but nothing in `BindGroupLayoutEntry`'s public API enforces this — the panic remains one omitted `.ty(..)` call away for any future caller | E5, E6 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/min/minwebgpu/src/binding_type.rs:21` (pre-fix) | `BindingType::Other`'s doc comment: "A placeholder for other or unhandled binding types" — documents it as expected, not exceptional | H1 ✅, H2 ❌ |
| E2 | `module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs:86-104` (pre-fix) | `impl From< BindGroupLayoutEntry > for web_sys::GpuBindGroupLayoutEntry` — every real variant maps to a `.set_*` call; only `Other` diverges, into `panic!( "The type of the binding entry was not set" )` | H1 ✅ (symptom) |
| E3 | `module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs:30-42` (pre-fix) | `BindGroupLayoutEntry::new()`'s body: `let ty = BindingType::Other;` — `Other` is the struct's own default, not a sentinel reachable only via misuse | H1 ✅ |
| E4 | `module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs:79-83` (pre-fix) | `pub fn ty( mut self, ty : impl Into< BindingType > ) -> Self` — an ordinary optional builder method; nothing marks it `#[must_use]` or otherwise signals it is mandatory before conversion | H2 ❌ |
| E5 | `module/blank/gpu_hal/src/device.rs:381-396` (pre-fix) | `create_bind_group_layout`'s loop always calls `.ty(..)` on `raw_entry` via one of 3 match arms covering `BindingType::{UniformBuffer,Texture,Sampler}` — exhaustive, so this call site never leaves `ty` as `Other` | H3 (current calls safe) |
| E6 | `examples/minwebgpu/deffered_rendering/src/main.rs:113-146` (pre-fix) | Every `.entry(..)` call passes an entry with `.ty(..)` already set, and every `.entry_from_ty(..)` call supplies a concrete `BindingType` by construction — neither path can reach `Other` | H3 (current calls safe) |

## Root Cause

```
BindGroupLayoutEntry::new()                    -> ty = BindingType::Other   (the type's own default)
BindingType::Other's doc comment                -> "a placeholder ... other or unhandled"  (documented as expected)
impl From<BindGroupLayoutEntry> for GpuBindGroupLayoutEntry
  BindingType::Buffer/Sampler/Texture/...       -> layout.set_*(..)          (handled)
  BindingType::Other                            -> panic!(..)                (H1, ✅ Root Cause)
```

An infallible `From` conversion was used for a type that has one documented input variant
(`Other`, the struct's own default, trivially reachable by never calling `.ty(..)`) with no
valid WebGPU representation. Rather than modeling this as a fallible conversion, the
implementation panicked — directly contradicting `Other`'s own doc comment, which frames it as
an expected placeholder rather than a programming error. H2 (defensive-programming panic) is
disproved by `Other` being the *default*, not a sentinel; H3 (currently-safe callers) is
confirmed true today but does not change that the API itself provides no static guarantee — it
is a live footgun for the first caller that omits `.ty(..)`. This confirms **H1 (✅ Root Cause)**.

## Why Not Caught

`module/min/minwebgpu` had **zero files under `tests/`** before this bug's fix — confirmed via
`find module/min/minwebgpu -iname tests` returning nothing, in contrast to sibling driver crates
`mingl` (which has `tests/tests.rs` plus a `tests/tests/` module tree) and `minwebgl` (which also
has none). No test of any kind exercised `BindGroupLayoutEntry`'s conversion, so the panic was
never triggered by any existing verification path. Compounding this:

```bash
$ find . -iname "*.yml" -o -iname "*.yaml" | grep -v target | xargs grep -l "wasm" 2>/dev/null
# (no output — no CI workflow file references wasm/wasm32 anywhere in the repo; same finding as BUG-046)
```

Unlike BUG-046, a live `cargo check -p minwebgpu --target wasm32-unknown-unknown --all-features`
(non-test) **does** succeed in this environment (the pre-existing, unrelated `mdmath_core`
blocker BUG-046-era investigations hit has since been resolved — see BUG-050) — so the source
fix itself was verified by a real compile, not only by cross-check (`## Fix Applied`). What
remains blocked is running the *new* regression test this fix adds
(`tests/bind_group_layout_entry_tests.rs`): compiling with `--tests` pulls in the crate's
existing `test_tools` dev-dependency's own `rand v0.8.7 → rand_core v0.6.4 → getrandom v0.2.17`
chain, and `getrandom v0.2.17` refuses to compile for `wasm32-unknown-unknown` without its `"js"`
feature enabled:

```bash
$ cargo check -p minwebgpu --target wasm32-unknown-unknown --all-features --tests
error: the wasm*-unknown-unknown targets are not supported by default, you may need to enable the "js" feature. For more information see: https://docs.rs/getrandom/#webassembly-support
   --> getrandom-0.2.17/src/lib.rs:346:9
error: could not compile `getrandom` (lib) due to 1 previous error
```

This is the exact same class of gap BUG-046 already documented and ruled out of scope for
`renderer` (`getrandom v0.2.17` lacking wasm32 support there too), now shown to affect
`minwebgpu` as well — triggered by the pre-existing `test_tools` dev-dependency
(`module/min/minwebgpu/Cargo.toml`, already present before this fix), not by anything this fix
adds. Two sibling crates already carry the fix for their own instance of this gap
(`module/helper/scene_script/Cargo.toml:39` and
`module/helper/primitive_generation/Cargo.toml:70`, both
`getrandom = { workspace = true, features = ["wasm_js"] }`); `minwebgpu` has never needed one
before because it never had a `tests/` directory to compile. Adding that override is a separate,
out-of-scope concern — not part of this bug's `## Fix Location` — since it is unrelated to the
panic-vs-`Result` defect and would be the first time `minwebgpu` exercises its wasm32 test target
at all.

## Fix Location

Root cause: `module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs:86-104` (pre-fix
line numbers):

```rust
// Before:
impl From< BindGroupLayoutEntry > for web_sys::GpuBindGroupLayoutEntry
{
  fn from( value: BindGroupLayoutEntry ) -> Self
  {
    let layout = web_sys::GpuBindGroupLayoutEntry::new( value.binding, value.visibility );

    match &value.ty
    {
      BindingType::Buffer( buffer ) => layout.set_buffer( &buffer ),
      BindingType::Sampler( sampler ) => layout.set_sampler( &sampler ),
      BindingType::Texture( texture ) => layout.set_texture( &texture ),
      BindingType::StorageTexture( texture ) => layout.set_storage_texture( &texture ),
      BindingType::ExternalTexture( texture ) => layout.set_external_texture( &texture ),
      BindingType::Other => panic!( "The type of the binding entry was not set" )
    }

    layout
  }
}

// After:
impl TryFrom< BindGroupLayoutEntry > for web_sys::GpuBindGroupLayoutEntry
{
  type Error = WebGPUError;

  fn try_from( value: BindGroupLayoutEntry ) -> Result< Self, Self::Error >
  {
    let layout = web_sys::GpuBindGroupLayoutEntry::new( value.binding, value.visibility );

    match &value.ty
    {
      BindingType::Buffer( buffer ) => layout.set_buffer( &buffer ),
      BindingType::Sampler( sampler ) => layout.set_sampler( &sampler ),
      BindingType::Texture( texture ) => layout.set_texture( &texture ),
      BindingType::StorageTexture( texture ) => layout.set_storage_texture( &texture ),
      BindingType::ExternalTexture( texture ) => layout.set_external_texture( &texture ),
      BindingType::Other => return Err( error::BindGroupError::TypeNotSet( value.binding ).into() )
    }

    Ok( layout )
  }
}
```

Since `From` guarantees infallibility (Rust's standard library provides a blanket `TryFrom` for
every `From`), the `From` impl could not simply gain a `TryFrom` alongside it — it had to be
**replaced**, which ripples to every caller that relied on the old infallible conversion. Four
additional locations needed updating to keep the workspace compiling, all direct, minimal
consequences of the signature change — no independent defects:

- `module/min/minwebgpu/src/error.rs` — new `WebGPUError::BindGroupError(BindGroupError::TypeNotSet(u32))` variant (no prior variant represented "binding type not set"; the `u32` carries the offending binding number).
- `module/min/minwebgpu/src/binding_type.rs` — `BindingType::Other`'s doc comment extended to state its new failure contract explicitly.
- `module/min/minwebgpu/src/descriptor/bind_group_layout.rs` — `BindGroupLayoutDescriptor::entry`/`entry_from_ty` changed from infallible (`Self`) to `Result< Self, WebGPUError >`, propagating via `?`.
- `module/min/minwebgpu/src/transform.rs` — removed `impl_to_web!( BindGroupLayoutEntry, GpuBindGroupLayoutEntry );`, since `AsWeb::to_web` is infallible by design and can no longer be implemented for this type.
- `module/blank/gpu_hal/src/device.rs:396` and `examples/minwebgpu/deffered_rendering/src/main.rs` (2 builder chains) — the only 2 real call sites in the workspace that convert a `BindGroupLayoutEntry`; both updated to propagate the new `Result` with `?` (confirmed via workspace-wide grep, `## Refs: src/`).

## Fix Applied

Applied exactly as documented above. Fix-time source comments use the standard 3-field form
(`Fix(BUG-051)` / `Root cause` / `Pitfall`) at every touched site: `bind_group_layout_entry.rs:90-101`
(root cause), `error.rs:93-99`, `binding_type.rs:21-29`, `bind_group_layout.rs:69-76`,
`transform.rs:37-46`, `gpu_hal/device.rs:396-403`, and both sites in
`examples/minwebgpu/deffered_rendering/src/main.rs:113-118,144-145`.

**Real compiles, not only cross-check** (stronger than BUG-046's fully-blocked baseline — the
unrelated `mdmath_core` blocker that would have affected this too was independently resolved via
BUG-050 before this fix's verification pass):

```bash
$ cargo check -p minwebgpu --target wasm32-unknown-unknown --all-features
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.29s     # exit 0

$ cargo check -p gpu_hal --target wasm32-unknown-unknown --features webgpu
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.23s     # exit 0

$ cargo check -p minwebgpu_deffered_rendering --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.09s     # exit 0

$ cargo clippy -p minwebgpu --target wasm32-unknown-unknown --all-features -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.27s     # exit 0, zero warnings
```

A workspace-wide `grep -rn "BindGroupLayoutEntry"` sweep (post-fix) confirmed no other call site
was missed — remaining matches are either `gpu_hal::BindGroupLayoutEntry` (a distinct, unrelated
plain-data type of the same name) or `wgpu::BindGroupLayoutEntry` (the third-party `wgpu` crate,
used only by the unrelated `minwgpu` driver).

**Regression test added** (`module/min/minwebgpu/tests/bind_group_layout_entry_tests.rs`, 5
`#[ wasm_bindgen_test ]` cases, `#[ cfg( target_arch = "wasm32" ) ]`-gated, marked
`// test_kind: bug_reproducer(BUG-051)`), covering: entry without `.ty(..)` → `Err` (was: panic);
entry with `.ty(..)` → `Ok`; the same two cases through `BindGroupLayoutDescriptor::entry`; and
`entry_from_ty` always succeeding (it can never construct an `Other` entry). Compiling this test
file (`--tests`) is blocked by the pre-existing `getrandom`/wasm32 gap documented in `## Why Not
Caught` — not by anything in this fix; the plain (non-`--tests`) build of the same crate, using
the exact same `TryFrom` impl the test exercises, is independently confirmed above. The MRE
(`## Minimum Reproducible Example`) provides the actually-executed red→green proof for this
defect pattern: exit 101 (panic) before, exit 0 (clean `Err`/`Ok`) after — both freshly
re-executed during this verification pass.

## Prevention

Add the same `getrandom = { workspace = true, features = ["wasm_js"] }` override
`scene_script`/`primitive_generation` already carry to `module/min/minwebgpu/Cargo.toml`, then
wire up an actual wasm32 test execution path (mirroring BUG-046's identical recommendation) so
`tests/bind_group_layout_entry_tests.rs` — and any future wasm32-gated test in this crate — can
actually run, not merely compile. Detection once both are in place:

```bash
cargo check -p minwebgpu --target wasm32-unknown-unknown --all-features --tests
```

should exit 0, and the 5 tests in `bind_group_layout_entry_tests.rs` should be collectible by a
wasm32 test runner.

**Pitfall:** An enum variant documented as a "placeholder" or "default" is, by construction,
reachable through ordinary use — never assume such a variant is an unreachable/defensive-only
case. Any conversion that cannot represent it must be `TryFrom`, not `From` plus a panic; and
because `From` and an explicit `TryFrom` cannot coexist on the same pair of types (the standard
library's blanket `impl<T, U: Into<T>> TryFrom<U> for T` would conflict), converting a
public-API `From` impl to `TryFrom` after the fact is a breaking signature change that must be
propagated to every real call site, not merely patched at the panic site.

## Generalized Version

**Broken assumption:** "A conversion (`From`) can be infallible because most of its input's
variants map cleanly" — false whenever the input type has even one documented placeholder,
default, or "unhandled/other" variant that has no valid representation in the output type.

Fails for any `impl From< Enum > for Target` where:
1. `Enum` has a variant documented (in its own doc comment) as a default, placeholder, or
   catch-all, AND
2. that variant is reachable via ordinary construction (e.g. it is the `Default`/`new()` value),
   AND
3. `Target` has no representation for it, so the `From` impl's match arm for that variant must
   either fabricate an invalid `Target` or abort (panic).

**Detection invariant:**
```
for every enum variant V documented as a default/placeholder/catch-all,
every infallible `From` conversion consuming that enum either has a valid mapping for V,
or does not exist (i.e. the conversion is TryFrom, not From).
```

## Verification

### Checklist

- [x] C1 — Is the root-cause fix (same site task `010` fixed) present: `TryFrom` conversion returning `Err( BindGroupError::TypeNotSet(..) )` instead of panicking on `BindingType::Other`? Confirmed via direct read of `module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs:125-146` — identical evidence to task `010`'s own C1 (this bug and that task converge on the same fix, filed independently the same day).
- [x] C2 — Do all 7 claimed `Fix(BUG-051)` sites (6 files, 2 sites in `deffered_rendering/main.rs`) carry the mandated 3-field comment? `grep -rn "Fix(BUG-051)" --include="*.rs" .` → exactly 7 matches: `bind_group_layout_entry.rs:113`, `error.rs:102`, `binding_type.rs:22`, `bind_group_layout.rs:90`, `transform.rs:38`, `gpu_hal/device.rs:571`, `deffered_rendering/main.rs:108,139` — all 7 present.
- [x] C3 — Path-citation drift check: this bug's own `## Fix Location`/`## Fix Applied`/`## Refs: src/` sections all cite `module/blank/gpu_hal/src/device.rs:396`. `git log --follow --diff-filter=R -- module/helper/gpu_hal/src/device.rs` shows commit `4469eafb` (2026-08-10, same day, after this bug's fix landed) renamed `module/{blank => helper}/gpu_hal/src/device.rs`. The fix content is present and correct at the current path (`module/helper/gpu_hal/src/device.rs:571-579`) — only this bug's own prose citation is now stale; not corrected here per this insertion's pure-insertion scope.
- [x] C4 — Is the documented, still-open `getrandom`/wasm32 gap (`## Prevention`, `## Why Not Caught`) still genuinely open (not silently fixed, not silently broken further)? `grep -n "getrandom" module/min/minwebgpu/Cargo.toml` → no match (override still absent, as documented). Live re-run: `cargo check -p minwebgpu --target wasm32-unknown-unknown --all-features --tests` → exit `101`, reproduces the exact same `getrandom` `compile_error!` ("the wasm*-unknown-unknown targets are not supported by default...") this bug's `## Why Not Caught` documents.
- [x] C5 — Does the regression test (`tests/bind_group_layout_entry_tests.rs`) carry the mandated `bug_reproducer(BUG-051)` marker and 5-section doc comment? Confirmed via direct read: `// test_kind: bug_reproducer(BUG-051)` at line 14; the doc comment on `entry_without_ty_yields_type_not_set_err_test` carries all 5 mandated sections (`## Root Cause`, `## Why Not Caught`, `## Fix Applied`, `## Prevention`, `## Pitfall`).

### Measurements

- [x] M1 — `BindGroupError` occurrences in `error.rs`: `3` (enum declaration, `#[ from ]` wiring into `WebGPUError`, doc-comment reference) (was: `0` — `git show 67cea248:module/min/minwebgpu/src/error.rs | grep -c "BindGroupError"` → `0`; `67cea248` is the last commit predating this fix).

### Invariants

- [x] I1 — `cargo check -p minwebgpu --target wasm32-unknown-unknown --all-features` → exit `0`, clean — real compile of the actual fixed, wasm32-gated code this bug's `## Fix Applied` section cites as its primary evidence.
- [x] I2 — `cargo clippy -p minwebgpu --target wasm32-unknown-unknown --all-features -- -D warnings` → exit `101`, **FAILS** — this is a **drift** from this bug's own recorded `## Fix Applied` result ("exit 0, zero warnings", captured 2026-08-10). Root cause: identical to task `010`'s I2 — the unrelated `browser_log` dependency (`module/helper/browser_log/src/panic.rs:82`) now violates `clippy::allow_attributes_without_reason`, introduced by commit `5f33be66` (2026-08-11 09:30:53, this morning), well after this bug's fix and its own verification pass. Not a regression of this bug's own fix: `cargo clippy -p minwebgpu --no-deps --all-targets --all-features -- -D warnings` → exit `0`, clean.
- [x] I3 — `cargo nextest run -p minwebgpu --all-features` → exit `4`, "no tests to run" (0 tests collected) — expected, crate-wide wasm32 gating (reused from task `010`'s I1, same crate, same verification pass).
- [x] I4 — `cargo check -p minwebgpu --target wasm32-unknown-unknown --all-features --tests` → exit `101` — expected; reproduces the still-open `getrandom` gap documented in `## Why Not Caught` (see C4); not a new failure, not caused by this bug's own fix.

### Anti-faking checks

- [x] AF1 — Guards against the root-cause fix silently reverting: `grep -c "panic!" module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs` must stay `0` (same guard as task `010`'s AF1 — both files fix the identical site).
- [x] AF2 — Guards against trusting a stale path citation in a future edit: this workspace has already renamed `gpu_hal` once (`module/blank` → `module/helper`) after this bug was filed; any future citation of a file/line in this bug's body must be re-verified against `git log --follow` before being trusted, not copied forward.
- [x] AF3 — Guards against conflating I2's `browser_log` failure with this bug reopening: before treating a red I2 as a regression, confirm `error: could not compile` names `browser_log`, and re-run the `--no-deps` variant to isolate minwebgpu's own code, exactly as this pass did.

## History

| Date       | Event  | Notes                                                                                                     |
|------------|--------|-------------------------------------------------------------------------------------------------------------|
| 2026-08-10 | filed  | Re-investigated `task/draft/010_minwebgpu_invariant_violating_panics.md`'s 3 alleged sites against current source; confirmed exactly 1 of 3 is real (the other 2 ruled out — no local doc comment documents recoverability). Root cause confirmed via source read + doc-comment cross-check + workspace-wide call-site grep before filing. |
| 2026-08-10 | confirmed | VERIFY_PASS — Tier 2 Dual-Role Self-Check, 8/8 dimensions PASS after one Fix-and-Recheck Loop round; MRE re-executed fresh and reproduces (exit 101 before, exit 0 after) |
| 2026-08-10 | completed | `From` → `TryFrom` fix applied at the root cause plus 6 ripple sites (`error.rs`, `binding_type.rs`, `bind_group_layout.rs`, `transform.rs`, `gpu_hal/device.rs`, `deffered_rendering/main.rs`), all with 3-field fix comments. Verified via real `cargo check`/`cargo clippy` against the real `wasm32-unknown-unknown` target for all 3 affected crates (minwebgpu, gpu_hal, minwebgpu_deffered_rendering) — all exit 0 — plus a new 5-case regression test (`bind_group_layout_entry_tests.rs`), whose *compilation* (not just the source fix) is blocked only by a pre-existing, out-of-scope `getrandom`/wasm32 gap (same class as BUG-046, documented transparently). Same-session, self-administered (filer = fixer = verifier) — Tier 2 Dual-Role Self-Check per governance/maav.rulebook.md's default, not an independent PROC16-style acceptance pass. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟡 | 🟢 | Adversarial pass: initial draft's `## Symptom` used the raw source-diff style BUG-043's own Verification Record flagged as a defect for that report — must show real captured terminal output instead | Rewrote `## Symptom` to show the MRE's real captured terminal output (panic vs. clean `Err`) |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | — | — |
| D3 | Cross-Reference Integrity | 🟡 | 🟢 | Adversarial pass: `## Fix Location`'s file/line citations for the 4 ripple sites were written before the Fix Documentation comments were added, shifting every subsequent line down | Re-grepped every touched file for exact post-comment line numbers immediately before writing `## Fix Applied`; corrected all citations |
| D4 | Root Cause Quality | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟡 | 🟢 | Adversarial pass: fix touches 3 crates (`minwebgpu`, `gpu_hal`, `minwebgpu_deffered_rendering`) — could look like scope creep | `## Fix Location` explicitly states the root cause is 100% within `minwebgpu`; the other 2 crates are necessary, minimal call-site ripples from `minwebgpu`'s own public API becoming fallible (confirmed via exhaustive workspace grep — no other call site exists), not independent defects |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 3 fixed | 3/3 |

**Reproduced:** YES — exit 101 (before) / exit 0 (after), 2026-08-10 (`/tmp/mre051/repro`,
`/tmp/mre051/repro_fixed`, verbatim output captured and matched into `## Symptom` and
`## Minimum Reproducible Example`).

**Aggregate verdict:** PASS — all 8 dimensions 🟢 after one Fix-and-Recheck Loop round
(`governance/maav.rulebook.md § MAAV : Fix-and-Recheck Loop`); self-administered, no subagent
dispatch (Verification Delegation would be forbidden per `file.rulebook.md § Report New Bug :
Step 9 - VERIFY Gate`).

## Refs: src/

- `module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs` — root cause: `From` → `TryFrom`, panic → `Err` (lines 90-118)
- `module/min/minwebgpu/src/error.rs` — new `BindGroupError::TypeNotSet(u32)` variant + `WebGPUError::BindGroupError` (lines 28-30, 93-109)
- `module/min/minwebgpu/src/binding_type.rs` — `BindingType::Other` doc comment extended with its failure contract (lines 21-35)
- `module/min/minwebgpu/src/descriptor/bind_group_layout.rs` — `entry`/`entry_from_ty` changed to `Result`-returning (lines 69-96)
- `module/min/minwebgpu/src/transform.rs` — removed invalid `impl_to_web!( BindGroupLayoutEntry, .. )` (lines 37-46)
- `module/blank/gpu_hal/src/device.rs` — propagate `?` at the one real call site (lines 396-404)
- `examples/minwebgpu/deffered_rendering/src/main.rs` — propagate `?` at both real call sites (lines 113-127, 144-153)

## Refs: tests/

- `module/min/minwebgpu/tests/bind_group_layout_entry_tests.rs` — new file, 5 `wasm_bindgen_test` cases, `// test_kind: bug_reproducer(BUG-051)`; compilation blocked by the pre-existing `getrandom`/wasm32 gap documented in `## Why Not Caught` (out of scope for this bug)
