# New shader_chunks_cli crate: unilang-based CLI for listing/inspecting WGSL shader chunks

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🎯 (Verified)
- **closes:** null
- **unit_type:** module
- **unit:** module/shader/shader_chunks_cli
- **verified_by:** self (Tier 2 Dual-Role Self-Check)
- **verification_date:** 2026-08-12
- **blocked_by:** 099
- **priority:** 3

## Goal

Add a new terminal CLI, `shader_chunks_cli`, for listing and inspecting the WGSL shader chunks managed by `shader_chunks` (relocated to `module/shader/shader_chunks` by task 099, which also adds the `ALL_CHUNKS`/`parse_tags`/`parse_description`/`parse_stage`/`parse_exports`/`try_compose` API this CLI depends on). Per explicit user instruction, the CLI must use real `unilang` for command dispatch, `cli_fmt` for its top-level help screen, and `data_fmt` for all table/tree rendering — modeled on `~/pro/lib/yrd_core/kbase/kbase/module/kbase`'s CLI design, but as its own separate crate (not folded into `shader_chunks` itself).

Motivated: explicit, accepted user request. Observable: `shader_chunks_cli list/get/tags/tree/compose` produce correct output for the 4 real bundled chunks; `docs/cli/` is populated per the CLI doc rulebook. Scoped: this task only builds the CLI crate and its docs — it does not modify `shader_chunks` itself (already done by task 099) or any example/consumer. Testable: `cargo test -p shader_chunks_cli`, `cargo clippy -p shader_chunks_cli -- -D warnings`, `assert_cmd` subprocess tests per command.

## In Scope

