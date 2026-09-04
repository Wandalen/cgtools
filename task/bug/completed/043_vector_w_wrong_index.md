# BUG-043: `Vector<E,4>::w()` returns the `z` component instead of the `w` component

- **Severity:** Medium
- **state:** Completed
- **Affects:** Any direct call to `Vector::<E,4>::w()` (the 4-component vector accessor) for any element type `E` — currently zero live call sites (see `## Impact`)
- **Component:** `module/math/ndarray_cg` — `vector::vec4::Vector<E,4>::w()`
- **repo_identity:** self
- **Filed:** 2026-08-09
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-09
- **Fixed:** 2026-08-09
- **Accepted By:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self — same-session Tier 2 Dual-Role Self-Check, no separate PROC16 acceptance actor)

## Symptom

```bash
# terminal output — wrong, current behavior (equivalent minimal reproduction; see ## Minimum Reproducible Example)
$ /tmp/mre043/repro
z = 3, w = 3        # <- w() should read 4, the true 4th component

# terminal output — correct, expected behavior once fixed
$ /tmp/mre043/repro
z = 3, w = 4
```

`w()` and `z()` are byte-for-byte identical in `module/math/ndarray_cg/src/vector/vec4/general.rs`:
both index `self.0[ 2 ]`. For any vector where the 3rd and 4th components differ,
`v.w() == v.z()` always holds — the true 4th element (`self.0[ 3 ]`) is never read by this
method.

## Impact

**Who is affected:** Any code calling `Vector::<E, 4>::w()` directly, for any of the crate's
4-element type aliases (`F32x4`, `F64x4`, `I32x4`, `I64x4`, `U32x4`, `U64x4` and their bool
vector sibling) — the bug lives in the generic `impl< E : MatEl > Vector< E, 4 >` block, so
it affects every `E`, not one specific numeric type.

**What breaks:** Silent — no panic, no compiler warning; `w()` simply returns the wrong
scalar (a duplicate of `z()`) with no signal to the caller.

**Magnitude — currently zero, confirmed by exhaustive search:** a workspace-wide grep for
every `.w()` call site outside `quaternion/` found exactly 3 matches
(`module/math/ndarray_cg/src/d2/mat3x3/general.rs:216-218`), and all 3 receivers are
`quat.w()` — i.e. `Quat<E>::w()`, not `Vector::<E,4>::w()`. `Quat<E>` wraps a `Vector<E,4>`
as its `.0` field but does **not** delegate its own `w()` to the vector's `w()`:
`quaternion/general.rs:27-31` defines an independent, correct `self.0[ 3 ]`-indexing method.
Every current caller of anything named `.w()` in this workspace therefore goes through the
correct quaternion path, never through the buggy vector path. The defect is real and
confirmed, but dormant: it will silently corrupt the first future caller that calls `.w()`
on a bare `Vector<E,4>` (as opposed to a `Quat<E>`), which is why Severity is Medium rather
than Critical/High — a live-callers audit found no current production impact, but the
public API itself is broken and will corrupt any new caller without warning until fixed.

**Entity Scope:** `None` — the affected code is an ordinary source file
(`src/vector/vec4/general.rs`), not an entity directory instance; `## Affected Entity
Collections` does not apply.

## How Discovered

```bash
$ grep -n "pub fn w\|pub fn z" module/math/ndarray_cg/src/vector/vec4/general.rs
32:    pub fn z( &self ) -> E
39:    pub fn w( &self ) -> E

$ sed -n '32,44p' module/math/ndarray_cg/src/vector/vec4/general.rs
    /// The `z` component of vector
    #[ inline ]
    pub fn z( &self ) -> E
    {
      self.0[ 2 ]
    }

    /// The `w` component of vector
    #[ inline ]
    pub fn w( &self ) -> E
    {
      self.0[ 2 ]        # <- should be self.0[ 3 ]
    }
```

Found during a routine read-through of `Vector<E,4>`'s accessor methods while investigating
an unrelated `todo.md` claim about integer vector math (2026-08-09 session) — not reported
by a user or a failing test, since no test in this workspace currently exercises `.w()` on a
bare `Vector<E,4>` (confirmed in `## Why Not Caught`).

## Minimum Reproducible Example

Fully self-contained — plain `rustc`, no cargo project, no external crates, no cgtools
paths. `Vector<E,4>` is a private, unpublished workspace crate, so a literal reproduction
against the real type isn't reachable from outside this repo; the script below reproduces
the exact defect *pattern* instead — a 4-element tuple-wrapped array with a `w()` accessor
copy-pasted from `z()` — structurally identical to the real bug at
`module/math/ndarray_cg/src/vector/vec4/general.rs:41-45`.

