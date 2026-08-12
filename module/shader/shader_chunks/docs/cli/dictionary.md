# Dictionary

Domain terms used throughout `docs/cli/`, alphabetical.

- **Chunk** — A named, reusable fragment of WGSL shader source, embedded at
  compile time via `include_str!`, carrying declared metadata (name,
  description, stage, tags, dependencies, exports) in `//@` comments.
- **Compose** — The act of concatenating one or more chunks into a single
  valid WGSL text, resolving dependency order automatically regardless of
  input order.
- **Dependency** — A chunk that another chunk's WGSL body calls into, and
  which therefore must be included alongside it for the composed output to
  compile.
- **Export** — A named function or item a chunk makes available to WGSL
  code that includes it (e.g. `hash21` exports a `fn hash21(...)`).
- **Registry** — The compiled-in, static list of every chunk
  (`shader_chunks_core::ALL_CHUNKS`) this CLI inspects; never runtime-discovered
  or loaded from the filesystem.
- **Stage** — The shader pipeline stage (e.g. fragment, vertex, or
  stage-agnostic) a chunk is written for.
- **Tag** — A free-form label attached to a chunk for discovery and
  categorization, independent of its dependency graph.
