# Format :: F03. plain_text

| Field | Value |
|-------|-------|
| ID | F03 |
| Output context | `compose` (composed WGSL preview); `preview` (build/validation summary line); `render` (write/validation summary line) |
| Trigger | Always for all three — none exposes a `format::` selection; each prints unstructured text directly |
| Structure | `compose`: raw WGSL source text, chunk bodies concatenated in dependency order, no added framing. `preview`: a `wrote <path> (<n> bytes wgsl, naga-validated)` line, a `target: <name>` line, then a `sliders:` header followed by one `  <property>  <min>..<max>  start <value>` line per declared slider. `render`: a `wrote <path> (<w>x<h> px, naga-validated)` line, `target:` and `time:` lines, then a `parameters:` header followed by one `  <property> = <value>` line per parameter (default unless overridden via `set::`) |
| Rendering mechanism | `compose`: `shader_chunks_core::try_compose`'s returned string. `preview`: `shader_chunks_preview::summary`'s returned string. `render`: `shader_chunks_render::summary`'s returned string. All three are printed as-is via `shader_chunks_cli_core::text_output` — no `data_fmt` pipeline, since none of the outputs is tabular or hierarchical data |
| Example | See below |

### Example

`compose hash21 value_noise` (hash21's body precedes value_noise's,
regardless of input order, since `try_compose` resolves dependency order
internally):

```text
fn hash21(p: vec2f) -> f32 { /* ... */ }

fn value_noise(p: vec2f) -> f32 { /* ... uses hash21 ... */ }
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

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.compose](../command/01_compose.md) | Raw composed WGSL, dependency-ordered |
| 2 | [.preview](../../../../shader_chunks_preview/docs/cli/command/01_preview.md) | Build/validation summary line, printed whether or not `serve::` runs afterward |
| 3 | [.render](../../../../shader_chunks_render/docs/cli/command/01_render.md) | Write/validation summary line; the image itself goes to the filesystem, never stdout |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
