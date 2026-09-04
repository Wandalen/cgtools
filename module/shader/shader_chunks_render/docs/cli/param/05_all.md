# Parameter :: 29. all

- **Fundamental Type:** [`Switch`](../../../../shader_chunks_query/docs/cli/type/07_switch.md)
  (unilang `Kind::Boolean`)
- **Constraints:**
  - Cannot be combined with `name`, `file::`, or `set::` — giving any of
    them alongside `all::1` rejects with exit 1 before any target is
    resolved or any chunk touched
  - When on, sweeps every entry in
    [`shader_chunks_core::CHUNKS`](../../../../shader_chunks_core/readme.md),
    writing `<out>/<name>.png` per chunk; `out::` is read as the output
    DIRECTORY instead of a file path, created if it doesn't already exist
  - A chunk whose shape isn't previewable is skipped, not failed — the
    batch continues; every other per-chunk error (naga validation, GPU,
    io) is a failure that also does not stop the batch, but flips the
    whole command's exit code to 1 once the sweep completes
- **Default:** `false` — omitting `all::1` renders exactly one target,
  same as before this parameter existed
- **Purpose:** Regenerates every bundled chunk's static-image artifact in
  one invocation — documentation preview sets, visual regression fixture
  refreshes — without a shell loop over `shader_chunks list format::names`.

### Examples
```bash
# Valid values
render all::1                              # every chunk, 256x256, into the cwd
render all::1 out::renders/ size::128      # every chunk, 128x128, into ./renders/ (created if missing)
render all::1 time::2.5                    # every chunk, frozen at time 2.5

# Invalid values (rejected with error)
render fbm3 all::1                    # error: cannot be combined with a target (`name`/`file::`) or `set::`, exit 1
render all::1 file::harness.wgsl      # error: cannot be combined with a target (`name`/`file::`) or `set::`, exit 1
render all::1 set::lacunarity:2.5     # error: cannot be combined with a target (`name`/`file::`) or `set::`, exit 1
```

### Notes
- Implemented by `render_all_to_png`/`batch_summary` in
  `shader_chunks_render/src/lib.rs`, reusing `render_to_png` per chunk —
  no separate GPU or naga-validation path from the single-target case.
- No `set::` overrides are available under `all::1`: a single override
  list can't cleanly apply across chunks with different declared
  parameters, so every chunk renders at its bundle-declared defaults.
- Distinguishing skip from failure relies on the same
  `PreviewError::Unpreviewable` nested error this crate's single-target
  path already rejects with exit 1 — under `all::1` that exact condition
  becomes a per-chunk skip instead, since one chunk's shape being
  unpreviewable says nothing about the rest of the registry.
- Member of no [parameter group](../../../../shader_chunks_query/docs/cli/param_group/readme.md) —
  like `out`/`size`/`time`/`set`, it shapes the rendered artifact rather
  than filtering/projecting/formatting query output.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.render](../command/01_render.md) | `false` | Mutually exclusive with `name`, `file::`, `set::` |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [Switch](../../../../shader_chunks_query/docs/cli/type/07_switch.md) | Boolean toggle | `bool` | `1`/`true`/`yes` on, `0`/`false`/`no` off |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
