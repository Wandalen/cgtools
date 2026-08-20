# Format :: F03. plain_text

| Field | Value |
|-------|-------|
| ID | F03 |
| Output context | `compose` (composed WGSL preview, or a write summary line under `out::`); `preview` (build/validation summary line); `render` (write/validation summary line); `validate` (all-clear message, or a findings report) |
| Trigger | Always for all four — none exposes a `format::` selection; each prints unstructured text directly |
| Structure | `compose`: raw WGSL source text, chunk bodies concatenated in dependency order, no added framing; with `out::<path>` given, this text is written to the file instead and stdout carries only a `wrote <path> (<n> bytes wgsl)` summary line. `preview`: a `wrote <path> (<n> bytes wgsl, naga-validated)` line, a `target: <name>` line, then a `sliders:` header followed by one `  <property>  <min>..<max>  start <value>` line per declared slider. `render`: a `wrote <path> (<w>x<h> px, naga-validated)` line, `target:` and `time:` lines, then a `parameters:` header followed by one `  <property> = <value>` line per parameter (default unless overridden via `set::`); under `all::1`, this single-target report is replaced by one `<name>: wrote <path>` / `<name>: skipped (<reason>)` / `<name>: failed (<error>)` line per chunk followed by a `<n> chunks: <r> rendered, <s> skipped, <f> failed` totals line. `validate`: either `registry is clean: <n> chunks, 0 findings` (zero findings), or a `<n> finding(s):` header followed by one blank-line-separated `[<chunk>] <check>: <message>` block per finding |
| Rendering mechanism | `compose`: `shader_chunks_core::try_compose`'s returned string, or (under `out::`) `shader_chunks_compose::compose_write`'s summary string. `preview`: `shader_chunks_preview::summary`'s returned string. `render`: `shader_chunks_render::summary`'s returned string. `validate`: `shader_chunks_validate::validate`'s returned string (`Ok` case), or `ValidateCliError::FindingsPresent`'s carried report string (`Err` case). All four are printed as-is via `shader_chunks_cli_core::text_output` — no `data_fmt` pipeline, since none of the outputs is tabular or hierarchical data |
| Example | See below |

### Example

`compose hash21 value_noise` (hash21's body precedes value_noise's,
regardless of input order, since `try_compose` resolves dependency order
internally):

```text
fn hash21(p: vec2f) -> f32 { /* ... */ }

fn value_noise(p: vec2f) -> f32 { /* ... uses hash21 ... */ }
```

`compose hash21 value_noise out::bundle.wgsl` (same composed text, written
to the file instead — stdout carries only the summary; exact byte count
varies):

```text
wrote bundle.wgsl (156 bytes wgsl)
```

`get`'s former `label: value` detail block moved to the
[`expanded`](../../../../shader_chunks_query/docs/cli/format/05_expanded.md) format when the query engine unified `list`
and `get` — chunk detail is now a *data view* with selectable fields, not
a hand-built string.

`preview fbm3 serve::0` (a value chunk — one synthesized `preview_scale`
slider; exact byte count and range vary with the chunk):

```text
wrote /path/to/shader_chunks_preview_web/-preview.json (/* n */ bytes wgsl, naga-validated)
target: fbm3
sliders:
  preview_scale  0.1..10  start 1
```

`render fbm3 size::128` (same bundle, frozen to a file — the parameter
lines carry the baked-in default values instead of slider ranges):

```text
wrote fbm3.png (128x128 px, naga-validated)
target: fbm3
time: 0
parameters:
  preview_scale = 8
```

`render all::1 out::renders/ size::128` (batch report — one line per
`shader_chunks_core::CHUNKS` entry, then a totals line; exact count and
names grow with the collection):

```text
fbm3: wrote renders/fbm3.png
fullscreen_triangle: skipped (chunk `fullscreen_triangle` is not previewable ...)
hash21: wrote renders/hash21.png
... (one line per registry entry)
<n> chunks: <n - 1> rendered, 1 skipped, 0 failed
```

`validate` (the real bundled registry is clean today):

```text
registry is clean: 50 chunks, 0 findings
```

A dirty registry instead prints a count header followed by one block per
finding, blank-line separated:

```text
2 finding(s):

[hash21] manifest_drift: description field disagrees with the manifest

[fbm3] wgsl_compile: <naga diagnostic text, possibly multi-line>
```

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.compose](../command/01_compose.md) | Raw composed WGSL, dependency-ordered |
| 2 | [.preview](../../../../shader_chunks_preview/docs/cli/command/01_preview.md) | Build/validation summary line, printed whether or not `serve::` runs afterward |
| 3 | [.render](../../../../shader_chunks_render/docs/cli/command/01_render.md) | Write/validation summary line; the image itself goes to the filesystem, never stdout |
| 4 | [.validate](../../../../shader_chunks_validate/docs/cli/command/01_validate.md) | All-clear message or blank-line-separated findings report; the only referencing command producing no artifact at all |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
