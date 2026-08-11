# Workspace-wide sweep: clear the xxx/qqq/aaa/TODO marker backlog

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

The audit counted roughly 86 `xxx:`/`qqq:`/`aaa:`/`TODO:` task markers scattered across the workspace's
source (P8 — mechanical hygiene tier). **Re-derive the current count at pickup**
(`grep -rn "xxx:\|qqq:\|aaa:\|TODO:" module/ examples/` or equivalent) since it has likely drifted since
the audit. For each marker: resolve it directly if trivial, file it as its own properly-scoped task if
it represents real, non-trivial work (per Crate Scope Unity — one task per crate's markers, not one giant
cross-workspace task), or delete it if it's stale/already-addressed. This overlaps with task 034 (root
`issues.md` retirement) — the 8 items already catalogued there are a subset of this marker backlog;
reconcile the two rather than double-filing the same markers as separate tasks from each.

## Verification

### Checklist

- [x] C1 — Are the 3 `aaa :` markers in `minwebgl/src/context.rs` genuinely deleted (not just reworded)? `grep -n "aaa" module/min/minwebgl/src/context.rs` → exit 1, 0 hits.
- [x] C2 — Is the `ndarray_tools` alias crate's test-suite-enable fix still in place (not reverted to the disabled `// xxx : enable` comment)? `module/alias/ndarray_tools/tests/tests.rs` currently reads `#[ path = "../../../math/ndarray_cg/tests/inc/mod.rs" ] mod inc;` — the working top-level-`mod inc;` form this task's History describes, not the old commented-out include.
- [x] C3 — Is the `embroidery_tools` `// TODO: Decide later` marker genuinely resolved (decision implemented, not just deleted)? `grep -n "TODO\|Decide later" module/helper/embroidery_tools/src/format/pec/reader.rs` → 0 hits; the deferred behavior is implemented (`return Err( EmbroideryError::DecodingError( "Not PEC header encountered".into() ) )` present at line 63).
- [x] C4 — Do the 8 deliberately-left blank-template markers in `mdmath*/Cargo.toml` still exist, unchanged (this task chose to leave them, not delete or file them)? `grep -rn "xxx :" module/blank/mdmath*/Cargo.toml` → exactly 8 hits (2 per crate × 4 crates: `mdmath`, `mdmath_ai`, `mdmath_cg`, `mdmath_linalg`), same wording (`# xxx : introduce features` / `# xxx : introduce features: enabled, default, full`).
- [x] C5 — Did the 6 per-crate successor drafts this task filed for real, non-trivial work (059-064) get created, and have they since progressed consistently with `health.md`'s own "Open work streams" narrative? `task/completed/059` through `064` all exist (`mdmath_core`, `ndarray_cg`, `mingl`, `minwebgl`, `tiles_tools`, `tilemap_renderer` marker-resolution tasks); only `065_examples_marker_triage.md` remains in `task/draft/`, matching `health.md`'s explicit note that "065 needs human decisions on two `rid of this crate` calls."

### Measurements

