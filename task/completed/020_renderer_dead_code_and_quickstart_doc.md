# Resolve renderer's Composer/raw.rs dead code and fix non-compiling Quick Start doc

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/renderer
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Three renderer hygiene items bundled together (P3, dead-code/hygiene bucket, Fix-in-place — decide
wire-in-vs-delete for the code items, then fix the doc): (1)
`module/helper/renderer/src/webgl/post_processing/composer.rs` — `Composer` has 5 in-file references
(struct, impl, doc comments, export at line 226) but zero references anywhere else in the workspace
(confirmed via workspace-wide grep this session) — decide whether to wire it into the actual render
pipeline or delete it; (2) `module/helper/renderer/src/webgl/material/raw.rs` — confirmed 0 bytes this
session — delete after confirming no `mod raw;` declaration still references it; (3) the crate's readme
Quick Start example doesn't compile against the current API — carried forward from the audit triage plan,
re-confirm the exact mismatch against current `module/helper/renderer/src/` before rewriting.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P3 (dead code)
  tier merged with a P5 (doc drift) item for the same crate, Fix-in-place / Delete-candidate bucket.

- **[2026-08-10]** `IMPLEMENTED` — All three Goal items executed.

  **Item 1 — `Composer` deleted (wire-in rejected), file renamed to match survivors.** Deadness
  re-derived fresh: workspace-wide grep finds zero `Composer` references outside its own file; the only
  wiring is `layer composer;` in `post_processing/mod.rs`. Decisive against wire-in: `Composer`'s one
  reason to exist — chaining passes — is its `render` method, and that method is a *commented-out
  block* (itself a hygiene violation); the struct never worked. All 14 real rendering examples chain
  passes manually via `SwapFramebuffer` and work — wiring `Composer` in would mean finishing an
  unstarted design and refactoring 14 working call sites (YAGNI). Intent check via whitelisted
  `git log -S "struct Composer"`: a single bulk-motion commit (`ffe4cecd` "Start of moving renderer"),
  no design intent anywhere. Deleted the struct + impl (53 lines incl. the commented-out block). The
  same file exports `Pass` (8 in-crate implementors) and `SwapFramebuffer` (`renderer.rs` + 14
  examples) — both verified live *before* deciding scope, so the file stays; renamed
  `composer.rs` → `pass.rs` so the filename matches its surviving content, updated
  `post_processing/mod.rs` (`layer pass;` with an accurate doc line). The rename is invisible to every
  consumer: all imports go through layer-propagated paths (`post_processing::{ Pass, SwapFramebuffer }`
  or `webgl::SwapFramebuffer`), none name `composer::` — verified by grep plus compiling all 14
  consumers. Also fixed `SwapFramebuffer`'s broken doc comment (first line was `//` instead of `///`,
  truncating the rendered doc).

  **Item 2 — `material/raw.rs` deleted.** Re-confirmed 0 bytes; `material/mod.rs` read in full and
  workspace-grepped — no `mod raw`/`layer raw` declaration exists anywhere, the file was completely
  orphaned.

  **Item 3 — Quick Start re-confirmed broken against current API, rewritten, and permanently gated.**
  Exact mismatches, each source-cited: `Renderer::new` returns `Result` (`renderer.rs:480`) but the
  snippet used it bare; `Renderer::render` takes `&mut self` and `&mut Scene` (`renderer.rs:681`) but
  the snippet took `&Renderer`; `ToneMappingPass::< ToneMappingAces >::new` takes only `gl`
  (`tonemapping.rs:68`) but the snippet passed width/height; `gl::context::ContexOptions` is a typo
  for `ContextOptions` (`minwebgl/src/context.rs`); glTF scenes are `Vec< Rc< RefCell< Scene > > >`
  (`loaders/gltf.rs:57`) but the snippet indexed them as plain `&Scene`; and the second snippet
  referenced an out-of-scope `canvas` with unimported types — it could never have compiled as written.
  Root cause of the rot: the readme was compiled by nothing (no doc-include in `lib.rs`), so no gate
  ever saw these blocks. Fix in two parts: (1) rewrote all three `rust` blocks (Basic Setup, Render
  Loop, Asset Loading) as compiling `rust,no_run` doc tests in house codestyle, distilled from the
  canonical `examples/minwebgl/postprocessing` pipeline; (2) wired the readme into the crate docs via
  `#![ cfg_attr( doc, doc = include_str!( ... ) ) ]` in `lib.rs` (the workspace pattern
  `primitive_generation` already uses, comment citing TASK-020) — Quick Start drift now fails
  `cargo test --doc` instead of rotting silently.

  **Verification** (all detached via `longrun`, package-scoped): battery log `-0026` — (1)
  `RUSTFLAGS="-D warnings" cargo check -p renderer` default: clean; (2) `--all-features`: clean; (3)
  `cargo nextest run -p renderer --all-features`: 79/79 (incl. the native-backend pixel tests); (4)
  `RUSTDOCFLAGS="-D warnings" cargo test --doc -p renderer --all-features`: 3/3, test names confirm
  they are exactly the three readme blocks (`lib.rs - (line 66/108/172)`), and renderer had zero
  pre-existing doc tests to confound the count; (5) `cargo clippy --all-targets --all-features
  -- -D warnings`: clean. Log `-0028` — (6) `cargo check` of all 14 real consumers
  (`minwebgl_gltf_viewer`, `morph_targets`, `animation_blending`, `minwebgl_sun_grid_lines`,
  `skeletal_animation`, `shadowmap`, `lottie_surface_rendering`, `text_rendering`, `postprocessing`,
  `character_control`, `pbr_lighting`, `renderer_with_outlines`, `curve_surface_rendering`,
  `animation_surface_rendering`): clean. **Mutation check of the new gate** (logs `-0029`/`-0030`):
  temporarily reintroduced the old snippet's wrong-arity call — doc test FAILED with E0061 on exactly
  the mutated block (2/3 others still passing), then restored and re-ran 3/3 green. The gate
  discriminates; the old content's defect class is executed-RED on record, not just asserted.

- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Self-administered Tier 2 Dual-Role Self-Check (see
  `## Verification Record`). Confirming pass re-verified deadness/orphanhood of both deletion targets
  fresh this session (not trusting the filing-time analysis), re-derived every Quick Start mismatch
  against current source with file:line citations before rewriting, and ran the full battery. The
  adversarial pass: (a) tried to refute "Composer is dead" via docs, git intent, and path-based
  imports — sustained (zero md mentions, single bulk-move commit, no `composer::` path anywhere);
  (b) probed whether deleting `Composer` regresses the readme's "Custom passes" claim — no, custom
  passes are `impl Pass`, which stays; (c) challenged the passing doc-test count as possibly
  pre-existing tests — refuted by test names (all three attribute to the readme include site) and by
  the crate having had no doc tests before; (d) mutation-checked the new gate rather than trusting a
  green first run (E0061 RED captured, then green restored); (e) audited the battery's two
  environment-caused interruptions (concurrent session deleted `target/debug` mid-build → E0460;
  transient disk-full → ENOSPC) and one self-caused one (three package names assumed from dir names —
  real names differ: `animation_blending`, `minwebgl_gltf_viewer`, `minwebgl_sun_grid_lines`;
  corrected by reading each `Cargo.toml`, not by shrinking the sweep) — all three interruptions
  re-run to green on record, none waved through. Working tree for the crate: exactly the six
  task-020 paths plus the five pre-existing uncommitted paths from the earlier native-pixel-test
  task — no strays, no backups. All 15 dimensions PASS; state → ✅ Completed.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | All three Goal items executed; the `pass.rs` rename is the completion of item 1's delete (a file named after its deleted occupant would be fresh doc drift), not scope growth | — |
