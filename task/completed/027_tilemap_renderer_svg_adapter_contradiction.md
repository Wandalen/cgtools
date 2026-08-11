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

## Verification

### Checklist

- [x] C1 — Is `TerminalBackend` (the phantom capability claim) fully absent from `src/`? `grep -rn "TerminalBackend" src/` → `0` hits.
- [x] C2 — Is the "full implementation" residue gone from every crate `*.md` doc? `grep -rln "full implementation" --include="*.md" .` (crate root) → `0` files.
- [x] C3 — Does `readme.md`'s capabilities table show the Terminal column all-empty (footnoted) and WebGL Sprites/Meshes/Batches as `yes` (not the old `stub`)? Read `readme.md:81-96` → Terminal column is `—` on every row with a footnote at line 93; Sprites/Meshes/Batches = `yes`/`yes` for SVG/WebGL.
- [x] C4 — Does `src/lib.rs` no longer carry the false "SvgBackend and TerminalBackend are stubs" note, with the tagline corrected? `grep -n "stub" src/lib.rs` → `0` hits; tagline reads "render to any backend (SVG and WebGL today; terminal planned)".
- [x] C5 — Does `docs/invariant/004`'s citation drop the "full implementation" phrase for the command-level/asset-level distinction the fix claims? Full read → states "its remaining holes are asset-level... not command-level"; no "full implementation" string remains anywhere in the file.
- [x] C6 — Does `roadmap.md`'s SVG-adapter bullet stay qualified (not an unconditional completeness claim), with an explicit pointer to its own gaps section? Read `roadmap.md:17` → "...Not complete, though — see "svg adapter gaps" below".

### Measurements

- [x] M1 — Crate `*.md` files carrying the false "full implementation"/stub-mismatch account: `0` (was: `2` — `readme.md` showed WebGL Sprites/Meshes/Batches as `stub` despite real implementations and listed `adapters::TerminalBackend` in architecture; `roadmap.md` carried an unqualified "full implementation" claim — both confirmed via `git show 4469eafb^:module/helper/tilemap_renderer/readme.md` and `...roadmap.md`, the commit immediately preceding this task's own fix, itself bundled into `4469eafb`).

### Invariants

- [x] I1 — Test suite (crate-scoped, all features): `cargo nextest run -p tilemap_renderer --all-features` → exit 0, 122/122 passed.
- [x] I2 — Compiler/lints: `cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings` → **exit 101** — genuine current drift, but not from this task's own files. Root cause: the workspace `Cargo.toml`'s `allow_attributes_without_reason` lint was flipped `"allow"` → `"warn"` by the current HEAD commit `5f33be66` ("feat: consolidate test infrastructure and refactor module architecture", 2026-08-11 — a 421-file mechanical pass, dated AFTER this task's 2026-08-10 completion); the matching codebase-wide reason-string sweep is tracked but unexecuted (`task/draft/058_workspace_allow_sweep_per_crate.md`, 📝 Draft, census "1905 sites workspace-wide"). `-D warnings` elevates the new `"warn"` to a hard error crate- and dependency-wide — confirmed independently failing in the unrelated `browser_log` crate too (pulled in transitively via the `adapter-webgl` feature). None of `readme.md`/`roadmap.md`/`src/lib.rs`/`docs/invariant/004` (this task's touched files) contain any `#[allow]` attribute. Scoped re-run excluding the known/tracked lint and the broken transitive dependency (`cargo clippy -p tilemap_renderer --all-targets --no-default-features --features enabled,adapter-svg,adapter-terminal,cli,scene-model -- -D warnings -A clippy::allow_attributes_without_reason`) → exit 0, clean.

### Anti-faking checks

- [x] AF1 — Guards against the 3-way contradiction quietly reappearing in only one of the touched files while the others stay fixed: re-run C1+C2+C6 together after any future SVG/Terminal doc edit — the original defect was specifically that the 3 sources *disagreed with each other*, so all three must stay consistent simultaneously, not just individually correct.
- [x] AF2 — Guards against task 058's future workspace-wide `#[allow]` sweep silently absorbing a real, newly-introduced clippy issue in this task's own files as "more of the same" pre-existing debt: I2's scoped command (`--features enabled,adapter-svg,adapter-terminal,cli,scene-model -- -D warnings -A clippy::allow_attributes_without_reason`) must still return exit 0 after task 058 lands — a failure there is a real regression, not the known pre-existing class.

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