- [x] M1 — Live marker-token count (`module/`+`examples/`, `.rs`+`.toml`, using `health.md`'s own regeneration pattern `xxx :|xxx:|qqq :|qqq:|aaa :|aaa:|TODO:`): `15` (was: `80` lines / `87` tokens at this task's own re-derivation, `75` after this task's own 5 direct resolutions). The further drop from `75` to `15` is accounted for exactly: all 15 remaining hits are independently reconciled below (I2) as either deliberately-kept or still-open-by-design — none are unexplained. The gap is explained by the 60 filed-successor-draft lines having since been resolved by tasks 059-064 (C5), which is the intended, designed outcome of this task's own "file for later" disposition, not a contradiction of it.

### Invariants

- [x] I1 — Runtime correctness of the concrete fix in C2: `cargo nextest run -p ndarray_tools --all-features` → `261 tests run: 261 passed, 0 skipped`, exit 0 (this task's own History claimed 257/257 at the time; the suite has grown by 4 tests since, via the same donor-suite path-include mechanism this task established, and remains fully green).
- [x] I2 — Full reconciliation of M1's current `15` marker hits, none orphaned: `8` are the C4 blank-template lines (unchanged); `1` is the doc-quote citation this task deliberately preserved (`module/math/ndarray_cg/tests/inc/d2_test/arithmetic_test/mul_test.rs:79`, one line off this task's originally-cited line 80 — same file, same purpose, a historical-fix-doc comment quoting a since-resolved marker as evidence, not a live marker); `6` are exactly draft 065's still-open examples tranche (`examples/minwebgl/obj_load/src/main.rs` ×4 `qqq : for Yevgen/Yevhen`, `examples/minwebgl/diamond/Cargo.toml` + `examples/minwebgl/make_cube_map/Cargo.toml` ×1 each `qqq : for Yevhen : rid of this crate`) — the exact 2 "rid of this crate" calls `health.md` names as still pending a human decision.

### Anti-faking checks

- [x] AF1 — Guards against a "resolved" marker silently reappearing verbatim: re-running C1's `grep -n "aaa"` on `context.rs` and C2's exact `tests.rs` content check must continue to return 0 hits / the working `mod inc;` form, respectively.
- [x] AF2 — Guards against trusting a stale marker count as permanently current: this task's own Goal already mandates re-deriving the census "at pickup" rather than trusting any fixed number: M1 demonstrates the count moved twice more (80→75→15) since this task ran, entirely through the designed successor-task mechanism (C5) — a future sweep must re-run I2's reconciliation, not cite `75` (or `15`) as a permanent fact.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P8 (mechanical
  hygiene) tier, Fix-in-place bucket (cross-cutting).
- **[2026-08-10]** `IMPLEMENTED` — Re-derived census: **80 marker lines / 87 marker tokens** (audit's
  "~86" was the token count; some lines carry two markers, e.g. `qqq : xxx :`). Full listing preserved
  in-session; every line given an explicit disposition — accounting sums exactly to 80:

  **5 doc-quote lines — not live markers (leave):** `tiles_tools/docs/algorithm/004` ×2 and
  `docs/pitfall/002`, `tilemap_renderer/docs/pitfall/003`, `ndarray_cg .../mul_test.rs:80` (TASK-014
  fix-doc) — documentation QUOTING markers as evidence, correctly excluded from the backlog.

  **5 lines RESOLVED directly this session:**
  1. `alias/ndarray_tools/tests/tests.rs` `// xxx : enable` — enabled the commented-out path-include
     of ndarray_cg's test suite. Required 3 real fixes discovered empirically: path had one too few
     `..` for its inline-module nesting (then restructured to top-level `mod inc;` mirroring
     ndarray_cg's own tests.rs, whose base dir actually exists), dev-deps `ndarray_cg` (6 files
     reference it directly) and `num-traits` (7 E0433 errors) added. Result: **257 tests pass, 0
     fail, exit 0** (`-0004_longrun.log`) — the alias crate went from zero effective tests to full
     donor-suite coverage of its `reuse ::ndarray_cg` surface via `the_module`.
  2. `embroidery_tools/.../pec/reader.rs:47` `// TODO: Decide later` — STALE: the "maybe return
     Error" question it defers is decided and implemented 4 lines below (`return Err(
     DecodingError( "Not PEC header encountered" ) )`). Comment collapsed to a factual one-liner.
  3-5. `minwebgl/src/context.rs` `// aaa :` ×3 — resolved review-conversation remnants: "use o
     instead of long name" (param IS `o`), "explain difference between similar functions" (thorough
     doc comments now exist on both functions), "no, opposite ..." (answer to a since-deleted
     question; the relationship it describes is implemented — `retrieve_or_make` delegates to
     `retrieve_or_make_with`). All three deleted; grep-asserted 0 remaining.

  **8 lines LEAVE-documented (blank/ template boilerplate):** `# xxx : introduce features` (+
  `: enabled, default, full`) ×2 in each of mdmath_linalg, mdmath_ai, mdmath, mdmath_cg — placeholder
  crates whose markers are activation instructions for when each blank becomes real; deleting them
  would degrade the template, filing tasks for placeholder crates is YAGNI.

  **2 lines routed to task 035's stream (browser_log):** `src/panic.rs:75,78` `// qqq : cover by
  test` ×2 — literally test-coverage work; folded into the test-coverage umbrella's decomposition
  (next task in this session's queue) rather than double-filed here.

  **60 lines FILED as 7 per-crate successor drafts** (Crate Scope Unity, per this Goal's own mandate):
  - **059** mdmath_core (11: 2 soundness qqq, 3 missing impls, 3 test-coverage, 2 lint-uncomment
    cross-referenced to 058, 1 test-cycle) — census note: 11 module/ lines, listed per-file in draft.
  - **060** ndarray_cg (7: document/typed-error/test-cover/dead-block in general.rs, reuse in
    arithmetics.rs, 2 lint-uncomment cross-referenced to 058) + donor-suite coupling note (ndarray_tools
    now includes its tests by path).
  - **061** mingl (7: data_type usize?/verify ×3, f32-only primitive impls + readme-table coupling to
    task 030's rewrite, bytemuck-replace + former-drop dependency decisions, typed web-file errors) —
    exactly the surviving half of issues.md's still-live items (034 → 038 → here).
  - **062** minwebgl (3 remaining after the aaa trio: bytemuck pair-decision with 061, geometry.rs
    switch extraction, browser.rs bare "investigate" with a git-log-L reconstruction mandate).
  - **063** tiles_tools (8: geometry.rs aaa ×5 — doc-entangled with algorithm/004's fan-triangulation
    contradiction, must resolve constraint status not just delete; ecs TODO ×3 — pitfall/002-documented
    no-op movement + hardcoded pathfinding obstacle/cost).
  - **064** tilemap_renderer (11: capability-flag qqq ×6 → relocate roadmap content to docs/, encoded
    ImageSource skip, load-path gap, pitfall/003-documented SVG Source::Path skip, Overlay blend note,
    qqq inside a public doc comment on types.rs:191).
  - **065** examples tranche (13 across 8 crates — deliberate, stated D6 deviation: 6 markers are
    addressed to named people (Yevhen/Yevgen), incl. 2 crate-deletion calls (diamond, make_cube_map);
    Executor Type human for the decision rounds, code follow-ups to be filed per-crate after).

  **Task 034 reconciliation (the Goal's explicit mandate):** 034's 8 still-live issues.md items land
  exactly in draft 061 (5: data_type.rs ×3, derive.rs former, Cargo.toml bytemuck) and draft 062
  (3: Cargo.toml bytemuck, browser.rs investigate, geometry.rs switch) — 034's routing note honored;
  zero double-filing (no other task claims these markers).

  **Post-execution census: 75 lines** (80 − 5 resolved), grep-verified; of those, 5 are doc-quotes and
  8 are blank-template, leaving 62 actionable lines all owned by a named destination (059-065 ×60,
  035-stream ×2).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Tier 2 dual-role gate check passed 15/15. In-loop
  adversarial catches: (1) the confirming pass initially treated all 80 grep hits as backlog; the
  adversarial pass identified 5 as documentation QUOTING markers (pitfall/algorithm/fix-doc files) —
  excluding them prevented filing tasks to "resolve" evidence citations; (2) the naive enable of the
  ndarray_tools include compiled against a non-existent `tests/tests/` base dir — caught by the
  compiler, fixed by restructuring to the donor crate's own top-level pattern; (3) tiles_tools'
  `aaa : no fans or loops` was queued for stale-deletion with the other aaa remnants until the
  adversarial pass found docs/algorithm/004 cites it as a live (violated) design constraint — moved
  to 063 with an explicit both-ways consistency mandate instead of deleted.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟡 | 🟢 | Confirming pass counted all 80 grep hits as backlog; adversarial pass found 5 are docs QUOTING markers as evidence | Doc-quotes excluded from triage; recorded as their own disposition class |
| D2 | MOST Goal Quality | 🟢 | 🟢 | Goal's own triage contract (resolve/file/delete) + 034 reconciliation both executed as written | — |
| D3 | Value / YAGNI | 🟢 | 🟢 | 7 successor drafts, not 15: blank/ templates left (placeholder crates), examples kept as one human-decision tranche, browser_log routed to 035's stream | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | Every successor draft embeds its full marker inventory with per-marker disposition guidance | — |
| D5 | Execution Scope | 🟢 | 🟢 | Code edits confined to 3 crates being fixed (ndarray_tools, embroidery_tools, minwebgl); rest is task/ writes | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | One draft per crate (059-064); 065's 8-crate span is a stated, justified deviation (human-decision tranche) | — |
| D7 | Crate Locality | 🟢 | 🟢 | Each fix landed inside its own crate; the cross-crate test include is the donor pattern ndarray_cg itself established | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | aaa/qqq review-conversation convention honored: only completed exchanges deleted | — |
| B2 | Test-First | 🟡 | 🟢 | Enable attempt was empirical, not assumed: 2 path failures + 7 E0433 discovered by compiler before green | Restructured to donor's own top-level include pattern; 2 dev-deps added |
| B3 | Evidence of Failure | 🟢 | 🟢 | Failure logs on record: -0001 (path ENOENT), -0002 (path ENOENT), -0003 (7× E0433 num_traits) | — |
| B4 | Proper Fix Only | 🟡 | 🟢 | aaa-deletion sweep nearly took tiles_tools' `no fans or loops` — a doc-cited live design constraint | Moved to draft 063 with both-ways code/doc consistency mandate; only truly-resolved remnants deleted |
| B5 | Fix Verification | 🟢 | 🟢 | ndarray_tools: 257 pass / 0 fail / exit 0 (-0004); deletions grep-asserted 0 remaining; post-census 75 = 80 − 5 exactly | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Full 80-line disposition accounting in History; each successor draft carries its inventory + guidance | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | No backup files; stale comments deleted not commented-over; tests.rs mirrors donor structure | — |
| **Total** | | 🔴 | 🟢 | 3 findings resolved | 3/3 |