| D2 | MOST Goal Quality | — | 🟢 | Motivated (dead abstraction, orphaned file, uncompilable Quick Start), Observable (grep/compile), Scoped (one crate), Testable (compile matrix + doc tests + consumer sweep) | — |
| D3 | Value / YAGNI | — | 🟢 | Null Hypothesis: skip → the crate's front-door example teaches an API that cannot compile (5 distinct contradictions), a stillborn abstraction keeps its export slot, and an empty file sits in `material/`. Wire-in of Composer was itself rejected on YAGNI grounds | — |
| D4 | Implementation Readiness | — | 🟢 | Wire-in-vs-delete fork resolved on evidence: commented-out core method, zero users, 14 working manual call sites, no recorded design intent | — |
| D5 | Execution Scope | — | 🟢 | Goal's own precondition steps honored: `mod raw;` absence confirmed before deleting raw.rs; Quick Start mismatch re-confirmed against current source (file:line for each defect) before rewriting; `Pass`/`SwapFramebuffer` liveness checked before deciding file fate | — |
| D6 | Crate Scope Unity | — | 🟢 | All edits inside `renderer` (readme.md, src/lib.rs, post_processing/{pass.rs,mod.rs}, material/raw.rs deletion) | — |
| D7 | Crate Locality | — | 🟢 | Fixes target the exact crate owning the dead code and the drifted doc; consumers untouched (verified unaffected by compilation, not assumption) | — |
| D8 | Crate Single Responsibility | — | 🟢 | No responsibility change — removed a non-functional abstraction and made docs match the real API | — |
| B1 | Rulebook Compliance | — | 🟢 | House codestyle in all snippets and edits; delete-don't-archive (commented-out render block deleted with its struct, not preserved); doc-include mirrors the existing workspace pattern | — |
| B2 | Test-First Requirement | — | 🟢 | The compile matrix is the RED/GREEN signal for doc drift; the new doc-test gate was itself mutation-checked (wrong-arity RED captured, E0061, log `-0029`) rather than trusted on first green | — |
| B3 | Evidence of Failure | — | 🟢 | Five source-cited API contradictions on record, plus an *executed* RED for the arity defect class via the mutation check — not analysis-only | — |
| B4 | Proper Fix Only | — | 🟢 | Root cause addressed (readme compiled by nothing → now doc-tested); snippets distilled from the canonical working example, not invented; deletion instead of stub/archive | — |
| B5 | Fix Verification | 🔴 | 🟢 | First battery run hit a mid-build `target/debug` deletion by a concurrent session (E0460), retry hit transient ENOSPC, and stage 6 had three wrong assumed package names — each re-run to green: `-0026` stages 1–5 (79/79 nextest, 3/3 doc tests, clippy clean), `-0028` all 14 consumers, `-0030` restored-green after mutation | Corrected package names from each Cargo.toml; re-ran to green |
| B6 | Knowledge Preservation | — | 🟢 | `lib.rs` doc-include carries a TASK-020 comment stating the why (drift now fails `cargo test --doc`); deletions need no site comment — git history is the record per house rules; `mod.rs` layer line documents the surviving content accurately | — |
| B7 | Code Cleanliness | — | 🟢 | `git status` scoped to the crate: exactly the six task-020 paths + five pre-existing paths from the earlier native-pixel-test task; no backup files; the one commented-out code block in the crate's post_processing was deleted, not kept | — |
| **Total** | | 🔴 | 🟢 | 1 (resolved in-loop) | 1/1 |

**Aggregate verdict:** PASS — all 15 dimensions clean on the final pass, zero Blocking Findings open.
The one in-loop finding (B5: battery interrupted twice by environment failures and once by assumed
package names) was driven to a fully green, evidence-logged state rather than partially accepted.
D1–D8 are the Readiness Verification Gate dimensions; B1–B7 apply because the Quick Start defect is a
genuine correctness fix (documented API that cannot compile), matching tasks 021/055's 15-dimension
precedent.
