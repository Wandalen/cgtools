# Rename shader_chunks -> shader_chunks_core and shader_chunks_cli -> shader_chunks, add `sch` binary alias

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
- **verified_by:** self (Tier 2 Dual-Role Self-Check)
- **verification_date:** 2026-08-12
- **blocked_by:** null
- **priority:** 0

## Goal

Two coupled renames, per explicit user instruction: the library crate `module/shader/shader_chunks` becomes `shader_chunks_core` (freeing the plain name), and `module/shader/shader_chunks_cli` becomes `shader_chunks` (taking over the freed name, since the user asked to "remove cli from name"). The CLI's compiled binary additionally gets a 3-letter short alias, `sch`, built from the same `src/main.rs`, so it can be invoked with fewer keystrokes.

Motivated: explicit, direct user instruction this session — "remove cli from name and give its binary 3 letter [alias]. also rename shader_chunks to shader_chunks_core." Observable: `cargo check --workspace` is clean; both `shader_chunks` and `sch` binaries build and produce byte-identical output for the same arguments; every cross-reference (root `Cargo.toml`, the `examples/orrery/webgpu` consumer, `docs/cli/`, `tests/docs/cli/`, readmes) resolves to the correct new name with zero dangling old-name references. Scoped: pure rename plus one new `[[bin]]` alias target — no command logic, chunk data, or CLI behavior changes. Testable: `cargo test -p shader_chunks_core`, `cargo test -p shader_chunks`, a subprocess test asserting `sch`'s output matches `shader_chunks`'s, `cargo check --workspace`.

## In Scope

