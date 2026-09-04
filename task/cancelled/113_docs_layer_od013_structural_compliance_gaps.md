# Fix docs/layer structural compliance gaps found in OD013 audit

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🚫 (Cancelled)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

A comprehensive gap audit of `docs/layer/` (requested: "what is still missing? comprehensive list of
gaps") checked all 6 layer instance files against
`$GENAI/dev/doc/doc_des.rulebook.md § OD013 : Common Doc Instance Requirements` (title format, heading
levels, Scope format, typed-reference-section ordering, bidirectionality) in addition to the
architectural gaps the docs already record about themselves. Three concrete, evidence-verified
structural defects surfaced that the architectural-gap content doesn't cover:

**1. Alphabetical-ordering violations (OD013 checklist rule 8 — typed reference sections must appear
alphabetically, `### Sources`/`### Tests` always last):**
- `docs/layer/002_l1_gpu_hal.md`: order is `### Layers` (line 59) → `### Explorations` (line 66) →
  `### Sources` (line 74). Wrong — `Explorations` (E) sorts before `Layers` (L).
- `docs/layer/006_l5_scene_script_and_runners.md`: order is `### Patterns` (line 44) → `### Layers`
  (line 51) → `### Render Stacks` (line 57) → `### Sources` (line 64). Wrong — `Layers` (L) sorts before
  `Patterns` (P).
- `docs/layer/004_l3_stack_engine.md` already gets this right (`Explorations` → `Layers` →
  `Render Stacks` → `Sources`) — usable as the in-repo reference for the correct order.

**2. Bidirectionality gap (OD013 checklist rule 10 — every typed-reference A→B needs a reciprocal B→A),
systemic across all 3 `render_stack/` files:** `docs/layer/004_l3_stack_engine.md` and
`docs/layer/006_l5_scene_script_and_runners.md` each carry a `### Render Stacks` section linking into
`docs/render_stack/001_d2.md`, `002_tile.md`, `003_d3.md` — but none of those three files link back.
Confirmed via `grep -ni "layer" docs/render_stack/00{1,2,3}*.md`: only prose mentions of the word
"layer," never an actual cross-reference into `docs/layer/`; none of the three has a `### Layers` H3
section at all. Needs a new `### Layers` typed-reference section added to each of the 3 render_stack
files (alphabetically placed among their existing `### Patterns` / `### Render Stacks` / `### Sources`
sections), with real relationship-description content — not a stub row.
- `render_stack/001_d2.md` ← referenced from `layer/004:51-57`, no back-link
- `render_stack/002_tile.md` ← referenced from `layer/006:57-63`, no back-link
- `render_stack/003_d3.md` ← referenced from both `layer/004` and `layer/006`, no back-link to either

**3. Missing cross-reference (content gap, not a formal rule violation):**
`docs/pattern/003_cross_stack_bridge_via_foundation_resources.md` is substantively about L0/L1
foundation-resource sharing between stacks (the exact subject matter of the layer ladder's bottom two
rungs) but isn't linked from any of the 6 `docs/layer/` files. Natural homes: `docs/layer/001_l0_drivers.md`
and/or `docs/layer/002_l1_gpu_hal.md` — needs a new row in each file's typed-reference sections, plus the
reciprocal row added to `pattern/003` itself.

**Open question — not yet actionable, needs a rulebook read first:**
`docs/layer/002_l1_gpu_hal.md`'s `### Explorations` section mixes 2 ADR files (`adr/002`, `adr/003`)
together with 1 actual exploration file (`explorations/001`). OD013's section-naming convention list
marks `### Decisions` as valid "only in `feature/` doc instances," so a non-`feature/` instance like
`layer/002` referencing an ADR has no clearly-designated section name in what's been read so far. Before
touching this: read `doc_des.rulebook.md § Architecture Documentation : Architecture Decision Records` to
find the intended convention for ADR cross-references from non-`feature/` doc instances, then decide
whether `002`'s mixed `Explorations` section is itself the accepted convention (nothing to fix) or a
genuine misfiling (needs a dedicated section or a rename).

All three concrete findings were verified directly against the live files this session (`grep`/`Read` on
`docs/layer/*.md`, `docs/render_stack/*.md`, `docs/pattern/003`) — not inferred from memory of the
architecture. Categories 1 and 2 are mechanical/low-risk fixes (reordering existing sections; adding
reciprocal cross-reference rows); no source code is touched.

## History

- **[2026-08-15]** `FILED` — Filed via lightweight Draft capture
  (`tsk.rulebook.md § Core Procedures : Procedure - Draft Task`, PROC8) at the user's request to
  "document all that" after a comprehensive `docs/layer` gap audit. Lightweight keyword scan of the
  Tasks Index (PROC8-S1) found no duplicate — task 089 mentions `docs/layer/002`'s texture-upload gap in
  its Purpose column, but that's an unrelated, already-closed architectural item, not this task's
  structural/OD013-compliance scope.
- **[2026-08-15]** `CANCELLED` — Reason: superseded by direct documentation fixes performed via
  `/doc_tsk`. Per `doc_tsk.md § Requirements` ("Documentation updates are performed DIRECTLY — never
  deferred to task files"), this task's entire scope turned out to be pure documentation work, which
  must never be task-tracked in the first place — it was validly filed under PROC8's lighter-weight
  rules (no doc-vs-implementation distinction at filing time), but doc_tsk's stricter categorization
  means it should resolve to a direct fix, not an implementation task. Fresh re-verification against
  live files (not reused from this task's own Goal text above) found: the 2 alphabetical-ordering
  findings and the render_stack→layer bidirectionality gap had already been resolved externally
  between this task's filing and this check — `docs/layer/002` and `docs/layer/006` are now correctly
  ordered, and all 3 `docs/render_stack/*.md` files now carry real `### Layers` back-references. The
  remaining 2 items were fixed directly this session: `docs/pattern/003` cross-referenced from
  `docs/layer/001` and `docs/layer/002` (new `### Patterns` sections, with a reciprocal `### Layers`
  section added to `pattern/003` itself, plus a `### Render Stacks` section closing an additional
  bidirectionality gap discovered against `render_stack/003` during this pass); and the ADR
  section-naming open question resolved by reading `doc_des.rulebook.md § Architecture Documentation :
  Architecture Decision Records` (OD058) plus the existing 4-file `### ADRs` precedent
  (`pattern/001`, `pattern/002`, `pattern/003`, `layer/002`) — `docs/layer/004`'s ADR reference was
  mislabeled as `### Explorations` and renamed to `### ADRs` to match. All 6 checks of
  `doc_des.rulebook.md § Documentation Workflow : Procedure - Consistency Validation` (PROC3) pass
  against the 4 touched files.
