# Rewrite embroidery_tools/readme.md to match the real API

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/embroidery_tools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

`module/helper/embroidery_tools/readme.md` (236 lines, read in full this session) documents an entirely
fictional API — `EmbroideryPattern`, `pattern.stitch_count()`/`.color_count()`/`.add_color()`/`.scale()`/
`.rotate()`/`.optimize()`, `Stitch::normal()`, `Color::rgb()`, `PesVersion::V6` — none of which exist in
the real source. The actual API (confirmed by grepping `src/embroidery_file.rs`, `thread.rs`,
`format/pes.rs`, `format/pec.rs` this session) is `EmbroideryFile` with `new()`, `stitches()`, `threads()`,
`stitch(dx,dy)`, `jump(dx,dy)`, `color_change(dx,dy)`, `trim()`, `end()`, `add_stitch_relative/absolute()`,
`add_thread()`, `bounds()`, `as_command_blocks()`; `Color`, `Thread`; `PESVersion` (not `PesVersion`);
`pec_threads() -> [Thread; 65]`. P4 (rewrite bucket) — rewrite the readme's Quick Start and API Reference
sections entirely from the real API surface; the "Current Status & Roadmap" / "Use Cases" prose sections
may be salvageable if reworded to stop implying the fictional API works.

## In Scope

- Rewrite `module/helper/embroidery_tools/readme.md`'s Quick Start, API Reference, Core Types table,
  Pattern Operations, and Thread Color Handling sections to use only the real, verified `EmbroideryFile` API
- Add a note documenting the crate-root re-export gap (`use embroidery_tools::*;` resolves nothing)
- Update Current Status & Roadmap: promote stitch-encoding normalization methods to Implemented, demote
  geometric transforms (scale/translate/rotate) to Planned

## Out of Scope

- Header, Features bullets, Installation, Supported Formats table, Use Cases section, and File Format
  Specifications/Coordinate Systems prose — left untouched (no fictional API references)
- The pre-existing "Coordinate Systems" mm-unit conversion claim — flagged as unverified but deliberately
  left unedited, outside mandatory scope
- Any change to the crate's actual source code — documentation-only rewrite

## Verification

### Checklist

- [x] C1 — Are the fictional API symbols (`EmbroideryPattern`, `PesVersion`, `.stitch_count(`, `Stitch::normal`, `.write_file(`, `.add_stitch(`, `.color_count(`, `.add_color(`, `Color::rgb(`) absent from the rewritten `readme.md`? Anchored grep (`grep -nE "EmbroideryPattern|PesVersion\b|\.stitch_count\(|Stitch::normal|\.write_file\(|\.add_stitch\(|\.color_count\(|\.add_color\(|Color::rgb\("`) → `0` matches (was `19` pre-fix, `git show ba2a6eb8:module/helper/embroidery_tools/readme.md`, same pattern). A looser substring grep turns up 2 false positives (`stitch_count` as a local variable name derived from the real `.stitches().len()`, and "no path-based `write_file()`" in the corrective prose) — inspected individually and confirmed non-fictional.
- [x] C2 — Does the documented `EmbroideryFile` API match the real signatures in `src/embroidery_file.rs`? `new()`, `stitches()`, `threads()`, `stitch(dx,dy)`, `jump(dx,dy)`, `color_change(dx,dy)`, `trim()`, `end()`, `add_stitch_relative()`, `add_stitch_absolute()`, `add_thread()`, `bounds()`, `as_command_blocks()` all confirmed present with matching signatures by direct source read.
- [x] C3 — Is the enum correctly named `PESVersion` (not `PesVersion`) with `V1`/`V6` variants, and is `pec_threads() -> [Thread; 65]` correctly signatured? Confirmed in `src/format/pes.rs:8-16` and `src/format/pec.rs:13`.
- [x] C4 — Is the "no path-based `write_file()` convenience" correction still accurate? `grep "pub fn" src/format/pes/writer.rs src/format/pec/writer.rs` shows only `write<W>(emb, writer, version)` (pes) and `write<W>(emb, writer)` (pec) — no `write_file` in either writer.
- [x] C5 — Is the crate-root re-export gap note ("`use embroidery_tools::*;` resolves nothing") still accurate? `src/lib.rs`'s `mod_interface!` block contains only `layer` declarations (`embroidery_file`, `stitch_instruction`, `format`, `thread`, `metadata`, `error`) — no `own use`/`orphan use` at crate root, confirmed by direct read.
- [x] C6 — Are geometric transforms (scale/translate/rotate) still correctly listed as absent/Planned rather than Implemented? Confirmed: no such methods exist anywhere in `EmbroideryFile`'s `impl` block (`src/embroidery_file.rs`, read in full).

