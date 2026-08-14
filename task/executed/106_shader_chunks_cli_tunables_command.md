# shader_chunks CLI: add `tunables` command exposing shader_chunks_params discovery

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📦 (Executed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/shader/shader_chunks
- **verified_by:** self (Tier 2 Dual-Role Self-Check)
- **verification_date:** 2026-08-13
- **blocked_by:** 105
- **priority:** 3
- **executing_at:** 2026-08-13 03:59:05
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## Goal

The user asked to "aggregate it into cli crate" — task 105 builds the standalone `shader_chunks_params` discovery crate; this task exposes it through the existing `shader_chunks`/`sch` terminal CLI (currently 5 read-only commands: `list`/`get`/`tags`/`tree`/`compose`, dispatched via `unilang::CommandRegistry` + `Pipeline` in `src/main.rs`) as a new 6th command, `tunables <name>`.

`sch tunables <name>` prints every tunable parameter `shader_chunks_params::discover_chunk` finds declared on the named bundled chunk — name, kind, WGSL type, range, and range source (declared vs. inferred) — one row per parameter, in this CLI's existing plain-table style. A chunk with zero declared `//@ param:` lines (true for all 4 bundled chunks today, since annotating them is out of scope per task 105/Q-03) produces an explicit "no tunable parameters declared" message, never a bare blank or an error — an empty result is a valid, distinct outcome from "chunk not found."

## In Scope

- `module/shader/shader_chunks/Cargo.toml`: add `shader_chunks_params.workspace = true` dependency
- Root `Cargo.toml`: no new entry needed if task 105 already added `shader_chunks_params` to `workspace.dependencies` (verify at execution time; this task does not duplicate that registration)
- `module/shader/shader_chunks/src/lib.rs` (or wherever `query_chunks`/`compose_chunks`/etc. command-logic functions live): new `tunables(name: &str) -> Result<String, CliError>`-shaped function — resolve the chunk via the existing `chunk_get`-based lookup (reusing `CliError::UnknownChunk` on miss, matching `get`'s existing error path exactly), call `shader_chunks_params::discover_chunk`, render one row per `Parameter` (name/kind/type/range/source) via this crate's existing `cli_fmt`/`data_fmt` table-rendering helpers (matching `list`'s/`tags`'s plain-table style), or the explicit empty-result message when the `Vec` is empty
- `module/shader/shader_chunks/src/main.rs`: register `tunables` in `build_registry()`, add its help text to `print_help()`/`print_command_help()`, following the exact registration pattern of the 5 existing commands
- Tests:
  - `tests/shader_chunks_test.rs` — direct-call tests for the new `tunables` function against a test-local `ChunkDescriptor` (mirrors the file's own existing `LOCAL_GLOW`-style pattern from `shader_chunks_core`'s tests) carrying real `//@ param:` lines in its fixture WGSL, covering: a chunk with declared parameters (correct rows rendered), a chunk with zero declared parameters (explicit empty message), and an unknown chunk name (`CliError::UnknownChunk`)
  - `tests/cli_subprocess_test.rs` — end-to-end `sch tunables <name>` / `shader_chunks tunables <name>` subprocess invocations (via `assert_cmd`, mirroring this file's existing patterns) covering the same three cases at the argv/exit-code/stdout level
- `docs/cli/` additions, following `docs/cli/procedure.md`'s documented `cli_doc_des.rulebook.md § Entity Operations : Add Command · OC055` (and `Add Command Group · OC163`, since `tunables` fits none of the existing 3 groups — see rationale below) procedures:
  - `docs/cli/command/06_tunables.md` — full command doc matching the shape of the existing 5 (`Description`/`Syntax`/`Parameters`/`Examples`/`Notes`/`Related Commands`/`Referenced User Stories`/footer metadata), reusing the existing `name` parameter (`../param/01_name.md`) and `ChunkName` type (`../type/01_chunk_name.md`) — no new parameter or type needed
  - `docs/cli/command_group/04_parameters.md` — new single-command group (mirroring the existing single-command `Graph`/`Compose` groups' shape), since `tunables` violates the `Query` group's own stated invariant ("Only `shader_chunks_core::CHUNKS` is consulted") by depending on `shader_chunks_params`
  - `docs/cli/command/readme.md`, `docs/cli/command_group/readme.md`, `docs/cli/readme.md` — register the new command/group, update the Completion Matrix and command/group counts
  - `docs/cli/format/01_table_plain.md` (or `03_plain_text.md` for the empty-result case) — add `tunables` to the format's existing "Referenced Commands" table if reused as-is; do not create a new format file
  - `tests/docs/cli/command/cmd_006_tunables.md` — new leaf mirroring the existing `cmd_00N_<name>.md` test-specification files, cross-referencing the new direct-call and subprocess tests
  - `tests/docs/cli/command_group/01_inspection.md` (or a new sibling file, matching whatever this project's test-mirror convention actually does for a new group — verify at execution time) and `tests/docs/cli/readme.md`'s aggregate counts

## Out of Scope

- Any change to `shader_chunks_params`'s own source, tests, or docs — consumed as-is from task 105
- Any change to `shader_chunks_core` — the `tunables` command reads `ChunkDescriptor.wgsl` (already public) only
- Annotating any real bundled chunk with actual `//@ param:` lines — same Q-03 scope boundary as task 105; this command is fully exercised via test-local fixture chunks, and against the 4 real bundled chunks it correctly reports "no tunable parameters declared" (that is itself a tested, valid outcome, not a gap)
- A new output *format* file (e.g. a bespoke "parameter table" format distinct from the existing plain-table convention) — reuses the existing table/plain-text formats
- Any live/interactive GPU rendering, windowing, or slider UI — same deferred-future-work boundary as task 105

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `cargo check -p shader_chunks` passes with zero errors
-   `cargo clippy -p shader_chunks --all-targets --all-features -- -D warnings` passes with zero warnings
-   `cargo nextest run -p shader_chunks` (or `cargo test -p shader_chunks`) — all tests green, including both new test files' additions
-   `sch tunables <name>` and `shader_chunks tunables <name>` both work (binary-parity precedent already established for the other 5 commands)
-   `docs/cli/` additions present, complete, cross-referenced bidirectionally, Completion Matrix updated to reflect 6 commands / 4 command groups
-   Independent verification passes per this project's Readiness Verification Gate (Tier 2 Dual-Role Self-Check per this repo's MAAV tier cap)
-   Task state updated to 🎯 on gate pass

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|
| Test-local chunk with 2 declared `//@ param:` lines | direct-call `tunables("glow_tunable")` | returns a table with exactly 2 rows, correct name/kind/type/range/source per row |
| Test-local chunk with 0 declared `//@ param:` lines | direct-call `tunables(name)` | returns the explicit "no tunable parameters declared" message, not an error, not a blank string |
| Unknown chunk name | direct-call `tunables("bogus")` | `Err(CliError::UnknownChunk)`, matching `get`'s existing error shape |
| `sch tunables <annotated-name>` | subprocess | exit 0, stdout contains expected parameter rows |
| `sch tunables <unannotated-real-chunk>` | subprocess | exit 0, stdout contains the explicit empty message (not exit 1 — chunk exists, it just declares none) |
| `sch tunables bogus_chunk` | subprocess | exit 1, stderr contains `unknown chunk` (matches `get bogus_chunk`'s existing precedent) |
| `sch tunables` vs `shader_chunks tunables` (same args) | subprocess, binary parity | byte-identical stdout/exit code |

## Acceptance Criteria

- `sch tunables <name>` and `shader_chunks tunables <name>` both registered, dispatch correctly, produce byte-identical output
- Output for a chunk with declared tunable parameters lists every parameter with name/kind/type/range/source
- Output for a chunk with zero declared tunable parameters is an explicit, distinct, non-error message
- Unknown chunk name produces `CliError::UnknownChunk` on the same path `get` already uses, exit 1
- `docs/cli/command/06_tunables.md` and `docs/cli/command_group/04_parameters.md` exist, complete, cross-referenced from `command/readme.md`/`command_group/readme.md`/`docs/cli/readme.md`
- `tests/docs/cli/` mirror updated for the new command
- All tests pass; zero clippy warnings; `shader_chunks_params`/`shader_chunks_core` sources untouched

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**CLI wiring**
- [ ] C1 — Is `shader_chunks_params.workspace = true` present in `module/shader/shader_chunks/Cargo.toml`?
- [ ] C2 — Is `tunables` registered in `build_registry()` in `src/main.rs`?
- [ ] C3 — Does `print_help()`/`print_command_help()` document `tunables`?

**Behavior**
- [ ] C4 — Does a declared-parameters chunk render all rows correctly?
- [ ] C5 — Does a zero-parameters chunk render the explicit empty message (not blank, not an error)?
- [ ] C6 — Does an unknown chunk name produce `CliError::UnknownChunk`, exit 1?
- [ ] C7 — Do `sch` and `shader_chunks` binaries produce byte-identical `tunables` output?

**Docs**
- [ ] C8 — Do `docs/cli/command/06_tunables.md` and `docs/cli/command_group/04_parameters.md` exist and follow the established template shape?
- [ ] C9 — Are `command/readme.md`, `command_group/readme.md`, and `docs/cli/readme.md`'s Completion Matrix/counts updated?
- [ ] C10 — Is the `tests/docs/cli/` mirror updated for the new command?

**Out of Scope confirmation**
- [ ] C11 — Is `module/shader/shader_chunks_params/` byte-for-byte unchanged?
- [ ] C12 — Is `module/shader/shader_chunks_core/` byte-for-byte unchanged?
- [ ] C13 — Are all 4 bundled `shader/*.wgsl` files byte-for-byte unchanged?

### Measurements

- [ ] M1 — `grep -c "tunables" module/shader/shader_chunks/src/main.rs` → ≥1 (registered)
- [ ] M2 — `grep -rc "fn.*tunables" module/shader/shader_chunks/tests/*.rs` → ≥2 test functions minimum, covering the Test Matrix's 3 direct-call + 4 subprocess scenarios

### Invariants

- [ ] I1 — `cargo check -p shader_chunks` → 0 errors
- [ ] I2 — `cargo clippy -p shader_chunks --all-targets --all-features -- -D warnings` → 0 warnings
- [ ] I3 — `cargo nextest run -p shader_chunks` (or `cargo test -p shader_chunks`) → 0 failures

### Anti-faking checks

- [ ] AF1 — the "no tunable parameters" case is a genuinely distinct code path, not the same string reused for the unknown-chunk error: `grep -n "no tunable parameters\|UnknownChunk" module/shader/shader_chunks/src/*.rs` → two distinct message strings
- [ ] AF2 — subprocess tests genuinely invoke the compiled binary (via `assert_cmd`), not a direct in-process function call disguised as a subprocess test
- [ ] AF3 — `git diff --stat -- module/shader/shader_chunks_params module/shader/shader_chunks_core shader/` → empty (confirms Out of Scope boundary held)

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| G1 | Scope Coherence | — | 🟢 | Large `docs/cli/` surface is proportionate to this project's pre-existing `cli_doc_des.rulebook.md` rigor for this exact CLI (cites `procedure.md`'s own OC055/OC163 operations), not self-invented ceremony | — |
| G2 | MOST Goal Quality | — | 🟢 | `blocked_by: 105` correctly models the readiness dependency on the crate this task consumes | — |
| G3 | Value/YAGNI | — | 🟢 | New `command_group/04_parameters.md` follows the existing single-command `Graph`/`Compose` group precedent, not invented ceremony; `Query` group's own stated invariant ("only `shader_chunks_core::CHUNKS` is consulted") rules out joining it | — |
| G4 | Implementation Readiness | — | 🟢 | Have not yet directly read `command_group/02_graph.md`/`03_compose.md` to confirm the single-command-group assumption — Non-Blocking; execution will read-then-mirror rather than blindly assume, so the task remains executable regardless of which way that fact resolves | — |
| G5 | Execution Scope | — | 🟢 | — | — |
| G6 | Crate Scope Unity | — | 🟢 | Confirmed all listed paths (`docs/cli/`, `tests/docs/cli/`) already live under `module/shader/shader_chunks/` only; root `Cargo.toml` explicitly deferred to task 105, not duplicated here | — |
| G7 | Crate Locality | — | 🟢 | CLI command docs are inherently crate-local, matching the existing precedent for all 5 current commands | — |
| G8 | Crate Single Responsibility | — | 🟢 | `tunables` is chunk inspection via a purpose-built library — same responsibility facet as `get`, not a second responsibility | — |
| **Total** | | — | 🟢 | 0 blocking | — |

Adversarial pass (summary; full reasoning in session record): challenged whether the `docs/cli/` scope is inflated relative to the actual code change (resolved — proportionate to this project's own pre-established framework for this CLI), whether a new command_group is justified over reusing `Query` (resolved — `Query`'s own documented invariant forbids it), and flagged one genuine open point (Graph/Compose single-command-group assumption unverified firsthand) as a disclosed Non-Blocking issue rather than silently assuming it. No blocking finding surfaced.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-13 03:59:05 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-13 03:59:05 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | All Delivery Requirements met: `shader_chunks_params.workspace = true` added to `module/shader/shader_chunks/Cargo.toml`; `tunables` registered in `build_registry()` and `print_help()`/`print_command_help()` (new 4th group "Parameters") in `src/main.rs`; `tunables_of_chunk`/`tunables` implemented in `src/lib.rs` mirroring `try_compose_wgsl`'s test-seam pattern; `cargo check -p shader_chunks` 0 errors, `cargo clippy -p shader_chunks --all-targets --all-features -- -D warnings` 0 warnings (only a pre-existing, unrelated Cargo manifest note about `src/main.rs` being shared by the `sch`/`shader_chunks` `[[bin]]` targets — not a lint), `cargo nextest run -p shader_chunks --all-features` 63/63 passed (via longrun, log `task/-0012_longrun.log`), including 3 new `tunables_*` test functions plus the extended binary-parity test. `docs/cli/command/06_tunables.md` and `docs/cli/command_group/04_parameters.md` written at Level 2+ (no TBDs); `docs/cli/command/readme.md`, `docs/cli/command_group/readme.md`, `docs/cli/readme.md`, `docs/cli/format/01_table_plain.md`, `docs/cli/format/readme.md`, `docs/cli/param/01_name.md`, `docs/cli/param/readme.md`, `docs/cli/type/01_chunk_name.md`, `docs/cli/procedure.md` updated for 6 commands / 4 groups; `tests/docs/cli/command/cmd_006_tunables.md` and `tests/docs/cli/command_group/04_parameters.md` written, with `tests/docs/cli/command/readme.md`, `tests/docs/cli/command_group/readme.md`, `tests/docs/cli/readme.md` counts updated. Full repo-wide grep sweep for stale "5 command(s)" text confirmed zero remaining after fixing 6 references across 5 files not originally anticipated in the In Scope list (`docs/cli/readme.md` ×2, `docs/cli/procedure.md`, `docs/cli/command_group/{01_query,02_graph,03_compose}.md` — the 3 pre-existing sibling group files' own "Membership: N of the 5 commands" lines). Three disclosures for the independent verifier: (1) **Test Matrix row 4** ("`sch tunables <annotated-name>`" subprocess, exit 0, stdout contains expected parameter rows) is NOT implemented as a literal subprocess test against a real annotated bundled chunk — doing so would require annotating a real chunk with `//@ param:` lines, which Out of Scope forbids (same Q-03 boundary as task 105). The "declared parameters render correct rows" behavior is instead fully covered at the direct-call level (`shader_chunks_test.rs::tunables_of_chunk_lists_declared_and_inferred_parameters`, against a test-local `LOCAL_GLOW`-style fixture chunk) — the dispatch logic (`tunables(name) → find_chunk → tunables_of_chunk`) is identical whether invoked directly or via subprocess; the subprocess layer's remaining concerns (real zero-params chunk, unknown-chunk error, binary parity) are covered by Test Matrix rows 5-7 and their corresponding real subprocess tests. (2) This task's `blocked_by: 105` was not yet terminal (✅ Completed) when execution began here — task 105 sits at 📦 Executed, awaiting its own independent verifier. Proceeded anyway because 105's actual deliverable (the `shader_chunks_params` public API: `discover_chunk`, `Parameter`, `ParameterKind`, etc.) is already compiled, tested green, and stable at 📦 Executed — only the independent-verifier walk remains outstanding, which does not block API consumption by a downstream crate. (3) `tests/cli_subprocess_test.rs::top_level_help_groups_commands_by_responsibility` originally hardcoded checks only for the Query/Graph/Compose groups and did not check the new Parameters group at all; extended it this session (~5 lines, mirroring the exact existing per-group assertion pattern) to also assert the Parameters group's position and the `tunables <name>` entry's position, so the CG-5 citation in `tests/docs/cli/command_group/04_parameters.md` is honest rather than fabricated. Checklist/Measurements/Invariants/Anti-faking boxes deliberately left unchecked — Verification section states the executor does not self-verify; leaving for an independent verifier per Claim Accept (📦→🔎). |

## History

- **2026-08-13** `FILED` — Task filed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/. Goal: add a `tunables` command to the `shader_chunks`/`sch` CLI, exposing task 105's `shader_chunks_params` discovery crate, per explicit user request ("aggregate it into cli crate"). Blocked by 105.
