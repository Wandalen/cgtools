# Parameter :: 1. out

- **Fundamental Type:** String (unilang `Kind::String`)
- **Constraints:** Must be a writable path at invocation time, resolved
  relative to the process's current directory; a path that cannot be
  written (e.g. a missing parent directory) returns
  `ComposeCliError::Io`, exit code 2. An existing file at the path is
  overwritten without prompting.
- **Default:** None — omitting `out::` prints the composed WGSL to
  stdout exactly as before this parameter existed. Unlike
  [`render`'s own `out`](../../../../shader_chunks_render/docs/cli/param/01_out.md),
  which always has a computed default filename, `compose`'s `out` has
  no default: its absence changes the *destination* of the output
  (stdout vs. file), not just the filename.
- **Purpose:** Names the file `.compose` writes the composed WGSL to,
  instead of printing it to stdout. The second parameter across the
  `shader_chunks` CLI naming a filesystem output path (after `render`'s
  own `out`) — every other parameter either filters/projects/formats
  stdout content or selects an input.

### Examples
```bash
# Valid values
compose hash21 value_noise                       # prints composed WGSL to stdout (no out::)
compose hash21 value_noise out::bundle.wgsl       # writes to bundle.wgsl, stdout gets a summary line instead
compose fbm3 transitive::1 out::fbm3_bundle.wgsl  # closure composed, then written to the given path

# Invalid values (rejected with error)
compose hash21 out::no_such_dir/x.wgsl            # io error: ..., exit 2 — the parent directory must exist
```

### Notes
- On success with `out::` given, stdout carries only the summary line
  `wrote <path> (<n> bytes wgsl)` — the composed text itself never
  reaches stdout in this mode, matching how `render`'s `out::` keeps
  the PNG bytes off stdout.
- The write happens via [`compose_write`](../../../../shader_chunks_compose/readme.md),
  called only after composition already succeeded — a name/dependency/cycle
  failure (exit 1) never reaches the write step, so a failed `compose`
  never leaves a partial or stale file at `out::`.
- On success the write is unconditional — an existing file at the path
  is replaced, matching `render`'s own re-run workflow (tweak the
  chunk set, run again, same artifact path).
- Member of no [parameter group](../../../../shader_chunks_query/docs/cli/param_group/readme.md) —
  like `render`'s `out`, it steers an artifact/side effect, not
  filtering/projection/formatting.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.compose](../command/01_compose.md) | None (stdout) | Only parameter in this crate naming a filesystem output path |

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
