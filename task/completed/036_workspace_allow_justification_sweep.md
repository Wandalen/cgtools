# Workspace-wide sweep: justify or remove unexplained #[allow] attributes

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
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
- **[2026-08-10]** `IMPLEMENTED` — Executed per the Goal's own decomposition note: sized the sweep
  (census: **1905 allow sites across 102 crates** — not one diff), built the decision-critical
  inheritance map (72/102 crates have `[lints] workspace = true`; non-inheriting: mdmath_core,
  ndarray_cg, embroidery_tools + 27 example crates, whose blanket allows largely suppress lints that
  are not even enabled for them), and discovered `[workspace.lints.clippy]` already centrally allows
  several of the commonly file-suppressed lints WITH justification comments (Cargo.toml 71-98) — so
  many file-level copies are pure redundancy, and example crates share a copy-pasted template block
  reducible to one template-level decision. Executed the Goal's named concrete instance empirically:
  removed all 8 blanket `#![allow(clippy::...)]` from `primitive_generation/src/text/ufo.rs`, ran
  scoped clippy — only 5 of 8 lints actually fire (3 were stale suppressions: cloned_instead_of_copied,
  unnecessary_cast, redundant_closure_for_method_calls — stayed removed). Fixed the code for 4:
  explicit_iter_loop ×4 (`&mut` loop forms), uninlined_format_args ×2, needless_continue
  (`_ => continue` → `_ => {}`, behavior-identical as last loop statement), semicolon_if_nothing_returned.
  Re-added 1 as a scoped fn-level attribute with justification: too_many_lines on `from_glif` (117-line
  linear XML event state machine). Verified: `cargo clippy -p primitive_generation --all-targets
  --all-features` now ZERO warnings, `cargo test -p primitive_generation --all-features` all pass
  (2+3 unit + 3 doc), both exit 0. Remainder decomposed into draft/058 (per-crate procedure proven here,
  census table, inheritance map, examples-template tranche).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Tier 2 dual-role gate check passed 15/15. In-loop
  adversarial catches: (1) the confirming plan assumed all 8 ufo.rs suppressions were live — empirical
  removal proved 3 stale, changing the fix from "justify 8" to "remove 3, fix 4, justify 1"; (2) the
  seemingly-equivalent `_ => continue` → `_ => {}` rewrite was verified against the surrounding control
  flow (match is the loop body's final statement) before applying — in any other position it would have
  changed behavior; (3) initial per-line-number fix targeting was abandoned after line drift between
  clippy passes — switched to content-anchored edits.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | Goal's decomposition note followed: census + inheritance map + concrete instance + successor draft | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value / YAGNI | 🟢 | 🟢 | One successor draft, not 25; examples tranche reduced to a single template decision | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | Successor carries the proven 5-step per-crate procedure | — |
| D5 | Execution Scope | 🟢 | 🟢 | Code edits confined to ufo.rs; draft/058 + index the only other writes | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | Proper-fix-over-suppression honored: 4 fixed in code, only 1 justified suppression retained | — |
| B2 | Test-First | 🟡 | 🟢 | Plan assumed all 8 suppressions live; empirical clippy run disproved 3 | Stale trio removed outright instead of justified |
| B3 | Evidence of Failure | 🟢 | 🟢 | Pre-fix clippy log: 9 warnings (5 unique lints) on record in -0002_longrun.log | — |
| B4 | Proper Fix Only | 🟡 | 🟢 | `_ => continue` → `_ => {}` is behavior-identical ONLY as the loop's final statement | Control-flow position verified before edit |
| B5 | Fix Verification | 🟢 | 🟢 | Post-fix: clippy ZERO warnings, tests 2+3+3 pass, both exit 0 | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Census, inheritance map, and procedure live in draft/058; justification comment lives on from_glif | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | No line-number-targeted edits survived (drift caught); content-anchored edits only | — |
| **Total** | | 🔴 | 🟢 | 2 findings resolved | 2/2 |