```bash
mkdir -p /tmp/mre043
cat > /tmp/mre043/repro.rs <<'EOF'
struct Vec4( [ f64 ; 4 ] );

impl Vec4
{
  fn z( &self ) -> f64 { self.0[ 2 ] }
  fn w( &self ) -> f64 { self.0[ 2 ] }   // copy-pasted from z(): should read self.0[ 3 ]
}

fn main()
{
  let v = Vec4( [ 1.0, 2.0, 3.0, 4.0 ] );
  println!( "z = {}, w = {}", v.z(), v.w() );
  assert_ne!( v.z(), v.w(), "w() must not equal z() for distinct components" );
}
EOF
rustc -O /tmp/mre043/repro.rs -o /tmp/mre043/repro 2>&1
/tmp/mre043/repro
echo "exit: $?"
```

**Expected:**
```
z = 3, w = 4
exit: 0
```

**Actual:**
```
z = 3, w = 3

thread 'main' panicked at /tmp/mre043/repro.rs:13:3:
assertion `left != right` failed: w() must not equal z() for distinct components
  left: 3.0
 right: 3.0
exit: 101
```

**Verify Command:** `/tmp/mre043/repro; test $? -eq 101` — **What:** demonstrates that a
`w()` accessor copy-pasted from `z()` returns the 3rd element instead of the 4th,
reproducing the exact invariant violated by `Vector::<E,4>::w()` at
`module/math/ndarray_cg/src/vector/vec4/general.rs:44`.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `w()`'s body is a copy-paste of `z()` that was never updated to index 3 | ✅ Root Cause | `vec4/general.rs:44` reads `self.0[ 2 ]`, byte-identical to `z()`'s body at `vec4/general.rs:36` | E1, E2, E3 |
| H2 | `w()` intentionally aliases `z()` as a deliberate (if confusingly named) design choice | ❌ Disproved | Doc comment at `vec4/general.rs:39` reads "The `w` component of vector" — states `w`, not an alias of `z`; no aliasing note anywhere in the file | E2 |
| H3 | The wrong index is a macro-expansion artifact (`mod_interface!` or a derive macro miscounting fields) | ❌ Disproved | `x()`/`y()`/`z()`/`w()` are hand-written plain methods inside a literal `impl` block (`vec4/general.rs:6-53`); the file's `mod_interface!` block (`vec4/general.rs:74-76`) is empty — no macro touches these method bodies | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/ndarray_cg/src/vector/vec4/general.rs:32-37` | `z()`'s body: `self.0[ 2 ]` — the correct, working pattern that `w()`'s body duplicates | H1 ✅ (symptom) |
| E2 | `module/math/ndarray_cg/src/vector/vec4/general.rs:39-45` | `w()`'s doc comment says "The `w` component of vector" but its body reads `self.0[ 2 ]` — identical to `z()`, not `self.0[ 3 ]` | H1 ✅, H2 ❌ |
| E3 | `git blame -L 39,44 module/math/ndarray_cg/src/vector/vec4/general.rs` → commit `18c3c31e2`, 2025-08-07 | All lines of `w()` (doc comment + signature + body) were authored together in one commit as plain hand-typed Rust — no `#[derive(...)]` or macro invocation wraps them, and the file's own `mod_interface!` block is empty (`general.rs:74-76`) | H1 ✅, H3 ❌ |

## Root Cause

```
z()   -> self.0[ 2 ]     (vec4/general.rs:36, correct)
w()   -> self.0[ 2 ]     (vec4/general.rs:44, wrong — copied from z(), index never bumped to 3)
```

`w()` was authored directly below `z()` in the same commit (`18c3c31e2`, 2025-08-07) by
copying `z()`'s body as a starting point for the new method and updating the doc comment
and signature name to `w`, but never updating the array index inside the body from `2` to
`3`. The doc comment (`vec4/general.rs:39`, "The `w` component of vector") states the
intended behavior correctly; only the body's literal index is wrong, which is exactly the
signature of a copy-paste slip rather than a deliberate design choice or a macro artifact
— confirming **H1 (✅ Root Cause)** over the disproved alternatives H2 and H3.

## Why Not Caught

No test in this workspace calls `.w()` on a bare `Vector<E,4>` at all — a workspace-wide
`grep -rn "\.w()" --include="*.rs" .` finds only 3 call sites total
(`module/math/ndarray_cg/src/d2/mat3x3/general.rs:216-218`), all on `Quat<E>` receivers
(which has its own independent, correct implementation), and zero matches anywhere under
any crate's `tests/` directory. `ndarray_cg`'s own test suite has no `vec4_test` module at
all — unlike `d2_test`, which has a dedicated `access_test/` covering 2D accessor and
indexing behavior in depth, there is no equivalent accessor-coverage test file for `vec4`
(confirmed via `find tests -type f -name "*.rs"` in `module/math/ndarray_cg` — no
`vec4`-named file exists). No invariant anywhere asserts `v.w() != v.z()` for a vector with
distinct components, so the constructor `Vector::<E,4>::new(x,y,z,w)` is never round-tripped
through `w()` in any existing test.