### Measurements

- [x] M1 — Fictional-symbol residue count in `readme.md` (same anchored pattern as C1): `0` (was: `19`, `git show ba2a6eb8:module/helper/embroidery_tools/readme.md`).
- [x] M2 — `readme.md` line count: `255` (was: `235`, same baseline commit) — net +20 lines from the added crate-root re-export note, manual `File`/`BufWriter` construction, and the Pattern Operations/Thread Color Handling rewrites.

### Invariants

- [x] I1 — Test suite (crate-scoped): `cargo nextest run -p embroidery_tools --all-features && cargo test -p embroidery_tools --doc --all-features` → exit 0; nextest 10/10 passed, doc-tests 0/0 (`readme.md` is a plain file here, not `include_str!`-embedded into a doc comment, so its code blocks are illustrative only — consistent with `src/lib.rs`, which has no `include_str!(...readme.md...)` anywhere).
- [x] I2 — Compiler/lints clean: `cargo clippy -p embroidery_tools --all-targets --all-features -- -D warnings` → exit 0, zero warnings (confirmed reproducible after two independent `cargo clean -p embroidery_tools` rebuilds).

### Anti-faking checks

- [x] AF1 — Guards against a fictional symbol being reintroduced by a future "helpful" doc edit: re-running C1/M1's anchored grep against `readme.md` must keep returning `0`.
- [x] AF2 — Guards against the crate-root re-export gap note going stale: if a future change adds `own use`/`orphan use` at the top level of `src/lib.rs`'s `mod_interface!` block, the readme's "resolves nothing" claim (readme.md:69) becomes false and must be updated in the same change.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P4 (doc rewrite)
  tier, Rewrite-from-scratch bucket.
- **[2026-08-10]** `IMPLEMENTED` — Rewrote `module/helper/embroidery_tools/readme.md` (236 lines) from
  ground truth instead of trusting the task's own real-API list at face value. Investigation, in order:
  (1) re-read the fictional readme.md in full, cataloguing every fictional symbol
  (`EmbroideryPattern`, `Stitch::normal/color_change()`, `Color::rgb/palette_index/thread()`,
  `StitchCommand`, `PesVersion`, `pattern.scale/translate/rotate/optimize()`, `pes::write_file`,
  `pec::write_file`); (2) read every real source file in full —
  `embroidery_file.rs`, `thread.rs`, `stitch_instruction.rs`, `metadata.rs`, `format.rs`, `format/pes.rs`,
  `format/pec.rs`, `error.rs`, `lib.rs` — plus a `grep -n "pub fn"` sweep across
  `format/{pec,pes}/{reader,writer}.rs` to get every exact real function signature; (3) discovered a
  discrepancy the task's own Goal text did not name: neither `pec::write` nor `pes::write` has a
  path-based `write_file()` convenience — both take a `Write + Seek` writer directly, so the accurate
  Quick Start must show manual `File`/`BufWriter` construction; (4) checked `Cargo.toml` — `enabled`
  (default) covers all read/write functionality used in the readme, `random` is a separate opt-in feature
  gating only `get_random_thread()`, not touched by any example; checked for `changelog.md` (exists, only
  documents the 0.1.0 initial release, no contradiction of readme claims), `docs/` (none — nothing to
  cross-link, unlike tiles_tools), `tests/`/`examples/` (neither exists; sample binaries live in
  `test_files/` and unit tests are inline `#[cfg(test)]` blocks); (5) built
  `cargo doc -p embroidery_tools --no-deps --all-features` and inspected the generated HTML module tree —
  confirmed the crate's `mod_interface!` layering re-exports nothing at the crate root at all (`own/`
  contains no struct/enum/fn pages), meaning even a symbol-corrected `use embroidery_tools::*;` would
  still resolve nothing; every type must be imported from its own submodule
  (`embroidery_file::EmbroideryFile`, `thread::{Color, Thread}`, `stitch_instruction::{Instruction,
  Stitch}`, `format::{pec, pes}`) — a structural finding beyond what the task's own symbol list
  anticipated. Resolution: rewrote both Quick Start examples, the Core Types table, the Pattern
  Operations block, the Thread Color Handling block, and both Integration Examples using only
  verified-real types/signatures/module paths; added an explicit note on the crate-root re-export gap;
  in Current Status & Roadmap, promoted stitch-encoding normalization
  (`fix_color_count()`/`interpolate_stop_as_duplicate_color()`/`interpolate_duplicate_color_as_stop()` —
  real, tested methods) from Planned to Implemented, and demoted geometric transforms
  (scale/translate/rotate — confirmed absent from the full `EmbroideryFile` impl block) to Planned; left
  the header, Features bullets, Installation, Supported Formats table, Use Cases section, and the
  File Format Specifications/Coordinate Systems prose untouched — none reference fictional API symbols,
  and Use Cases/Roadmap prose was explicitly optional-reword per the task's own Goal. Verification:
  extracted every runnable function into a temporary `examples/-scratch_readme_check.rs` and ran
  `cargo check -p embroidery_tools --example=-scratch_readme_check --all-features` — compiled clean (2
  warnings, both artifacts of the scratch harness itself: an unused `mut` and a missing-crate-docs lint,
  neither reflecting the readme content); deleted the scratch file and its now-empty `examples/` directory
  immediately after, confirmed gone via a repeat `cargo check` reporting "no example target". Adversarial
  pass additionally: grepped the finished file for every fictional symbol name (clean — the only
  remaining hits are the two prose lines that explicitly correct the record); confirmed the `_ => {}`
  wildcard arm is required and present for matching against the `#[non_exhaustive]` `Instruction` enum;
  confirmed `Thread`'s `..Default::default()` usage against `format/pec.rs`'s own identical pattern before
  relying on it; flagged (but deliberately left unedited, outside mandatory scope) the pre-existing
  "Coordinate Systems" mm-unit claim as unverified either way after a quick grep for conversion-factor
  evidence in the pes/pec reader/writer source came back empty.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Tier 2 Dual-Role Self-Check (D1-D8; documentation-accuracy
  rewrite task, not a code-defect fix, so B1-B7 Bug-Fixing Quality Requirements do not apply — consistent
  with tasks 012/019/023/025/034 this session). Confirming pass re-read the full rewritten `readme.md` and
  cross-checked every code sample against the verified-real signatures. Adversarial pass re-ran the
  fictional-symbol grep sweep on the finished file, re-verified the deleted scratch example left no trace,
  and re-checked the `cargo check` compile result reflected the file's actual final content (not a stale
  intermediate draft) by confirming the scratch file's content matched the readme's code blocks
  verbatim at the time of the check. All 8 dimensions 🟢, see Verification Record below. State set to
  ✅ Completed; moved `draft/` → `completed/`.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Draft-stage Goal-only format; Goal names the exact file, catalogs the specific fictional symbols, and specifies the real API + P4 rewrite-bucket scope | — |
