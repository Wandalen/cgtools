# Relocate shader chunks to shader/, move shader_chunks to module/shader/, add tag manifest field + inspection API

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** task
- **verification_date:** 2026-08-12 20:04:12
- **blocked_by:** null
- **priority:** 0
- **executing_at:** 2026-08-12 19:54:53
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **in_motion:** false
- **accepting_at:** 2026-08-12 19:55:09
- **accepting_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **completed_at:** 2026-08-12 20:04:12
- **completed_by:** task

## Goal

`shader_chunks` currently lives at `module/min/shader_chunks/`, with its 4 bundled `.wgsl` chunks under its own `src/chunks/`. The user wants the chunk *source* relocated to a repo-root `shader/` directory (sibling to `module/`), the crate itself relocated to `module/shader/shader_chunks/`, a new `//@ tags: group:tag, ...` manifest field on every chunk, and a small set of new public parsers (`ALL_CHUNKS`, `parse_tags`, `parse_description`, `parse_stage`, `parse_exports`, `try_compose`) so a later CLI (task 100) can enumerate and inspect chunks without hardcoding them. This is foundation work for task 100 — the CLI cannot be built against a crate that doesn't yet expose "list every chunk" or "read this chunk's tags" as a public API.

Motivated: explicit, accepted user request (analysis plan approved verbatim — "do all that. apply changes"). Observable: `shader/*.wgsl` exist at repo root, `module/shader/shader_chunks/` compiles and passes its test suite, `module/min/shader_chunks/` no longer exists. Scoped: this task only moves files and extends `shader_chunks`'s own manifest-parsing surface — it does not touch the CLI crate (task 100) or any consumer's rendering logic. Testable: `cargo test -p shader_chunks`, `cargo clippy -p shader_chunks -- -D warnings`, plus grep-based path/reference checks below.

## In Scope

