# Reconsider a properly-architected vectorizer if a real consumer emerges

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📝 (Draft)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/vectorizer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

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

## History

- **[2026-08-10]** `FILED` — Filed via lightweight Draft capture
  (`tsk.rulebook.md § Core Procedures : Procedure - Draft Task`, PROC8) at the user's explicit request to
  keep the vectorizer question "open for now" after reviewing task 023's DELETE decision and execution —
  explicitly without reverting the already-executed, already-verified code deletion. Classified via
  `tsk.rulebook.md § Task File : Deduplication Search` as Case E (closed task 023 exists, but scope
  differs: 023's scope — "decide fix vs delete" — is fully resolved; this task's scope — "watch for a
  future revival trigger" — is not the same question and isn't resolvable now). Cross-linked to 023 via
  `**Related Tasks:**` on both sides.
