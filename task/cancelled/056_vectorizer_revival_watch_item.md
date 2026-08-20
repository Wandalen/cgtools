# Reconsider a properly-architected vectorizer if a real consumer emerges

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🚫 (Cancelled)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/vectorizer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **cancelled_at:** 2026-08-19 10:59:47
- **cancelled_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

`module/helper/vectorizer` (raster-to-vector image conversion, VTracer-based) was deleted by task 023
(`task/completed/023_vectorizer_feature_gate_decision.md`) after investigation found the real blocker
was a genuine architectural inversion, not a missing `#[cfg]` gate: core vectorization logic
(`actions::layers`/`actions::clusters`) depended directly on CLI-only `clap`-derived config types
(`commands::raster::vectorize::*`, gated behind `feature = "cli"`), so the library-only build path had
never actually compiled. Zero cross-references and zero tests existed anywhere in the workspace at
deletion time, so DELETE was the correct call for a P3 backlog item with no current consumer — fully
recoverable via git history (pre-deletion tree at commit `2be3d2cc`) if ever needed again.

**This is explicitly a tracking placeholder, not active work.** No implementation should begin
speculatively. If a real, concrete consumer need for raster-to-vector conversion emerges later, revisit
building a correctly-architected version: config types owned by `actions` (or a shared location), with
`commands` (CLI) depending on `actions` instead of the reverse, plus a real test suite (fixture images +
output assertions) written from the start rather than retrofitted. Until that trigger condition exists,
no further action is needed on this task.

**Related Tasks:** `023` (`task/completed/023_vectorizer_feature_gate_decision.md`) — this task exists
solely to keep the door open on 023's DELETE decision without disturbing 023's own terminal, verified
state. `✅ Completed` is terminal in this project's task system (v5.13 — the REOPEN transition was
removed; see `tsk.rulebook.md § Vocabulary : Regression Event`), so a distinct, cross-linked task is the
correct mechanism for "revisit later," not reopening 023 itself.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 10:59:47 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CANCEL | task cancelled |

## History

- **[2026-08-10]** `FILED` — Filed via lightweight Draft capture
  (`tsk.rulebook.md § Core Procedures : Procedure - Draft Task`, PROC8) at the user's explicit request to
  keep the vectorizer question "open for now" after reviewing task 023's DELETE decision and execution —
  explicitly without reverting the already-executed, already-verified code deletion. Classified via
  `tsk.rulebook.md § Task File : Deduplication Search` as Case E (closed task 023 exists, but scope
  differs: 023's scope — "decide fix vs delete" — is fully resolved; this task's scope — "watch for a
  future revival trigger" — is not the same question and isn't resolvable now). Cross-linked to 023 via
  `**Related Tasks:**` on both sides.
- **[2026-08-19]** `NOTE` — Resolved by explicit user decision: rather than continuing to wait for an
  in-workspace revival trigger, the pre-deletion crate is relocated outside the cgtools repository
  entirely, to a standalone sibling package, so it no longer needs cgtools' own architecture/lint
  constraints to exist. This repository's own tracked content must never reference absolute or relative
  paths to files outside the repository, so the relocation destination is intentionally not spelled out
  here. Recovered
  all 20 files (3870 lines) from the pre-deletion tree — confirmed via `git show --stat 2be3d2cc` (the
  deletion commit itself; note this task's own text above says "pre-deletion tree at commit `2be3d2cc`,"
  which is imprecise — `2be3d2cc` is the deletion commit, the pre-deletion tree is its parent,
  `2be3d2cc~1`) via whitelisted `git show 2be3d2cc~1:<path>` reads for each file, redirected to the new
  location; line counts match the deletion diff's own stat (Cargo.toml 96/96 exact; two files off by
  exactly 1 from a trailing-newline counting convention difference between `wc -l` and git's diff stat,
  not a content loss). `Cargo.toml`'s `workspace = true` dependency/lint inheritance doesn't resolve
  outside a Cargo workspace, so every dependency was inlined to the version pinned in cgtools' root
  `Cargo.toml` as of the pre-deletion parent commit `2be3d2cc~1` (not today's HEAD — today's versions
  are newer and broke the standalone build with 16 compile errors on first attempt, consistent with this
  session's recurring finding that these workspace-internal utility crates — `mod_interface`,
  `derive_tools`, `error_tools` — get breaking API changes across versions; the vintage-correct pins
  fixed it cleanly). `repository`/`homepage`/`documentation` metadata fields removed (they pointed at the
  cgtools GitHub location, no longer accurate); `changelog.md`'s own `[0.1.0]` release-tag link left
  unchanged since it documents a real past release, not a claim about current location. `[lints]
  workspace = true` dropped (cgtools' custom lint policy no longer applies once outside cgtools).
  Verified live: `cargo check --all-features` at the new standalone location exits 0 clean (no `tests/`
  directory existed at deletion time to re-verify — matches task 023's own "zero tests existed" finding).
  cgtools' own workspace is unaffected — no files changed inside this repo other than this task file
  itself. Closing this task via `tsk .cancel 056` (the correct terminal transition for an open Draft-state
  placeholder task whose watch condition is now resolved by relocation rather than by an in-workspace
  revival).
