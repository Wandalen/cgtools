# Resolve tilemap_renderer's 3-way SVG-adapter documentation contradiction

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tilemap_renderer
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

`module/helper/tilemap_renderer`'s documentation set (`docs/feature/003_terminal_backend_adapter.md`,
`docs/feature/readme.md`, `docs/invariant/readme.md`, `docs/pattern/readme.md`, `docs/pitfall/readme.md`,
`readme.md`, `roadmap.md` — all touched by this repo's recent docs-entity migration commits) contains a
3-way contradiction about the SVG backend adapter's actual status/capability, per the audit triage plan.
P4 (rewrite bucket) — **the exact 3-way contradiction's specific claims were not preserved precisely
through this session's context compaction; re-derive by reading the current SVG adapter source against
each of the doc instances above before rewriting**, then produce one consistent account across all
touched doc files rather than fixing only one of the three.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P4 (doc rewrite)
  tier, Rewrite-from-scratch bucket. Flagged: citation detail needs re-derivation at pickup.
- **[2026-08-10]** `IMPLEMENTED` — Re-derived the contradiction from source, then rewrote the stale
  accounts to one consistent story. **The 3-way contradiction as re-derived:** Story A (`readme.md`)
  claimed `adapter-svg | complete` plus phantom Terminal capability — `adapters::TerminalBackend` listed
  in architecture, capabilities table said Terminal renders Paths/Text (`yes`/`yes`), tree said
  "ASCII/Unicode terminal output", tagline said "render to any backend — SVG, WebGL2, or terminal", and
  the ScreenSpaceSprite known-issue called terminal "a no-op for this variant" (implying other variants
  work). Story B (`roadmap.md`) said "SVG adapter — full implementation" while the same file carries
  "### svg adapter gaps" (font loading, `Source::Path` geometries, image Y-flip), and its terminal-gaps
  list implied paths/text already work. Story C (`docs/feature/`) is source-verified: 001 tracks SVG ⚠️
  partial (font selection unimplemented — `Assets.fonts` accepted then ignored, no `@font-face`, no
  `font-family`; `svg.rs:141-146`, `svg.rs:1542-1545`), 003 tracks Terminal ⏸️ (7-line stub, no `Backend`
  impl or type — `src/adapters/terminal.rs`). **Anchor = Story C** (matches source; `svg.rs:679` confirms
  text-on-path IS implemented, so the gap really is fonts, not text). **Rewrites:** `readme.md` — tagline,
  architecture bullet, tree, features table (svg → partial with font caveat; terminal → honest stub),
  capabilities table (Terminal column all-empty with footnote; SVG Text footnoted ¹ for the font gap;
  blend footnote renumbered ²), depth note ("SVG and terminal adapters" → "The SVG adapter"),
  ScreenSpaceSprite reword; `roadmap.md` — SVG completed-bullet qualified with explicit pointer to its
  own gaps section, tree annotated, terminal-gaps section reframed ("no `Backend` implementation or type
  exists yet", basics bullet added); `src/lib.rs` — tagline fixed and the false doc note deleted ("Note:
  SvgBackend and TerminalBackend are stubs" — wrong in the OPPOSITE direction: SVG is the most complete
  adapter, and `TerminalBackend` doesn't exist); `docs/invariant/004` — its citation quoted the exact
  roadmap phrase this task changed, reworded to the command-level/asset-level distinction. Also fixed
  `readme.md`'s WebGL capability column (Sprites/Meshes/Batches said `stub`, contradicting its own
  footnote two lines below and `webgl.rs:360/381/1152-1153` real implementations — flipped to `yes`).
  Verification: residue greps clean (`TerminalBackend` gone from `src/`, "full implementation" gone from
  all `*.md`, per-claim terminal/stub hits reviewed); `cargo check -p tilemap_renderer --features
  adapter-svg` exit 0 (`-0001_longrun.log` in crate dir, 20s).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — 15-dimension Tier 2 gate passed (see Verification
  Record). Three findings resolved in-loop: the task's own file list omitted `src/lib.rs`, which carried
  the falsest claim of all (SVG called a stub); my roadmap rewrite orphaned `docs/invariant/004`'s
  quotation of the old phrase; and the WebGL capability column self-contradicted the footnote below it.
  Moved draft/ → completed/.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟡 | 🟢 | Task's named file list (7 doc files) omitted `src/lib.rs`, whose doc comment carried the inverse falsehood ("SvgBackend … are stubs") plus a reference to the nonexistent `TerminalBackend` type; found by repo-wide grep, not the file list | Fixed lib.rs tagline; deleted the false note |
| D2 | MOST Goal Quality | 🟢 | 🟢 | Goal's re-derivation mandate honored: every rewritten claim anchored to source lines before editing | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | Considered implementing the missing terminal backend or font support instead of documenting their absence; rejected — task is doc reconciliation, no committed need | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | Edits confined to tilemap_renderer crate + task file; two files beyond the named list (lib.rs, invariant/004) justified by the Goal's "one consistent account" mandate | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | Doc-consistency validation run as genuine two-pass check; no new doc instances created (no Catalog anti-pattern) | — |
| B2 | Test-First | 🟢 | 🟢 | Contradiction verdict derived from source BEFORE any rewrite (svg.rs fonts/textPath, terminal.rs stub, webgl.rs sprite/mesh/batch) | — |
| B3 | Evidence of Failure | 🟡 | 🟢 | Confirming pass scoped to SVG/terminal claims per task title; adversarial per-cell re-read of the capabilities table found the WebGL column claiming `stub` for Sprites/Meshes/Batches while the footnote two lines below says they work | Verified real impls at `webgl.rs:360/381/1152-1153`, flipped to `yes` |
| B4 | Proper Fix Only | 🟢 | 🟢 | False lib.rs note deleted outright, not annotated; no status duplication reintroduced (status lives in features table + docs/feature) | — |
| B5 | Fix Verification | 🟡 | 🟢 | My own roadmap edit orphaned `docs/invariant/004:33`'s verbatim quotation of "SVG adapter — full implementation" — a new dangling reference the fix itself would have created | Post-edit "full implementation" grep caught it; citation reworded to command-level vs asset-level distinction; final grep clean |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Full re-derived contradiction + source anchors recorded in IMPLEMENTED entry for future pickup | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | Only comment/doc lines touched in src/; `cargo check --features adapter-svg` exit 0, zero warnings; longrun log hyphen-prefixed | — |
| **Total** | | 🔴 | 🟢 | 3 findings resolved | 3/3 |
