# Workspace-wide sweep: adopt docs/ entity structure in remaining crates

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

The audit found only 5 of 27 workspace crates have adopted the `docs/` doc-entity structure
(`docs/feature/`, `docs/invariant/`, `docs/api/`, etc.) that recent commits (`refactor: migrate ...
documentation to docs/ entity structure`, visible in this repo's own git log) are actively rolling out
elsewhere (P8 — mechanical hygiene tier). **Re-derive the current 5/27 count at pickup** — this repo's
docs migration is actively in progress per its own recent commit history, so the true count has likely
already moved since this finding was made. For each remaining crate, migrate its existing scattered docs
(readme.md sections, standalone `.md` files) into the appropriate `docs/` doc-entity subdirectories,
following whatever pattern the 5 already-migrated crates establish. Likely worth decomposing per-crate at
pickup, same as tasks 035/036.

## In Scope

- Re-deriving the workspace-wide `docs/` doc-entity adoption census across all 30 `module/` crates (originally estimated 5/27)
- Verifying the 8 already-adopted crates (`line_tools`, `renderer`, `scene_script`, `tilemap_renderer`, `tilemap_scene`, `tiles_tools`, `minwebgpu`, `minwgpu`) use correctly typed `docs/` subdirectories with no untyped loose files
- Verifying the zero-`spec.md` prohibition and the roadmap.md companion-file exception hold workspace-wide
- Documenting the finding that no migration backlog exists, per `rulebook.md § Documentation layout`'s Documentation Necessity Test

## Out of Scope

- Actually migrating any of the remaining 22 crates' docs into `docs/` doc-entity subdirectories — the audit concluded this isn't warranted, since those crates carry no design documentation to migrate
- Creating `docs/` scaffolding for crates without content warranting it (would violate the Documentation Necessity Test / Catalog Doc Entity anti-pattern)

## Verification

### Checklist

- [x] C1 — Is the re-derived denominator (30 `module/` crates) still accurate? `for d in module/{alias,helper,math,min,blank}/*; do [ -f "$d/Cargo.toml" ] && echo "$d"; done | wc -l` → `30`.
- [x] C2 — Do exactly the 8 claimed crates (and no others) carry `docs/`? Directory scan across all 30 → `line_tools, renderer, scene_script, tilemap_renderer, tilemap_scene, tiles_tools, minwebgpu, minwgpu` — identical set, same 8, same names, as claimed.
- [x] C3 — Is the "zero `spec.md`" prohibition-check still true? `find module examples -iname spec.md` → `0` hits (workspace-wide, broader than this task's own module-only claim).
- [x] C4 — Is "zero untyped loose files at any `docs/` root" still true for all 8 adopted crates? `find <crate>/docs -maxdepth 1 -type f` for each of the 8 → empty every time (all content lives in typed subdirectories: `feature/`, `invariant/`, `pitfall/`, `api/`, `algorithm/`, `pattern/`, ...).
- [x] C5 — Are all 3 `roadmap.md` companion files (the one permitted crate-root `.md` exception besides readme/license/changelog) inside already-adopted crates? `find module -iname roadmap.md` → `tilemap_scene/roadmap.md`, `tiles_tools/roadmap.md`, `tilemap_renderer/roadmap.md` — all 3 are members of the C2 8-crate adopted set.

### Measurements

- [x] M1 — `docs/`-adopted crate count: `8/30` (was: `5/27`, the original 2026-08-08 audit figure this task's own Goal explicitly flagged as stale-by-design and re-derived at pickup — both figures are the task's own cited before/after, not a code change this task made).

### Invariants

- [x] I1 — Directory-presence re-scan (the mechanical equivalent of a test suite for a docs-structure claim): looped `[ -d "$d/docs" ]` check across all 30 `module/` crates → 8 matches, byte-identical crate list to C2.
- [x] I2 — Prohibition re-scan: `find module examples -iname spec.md` → exit 0, `0` results.

### Anti-faking checks

- [x] AF1 — Guards against filler `docs/` trees being created later just to inflate the adoption count: C2's spot check confirms all 8 use genuine typed doc-definition subdirectories with real content (not empty scaffolding) — a future count that includes an empty `docs/` directory with no typed subdirectory content would be gaming this metric, not satisfying it.
- [x] AF2 — Guards against a future crate accumulating scattered design `.md` files without a corresponding `docs/` migration going unnoticed: re-running C3's `spec.md` search plus a scan for crate-root loose `.md` files beyond `readme`/`license`/`changelog`/`roadmap` is the named mechanical trigger (per this task's own dissolution reasoning) for revisiting this task's "no migration backlog" conclusion — it is not a one-time check.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P8 (mechanical
  hygiene) tier, Fix-in-place bucket (cross-cutting).
- **[2026-08-10]** `IMPLEMENTED` — Re-derivation dissolved the finding. Current census: **8 of 30**
  module crates carry `docs/` (line_tools, renderer, scene_script, tilemap_renderer, tilemap_scene,
  tiles_tools, minwebgpu, minwgpu — up from the draft's stale 5/27, confirming the Goal's prediction the
  count had moved), and every one of the 8 uses correct typed doc definition instances (feature/,
  invariant/, pitfall/, api/, algorithm/, pattern/, ...). Decisively: the governing rule —
  `rulebook.md § Documentation layout` (local rulebook, overrides global) — mandates the FORM when
  design documentation exists, not universal presence: "`docs/` — ... present only when the crate has
  content that warrants it (see doc_des.rulebook.md's Documentation Necessity Test)" and "Applies to all
  crates ... that carry design documentation." The remaining 22 crates carry NO design documentation to
  migrate: zero scattered design .md files (the only crate-root .md beyond readme/license/changelog are
  3 `roadmap.md` files — a rulebook-sanctioned companion file, all three in already-adopted crates), zero
  crate-root `spec.md` anywhere in module/ or examples/ (the rule's one prohibition — verified by find),
  zero untyped loose files at any docs/ root (verified). Creating docs/ trees for crates with no
  warranting content would violate the Documentation Necessity Test the rulebook itself cites (the
  Catalog Doc Entity anti-pattern). No migration backlog exists; adoption correctly grows organically as
  crates accrue design content.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Tier 2 dual-role gate check passed 15/15. In-loop
  adversarial catch: the confirming pass initially accepted the draft's "migrate the remaining crates"
  mandate at face value; the adversarial pass asked whether any rule actually requires universal
  adoption — the local rulebook's conditional clause disproved the premise before any filler doc
  structure was generated.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | Goal's re-derivation mandate executed: fresh census (8/30), form-compliance check, prohibition checks | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value / YAGNI | 🟡 | 🟢 | Draft's premise (universal adoption as goal) would have produced filler docs/ trees — Catalog Doc Entity anti-pattern | Premise disproved via local rulebook's conditional clause; zero filler created |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | Zero repository changes needed; record + index only | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | Local-first honored: rulebook.md § Documentation layout read and cited as governing authority | — |
| B2 | Test-First | 🟢 | 🟢 | Each conclusion backed by a runnable check (find spec.md, docs/-root loose-file scan, census loop) | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | No failure exists: 0 spec.md, 0 untyped instances, 0 scattered design docs — all verified empty | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Dissolution documented rather than busy-work migration performed | — |
| B5 | Fix Verification | 🟢 | 🟢 | 8 adopted crates spot-verified to use typed subdirs (feature/invariant/pitfall/api/...) | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Census + rule interpretation recorded here with rulebook citation | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved | 1/1 |