- Move `module/min/shader_chunks/src/chunks/{hash21,value_noise,fbm3,fullscreen_triangle}.wgsl` → `shader/{hash21,value_noise,fbm3,fullscreen_triangle}.wgsl` (repo root, flat, no `chunks/` subdirectory)
- Move the crate directory `module/min/shader_chunks/` → `module/shader/shader_chunks/` (Cargo.toml, src/lib.rs, readme.md, tests/)
- Add a `//@ tags:` manifest line to all 4 moved chunks (comma-separated `group:tag` entries, mirroring `depends_on`'s style):
  - `hash21.wgsl` → `//@ tags: category:hash`
  - `value_noise.wgsl` → `//@ tags: category:noise`
  - `fbm3.wgsl` → `//@ tags: category:noise, technique:fractal`
  - `fullscreen_triangle.wgsl` → `//@ tags: category:vertex`
- Update `src/lib.rs`'s 4 `include_str!` paths to reach the new chunk location (`"../../../../shader/<name>.wgsl"` from `module/shader/shader_chunks/src/`)
- Add to `src/lib.rs` (mirroring the existing `manifest_field`/`parse_name`/`parse_depends_on` style exactly):
  - `ALL_CHUNKS: &[&str]` const — `&[HASH21, VALUE_NOISE, FBM3, FULLSCREEN_TRIANGLE]`
  - `parse_tags(wgsl: &str) -> Vec<(&str, &str)>` — mandatory line (may be empty), panics on an entry with no `:` separator
  - `parse_description(wgsl: &str) -> &str` — mandatory field, same panic contract as `parse_name`
  - a private `manifest_field_opt` helper (non-panicking `Option<&str>` variant of `manifest_field`) + public `parse_stage(wgsl: &str) -> Option<&str>` built on it
  - a private `manifest_field_all` helper (collects every matching `//@ key:` line, not just the first) + public `parse_exports(wgsl: &str) -> Vec<&str>` built on it
  - `ComposeError` enum (`CyclicDependency(String)` / `MissingDependency { chunk: String, missing: String }`), implementing `std::fmt::Display` + `std::error::Error` — plain `std`, no new dependency (crate currently has zero non-`mod_interface` dependencies; keep it that way)
  - `try_compose(chunks: &[&str]) -> Result<String, ComposeError>` — non-panicking twin of `compose`, same topological-sort logic, returns `Err` instead of panicking on a cycle or missing dependency; existing `compose` stays untouched (still panics) for its existing trusted call sites
  - register every new public item in the existing `mod_interface!` block
- Update `tests/shader_chunks_test.rs`: replace its private `ALL_CHUNKS` const and `manifest_fields` helper with the library's own new `ALL_CHUNKS`/`parse_exports` (deletes the duplication the test file currently carries); add new tests per Test Matrix below
- Update `module/shader/shader_chunks/readme.md`: chunk source location, tag field, new API surface, updated consumer link
- `Cargo.toml` (workspace root): remove `"module/min/shader_chunks",` from `# Min modules`; add a new `# Shader modules` section with `"module/shader/shader_chunks",`; move `[workspace.dependencies.shader_chunks]` out of the `# = min` block into a new `# = shader` block with `path = "module/shader/shader_chunks"`
- `examples/orrery/webgpu/readme.md:7`: update the relative link from `../../../module/min/shader_chunks/readme.md` to `../../../module/shader/shader_chunks/readme.md`

## Out of Scope

- The new `shader_chunks_cli` crate — filed separately as task 100, `blocked_by` this task
- Any change to `examples/orrery/webgpu`'s own shader/scene code — it only consumes `shader_chunks::compose()` on the same 4 chunks, unaffected by the relocation beyond the readme link fix above
- `rulebook.md`'s L0-L5 rendering-layer placement ladder classification for `shader_chunks` — pre-existing gap noted during planning, not caused by this move, not this task's concern unless the user asks separately
- Publishing `shader_chunks` to crates.io, or preserving `cargo publish` portability — the repo-root `shader/` placement the user explicitly chose makes `include_str!` reach outside the crate directory, which breaks publish-tarball inclusion; accepted tradeoff per the delivered plan (crate isn't in the root readme's published "Core Crates" list today)
- Renaming any chunk file or its `//@ name:` value — only location and manifest fields change

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `cargo test -p shader_chunks --all-features` passes with zero failures
-   `cargo clippy -p shader_chunks --all-targets --all-features -- -D warnings` passes with zero warnings
-   `module/min/shader_chunks/` no longer exists on disk
-   `shader/{hash21,value_noise,fbm3,fullscreen_triangle}.wgsl` exist at repo root
-   `cargo check --workspace` passes (confirms the workspace Cargo.toml edits and the `examples/orrery/webgpu` consumer still resolve)
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|
| Each of the 4 real bundled chunks | `parse_tags` | Returns the exact `(group, tag)` pairs listed above, in file order |
| A chunk with a malformed tags entry (no `:`) | `parse_tags` | Panics with a message naming the offending entry |
| `ALL_CHUNKS` | length check | `ALL_CHUNKS.len() == 4`, matching the actual `.wgsl` file count under `shader/` |
| `fullscreen_triangle` (has `stage: vertex`) | `parse_stage` | Returns `Some("vertex")` |
| `hash21`/`value_noise`/`fbm3` (no `stage:` line) | `parse_stage` | Returns `None` (no panic) |
| `fullscreen_triangle` (2 `export:` lines) | `parse_exports` | Returns both signatures, in file order |
| `hash21` (1 `export:` line) | `parse_exports` | Returns exactly 1 signature |
| Each of the 4 real bundled chunks | `parse_description` | Returns the exact description text from the existing `//@ description:` line (already present, previously unparsed) |
| `[VALUE_NOISE, FBM3]` (missing `hash21`) | `try_compose` | Returns `Err(ComposeError::MissingDependency { .. })`, does not panic |
| Two chunks with mutual `depends_on` | `try_compose` | Returns `Err(ComposeError::CyclicDependency(..))`, does not panic |
| `[FBM3, FULLSCREEN_TRIANGLE, VALUE_NOISE, HASH21]` (valid, out of order) | `try_compose` | Returns `Ok(String)` byte-identical to `compose`'s existing output for the same input |

## Acceptance Criteria

-   `shader/hash21.wgsl`, `shader/value_noise.wgsl`, `shader/fbm3.wgsl`, `shader/fullscreen_triangle.wgsl` exist at repo root, each with a `//@ tags:` line
-   `module/shader/shader_chunks/` exists and contains a working crate; `module/min/shader_chunks/` does not exist
-   `shader_chunks::{ALL_CHUNKS, parse_tags, parse_description, parse_stage, parse_exports, try_compose, ComposeError}` are all public and re-exported via `mod_interface!`
-   `tests/shader_chunks_test.rs` uses the library's own `ALL_CHUNKS`/`parse_exports` instead of private duplicates
-   Workspace root `Cargo.toml` member list and `[workspace.dependencies.shader_chunks]` both point at `module/shader/shader_chunks`
-   `examples/orrery/webgpu/readme.md`'s `shader_chunks` link resolves to the new path
-   `cargo test -p shader_chunks --all-features` and `cargo clippy -p shader_chunks --all-targets --all-features -- -D warnings` both exit 0
-   `cargo check --workspace` exits 0

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**Relocation**
- [ ] C1 — Do all 4 `.wgsl` files exist under repo-root `shader/` and nowhere else?
- [ ] C2 — Does `module/min/shader_chunks/` no longer exist?
- [ ] C3 — Does `module/shader/shader_chunks/` contain Cargo.toml, src/lib.rs, readme.md, tests/?

**Tags + new API**
- [ ] C4 — Does every chunk carry a `//@ tags:` line matching the values listed in In Scope?
- [ ] C5 — Are `ALL_CHUNKS`, `parse_tags`, `parse_description`, `parse_stage`, `parse_exports`, `try_compose`, `ComposeError` all present in `mod_interface!`'s public surface?
- [ ] C6 — Does `try_compose` share `compose`'s topological-sort logic (no divergent reimplementation) and only differ in panic-vs-Result?
- [ ] C7 — Does the test file's private `ALL_CHUNKS`/`manifest_fields` duplication no longer exist?

**Touch points**
- [ ] C8 — Does workspace root `Cargo.toml` reference `module/shader/shader_chunks` in both the member list and `[workspace.dependencies.shader_chunks]`?
- [ ] C9 — Does `examples/orrery/webgpu/readme.md` link resolve?

### Measurements

- [ ] M1 — `find /home/user1/pro/lib/yrd_gamedev/cgtools/shader -name '*.wgsl' | wc -l` → 4
- [ ] M2 — `test -d /home/user1/pro/lib/yrd_gamedev/cgtools/module/min/shader_chunks && echo EXISTS || echo GONE` → `GONE`
- [ ] M3 — `grep -c "module/min/shader_chunks" /home/user1/pro/lib/yrd_gamedev/cgtools/Cargo.toml /home/user1/pro/lib/yrd_gamedev/cgtools/examples/orrery/webgpu/readme.md` → 0 in both

### Invariants

- [ ] I1 — `cargo test -p shader_chunks --all-features` → 0 failures
- [ ] I2 — `cargo clippy -p shader_chunks --all-targets --all-features -- -D warnings` → 0 warnings
- [ ] I3 — `cargo check --workspace` → 0 errors

### Anti-faking checks

- [ ] AF1 — `try_compose` is not a copy-pasted reimplementation that happens to also work: `grep -n "fn visit" src/lib.rs` should show the topological-sort helper reused by both `compose` and `try_compose`, not duplicated under a second name
- [ ] AF2 — the new tests in the Test Matrix actually exercise the new functions by name (`grep -n "parse_tags\|parse_stage\|parse_exports\|try_compose" tests/shader_chunks_test.rs` finds real call sites, not just the import list)

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| G1 | Scope Coherence | — | 🟢 | In Scope enumerates exact file moves, exact new function signatures, exact 2 touch-point files; Out of Scope explicitly excludes the CLI crate, the orrery example's own logic, and the rendering-layer ladder gap. | — |
| G2 | MOST Goal Quality | — | 🟢 | Motivated (explicit accepted plan), Observable (files exist at new paths, old path gone, tests pass), Scoped (one crate + 2 touch-point files), Testable (cargo test/clippy/check + grep measurements). | — |
| G3 | Value/YAGNI | — | 🟢 | Null Hypothesis: without this, task 100 (the CLI the user explicitly asked for) cannot be built — there's no way to enumerate chunks or read tags today. New API additions are the minimum set task 100 actually needs (verified against task 100's own command list before writing this task), not speculative — `parse_description`/`parse_stage`/`parse_exports` were already anticipated by the crate's own doc comment ("unread by the composer itself" — future-tool-facing), not invented here. | — |
| G4 | Implementation Readiness | — | 🟢 | Exact current `manifest_field`/`parse_name`/`parse_depends_on`/`compose`/`visit` source read in full this session; exact `include_str!` path math for the new location computed (`../../../../shader/`); exact touch points confirmed via repo-wide grep (only 2 files). | — |
| G5 | Execution Scope | — | 🟢 | All paths (`shader/`, `module/shader/shader_chunks/`, root `Cargo.toml`, `examples/orrery/webgpu/readme.md`) resolve inside this repository. | — |
| G6 | Crate Scope Unity | — | 🔴→🟢 | Initial read: deliverables span the crate itself, a non-crate repo-root directory, the workspace-root manifest, and one file in a different crate — fails a strict single-crate reading. Resolved: this is a workspace-level restructuring task by nature (matches precedent tasks 008/022/031/035-039, all `unit_type: workspace`), not a module-level one; reclassified `unit_type` to `workspace` rather than forcing a false single-crate framing. | Reclassified unit_type to workspace |
| G7 | Crate Locality | — | 🟢 | Given the workspace-level classification, every deliverable is either the crate being moved or a minimal, factual companion reference update (manifest path, one doc link) — no logic pushed up to an aggregator that doesn't own it. | — |
| G8 | Crate Single Responsibility | — | 🟢 | `shader_chunks`'s responsibility ("manifest-driven WGSL chunk composition") is unchanged — this task relocates it and extends its manifest-reading surface along the same axis (parsing more of the manifest it already defines), it doesn't bolt on a second responsibility. | — |
| **Total** | | — | 🟢 | — | — |

