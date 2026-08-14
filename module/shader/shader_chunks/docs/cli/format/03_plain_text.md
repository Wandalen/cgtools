# Format :: F03. plain_text

| Field | Value |
|-------|-------|
| ID | F03 |
| Output context | `compose` (composed WGSL preview) |
| Trigger | Always — `compose`'s output is shader source, not a data view; there is no `format::` selection |
| Structure | Raw WGSL source text, chunk bodies concatenated in dependency order, no added framing |
| Rendering mechanism | `shader_chunks_core::try_compose`'s returned string, printed as-is — no `data_fmt` pipeline, since the output is code, not tabular or hierarchical data |
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
[`expanded`](05_expanded.md) format when the query engine unified `list`
and `get` — chunk detail is now a *data view* with selectable fields, not
a hand-built string.

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.compose](../command/05_compose.md) | Raw composed WGSL, dependency-ordered |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
