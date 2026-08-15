# Parameter :: 24. out

- **Fundamental Type:** String (unilang `Kind::String`)
- **Constraints:** Must be a writable path at invocation time, resolved
  relative to the process's current directory; a path that cannot be
  written (e.g. a missing parent directory) returns
  `RenderCliError::Io`, exit code 2. An existing file at the path is
  overwritten without prompting.
- **Default:** `<target>.png` — derived from the target: `<name>.png`
  for a bundled chunk, the file stem + `.png` for a `file::` target
  (`render.png` when the path has no stem), always in the current
  directory.
- **Purpose:** Names the PNG file `.render` writes its single frame to —
  the CLI's only parameter naming a filesystem output path. Every other
  command's output goes to stdout (or, for `.preview`, into
  `shader_chunks_preview_web`'s fixed bundle location).

### Examples
```bash
# Valid values
render fbm3                          # writes fbm3.png in the current directory
render fbm3 out::doc/fbm3_256.png    # explicit path, used verbatim
render file::./scratch/-draft.wgsl   # derives -draft.png from the file stem

# Invalid values (rejected with error)
render fbm3 out::no_such_dir/x.png   # io error: ..., exit 2 — the parent directory must exist
```

### Notes
- The effective default is computed by `out_path_of`
  (`shader_chunks_render/src/lib.rs`) and unit-tested for all three
  arms — name target, file target with a stem, and explicit `out::`
  winning over both (`out_path_default_derives_from_the_target`).
- The path is only ever touched on success: naga validation, GPU
  rendering, and readback all complete before the write, so a failed
  render never leaves a partial PNG and never clobbers a previous good
  one (subprocess-tested via the unknown-chunk case asserting the file
  stays absent).
- On success the write is unconditional — an existing file is replaced,
  matching the re-render workflow (tweak `time::`/`size::`, run again,
  same artifact path).
- Member of no [parameter group](../../../../shader_chunks_query/docs/cli/param_group/readme.md) — like
  `file`/`serve`, it steers an artifact/side effect, not
  filtering/projection/formatting.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.render](../command/01_render.md) | `<target>.png` | Only parameter in the CLI naming a filesystem output path |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| String | String | `String` | Must be a writable path at invocation time |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