## Fix Location

`module/math/ndarray_cg/src/vector/vec4/general.rs:44`:

```rust
// Before:
pub fn w( &self ) -> E
{
  self.0[ 2 ]
}

// After:
pub fn w( &self ) -> E
{
  self.0[ 3 ]
}
```

One-line fix; no other locations affected — `Quat<E>::w()` (`quaternion/general.rs:27-31`)
already independently indexes `self.0[ 3 ]` and does not delegate to this method.

## Fix Applied

Applied exactly as documented above (`vector/vec4/general.rs:44`: `self.0[ 2 ]` → `self.0[ 3 ]`),
with the fix-time comment upgraded from the filing-time backreference to the standard 3-field
form (`Fix(BUG-043)` / `Root cause` / `Pitfall`, `vector/vec4/general.rs:40-43`). New reproducer:
`tests/inc/vec4_test.rs::accessor_test` (`// test_kind: bug_reproducer(BUG-043)`), asserting
`x()`/`y()`/`z()`/`w()` against a 4-distinct-component vector for one integer (`I32x4`) and one
float (`F32x4`) type, plus `w() != z()`. Confirmed failing before the fix
(`cargo nextest run -p ndarray_cg --all-features` → `FAIL ... inc::vec4_test::accessor_test`) and
passing after (`229 tests run: 229 passed, 0 skipped`).

## Prevention

Add a dedicated `vec4_test` accessor test asserting `Vector::new(1,2,3,4).w() == 4` (and
equivalently for `x()`/`y()`/`z()`) across at least one integer and one float element type,
plus a check that `v.w() != v.z()` whenever the 3rd and 4th constructor arguments differ.
Detection:

```bash
cargo test -p ndarray_cg vec4
```

should exist and pass for a case built with 4 distinct components.

**Pitfall:** A same-commit, copy-adjacent accessor (`w()` written directly below `z()`,
same body shape, only the doc comment and name updated) is exactly the pattern most likely
to carry a silent stale-index slip — when adding an accessor by copying its neighbor,
always independently verify the copied index against the field it now names, not just the
surrounding doc/signature text.

## Generalized Version

**Broken assumption:** "An accessor method's name/doc comment accurately reflects the array
index its body reads" — false whenever the method was authored by copying a sibling
accessor's body without updating the literal index inside it.

Fails for any accessor pair `{a(), b()}` where:
1. `b()` was authored by copying `a()`'s body as a template, AND
2. the doc comment and method name were updated to `b`, but the internal index literal was
   not updated from `a`'s index to `b`'s, AND
3. no test independently asserts `b()`'s return value against a component that differs from
   `a()`'s.

**Detection invariant:**
```
for every constructed value with N pairwise-distinct components,
component_accessor[i](v) == v[i]  for all i in 0..N
```

## Verification

### Checklist

- [x] C1 — Does `Vector<E,4>::w()` now correctly read `self.0[3]` (not `self.0[2]`)? `grep -n -A3 "fn w(" src/vector/vec4/general.rs` → `pub fn w( &self ) -> E` at line 45, body `self.0[ 3 ]`, with an adjacent `Fix(BUG-043)` comment stating the root cause and pitfall.
- [x] C2 — Does the reproducer test exist and genuinely cover the fixed behavior? `tests/inc/vec4_test.rs::accessor_test` exists, marked `// test_kind: bug_reproducer(BUG-043)`, and asserts `v.w() == 4`/`v.w() != v.z()` for both an integer (`I32x4`) and a float (`F32x4`) type.
- [x] C3 — Is `Quat<E>::w()` confirmed still an independent implementation (never delegating to the fixed `Vector::w()`, so this fix carried no risk of a double-fix or regression there)? Confirmed in `src/quaternion/general.rs`: `Quat::w()` has its own body, `self.0[ 3 ]`, with no call into `Vector::w()`.
- [x] C4 — Does the "currently zero live call sites beyond `Quat::w()`" claim still hold today (i.e. no new bare-`Vector<E,4>::w()` caller has been introduced since the fix that would need this exact regression check)? Workspace-wide `grep -rn "\.w()" --include="*.rs" .` (excluding `quaternion/` and the test file itself) → still exactly 3 matches, all `quat.w()` receivers in `module/math/ndarray_cg/src/d2/mat3x3/general.rs:235-237` — unchanged in count and location from the bug's own filing-time audit.

### Measurements

