# Remove unused `animation` dependency from `tiles_tools`

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tiles_tools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **verification_date:** 2026-08-16 05:36:58
- **blocked_by:** null
- **in_motion:** false
- **accepting_at:** 2026-08-16 05:23:28
- **accepting_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **accepted_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ (independent acceptance-verification session — see Outcomes § Acceptance Results)
- **accepted_at:** 2026-08-16
- **priority:** 0
- **completed_at:** 2026-08-16 05:36:58
- **completed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

Restore the "GPU-free by dependency surface" invariant `docs/layer/005_l4_scene_model.md`
and `tilemap_scene`'s own [invariant/003](../../module/helper/tilemap_scene/docs/invariant/003_compiles_to_renderer_commands_only.md)
claim for the tile stack's scene model — currently false. `tiles_tools`'s
default-on `enabled` feature unconditionally lists `"dep:animation"`
(`Cargo.toml` line 41), and `animation` is declared as an optional
workspace dependency (line 80) with no feature gate narrowing it back out —
so every default-featured build of `tiles_tools` (and everything that
depends on it, including `tilemap_scene`) pulls `animation` in, which
transitively pulls in `minwebgl`/`mingl`'s GL-context layers. Confirmed via
`cargo tree -p tilemap_scene -i minwebgl`, which shows the path
`minwebgl → animation → tiles_tools → tilemap_scene`. Yet `tiles_tools`'s
own source never references the dependency at all:
`grep -rn "animation::" module/helper/tiles_tools/src/` returns zero
matches. This is dead weight, not a real dependency — removing it is a
direct code fix, not a documentation softening, because the invariant it
violates is a real, valuable, already-committed one (native/headless
testability of the tile stack's scene model) that the codebase already
claims to hold. Matters now because the 2026-08-15/16 `docs/layer` gap
audit flagged this as one of 3 High-severity findings against
`docs/layer/005_l4_scene_model.md`. Bounded to a 2-line `Cargo.toml` edit in
this one crate. Testable: `cargo tree -p tilemap_scene -i minwebgl` returns
"package ID specification... did not match any packages" (empty result)
where it previously returned a real path.

## In Scope

- `module/helper/tiles_tools/Cargo.toml`: remove `"dep:animation"` from the
  `enabled` feature list; remove the
  `animation = { workspace = true, optional = true }` line from
  `[dependencies]`.

## Out of Scope

- `animation`'s own source, or any of its other consumers (`renderer`,
  `scene_script`, `primitive_generation`) — this task only removes
  `tiles_tools`'s own unused pull-in; every other consumer's dependency on
  `animation` is unaffected and untouched.
- `primitive_generation`'s similar math-only feature gate on `minwebgl` —
  already flagged as its own, separate, open classification gap in
  `docs/layer/001_l0_drivers.md`'s Beside-the-Ladder Consumers section; not
  this task's scope.
- Any source-code change — `grep -rn "animation::" module/helper/tiles_tools/src/`
  already confirms zero call sites exist to update.
- Re-auditing every other crate's dependency surface for similar unused
  pulls — scoped to `tiles_tools` (and its effect on downstream
  `tilemap_scene`) only.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Removal lands with zero behavior change to any currently-passing test —
    the dependency is confirmed unused, so no call site needs updating
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a command that demonstrated the
    problem before the implementing change landed
-   Minimum change to satisfy Test Matrix — no features beyond requirements
-   `verb/test` passes with zero failures and zero warnings
-   Independent verification passes per `§ Acceptance Verification :
    Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to
    `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|--------------------|
| T01 | `cargo tree -p tilemap_scene -i minwebgl` after the `Cargo.toml` edit | Workspace dependency graph | Returns no matching package (was: a real `minwebgl → animation → tiles_tools → tilemap_scene` path) |
| T02 | `cargo build -p tiles_tools --all-features` | Full-feature build | Compiles clean — confirms no hidden call site depended on `animation` being pulled in |
| T03 | `cargo test -p tiles_tools` (default features) and `cargo test -p tilemap_scene` | Pre-existing test suites of the crate and its direct downstream consumer | Zero regressions — every test that passed before the edit still passes |

## Acceptance Criteria

-   `Cargo.toml`'s `enabled` feature list no longer contains `"dep:animation"`
-   `Cargo.toml`'s `[dependencies]` no longer declares `animation`
-   `cargo tree -p tilemap_scene -i minwebgl` returns no matching package
-   `cargo build -p tiles_tools --all-features` and `cargo test -p tiles_tools`
    both succeed
-   `cargo test -p tilemap_scene` shows zero regressions
-   Every Test Matrix row has a corresponding passing command

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting (via EXEC_COMPLETE, ⚙️ → 📦, followed by an acceptance CLAIM, 📦 → 🔎).

### Checklist

