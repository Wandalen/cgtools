# Format :: F03. plain_text

| Field | Value |
|-------|-------|
| ID | F03 |
| Output context | `get` (full chunk detail) and `compose` (composed WGSL preview) |
| Trigger | Always — neither command has a table/tree shape; both print a plain `format!`-built string via `println!` |
| Structure | `get`: fixed `label: value` lines, one per field (`name`, `description`, `stage`, `tags`, `depends_on`, `exports`), in that order. `compose`: raw WGSL source text, chunk bodies concatenated in dependency order, no added framing |
| Rendering mechanism | Hand-built `format!` string in `src/lib.rs` (`get_chunk`, deferring to `shader_chunks::try_compose` for `compose`) — no `data_fmt` table/tree pipeline, since neither output is tabular or hierarchical |
| Example | See below |

### Example

`get hash21`:

```text
name: hash21
description: Single-value hash of a 2D point into [0, 1).
stage: None
tags: category:hash
depends_on: (none)
exports:
  fn hash21(p: vec2f) -> f32
```

`compose hash21 value_noise` (hash21's body precedes value_noise's,
regardless of input order, since `try_compose` resolves dependency order
internally):

```text
fn hash21(p: vec2f) -> f32 { /* ... */ }

fn value_noise(p: vec2f) -> f32 { /* ... uses hash21 ... */ }
```

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.get](../command/02_get.md) | Fixed 6-line label:value detail block |
| 2 | [.compose](../command/05_compose.md) | Raw composed WGSL, dependency-ordered |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