- [x] M1 — `Vector<E,4>::w()`'s body: now `self.0[ 3 ]` (was: `self.0[ 2 ]`, cite `git show 9b71cf39^:module/math/ndarray_cg/src/vector/vec4/general.rs` — pre-fix, `w()` was byte-identical to `z()`'s body; `9b71cf39` is the exact fix commit, confirmed via `git diff 9b71cf39^ 9b71cf39` matching this bug's own `## Fix Applied` section exactly).
- [x] M2 — `accessor_test` reproducer presence in `tests/inc/vec4_test.rs`: now `1` test (was: file did not exist at all — `git show 9b71cf39^:module/math/ndarray_cg/tests/inc/vec4_test.rs` → `fatal: path ... exists on disk, but not in '9b71cf39^'`, confirming the file, and its test, is wholly new to the fix commit).

### Invariants

- [x] I1 — Test suite (crate-scoped): `cargo nextest run -p ndarray_cg --all-features` (via `longrun`) → exit 0, 261/261 passed, 0 skipped (includes `vec4_test::accessor_test`, confirmed in the pass list).
- [x] I2 — Compiler/lints clean: `cargo clippy -p ndarray_cg --all-targets --all-features -- -D warnings` (via `longrun`) → exit 0, zero warnings/errors.

### Anti-faking checks

- [x] AF1 — Guards against `w()`'s index silently reverting to `2` (e.g. a careless copy-paste from `z()` again during an unrelated refactor): re-running C1/M1 (`self.0[ 3 ]`) and `accessor_test` (I1) must both still hold; the test's `assert_ne!( v.w(), v.z() )` exists specifically to catch this exact regression mode.
- [x] AF2 — Guards against a *new* bare-`Vector<E,4>::w()` call site being added elsewhere without re-auditing that it now receives the corrected accessor: C4's workspace-wide `.w()` grep is the re-check mechanism — any new match found outside `quaternion/`/`vec4_test.rs` should be spot-verified against the fixed `self.0[3]` body before being trusted.

## History

| Date       | Event  | Notes                                                                                                     |
|------------|--------|-------------------------------------------------------------------------------------------------------------|
| 2026-08-09 | filed  | Found during unrelated `todo.md` investigation; root cause confirmed via source read + `git blame` + workspace-wide `.w()` call-site grep before filing |
| 2026-08-09 | confirmed | VERIFY_PASS — Tier 2 Dual-Role Self-Check, 8/8 dimensions PASS after one Fix-and-Recheck Loop round; MRE re-executed and reproduces (exit 101) |
| 2026-08-09 | completed | Reproducer test added (`tests/inc/vec4_test.rs::accessor_test`), confirmed failing pre-fix, fix applied (`vector/vec4/general.rs:44`), confirmed passing post-fix (`cargo nextest run -p ndarray_cg --all-features`: 229/229 passed). Same-session, self-administered (filer = fixer = verifier) — Tier 2 Dual-Role Self-Check per `governance/maav.rulebook.md`'s default, not an independent PROC16-style acceptance pass. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟡 | 🟢 | Adversarial pass: `## Symptom` showed a raw source-code diff instead of actual terminal output (check 201) | Rewrote `## Symptom` to show the MRE's real captured terminal output (wrong vs. correct) |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Adversarial pass: `## Minimum Reproducible Example`'s **Actual** block paraphrased the real `rustc` panic text instead of matching it verbatim (check 206) | Re-ran the MRE fresh, captured verbatim output, corrected the **Actual** block |
| D3 | Cross-Reference Integrity | 🟡 | 🟢 | Adversarial pass: H3/E3's `mod_interface!`/impl-block line citations (`6-52`, `73-75`) were stale by one line — the `// BUG-043 ...` backreference comment added at filing time shifted every subsequent line down by one | Corrected citations to `6-53` (impl block) and `74-76` (`mod_interface!` block); re-verified every other `general.rs:NNN` citation in the file against current source line-by-line |
| D4 | Root Cause Quality | 🟡 | 🟢 | Adversarial pass: `## Root Cause` prose explained the mechanism but never explicitly cited `H1` by ID (check 401) | Added explicit "confirming **H1 (✅ Root Cause)**" citation |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 4 fixed | 4/4 |

**Reproduced:** YES — exit 101, 2026-08-09 (`/tmp/mre043/repro`, verbatim output captured and matched into `## Minimum Reproducible Example`).

**Aggregate verdict:** PASS — all 8 dimensions 🟢 after one Fix-and-Recheck Loop round (`governance/maav.rulebook.md § MAAV : Fix-and-Recheck Loop`); self-administered, no subagent dispatch (Verification Delegation would be forbidden per `file.rulebook.md § Report New Bug : Step 9 - VERIFY Gate`).

## Refs: src/

- `module/math/ndarray_cg/src/vector/vec4/general.rs` — fix `w()`'s body from `self.0[ 2 ]` to `self.0[ 3 ]` (line 44); backreference comment already added at line 40