Desired answer for every question is YES.

**Dependency removal**
- [x] C1 — Is `"dep:animation"` absent from the `enabled` feature list in `module/helper/tiles_tools/Cargo.toml`?
- [x] C2 — Is the `animation = { workspace = true, optional = true }` line absent from `[dependencies]`?

**Out of Scope confirmation**
- [x] C3 — Is `module/helper/tiles_tools/src/` unmodified (`git diff` shows `Cargo.toml` only)?
- [x] C4 — Do `renderer`, `scene_script`, and `primitive_generation`'s own `Cargo.toml` files remain unmodified?

### Measurements

- [x] M1 — `cargo tree -p tilemap_scene -i minwebgl 2>&1` → contains "did not match any packages" (was: a real dependency path)
- [x] M2 — `cargo tree -p tiles_tools -i minwebgl 2>&1` → contains "did not match any packages" (was: a real dependency path)

### Invariants

- [x] I1 — test suite: `verb/test` → 0 failures
- [x] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check -p tiles_tools --all-features` → 0 warnings

### Anti-faking checks

- [x] AF1 — M1/M2's "no match" result is verified by reading the actual command output (not merely a zero exit code, since `cargo tree -i` can exit non-zero on a true no-match) — confirm the specific "did not match any packages" text appears

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | — | — |

## Outcomes

### Acceptance Results

- **Verified by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ (independent acceptance verifier — did not implement this task; fresh read of all artifacts this session)
- **Date:** 2026-08-16
- **Verdict:** PASS

#### Checklist

- [x] C1 — Is `"dep:animation"` absent from the `enabled` feature list in `module/helper/tiles_tools/Cargo.toml`? — YES: direct read of the current file (lines 33-41) shows `enabled = [ "dep:error_tools", "dep:former", "dep:mod_interface", "dep:hecs", "dep:pathfinding", "dep:serde", "dep:ndarray_cg" ]` — no `"dep:animation"` entry present.
- [x] C2 — Is the `animation = { workspace = true, optional = true }` line absent from `[dependencies]`? — YES: direct read of `[dependencies]` (lines 69-90) contains no `animation` entry. `git show de61eec9 -- module/helper/tiles_tools/Cargo.toml` confirms the exact 2-hunk diff that removed both the feature-list entry and the dependency declaration (plus its `# Animation` comment).
- [x] C3 — Is `module/helper/tiles_tools/src/` unmodified (`git diff` shows `Cargo.toml` only)? — YES, with a documented caveat: `git diff HEAD -- module/helper/tiles_tools/` and `git status --porcelain -- module/helper/tiles_tools/` both return empty (fully committed, zero uncommitted diff anywhere under the crate right now). The commit carrying this task's fix (`de61eec9`, a bundled commit per this repo's known concurrent-actor-authoring pattern) also touches ~13 other `tiles_tools/src/`+`tests/` files in the same commit — content-inspected, not just stat-counted: e.g. `src/ecs/components.rs`'s hunk is a `Fix(BUG-132)` to `tiles_tools`'s own internal ECS `Animation` *component* (a sprite frame-timer advance-loop bug), an entirely different `Animation` than the external `animation` crate this task removes. `grep -rn "animation::" module/helper/tiles_tools/src/` → zero matches (exit 1), confirming the crate never called into the removed external crate, so this task's own scope never required a source change, and the bundled src/ changes are unrelated (test-coverage-expansion) work riding the same commit, not scope creep by this task's implementation.
- [x] C4 — Do `renderer`, `scene_script`, and `primitive_generation`'s own `Cargo.toml` files remain unmodified? — YES, with a documented caveat: `renderer/Cargo.toml` and `scene_script/Cargo.toml` show no recent commit touching them and zero uncommitted diff; both still declare their own intentional, untouched `animation` dependencies (`renderer/Cargo.toml:63` `animation = { workspace = true, optional = true }`; `scene_script/Cargo.toml:32` `animation = { workspace = true }`). `primitive_generation/Cargo.toml` WAS modified in the same bundled commit `de61eec9`, but its diff is entirely a `kurbo` dev-dependency addition plus 3 new `[[test]]` registrations gated on `text`/`font-processing` features — zero mention of `animation` or dependency-surface concerns anywhere in that hunk; unrelated bundled work, not this task's own scope creep.

#### Measurements