- New crate `module/shader/shader_chunks_cli/`:
  - `Cargo.toml`: `name = "shader_chunks_cli"`, deps on `shader_chunks.workspace = true` (once task 099 lands), `unilang.workspace = true`, `cli_fmt` (workspace, feature `cli_help_template` only), `data_fmt` (workspace, features `table_plain, tree_hierarchical, tree_aligned`), `error_tools.workspace = true`, `mod_interface.workspace = true`
  - `src/lib.rs`: testable command logic — one function per command, each taking already-parsed arguments and returning `Result<String, CliError>` (the string is the fully-rendered output ready to print; keeping rendering inside the testable function, not `main.rs`, is what makes the direct-call test tier possible)
  - `src/main.rs`: thin entry point — builds the `unilang::registry::CommandRegistry` (one `CommandDefinition::former()...end()` + `CommandRoutine` pair per command, registered via `registry.register_with_routine`), dispatches via `unilang::pipeline::Pipeline::new(registry).process_command_from_argv_simple(&argv)`, prints `CommandResult`, maps a `CliError` to a process exit code
  - `CliError` enum (`error_tools`-based, in `src/lib.rs`): causation-based, e.g. `UnknownChunk(String)` / `Compose(shader_chunks::ComposeError)` / `Render(String)`, each mapped to an exit code in `main.rs` (validation-style errors → 1, everything else → 2) — mirrors kbase's own two-layer error split (neutral `shader_chunks::ComposeError` in the library from task 099, causation-based `CliError` here)
  - Five commands, each backed by a `src/lib.rs` function:
    - `list` — table of all `ALL_CHUNKS` entries: name / description / tags / depends_on, via `data_fmt`'s `RowBuilder` → `build_view()` → `TableFormatter` → `Format::format()` pipeline (`table_plain` feature)
    - `get <name>` — full detail for one chunk (name, description, stage, tags, depends_on, exports), plain `println!`-style text, not a table
    - `tags` — every distinct `group:tag` pair and which chunks carry it, `data_fmt` table
    - `tree [name]` — dependency tree for one chunk (or, with no argument, every chunk with no dependents as roots), rendered via `data_fmt`'s `TreeNode<ColumnData>` + `TreeFormatter::format_aligned()` (`tree_hierarchical`/`tree_aligned` features)
    - `compose <name...>` (`multiple: true` String argument, mirroring unilang's own `ArgumentAttributes { multiple: true, .. }` pattern) — preview composed WGSL output via `shader_chunks::try_compose`, printing the `ComposeError` message on failure instead of panicking
  - `mod_interface!` block registering the crate's public `src/lib.rs` surface (command functions + `CliError`)
- Tests (two-tier, mirroring kbase's own split):
  - `tests/shader_chunks_cli_test.rs` (or per-command files) — direct-call tests against `src/lib.rs` functions, no subprocess, covering all 5 commands against the 4 real bundled chunks plus at least one error path each (unknown chunk name, missing dependency via `compose`)
  - `tests/cli_subprocess_test.rs` — `assert_cmd`-based (`Command::new(cargo::cargo_bin!("shader_chunks_cli"))`), functional assertions (`stdout.contains(...)`) for at least: `list`, `get hash21`, `tags`, `tree fbm3`, `compose hash21 value_noise`, plus one failure case (unknown chunk) asserting non-zero exit
  - `tests/readme.md` — Responsibility Table for the new test files
- `docs/cli/` for the new crate, per `cli_doc_des.rulebook.md`, targeting L2-L3 completion:
  - `readme.md`, `index.md`, `procedure.md`
  - `command/` — one instance per command (`list`, `get`, `tags`, `tree`, `compose`)
  - `format/` — output format documentation (table for `list`/`tags`, tree for `tree`, plain text for `get`/`compose`)
  - `param/` — `name` (positional-ish, `get`/`tree`), `name...` (multiple, `compose`)
  - `param_group/` — if any commands share a parameter shape worth grouping (evaluate at implementation time; do not force one if the 5 commands' params don't actually cluster)
  - `type/` — the one domain entity this CLI operates on ("chunk")
  - explicitly SKIP `user_story` (needs ≥5 instances per the rulebook's own threshold; this CLI has 5 commands total, not enough distinct user stories to justify the collection) and `command_noun`/`command_verb` (needs ≥3 domain nouns; this CLI has exactly one entity, "chunk") — both disproportionate at this scale
  - `tests/docs/cli/` mirror directory per the rulebook's own entity table
- `module/shader/shader_chunks_cli/readme.md`: crate purpose, command list, example invocations, link to `docs/cli/`
- Root `Cargo.toml`:
  - Add `"module/shader/shader_chunks_cli",` to the new `# Shader modules` member-list section (created by task 099)
  - Bump `[workspace.dependencies.unilang]` from `0.26.0` to `0.58.2` (confirmed working version — kbase's own `unilang_cli_guard`/`query_cli`/`unilang`'s own demo CLI all resolve cleanly at this version; the existing pin is unused by any crate today, so this is a safe, uncontested bump)
  - Add `[workspace.dependencies.cli_fmt]` `version = "=0.13.1"` (exact pin, matching the confirmed-working version in kbase's own Cargo.lock)
  - Add `[workspace.dependencies.data_fmt]` `version = "=0.7.1"` (exact pin, same source)

## Out of Scope

- Any change to `shader_chunks` itself — already done by task 099, this task only consumes its public API
- Replicating kbase's own `unilang_cli_guard` argv-hardening helpers (bare-`.help`/`.` shorthand, `--` end-of-options stripping, duplicate-key rejection) — those exist to harden a large, many-command production CLI across a whole monorepo; this is a 5-command read-only inspection tool where `unilang`'s own built-in `Pipeline` dispatch and `HelpGenerator` are sufficient. If a concrete gap surfaces during implementation (e.g. a genuinely confusing error on bad input), fix that specific gap directly rather than importing the whole guard suite
- A YAML command manifest — `unilang`'s own demo CLI (`unilang-0.58.2/src/bin/unilang_cli/demo_commands.rs`) defines commands programmatically via `CommandDefinition::former()...end()`; this crate follows that pattern (no manifest file to keep in sync)
- Any REPL / interactive mode — argv dispatch only, matching every real-world consumer example found (`query_cli`, unilang's own `unilang_cli` binary)
- Publishing either crate to crates.io
- `docs/pattern/` entry for the manifest-driven-chunk or CLI-dispatch pattern — deferred to a later `doc_pln`/`doc_tsk` discovery pass, not pre-decided here

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `cargo test -p shader_chunks_cli --all-features` passes with zero failures
-   `cargo clippy -p shader_chunks_cli --all-targets --all-features -- -D warnings` passes with zero warnings
-   Every one of the 5 commands has both a direct-call test and a subprocess test
-   `docs/cli/` reaches L2-L3 per `cli_doc_des.rulebook.md`, with no TBD markers or empty required sections
-   `cargo check --workspace` passes
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|
| No arguments | `main.rs` dispatch | Prints `cli_fmt`-rendered top-level help/command list, exits 0 |
| `list` | direct-call + subprocess | Table with 4 rows (one per real chunk), columns name/description/tags/depends_on |
| `get hash21` | direct-call + subprocess | Full detail text including name, description, `stage: None`, tags, depends_on, exports |
| `get bogus_chunk` | direct-call + subprocess | `CliError::UnknownChunk`, non-zero exit, no panic |
| `tags` | direct-call + subprocess | Table listing every distinct `group:tag` pair (from task 099's tag values) and its carrying chunk(s) |
| `tree fbm3` | direct-call + subprocess | Tree showing `fbm3 -> value_noise -> hash21` |
| `tree` (no argument) | direct-call | Forest rooted at every chunk nothing else depends on |
| `compose hash21 value_noise` | direct-call + subprocess | Composed WGSL text, hash21 before value_noise regardless of input order |
| `compose value_noise` (missing hash21) | direct-call + subprocess | `CliError::Compose(ComposeError::MissingDependency)`, non-zero exit, no panic |
| `compose` (cyclic input, synthetic test fixture) | direct-call | `CliError::Compose(ComposeError::CyclicDependency)`, non-zero exit |

## Acceptance Criteria

-   `module/shader/shader_chunks_cli/` exists, builds, and its binary runs all 5 commands correctly against the 4 real bundled chunks
-   `unilang::registry::CommandRegistry` + `unilang::pipeline::Pipeline` are the actual dispatch mechanism (verifiable via `grep -n "unilang::" src/main.rs`)
-   `cli_fmt` is used for the top-level help screen only (`grep -n "cli_fmt::" src/`); every per-command detail screen is plain text
-   `data_fmt` renders every table (`list`, `tags`) and tree (`tree`) — no manual `println!` column alignment (`grep -n "data_fmt::" src/lib.rs` shows real usage, not just an unused import)
-   `docs/cli/` has `readme.md`, `index.md`, `procedure.md`, and populated `command/`, `format/`, `param/`, `type/` subdirectories, each command/format/param/type instance with no TBD markers
-   `tests/docs/cli/` mirror directory exists
-   `cargo test -p shader_chunks_cli --all-features` and `cargo clippy -p shader_chunks_cli --all-targets --all-features -- -D warnings` both exit 0
-   `cargo check --workspace` exits 0

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**Crate + dispatch**
- [ ] C1 — Does `module/shader/shader_chunks_cli/` contain Cargo.toml, src/lib.rs, src/main.rs, tests/, readme.md?
- [ ] C2 — Does `main.rs` build a real `CommandRegistry` and dispatch via `Pipeline::process_command_from_argv_simple`?
- [ ] C3 — Does every command have a `CommandDefinition` registered with a matching `CommandRoutine`?

**Commands**
- [ ] C4 — Do all 5 commands (`list`, `get`, `tags`, `tree`, `compose`) exist and produce correct output against the 4 real chunks?
- [ ] C5 — Does `list`/`tags` use `data_fmt`'s table pipeline (not manual alignment)?
- [ ] C6 — Does `tree` use `data_fmt`'s `TreeNode`/`TreeFormatter`?
- [ ] C7 — Does `compose` use `shader_chunks::try_compose` (task 099's fallible API), not the panicking `compose`?

**Errors**
- [ ] C8 — Does `get`/`compose` on bad input return a `CliError`, exit non-zero, and never panic?

**Docs**
- [ ] C9 — Does `docs/cli/` have all required L2-L3 sections per `cli_doc_des.rulebook.md`, with zero TBD markers?
- [ ] C10 — Does `tests/docs/cli/` mirror exist?

### Measurements

- [ ] M1 — `cargo run -p shader_chunks_cli -- list 2>&1 | grep -c hash21` → ≥1
- [ ] M2 — `cargo run -p shader_chunks_cli -- compose value_noise 2>&1; echo "exit=$?"` → non-zero exit, no panic backtrace in output
- [ ] M3 — `find /home/user1/pro/lib/yrd_gamedev/cgtools/module/shader/shader_chunks_cli/docs/cli -name "*.md" | xargs grep -l "TBD" | wc -l` → 0

### Invariants

- [ ] I1 — `cargo test -p shader_chunks_cli --all-features` → 0 failures
- [ ] I2 — `cargo clippy -p shader_chunks_cli --all-targets --all-features -- -D warnings` → 0 warnings
- [ ] I3 — `cargo check --workspace` → 0 errors

### Anti-faking checks

- [ ] AF1 — `grep -rn "println!" src/lib.rs` for `list`/`tags`/`tree` shows no hand-rolled column alignment (e.g. manual `{:<20}` width padding) standing in for `data_fmt` — if found, `data_fmt` is decorative, not actually driving the output
- [ ] AF2 — subprocess tests assert on actual command output content (`stdout.contains(...)`), not merely exit code 0 — an empty-output false-pass would not be caught by exit code alone

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| G1 | Scope Coherence | — | 🟢 | In Scope names every file/dir to create, every dependency to add, every command's exact backing mechanism; Out of Scope explicitly excludes touching `shader_chunks` itself, the argv-guard suite, YAML manifests, REPL mode, publishing, and a new pattern doc. | — |
| G2 | MOST Goal Quality | — | 🟢 | Motivated (explicit user instruction: "must use unilang... cli_fmt... data_fmt"), Observable (5 commands produce correct output against real chunks), Scoped (one new crate + its docs), Testable (cargo test/clippy + Test Matrix's 10 scenarios). | — |
| G3 | Value/YAGNI | — | 🟢 | Null Hypothesis: without this CLI, inspecting a chunk's tags/deps requires opening the .wgsl file directly — the explicit ask is for terminal tooling. Scope was actively trimmed against YAGNI during filing: dropped the argv-guard suite (unnecessary at 5-command scale), dropped a YAML manifest (unilang's own demo CLI shows the programmatic builder is the simpler real pattern), dropped `user_story`/`command_noun`/`command_verb` doc collections (below their own rulebook-stated instance thresholds). | — |
| G4 | Implementation Readiness | — | 🟢 | Verified directly against unilang 0.58.2's actual cached source this session (not assumed from memory): `CommandDefinition::former()...end()`/`ArgumentDefinition::former()...end()` builder shape confirmed in `unilang-0.58.2/src/bin/unilang_cli/demo_commands.rs`; `Pipeline::new(registry).process_command_from_argv_simple(&argv) -> CommandResult{command,outputs,success,error}` confirmed in `pipeline/argv.rs`+`pipeline/result.rs`; `CommandRoutine` closure signature `Fn(cmd, ctx) -> Result<OutputData, ErrorData>` with `cmd.arguments.get(key) -> Option<&Value>` and `ArgumentAttributes{optional,multiple,default,sensitive,interactive}` (the `multiple: true` flag needed for `compose <name...>`) all confirmed from the same demo file. `cli_fmt`/`data_fmt` exact versions (`=0.13.1`/`=0.7.1`) cross-confirmed against kbase's own resolved Cargo.lock. `try_compose`/`ComposeError` (task 099's deliverable) is this task's direct dependency — `blocked_by: 099` records the ordering. | — |
| G5 | Execution Scope | — | 🟢 | All paths (`module/shader/shader_chunks_cli/`, its `docs/cli/`, `tests/docs/cli/`, root `Cargo.toml`) resolve inside this repository. | — |
| G6 | Crate Scope Unity | — | 🟢 | Every deliverable targets exactly one crate (`shader_chunks_cli`) plus its own docs; the one workspace-root `Cargo.toml` touch is the minimal member-list+dependency-table registration every new crate requires, not scope creep. | — |
| G7 | Crate Locality | — | 🟢 | Command logic, tests, and docs all live inside `shader_chunks_cli` itself — nothing pushed up to an aggregator or sideways into `shader_chunks`. | — |
| G8 | Crate Single Responsibility | — | 🟢 | "Terminal CLI for listing/inspecting shader_chunks' WGSL chunks" states in one sentence without "and" — all 5 commands are read-only inspection views over the same one data source. | — |
| **Total** | | — | 🟢 | — | — |

Adversarial pass: strongest challenge is "is a whole second crate justified, or should this just be a `[[bin]]` target inside `shader_chunks` itself (as kbase itself does — lib+bin in one crate)?" — checked against the user's own explicit wording, "cover both crate[s]" (plural), which only makes sense if two crates exist; kept as a deliberate, flagged divergence from kbase's own structure rather than an oversight. Second challenge: "does dropping the argv-guard suite leave a real robustness gap?" — kbase's guards exist because unilang's own leniency (bare-token auto-dot-prefixing, `.help`/`.version` combined with other tokens silently no-op'ing) is dangerous at production multi-command scale with real users; at 5 read-only inspection commands used by the same developers who wrote them, that risk is materially lower, and the Out of Scope entry names the exact escape hatch (fix a concrete gap directly) if implementation proves otherwise. No blocking finding survives.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-08-12** `FILED` — Task filed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/. Goal: new shader_chunks_cli crate using real unilang/cli_fmt/data_fmt, modeled on kbase's CLI design, per user-approved plan. blocked_by 099 (needs its ALL_CHUNKS/try_compose API).
