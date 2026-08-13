# Dictionary

Domain terms used throughout `docs/cli/`, alphabetical.

- **Chunk** — A named, reusable fragment of WGSL shader source, embedded at
  compile time via `include_str!`, carrying declared metadata (name,
  description, stage, tags, dependencies, exports) in `//@` comments.
- **Compose** — The act of concatenating one or more chunks into a single
  valid WGSL text, resolving dependency order automatically regardless of
  input order. The named set must be dependency-complete — strict by
  default; `transitive::1` widens it to its full dependency closure first.
- **Dependency** — A chunk that another chunk's WGSL body calls into, and
  which therefore must be included alongside it for the composed output to
  compile.
- **Export** — A named function or item a chunk makes available to WGSL
  code that includes it (e.g. `hash21` exports a `fn hash21(...)`).
- **Field** — One queryable attribute of a chunk record (`name`,
  `description`, `stage`, `tags`, `depends_on`, `exports`, `source`) —
  the unit of projection via `fields::`.
- **Filtering** — Narrowing *which chunks* a query keeps (pattern, tags,
  stage, dependency relationships, roots/leaves) — the first stage of the
  query pipeline.
- **Formatting** — Shaping how a query result is ordered, paged, and
  rendered (`format::`, `sort::`, `order::`, `limit::`, `offset::`,
  `heading::`, `width::`) — the last stage of the query pipeline.
- **Leaf** — A chunk with no dependencies of its own; selectable via
  `leaves::1`.
- **Projection** — Choosing *what is shown* about each kept chunk — a
  field subset via `fields::`, or just the total via `count::1` — the
  middle stage of the query pipeline.
- **Query** — A `list`/`get` invocation: select a candidate set, filter
  it, project fields, and render — one shared engine
  (`query_chunks`) behind both commands, differing only in defaults.
- **Registry** — The compiled-in, static table of every chunk
  (`shader_chunks_core::CHUNKS`) this CLI inspects; never runtime-discovered
  or loaded from the filesystem.
- **Root** — A chunk no other chunk depends on; a natural entry point,
  selectable via `roots::1` and rendered by `tree`'s forest view.
- **Selection** — Fixing the candidate set a query starts from: the
  positional `names` (in order, duplicates allowed) for `get`/`list`, or
  every registry chunk when `list` gets no names.
- **Stage** — The shader pipeline stage (e.g. fragment, vertex, or
  stage-agnostic) a chunk is written for.
- **Tag** — A free-form `group:tag` label attached to a chunk for
  discovery and categorization, independent of its dependency graph;
  selectable via `tag::` selectors (exact pair or bare tag).
- **Tunable** — A value a chunk declares as meant-to-be-adjusted via a
  `//@ param:` comment line, carrying a name, kind, WGSL type, and range;
  the range's source is either *declared* (an explicit `range(min, max)`
  clause) or *inferred* (heuristic, via
  `shader_chunks_params::infer_range`). Listed by `tunables`.