| D2 | MOST Goal Quality | — | 🟢 | Motivated (every original code sample fails to compile against the real crate), Observable (fictional symbols either exist in source or don't; code either compiles or doesn't), Scoped (1 file), Testable (grep for fictional symbol names; `cargo check` against extracted code blocks) | — |
| D3 | Value / YAGNI | — | 🟢 | Null Hypothesis: skip → the very first two lines of the Quick Start already fail to compile (`EmbroideryPattern`/`stitch_count()` don't exist) for any developer who copies it; concrete, already-existing need, not speculative | — |
| D4 | Implementation Readiness | — | 🟢 | Real API confirmed via full reads of every source module, a `pub fn` signature sweep across all 4 reader/writer files, a `cargo doc` ground-truth check of the actual public module tree, and a compiled `cargo check` proof of the final examples | — |
| D5 | Execution Scope | — | 🟢 | Single file edited (`module/helper/embroidery_tools/readme.md`); scratch verification artifact (`examples/-scratch_readme_check.rs`) created and fully removed in the same session, confirmed gone via repeat `cargo check` | — |
| D6 | Crate Scope Unity | — | 🟢 | Edit confined entirely to `module/helper/embroidery_tools/` (readme.md plus the deleted scratch example) | — |
| D7 | Crate Locality | — | 🟢 | Fix applied directly in the owning crate's own readme, no aggregator or workspace-level doc touched | — |
| D8 | Crate Single Responsibility | — | 🟢 | No responsibility change — correcting documentation accuracy only | — |
| **Total** | | 🔴 | 🟢 | 0 | 0/0 |

**Aggregate verdict:** PASS — all 8 dimensions clean on both passes, zero Blocking Findings. One
Non-Blocking residual was explicitly flagged rather than silently dropped: the pre-existing "Coordinate
Systems" prose claim (mm units, automatic machine-specific conversion) is unverified either way — a
targeted grep for conversion-factor evidence in the format reader/writer source found nothing conclusive,
and settling it definitively would require a binary-format-level investigation disproportionate to a
readme-wording task; the claim was left untouched (not newly introduced by this task, and outside the
mandatory Quick Start/API Reference scope). D1–D8 are the Readiness Verification Gate dimensions, reused
at completion per this session's established precedent for decision/hygiene/doc-rewrite tasks (matching
tasks 012/019/023/025/034) — not a defect fix, so Bug-Fixing Task Quality Requirements (B1–B7) do not
apply.