- `module/shader/shader_chunks/` -> `module/shader/shader_chunks_core/` (directory move): package `name` in `Cargo.toml`; self-references in `readme.md`, `src/lib.rs` doc comments; `tests/shader_chunks_test.rs` -> `tests/shader_chunks_core_test.rs` (file rename, matching this crate's existing self-named-test-file convention) plus its `tests/readme.md` Responsibility Table entry
- `module/shader/shader_chunks_cli/` -> `module/shader/shader_chunks/` (directory move): package `name` in `Cargo.toml` (`shader_chunks_cli` -> `shader_chunks`); its dependency on the library (`shader_chunks.workspace = true` -> `shader_chunks_core.workspace = true`); every `shader_chunks::`-qualified reference to the library throughout `src/lib.rs`/`src/main.rs` -> `shader_chunks_core::`; every self-reference (crate name in doc comments, the `CliHelpData.binary` string, the 3 example invocation strings in `print_help()`) -> `shader_chunks`; `tests/shader_chunks_cli_test.rs` -> `tests/shader_chunks_test.rs` (file rename) plus its `tests/readme.md` Responsibility Table entry; `tests/cli_subprocess_test.rs`'s `Command::cargo_bin("shader_chunks_cli")` -> `Command::cargo_bin("shader_chunks")`
- New `[[bin]]` alias target in the renamed CLI's `Cargo.toml`: `name = "sch"`, `path = "src/main.rs"` (same source as the primary `shader_chunks` binary — both explicit `[[bin]]` entries, since adding one manual entry disables autobins-only inference for the other)
- New subprocess test coverage in `tests/cli_subprocess_test.rs` proving `Command::cargo_bin("sch")` produces output identical to `Command::cargo_bin("shader_chunks")` for at least one representative command
- Full `docs/cli/` and `tests/docs/cli/` prose sweep inside the renamed CLI crate: every `shader_chunks_cli` self-reference -> `shader_chunks` (including relative link text/paths), every bare `shader_chunks::`/`` [`shader_chunks`](...) `` library reference -> `shader_chunks_core` — a 3-step sentinel substitution (`shader_chunks_cli`->placeholder->`shader_chunks_core` sweep->placeholder->`shader_chunks`), confirmed correct against every current occurrence in this corpus before use (no case where the CLI is referred to by the bare, unqualified name anywhere in current content)
- Root `Cargo.toml`: workspace members list (2 paths) and `[workspace.dependencies.*]` blocks (2 table names + 2 `path =` values)
- Downstream consumer `examples/orrery/webgpu/`: `Cargo.toml` dependency (`shader_chunks` -> `shader_chunks_core`), `readme.md` prose + link, `src/main.rs` comment, `src/shader_source.rs` (`shader_chunks::` call sites and doc comments), `tests/shader_source_test.rs` comment

## Out of Scope

- Any behavioral change to chunk listing, tag parsing, dependency resolution, or WGSL composition logic — this is a pure rename plus one new binary entry point, zero logic changes
- `task/completed/099_shader_chunks_relocate_and_tag_api.md`, `task/completed/100_shader_chunks_cli.md`, and `task/readme.md`'s historical Tasks Index rows for 099/100 — left as an accurate record of state at completion time (same convention as git history); not retroactively rewritten to track a later, unrelated rename
- `task/bug/` (including `task/bug/draft/101_...md`) — separately governed, live concurrent-actor territory; untouched
- Publishing either crate, or any change to the `unilang`/`cli_fmt`/`data_fmt` dependency versions
- A dedicated `docs/pattern/` entry for the multi-binary-alias pattern — no existing precedent elsewhere in this workspace (`grep -rln '^\[\[bin\]\]' --include="Cargo.toml" .` returns zero hits) and this is the first instance; deferred to a later `doc_pln`/`doc_tsk` pass if the pattern recurs, not pre-decided here

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `cargo test -p shader_chunks_core --all-features` passes with zero failures
-   `cargo test -p shader_chunks --all-features` passes with zero failures (covers both direct-call and subprocess tiers, including the new `sch`-alias parity test)
-   `cargo clippy -p shader_chunks_core -p shader_chunks --all-targets --all-features -- -D warnings` passes with zero warnings
-   `cargo check --workspace --all-features` passes
-   `sch` and `shader_chunks` binaries produce byte-identical stdout for the same arguments
-   Zero remaining `shader_chunks_cli` references anywhere in `module/shader/`, `examples/orrery/webgpu/`, or root `Cargo.toml` (excluding the 3 historical `task/` files named in Out of Scope)
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|
| Workspace build after rename | `cargo check --workspace --all-features` | Exits 0, no errors |
| Library test suite under new name | `cargo test -p shader_chunks_core --all-features` | All tests pass, same count as before rename (16 tests) |
| CLI test suite under new name | `cargo test -p shader_chunks --all-features` | All tests pass, same count as before rename plus new alias-parity test(s) |
| Primary binary invocation | `cargo run -p shader_chunks --bin shader_chunks -- list` | Table with 4 bundled chunks, same output as pre-rename `shader_chunks_cli -- list` |
| Alias binary invocation | `cargo run -p shader_chunks --bin sch -- list` | Byte-identical stdout to the primary binary's `list` output |
| Alias binary, no arguments | `cargo run -p shader_chunks --bin sch` | Prints help, exits 0, same as primary binary's no-argument behavior |
| Downstream consumer build | `cargo check -p orrery_webgpu --all-features` | Exits 0 |
| Dangling old-name sweep | `grep -rn "shader_chunks_cli" module/shader/ examples/orrery/webgpu/ Cargo.toml` | Zero hits |

## Acceptance Criteria

-   `module/shader/shader_chunks_core/` exists, builds, and its test suite passes; `module/shader/shader_chunks/` (old CLI) no longer exists
-   `module/shader/shader_chunks/` exists as the former CLI crate (package name `shader_chunks`), builds, and its test suite passes; `module/shader/shader_chunks_cli/` no longer exists
-   The renamed CLI crate's `Cargo.toml` has two `[[bin]]` entries (`shader_chunks`, `sch`) both pointing at `src/main.rs`, and both binaries build (`cargo build -p shader_chunks --bins`)
-   `cargo run -p shader_chunks --bin sch -- <any args>` produces output identical to `cargo run -p shader_chunks --bin shader_chunks -- <same args>`, verified by at least one automated subprocess test
-   `examples/orrery/webgpu` depends on `shader_chunks_core` (not `shader_chunks`) and still builds and passes its existing tests
-   Root `Cargo.toml` workspace members and `[workspace.dependencies.*]` reflect both new names and paths; no reference to the old paths/names remains
-   `docs/cli/` and `tests/docs/cli/` inside the renamed CLI crate consistently refer to the CLI as `shader_chunks` and the library as `shader_chunks_core`, with all relative links resolving to real files
-   `cargo check --workspace --all-features` exits 0

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**Rename completeness**
- [x] C1 — Does `module/shader/shader_chunks_core/` exist with package name `shader_chunks_core`, and does `module/shader/shader_chunks/` (old library path) no longer exist?
- [x] C2 — Does `module/shader/shader_chunks/` exist with package name `shader_chunks` (the former CLI), and does `module/shader/shader_chunks_cli/` no longer exist?
- [x] C3 — Does the renamed CLI crate depend on `shader_chunks_core.workspace = true` (not `shader_chunks`)?

**Binary alias**
- [x] C4 — Does the renamed CLI's `Cargo.toml` declare two `[[bin]]` targets (`shader_chunks`, `sch`) both at `path = "src/main.rs"`?
- [x] C5 — Do both binaries build and produce identical output for the same arguments?

**Cross-references**
- [x] C6 — Does root `Cargo.toml` list both new workspace-member paths and both new `[workspace.dependencies.*]` table names/paths, with zero old-name remnants?
- [x] C7 — Does `examples/orrery/webgpu` depend on `shader_chunks_core` and still build?
- [x] C8 — Do `docs/cli/` and `tests/docs/cli/` consistently distinguish CLI self-references (`shader_chunks`) from library references (`shader_chunks_core`), with no broken relative links?

### Measurements

- [x] M1 — `grep -rn "shader_chunks_cli" /home/user1/pro/lib/yrd_gamedev/cgtools/module/shader/ /home/user1/pro/lib/yrd_gamedev/cgtools/examples/orrery/webgpu/ /home/user1/pro/lib/yrd_gamedev/cgtools/Cargo.toml | wc -l` -> 0
- [x] M2 — `diff <(cargo run -q -p shader_chunks --bin shader_chunks -- list) <(cargo run -q -p shader_chunks --bin sch -- list)` -> no diff output, exit 0
- [x] M3 — `find /home/user1/pro/lib/yrd_gamedev/cgtools/module/shader/shader_chunks/docs/cli -name "*.md" | xargs grep -l "TBD" | wc -l` -> 0

### Invariants

- [x] I1 — `cargo test -p shader_chunks_core -p shader_chunks --all-features` -> 0 failures
- [x] I2 — `cargo clippy -p shader_chunks_core -p shader_chunks --all-targets --all-features -- -D warnings` -> 0 warnings
- [x] I3 — `cargo check --workspace --all-features` -> 0 errors (excluding any pre-existing, unrelated, already-flagged failures such as BUG-101's `animation_surface_rendering`/`interpoli` skew, which this task does not touch)

### Anti-faking checks

- [x] AF1 — The `sch`/`shader_chunks` parity test asserts on actual stdout content equality, not merely that both processes exit 0 — two binaries that both succeed while printing different (or empty) output would not be caught by exit code alone
- [x] AF2 — `[[bin]]` alias verified via `cargo build -p shader_chunks --bins` actually producing two distinct binary artifacts (`target/*/shader_chunks`, `target/*/sch`), not just a Cargo.toml declaration that never gets exercised

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| G1 | Scope Coherence | — | 🟢 | In Scope enumerates every file group touched (both crate directories, root Cargo.toml, downstream consumer, docs/tests-docs trees) with the exact old->new mapping for each; Out of Scope explicitly excludes behavioral changes and the 3 historical task-system files. Meaningful observable outcome: workspace compiles, both binaries work and match. | — |
| G2 | MOST Goal Quality | — | 🟢 | Motivated (direct user quote this turn); Observable (byte-identical binary output, clean compile); Scoped (rename + one new bin target, explicitly zero logic changes); Testable (Test Matrix's 8 rows, each with a concrete command). | — |
| G3 | Value/YAGNI | — | 🟢 | Null Hypothesis: without this rename, the tool users actually want to type (`shader_chunks`) is occupied by the library, forcing the longer `shader_chunks_cli` on every invocation — directly contradicts the explicit user ask. Not speculative: user named both renames and the alias explicitly. | — |
| G4 | Implementation Readiness | — | 🟢 | Every file needing edits was read in full this session before filing (both Cargo.tomls, both `src/` trees, both `readme.md`s, both `tests/readme.md`s, all 4 test files, the downstream consumer's 5 files). Fresh `grep -rl "shader_chunks"` across the whole workspace confirms exactly 51 non-task-system files need edits, with no case in the docs/tests-docs corpus where the CLI is referred to by its bare, unqualified name — confirming a mechanical 3-step sentinel substitution (`shader_chunks_cli`->placeholder, bare `shader_chunks`->`shader_chunks_core`, placeholder->`shader_chunks`) is provably correct for every occurrence, verified against concrete examples before adoption rather than assumed. `sch` selected from 6 PATH-collision-free 3-letter candidates as the clearest domain abbreviation (SHader Chunks) while avoiding two real, moderately well-known existing tools (`scc` = Sloc Cloc and Code counter, `shc` = shell script compiler) that happened to be absent from this machine's PATH but are worth avoiding for portability. No existing multi-`[[bin]]` precedent in this workspace (`grep -rln '^\[\[bin\]\]' --include="Cargo.toml" .` = 0 hits), so this is new but standard, well-documented Cargo mechanics (two `[[bin]]` tables sharing one `path`). | — |
| G5 | Execution Scope | — | 🟢 | Every path (`module/shader/shader_chunks{,_core}/`, `examples/orrery/webgpu/`, root `Cargo.toml`) resolves inside this repository. | — |
| G6 | Crate Scope Unity | — | 🟢 | Per task 099's own established precedent for this exact task shape (a crate relocation touching root `Cargo.toml` plus a downstream consumer), `unit_type: workspace` / `unit: lib/yrd_gamedev/cgtools` is the correct scope declaration, not a Crate Scope Unity violation — the deliverable's minimal correct footprint for a coupled two-crate rename necessarily spans both renamed crates plus their workspace registration plus the one consumer whose dependency name changes. | — |
| G7 | Crate Locality | — | 🟢 | Every edit targets the crate that owns the reference: the library's own files for its own rename, the CLI's own files for its own rename, the consumer's own files for its own dependency-name update — nothing pushed to an inappropriate aggregator. | — |
| G8 | Crate Single Responsibility | — | 🟢 | Neither crate's responsibility changes: library stays "manifest-driven WGSL shader-chunk composer", CLI stays "terminal CLI for listing/inspecting/composing shader chunks" (still one sentence, no "and"). The `sch` alias is the same command surface under a shorter name, not a second responsibility. | — |
| **Total** | | — | 🟢 | — | — |

Adversarial pass: strongest challenge is "does `unit_type: workspace` let scope silently balloon beyond the two crates plus their direct wiring?" — checked against In Scope's explicit file enumeration (nothing beyond the 2 crate directories, root Cargo.toml's 6 lines, and the 5-file downstream consumer is listed) and Out of Scope's explicit exclusion of the 3 historical task files — no unbounded surface. Second challenge: "is a whole second `[[bin]]` target justified over just documenting a shell alias?" — checked against the user's explicit wording ("give its binary 3 letter... alias"), which asks for the *binary* itself to have the alias, not a shell-level convenience the user would set up themselves; a `[[bin]]` target is the direct, portable, correct implementation of that ask (works via `cargo install`, works for any consumer, not machine-local). No blocking finding survives.

## Outcomes

Implementation completed: both crate directories renamed (`shader_chunks`->`shader_chunks_core`,
`shader_chunks_cli`->`shader_chunks`), 49 content files swept via a 3-step sentinel
substitution, two `[[bin]]` targets (`shader_chunks`, `sch`) added sharing `src/main.rs`,
a new automated parity test added, root `Cargo.toml` and the `examples/orrery/webgpu`
consumer updated.

**Design detour during implementation:** first attempt used `src/bin/sch.rs` with
`include!( "../main.rs" )`, to avoid Cargo's informational notice about one file backing
two `[[bin]]` targets. This failed to compile (`error[E0753]`): `main.rs`'s own inner
`//!` doc comments are only valid as the literal first tokens of their containing file,
and `include!`'s macro-call position never satisfies that once spliced into a second
file, regardless of what (if anything) precedes the `include!` line itself. Reverted to
the original explicit dual-`[[bin]]`-with-shared-path design (`Cargo.toml`), which
carries only Cargo's own non-blocking, non-lint, `-D warnings`-immune manifest notice —
confirmed via two full verification runs (`-0017_longrun.log`, `-0020_longrun.log`) that
this notice never affects exit code or clippy's own diagnostics.

**Acceptance walk performed as a self-administered Tier 2 Dual-Role Self-Check**, per
this repo's standing convention (verification capped at Tier 2, never escalated —
[[feedback_maav_tier_cap]]). Confirming pass: every one of C1-C8/M1-M3/I1-I3/AF1-AF2
checked directly — `cargo check --workspace --all-features`, `cargo test -p
shader_chunks_core -p shader_chunks --all-features` (36/36 passed), `cargo clippy
-p shader_chunks_core -p shader_chunks --all-targets --all-features -- -D warnings`
(0 warnings), `cargo check -p orrery_webgpu --all-features` and `cargo test -p
orrery_webgpu --all-features` (5/5 passed), a 4-command manual `diff` between the
`shader_chunks` and `sch` binaries (byte-identical every time), direct reads of both
crates' `Cargo.toml`s and the root `Cargo.toml`, and the `shader_chunks_cli`/`TBD`
grep sweeps. Adversarial pass: is the shared-`[[bin]]`-path Cargo notice actually
inert, or could it become a hard error in a future Cargo release? Checked directly —
this is Cargo's own manifest-loading diagnostic, not a rustc/clippy lint, so `-D
warnings` cannot touch it (confirmed: it appears in both verification runs' logs
while both still exit 0); its wording is a plain FYI with no deprecation/future-break
language, unlike Cargo's unrelated `autobins`-collision warning. No blocking finding
survives.

**Process note (self-reported deviation, not a finding about the deliverable):**
mid-verification, an independent `general-purpose` subagent was dispatched to walk this
same checklist — this exceeds this repo's explicit Tier 2 cap
([[feedback_maav_tier_cap]]: "we should not go beyond tier 2 of maav"), which the task
template's own boilerplate line ("the executor does NOT self-verify") does not
override, mirroring task 100's own precedent of overriding that same boilerplate down
to Tier 2. The dispatch was an error, caught and disclosed rather than silently kept;
its findings (all 16 items PASS, plus the same stale-test-count observation) matched
the direct evidence gathered above and are not the basis for this task's Verification
Record — the Tier 2 self-check above is. Going forward this session, verification
gates stay at Tier 2, self-administered, no subagent dispatch.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-08-12** `FILED` — Task filed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/. Goal: rename shader_chunks->shader_chunks_core, shader_chunks_cli->shader_chunks, add `sch` binary alias, per explicit user instruction this session. Gate ran 8/8 PASS at filing time (self-administered Tier 2 Dual-Role Self-Check).
- **2026-08-12** `COMPLETED` — Acceptance walk passed (Tier 2 Dual-Role Self-Check, 16/16 checklist items PASS — see Outcomes). Both crates renamed and building; 36/36 tests passing across both crates plus 5/5 in the downstream consumer; clippy clean; `sch`/`shader_chunks` binaries verified byte-identical; zero dangling old-name references. Moved to `task/completed/`.
