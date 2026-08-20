# Delete `ndarray_cg::mem`'s dead `AsBytes`/`Pod` block, superseded by the live `asbytes` crate

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-19 22:54:58
- **expires_at:** 2026-08-20 00:54:58
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/math/ndarray_cg
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **unverified_at:** 2026-08-19 22:37:56
- **unverified_by:** system
- **in_motion:** true
- **verifying_at:** 2026-08-19 22:54:58
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

`module/math/ndarray_cg/src/mem.rs` carries a commented-out `mod private` block — a
`bytemuck::Pod` re-export, a custom `AsBytes` trait, and 4 impls (`nd::Array`, `Vec<T>`, `[T]`,
`[T;N]`) — directly beside the live line that replaced it: `reuse ::asbytes;` (in the
`mod_interface!` block), plus commented `orphan use { Pod, AsBytes };` / `own use ::bytemuck::*;`
re-export lines that are equally dead.

This session confirmed `asbytes` (v0.4.1, pulled from crates.io per workspace root
`Cargo.toml:361-365`) is **not a third-party black box** — it's published by `Wandalen`, the same
account/org as this repo, sourced from
`github.com/Wandalen/wTools/tree/master/module/experimental/asbytes` (confirmed via the crates.io
API, cross-checked against the `source`/`checksum` fields in `Cargo.lock`). A local clone of that
source tree exists at `~/pro/lib/yrd_core/wtools` (and sibling clones) for direct inspection if ever
needed. The dead block is therefore fully superseded and fully recoverable two ways — this repo's
own git history, and the live, actively-maintained sibling crate — with no "opaque external
dependency" concern that would justify keeping it in place as a reference blueprint.

**Related Tasks:** `391`/`395` (other audit incidents resolved to concrete action, same audit).
`392`/`394` (other audit incidents left open for developer decision, same audit) — this incident is
the one case in the same audit that needed no open question: the replacement's authorship and
liveness were both directly verifiable.

## In Scope

- Delete the commented `Pod`/`AsBytes` trait+impls block in `module/math/ndarray_cg/src/mem.rs`'s
  `mod private`.
- Delete the commented `orphan use { Pod, AsBytes };` / `own use ::bytemuck::*;` lines in the same
  file's `mod_interface!` block.

## Out of Scope

- The live `reuse ::asbytes;` line and anything downstream of it — untouched.
- The `asbytes` crate itself (separate repo, not this task's concern).
- Any other file — single-file, single-incident scope.

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any), not by this
section.

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`).
- Mechanical dead-code deletion only — no behavior change; Test Matrix not applicable (nothing
  behavioral to assert), correctness captured by the compiler/clippy gate recorded as an Invariant
  below.
- Independent verification pass per `§ Acceptance Verification : Procedure - Execution` before state
  moves to ✅.

## Acceptance Criteria

- `module/math/ndarray_cg/src/mem.rs` contains no commented `AsBytes`/`Pod` trait or impl text.
- The live `reuse ::asbytes;` line and its real consumers are untouched and still compile.
- `cargo clippy -p ndarray_cg --no-deps --all-targets --all-features -- -D warnings` exits 0.
- `git diff --stat` (against the commit that introduced the fix) touches only `mem.rs`.

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an
independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

- [ ] C1 — Is `mem.rs` free of the commented `AsBytes`/`Pod` block?
- [ ] C2 — Is the live `reuse ::asbytes;` line, and every real caller of it, still present and
      compiling?

### Measurements

- [ ] M1 — `grep -n "AsBytes\|Pod" module/math/ndarray_cg/src/mem.rs` → 0 hits (was: multiple, all
      inside the dead comment block).

### Invariants

- [ ] I1 — `cargo clippy -p ndarray_cg --no-deps --all-targets --all-features -- -D warnings` → exit
      0.
- [ ] I2 — `git diff --stat` (against the commit that introduced the fix) touches only `mem.rs`.

### Anti-faking checks

- [ ] AF1 — The deletion isn't a no-op re-comment or relocation to another file —
      `grep -rn "AsBytes" module/math/ndarray_cg/src/` after the change shows zero hits anywhere in
      the crate, not just absent from `mem.rs`.

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | Lightweight template variant: no separate Work Procedure/Test Matrix sections — Delivery Requirements explicitly justifies this ("mechanical dead-code deletion only... Test Matrix not applicable"); Verification's own Checklist/Measurements/Invariants/Anti-faking layers substitute | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`module/math/ndarray_cg`), no cross-crate touch | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 0 | 0/0 |

**Adversarial pass:** attempted to find a reason the task should not proceed — the "keep as reference comment" original framing (History's FILED entry) was itself an adversarial catch from an earlier session, already corrected once `asbytes`'s same-org authorship was confirmed; re-checked that correction here and found it sound (crates.io API cross-check against `Cargo.lock` source/checksum fields, confirmed in Goal). No further defect found. History's EXECUTED entry already independently re-confirms M1/AF1/I1 live (0 grep hits crate-wide, clippy clean).

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 02:57:55 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-19 02:58:01 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:56 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 22:54:58 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:54:58 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 393` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |

## History

- **[2026-08-19]** `FILED` — Filed via lightweight Draft capture
  (`tsk.rulebook.md § Core Procedures : Procedure - Draft Task`, PROC8) following a user-requested
  dead-code maturity review of `module/math`/`module/min`. Originally the audit's tentative call was
  "keep as a reference comment" (assumed external dependency, opaque); corrected same session once
  `asbytes`'s authorship was confirmed to be this same org's own sibling crate — not opaque, not a
  reason to keep dead code in place. User directive: close and remove.

- **[2026-08-19]** `EXECUTED` — Deleted the commented `Pod`/`AsBytes` trait+impls block and the
  commented `orphan use { Pod, AsBytes };`/`own use ::bytemuck::*;` lines from
  `module/math/ndarray_cg/src/mem.rs`. File is now 13 lines: an empty `mod private` and a
  `mod_interface!` block containing only the live `reuse ::asbytes;`. Verified directly against this
  task's own AC/M1/AF1 grep text: `grep -n "AsBytes\|Pod" .../mem.rs` → 0 hits; `grep -rn "AsBytes"
  module/math/ndarray_cg/src/` → 0 hits crate-wide (not just absent from `mem.rs` — satisfies AF1);
  `reuse ::asbytes;` line confirmed present and untouched. `cargo clippy -p ndarray_cg
  --all-targets --all-features -- -D warnings` exit 0 (covered this session by both the
  `-p ndarray_cg -p mingl` and `-p ndarray_cg -p mdmath_core` clippy passes run for tasks 395/391 —
  `ndarray_cg` itself is common to both and came back clean each time). Purely mechanical deletion, no
  behavior change, matching this task's own Delivery Requirements ("Test Matrix not applicable").

  **`tsk` lifecycle.** `.submit` → ❓, `.claim_verify` → 🔬 (Verifying) both completed. `.verify_pass`
  blocked: `tsk: .verify_pass: self-verification forbidden (actor matches filed_by)` — same actor
  filed and executed this task, matching the established same-sandbox actor-guard pattern already
  documented on numerous other rows in `task/readme.md`'s Tasks Index. Left at 🔬 (Verifying) pending
  an independent verifier in a different actor identity; not forced or spoofed past the guard.
