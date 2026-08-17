# Command Group :: 5. Preview

### Pattern

Live browser artifact production: build a
[`shader_chunks_preview_core::PreviewBundle`](../../../../shader_chunks_preview_core/readme.md)
from a bundled chunk or a local WGSL file, naga-validate the composed
WGSL, write it as `-preview.json` into the `shader_chunks_preview_web`
runner crate, and — by default — launch that runner in the browser.

### Purpose

Let a shader author see a chunk rendered live, with its `//@ param:`
uniforms wired to sliders, without hand-writing a WGPU harness — the
terminal-native counterpart to opening the browser and wiring up
`shader_chunks_preview_web` by hand.

### Semantic Coherence Test

"The member command produces a live, browser-servable rendering artifact
from the compiled-in chunk registry (or a local file), not just a text or
metadata view of it." `.preview` is the only command whose output is a
running dev server (or, with `serve::0`, a written bundle file plus a
summary) rather than stdout content alone.

### Why NOT Merge Into Compose

Both `.preview` and `.compose` start from chunk WGSL and resolve it into
a single composed program, but their invariants diverge exactly where it
matters: [Compose](../../../../shader_chunks_compose/docs/cli/command_group/01_compose.md)'s own invariant is "no side effects
outside stdout content and process exit code — the composed WGSL is
printed, never written to a file." `.preview` breaks that by design — it
always writes `-preview.json` to `shader_chunks_preview_web`'s crate
directory, and by default spawns a blocking browser dev-server subprocess
via `action/browser_serve`. Merging would put a side-effecting,
server-launching command inside a group whose entire contract is
"stdout only."

### Invariants

- Bundle contents are idempotent for identical input (same chunk or file
  content) — but the command itself is NOT side-effect-free: `-preview.json`
  is always (re)written on success, unconditionally overwriting any
  previous bundle in that path.
- Naga validation runs before any write — a chunk that fails to parse or
  validate leaves `-preview.json` untouched and exits non-zero (1); no
  partial or stale-looking bundle is ever written on failure.
- Exactly one target is required: `name` (positional) or `file::`, never
  both, never neither — violating this fails loudly with exit 1 before
  any chunk lookup or file read is attempted.
- `serve::0` skips the dev-server hand-off and returns the summary as
  normal command output instead — this is what makes the command testable
  end-to-end via subprocess without a browser.

### Referenced Commands

| # | Command | Relationship |
|---|---------|---------------|
| 1 | [`.preview`](../command/01_preview.md) | Member — live browser preview of one chunk |

**Membership:** 1 of the 8 commands across the `shader_chunks` family; the
full partition across all 6 command groups (spanning all 5 leaf CLIs) is
stated in [the family index](../../../../shader_chunks/docs/cli/readme.md).
A single-member group is deliberate — the boundary is output-species (a
live rendering artifact, with real filesystem and subprocess side
effects), not command count.

### Referenced Tests

| File | Relationship |
|------|--------------|
| [`../../../tests/docs/cli/command_group/01_preview.md`](../../../tests/docs/cli/command_group/01_preview.md) | Group-level test specification |
| [`../../../../shader_chunks_preview/tests/preview_cli_test.rs`](../../../../shader_chunks_preview/tests/preview_cli_test.rs) | `name_target_prepares_a_validated_bundle`, `unknown_name_is_rejected_with_the_shared_unknown_chunk_text`, `missing_file_is_an_io_error_with_exit_code_2`, `preview_without_serve_writes_the_bundle_into_the_web_runner_crate`, `subprocess_preview_serve_0_succeeds_and_prints_the_summary`, `subprocess_preview_with_unknown_name_fails_with_exit_1`, `subprocess_preview_with_no_target_fails_loudly`, `subprocess_help_lists_the_preview_group` |

### Typical Patterns

Discover with [Query](../../../../shader_chunks_query/docs/cli/command_group/01_query.md), confirm what tunables exist with
[Parameters](../../../../shader_chunks_params/docs/cli/command_group/01_parameters.md)'s `tunables <name>`, then
`preview <name>` to see it live with those same parameters wired to
sliders — `serve::0` first if only the bundle (not a browser tab) is
wanted, e.g. in a script or CI check.

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`../readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
