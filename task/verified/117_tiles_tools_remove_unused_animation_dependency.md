# Remove unused `animation` dependency from `tiles_tools`

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **started_at:** 2026-08-16
- **expires_at:** null
- **round:** 1
- **state:** ⚙️ (Executing)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tiles_tools
- **verified_by:** user1@w002
- **verification_date:** 2026-08-16
- **blocked_by:** null

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
- [ ] C1 — Is `"dep:animation"` absent from the `enabled` feature list in `module/helper/tiles_tools/Cargo.toml`?
- [ ] C2 — Is the `animation = { workspace = true, optional = true }` line absent from `[dependencies]`?

**Out of Scope confirmation**
- [ ] C3 — Is `module/helper/tiles_tools/src/` unmodified (`git diff` shows `Cargo.toml` only)?
- [ ] C4 — Do `renderer`, `scene_script`, and `primitive_generation`'s own `Cargo.toml` files remain unmodified?

### Measurements

- [ ] M1 — `cargo tree -p tilemap_scene -i minwebgl 2>&1` → contains "did not match any packages" (was: a real dependency path)
- [ ] M2 — `cargo tree -p tiles_tools -i minwebgl 2>&1` → contains "did not match any packages" (was: a real dependency path)

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check -p tiles_tools --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — M1/M2's "no match" result is verified by reading the actual command output (not merely a zero exit code, since `cargo tree -i` can exit non-zero on a true no-match) — confirm the specific "did not match any packages" text appears

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

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-16]** `FILED` — Task filed via `/doc_tsk` Phase 3 (docs/layer gap audit): remove `tiles_tools`'s unused default-on `animation` dependency, restoring the GPU-free-by-dependency-surface invariant for `tilemap_scene`.

## Related Documentation

- `docs/layer/005_l4_scene_model.md` — the layer doc whose "GPU-free by dependency surface" claim this task restores to true
- `module/helper/tilemap_scene/docs/invariant/003_compiles_to_renderer_commands_only.md` — the crate-level invariant this task's fix upholds
- `module/helper/tiles_tools/Cargo.toml` — the file this task edits
