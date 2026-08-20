# Parameter :: 22. file

- **Fundamental Type:** String (unilang `Kind::String`)
- **Constraints:** Must name a readable path, resolved relative to the
  process's current directory at invocation time; an unreadable or
  missing path returns `PreviewCliError::Io`, exit code 2. Mutually
  exclusive with, and jointly required alongside, `name` — giving both or
  neither fails with exit 1 before either is resolved.
- **Default:** — (no default; exactly one of `name`/`file::` is required)
- **Purpose:** Names a local `.wgsl` chunk file to preview or render
  instead of a bundled chunk — the file's own text (manifest header
  included) is read and passed through the same bundle-building and
  naga-validation path as a bundled chunk's `wgsl` field.

### Examples
```bash
# Valid values
preview file::shader/fbm3/fbm3.wgsl serve::0   # previews a local file
preview file::./scratch/draft_chunk.wgsl                 # relative path, any location

# Invalid values (rejected with error)
preview file::does_not_exist.wgsl   # io error: reading `does_not_exist.wgsl`: ..., exit 2
preview fbm3 file::fbm3.wgsl        # both name and file:: given, exit 1
```

`.render` accepts the same values with the same failure modes:
`render file::./scratch/draft_chunk.wgsl out::draft.png` renders the
local file to a PNG instead of serving it.

### Notes
- Unlike `name`, `file::` performs no lookup against
  `shader_chunks_core::CHUNKS` — any readable WGSL text is accepted,
  bundled or not, which is what makes `.preview` usable on a chunk still
  under development, and what makes `file::` `.render`'s escape hatch
  for chunks outside the previewable shapes (a hand-written
  fragment-stage harness).
- A file that reads successfully but fails naga parse/validation still
  exits 1 (`PreviewCliError::Validation`), same as a bundled chunk would
  — `file::`'s own failure mode is narrower (`Io`, exit 2), covering only
  the read itself.
- Member of no [parameter group](../../../../shader_chunks_query/docs/cli/param_group/readme.md) — like
  `name`/`names`, it is a target selector, not a filter/projection/format
  modifier.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.preview](../command/01_preview.md) | — | Alternative to `name`; exactly one of the two is required |
| 2 | [.render](../../../../shader_chunks_render/docs/cli/command/01_render.md) | — | Same exclusivity; the local file becomes the rendered frame's source |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| String | String | `String` | Must be a readable path at invocation time |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
