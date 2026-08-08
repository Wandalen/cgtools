# Workspace-wide sweep: justify or remove unexplained #[allow] attributes

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📝 (Draft)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

`module/helper/primitive_generation/src/text/ufo.rs` has 8 blanket `#![allow(clippy::...)]` attributes
(lines 4-11, confirmed by direct read this session) with zero justification comments — used as the
concrete first-hand example of a systemic pattern the audit found repeated across the workspace (P8 —
mechanical hygiene tier). Sweep every `#[allow(...)]`/`#![allow(...)]` attribute workspace-wide
(`grep -rn "#!\?\[allow("`); for each, either add a one-line comment explaining the specific reason the
lint is suppressed, or remove the attribute and fix the underlying lint if it's not actually justified.
**This is a large, mechanical, cross-cutting sweep — likely worth decomposing per-crate at pickup** rather
than one giant diff, similar to task 035's own decomposition note.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P8 (mechanical
  hygiene) tier, Fix-in-place bucket (cross-cutting).