- [x] M1 — `cargo tree -p tilemap_scene -i minwebgl 2>&1` → `error: package ID specification \`minwebgl\` did not match any packages` — MET (expected: output contains "did not match any packages"; was previously a real `minwebgl → animation → tiles_tools → tilemap_scene` path per this task's own Goal section).
- [x] M2 — `cargo tree -p tiles_tools -i minwebgl 2>&1` → `error: package ID specification \`minwebgl\` did not match any packages` — MET (same expected text, same command shape scoped to `tiles_tools` directly).

#### Invariants

- [x] I1 — test suite: `verb/test` → 0 failures — HOLD. Full workspace suite launched detached this session via `longrun .launch dir::/home/user1/pro/lib/yrd_gamedev/cgtools -- verb/test` (Durable Log `-0033_longrun.log`, pid 1923241, exit 0, elapsed 148s). Results: native `cargo nextest run --all-features --workspace` → "1834 tests run: 1834 passed, 0 skipped"; `cargo test --doc --all-features --workspace` → every doc-test suite 0 failed (e.g. tiles_tools: "40 passed; 0 failed"); `cargo clippy --all-targets --all-features --workspace -- -D warnings` → clean (script runs under `set -euo pipefail`, so a clippy failure would have aborted before the wasm32 stages, which did run); wasm32 check → "52 example(s) checked, 0 failed"; wasm32 test → "3 crate(s) tested, 0 failed". A prior log (`-0032_longrun.log`, completed 05:12:39) existed but was NOT reused as evidence: file-mtime check showed `line_tools` shader/test files were edited at 05:15-05:22, after that run completed, so its result no longer reflected the current tree — re-ran fresh instead per the staleness-check requirement.
- [x] I2 — compiler clean: `RUSTFLAGS="--cfg web_sys_unstable_apis -D warnings" cargo check -p tiles_tools --all-features` → 0 warnings — HOLD. (`--cfg web_sys_unstable_apis` added to the override per this repo's documented `.cargo/config.toml`-clobber pitfall — a bare `RUSTFLAGS="-D warnings"` would silently drop the required cfg.) First pass returned a suspiciously fast cached "Finished ... in 0.36s" with no "Compiling" line, so it was re-verified by forcing a genuine rebuild (`touch src/lib.rs`, re-run with `-v`): confirmed `Dirty tiles_tools ... the file ... has changed`, followed by a real `rustc` invocation whose full argument list ends `--cfg web_sys_unstable_apis -D warnings` (both flags present — override did not clobber the required cfg) and exits 0 with zero warnings. Bonus corroboration: that invocation's `--extern` list (`bincode`, `error_tools`, `former`, `hecs`, `mod_interface`, `ndarray_cg`, `pathfinding`, `ron`, `rustc_hash`, `serde`, `serde_json`) contains no `--extern animation`, independently confirming M2/C1/C2 at the compiled-artifact level.

#### Anti-faking checks

- [x] AF1 — M1/M2's "no match" result verified by reading actual command output text, not exit code alone — PASS. Both M1 and M2's raw stderr text contains the literal substring "did not match any packages" (quoted above, read directly from command output). Exit code was 101 (non-zero) in both cases — consistent with this task's own warning that `cargo tree -i` exits non-zero on a genuine no-match, so exit code alone would have been ambiguous; the literal text, not the exit code, is what was relied on.

**Falsification notes (adversarial pass):** attempted to disprove C1/C2 by reading the raw file rather than trusting the task description's claim of what changed. Attempted to disprove C3/C4 by expanding scope from "is the named crate's Cargo.toml touched" to "read every diff hunk in the enclosing bundled commit that touches these paths" — found real modifications to `tiles_tools/src/` and `primitive_generation/Cargo.toml` that a shallower `git status`-only check would have flagged as violations, then content-verified each was unrelated to `animation` (BUG-132's own internal `Animation` component; `kurbo`/test-registration additions) rather than accepting "the file changed" or "the file didn't change" at face value. Attempted to disprove I1 by checking for tree drift since the last existing full-suite log instead of citing it uncritically — found genuine drift (line_tools) and re-ran rather than citing stale evidence. Attempted to disprove I2 by treating a too-fast "Finished" as a stale-cache red flag rather than accepting it, then forced and confirmed a genuine recompilation. No Blocking findings on any of the 9 items; two Non-Blocking documentation caveats recorded on C3/C4 above (bundled-commit noise, content-verified unrelated).

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-16 05:22:56 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | EXEC_COMPLETE | execution complete |
| 2026-08-16 05:23:28 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-16 05:36:58 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | ACCEPTANCE_PASS | acceptance passed |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-16]** `FILED` — Task filed via `/doc_tsk` Phase 3 (docs/layer gap audit): remove `tiles_tools`'s unused default-on `animation` dependency, restoring the GPU-free-by-dependency-surface invariant for `tilemap_scene`.

## Related Documentation

- `docs/layer/005_l4_scene_model.md` — the layer doc whose "GPU-free by dependency surface" claim this task restores to true
- `module/helper/tilemap_scene/docs/invariant/003_compiles_to_renderer_commands_only.md` — the crate-level invariant this task's fix upholds
- `module/helper/tiles_tools/Cargo.toml` — the file this task edits