Adversarial pass: strongest challenge is "is `try_compose` genuinely needed by this task, or is it scope creep belonging to task 100?" — checked against task 100's `compose` command, which takes ad hoc CLI arguments (untrusted combinations) where a raw panic is bad UX; the fallible twin has to live in `shader_chunks` itself (not the CLI) because the CLI has no access to `compose`'s private `visit`/cycle-detection internals otherwise — so it's correctly this task's deliverable, task 100 just consumes it. Second challenge: "does the repo-root `shader/` placement quietly break something else that depends on `module/min/shader_chunks/src/chunks/`?" — repo-wide grep for `shader_chunks/src/chunks` confirms zero references outside the crate's own `lib.rs`/tests, so no hidden coupling. No blocking finding survives; G6's initial fail was resolved by reclassification, not by narrowing scope.

## Outcomes

### Acceptance Results

- **Verified by:** self (Tier 2 Dual-Role Self-Check) — same session that performed the
  implementation; no separate verifier session dispatched. Resolved actor identity:
  `user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/` (matches this task's own
  `executing_by`/`accepting_by` fields).
- **Independence note:** this is a same-session self-check, not independent verification —
  consistent with this project's own standing verification cap (never escalate cgtools
  acceptance walks beyond Dual-Role Self-Check). Disclosed per instruction rather than framed as
  something it isn't: even a separately-dispatched verifier session would resolve to the same
  actor identity on this host (`scope get::id` is deterministic per host+user+cwd, not
  per-session — the same BUG-197 collision task 093's own Outcomes disclosed), so a nominal
  second dispatch would not have bought genuine identity independence either. What this walk
  does provide: two distinct, explicitly-separated passes — a confirming pass (the Checklist/
  Measurements/Invariants/Anti-faking walk below, all re-run fresh against the now-actually-
  implemented code rather than reused from the pre-implementation Verification Record above) and
  an adversarial pass (a repo-wide sweep for stray old-path references and Cargo.toml
  duplicate/orphaned entries, specifically hunting for what the confirming pass could plausibly
  miss).
- **Date:** 2026-08-12
- **Verdict:** PASS

#### Checklist

- [x] C1 — Do all 4 `.wgsl` files exist under repo-root `shader/` and nowhere else? — YES:
      `find shader -maxdepth 1 -name '*.wgsl'` → `shader/fbm3.wgsl`, `shader/fullscreen_triangle.wgsl`,
      `shader/hash21.wgsl`, `shader/value_noise.wgsl` (4 files, see M1). Repo-wide
      `find . -name '*.wgsl'` (excluding `target/`) shows every other hit belongs to unrelated crates
      (`renderer`'s own webgpu shaders, `minwebgpu`/`minwgpu` examples) — none under any
      `shader_chunks`-related path outside `shader/`.
- [x] C2 — Does `module/min/shader_chunks/` no longer exist? — YES:
      `test -d module/min/shader_chunks && echo EXISTS || echo GONE` → `GONE` (see M2).
- [x] C3 — Does `module/shader/shader_chunks/` contain Cargo.toml, src/lib.rs, readme.md, tests/? —
      YES: `ls module/shader/shader_chunks/` → `Cargo.toml`, `readme.md`, `src`, `tests`; `src/lib.rs`
      (294 lines) and `tests/shader_chunks_test.rs` (175 lines) both freshly read in full this session
      and confirmed non-empty with real content.
- [x] C4 — Does every chunk carry a `//@ tags:` line matching the values listed in In Scope? — YES:
      `grep -n "//@ tags:" shader/*.wgsl` → `hash21.wgsl:3: category:hash`,
      `value_noise.wgsl:3: category:noise`, `fbm3.wgsl:3: category:noise, technique:fractal`,
      `fullscreen_triangle.wgsl:3: category:vertex` — byte-for-byte matches the In Scope table.
- [x] C5 — Are `ALL_CHUNKS`, `parse_tags`, `parse_description`, `parse_stage`, `parse_exports`,
      `try_compose`, `ComposeError` all present in `mod_interface!`'s public surface? — YES:
      `src/lib.rs:278-294` — the closing `::mod_interface::mod_interface! { ... }` block lists all 7 by
      name (`own use ALL_CHUNKS;`, `own use parse_tags;`, `own use parse_description;`,
      `own use parse_stage;`, `own use parse_exports;`, `own use try_compose;`,
      `own use ComposeError;`), alongside the pre-existing `HASH21`/`VALUE_NOISE`/`FBM3`/
      `FULLSCREEN_TRIANGLE`/`parse_name`/`parse_depends_on`/`compose`.
- [x] C6 — Does `try_compose` share `compose`'s topological-sort logic (no divergent reimplementation)
      and only differ in panic-vs-Result? — YES: `src/lib.rs:201-204` — `compose` is now a 1-line
      wrapper, `try_compose( chunks ).unwrap_or_else( | err | panic!( "{err}" ) )`; `try_compose`
      (216-231) builds `entries` then calls the single `visit` helper (239-274) per top-level entry.
      See AF1 — exactly one `fn visit` exists in the file.
- [x] C7 — Does the test file's private `ALL_CHUNKS`/`manifest_fields` duplication no longer exist? —
      YES: `tests/shader_chunks_test.rs:5-10` imports `ALL_CHUNKS` and `parse_exports` directly from
      the `shader_chunks` crate; full-file read confirms no local `const ALL_CHUNKS` or
      `fn manifest_fields` definition anywhere in the file.
- [x] C8 — Does workspace root `Cargo.toml` reference `module/shader/shader_chunks` in both the member
      list and `[workspace.dependencies.shader_chunks]`? — YES: `grep -n "module/shader/shader_chunks"
      Cargo.toml` → line 26 (member list: `"module/shader/shader_chunks",`) and line 220
      (`path = "module/shader/shader_chunks"` under the `[workspace.dependencies.shader_chunks]` table
      at line 218). `grep -n "shader_chunks" Cargo.toml` shows exactly these 3 lines total — no
      orphaned or duplicate old-style entry survives (see adversarial note below).
- [x] C9 — Does `examples/orrery/webgpu/readme.md` link resolve? — YES:
      `examples/orrery/webgpu/readme.md:7` links
      `[shader_chunks](../../../module/shader/shader_chunks/readme.md)`;
      `test -f module/shader/shader_chunks/readme.md` confirms the target exists — 3×`../` from
      `examples/orrery/webgpu/` reaches repo root, then `module/shader/shader_chunks/readme.md`
      resolves correctly.

#### Measurements

- [x] M1 — `find /home/user1/pro/lib/yrd_gamedev/cgtools/shader -name '*.wgsl' | wc -l` → `4` — MET.
- [x] M2 — `test -d /home/user1/pro/lib/yrd_gamedev/cgtools/module/min/shader_chunks && echo EXISTS ||
      echo GONE` → `GONE` — MET.
- [x] M3 — `grep -c "module/min/shader_chunks" /home/user1/pro/lib/yrd_gamedev/cgtools/Cargo.toml
      /home/user1/pro/lib/yrd_gamedev/cgtools/examples/orrery/webgpu/readme.md` →
      `Cargo.toml:0`, `examples/orrery/webgpu/readme.md:0` — MET (0 in both; `grep -c` exits 1 on a
      zero-match count, which is the expected/correct signal here, not a command failure).

#### Invariants

- [x] I1 — `cargo test -p shader_chunks --all-features` → HOLD: `16 passed; 0 failed; 0 ignored;
      0 measured; 0 filtered out` (nextest run, `task/-0026_longrun.log` lines 13/31); doc-test pass
      shows `0 passed; 0 failed` (crate's doc comments carry no runnable examples — expected for a
      manifest-parsing library). Detached via `longrun .launch`, exit 0, elapsed 92s total for the
      full I1+I2+I3 chain.
- [x] I2 — `cargo clippy -p shader_chunks --all-targets --all-features -- -D warnings` → HOLD: part of
      the same `&&`-chained detached launch, overall exit 0; `grep -n "warning" task/-0026_longrun.log`
      → zero matches across the entire log.
- [x] I3 — `cargo check --workspace` → HOLD: exit 0; log shows `Finished \`dev\` profile
      [unoptimized + debuginfo] target(s) in 1m 30s` after checking numerous workspace members
      (`cg_tools`, `mdmath_linalg`, `minwebgpu_hello_triangle_quickstart`, `jewelry_site`,
      `minwebgpu_deffered_rendering`, `minwebgpu_renderer_pbr_scene`, `behaviour_tree`,
      `minwebgpu__`, and others) — full-workspace resolution confirmed, including the real
      `orrery_webgpu` consumer.

#### Anti-faking checks

- [x] AF1 — `grep -n "fn visit" src/lib.rs` → single hit at line 239 — confirms the topological-sort
      helper is reused by both `compose` and `try_compose`, not duplicated under a second name (see
      C6 for the call-graph detail).
- [x] AF2 — `grep -n "parse_tags\|parse_stage\|parse_exports\|try_compose"
      tests/shader_chunks_test.rs` → 20 real call sites across `parse_stage_is_some_only_for_the_
      vertex_chunk`, `parse_exports_counts_match_each_chunk`, `parse_tags_reads_every_bundled_chunk`,
      `parse_tags_panics_on_malformed_entry`, `try_compose_matches_compose_output_on_success`,
      `try_compose_returns_err_on_missing_dependency`, `try_compose_returns_err_on_cyclic_dependency`
      — genuine assertions against real return values (e.g.
      `assert_eq!( parse_tags( FBM3 ), vec![ ( "category", "noise" ), ( "technique", "fractal" ) ] )`),
      not merely present in the import list.

**Adversarial-pass evidence (beyond the checklist's own literal items):** repo-wide
`grep -rn "module/min/shader_chunks" --include="*.md" --include="*.toml" --include="*.rs" .` returns
matches only inside this task file's own historical/planning prose (Goal, In Scope, Acceptance
Criteria, Checklist text) — zero live/functional references survive anywhere else in the repository.
`grep -n "shader_chunks" Cargo.toml` returns exactly 3 lines (member-list entry, `[workspace.
dependencies.shader_chunks]` header, `path =` line) — no leftover duplicate or orphaned entry from
the pre-move state. No blocking finding survives either pass.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-12 19:54:53 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-12 19:55:07 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | execution complete |
| 2026-08-12 19:55:09 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-12 20:04:12 | task | ACCEPTANCE_PASS | acceptance passed |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-08-12** `FILED` — Task filed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/. Goal: relocate shader_chunks' source and crate per user-approved plan, add tag manifest field and inspection API needed by task 100.
- **2026-08-12** `ACCEPTANCE_PASS` — Acceptance verification passed (Tier 2 Dual-Role Self-Check; see `## Outcomes`). State ✅ Completed.
