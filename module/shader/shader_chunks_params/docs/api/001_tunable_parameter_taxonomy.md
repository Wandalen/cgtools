# API: Tunable Parameter Taxonomy

### Scope

- **Purpose**: Define the 5-kind tunable-parameter taxonomy, the `//@ param:` grammar chunk authors write to declare a tunable, and the Rust types/functions this crate exposes for discovering them.
- **Responsibility**: Document `ParameterKind`, `ValueType`, `RangeSource`, `Range`, `Parameter`, and the `discover`/`discover_chunk` functions — the complete public surface for turning `//@ param:` manifest lines into structured data.
- **In Scope**: The `//@ param:` line grammar; every public type and function in `src/lib.rs`; the malformed-directive panic contract.
- **Out of Scope**: How a range is chosen when a line declares none (see [`algorithm/001`](../algorithm/001_range_inference_heuristic.md), which states the HOW this API's `range` field relies on); annotating any real bundled `shader/*.wgsl` chunk (no consumer does so yet — see this crate's [`readme.md`](../../readme.md)).

### Abstract

A shader chunk's manifest header — the `//@`-prefixed comment block `shader_chunks_core` already reads for `name`/`description`/`tags`/`depends_on`/`export` — may additionally carry one `//@ param:` line per tunable value the chunk exposes: a plain function argument, a compile-time define directive, a uniform-buffer field, a vertex attribute, or a bound texture. `discover` reads every such line out of raw WGSL text, in file order, resolving each one into a `Parameter` — never executing, binding, or animating anything. Discovery is purely textual: the same trust model `shader_chunks_core::manifest_field` already applies to authored manifest content applies here, so a malformed line panics rather than returning an error a caller might silently ignore.

### Grammar

```text
//@ param: <name> <kind> <type> [range(min, max)]
```

- **`<name>`**: the parameter's identifier, copied verbatim from the adjacent real WGSL declaration.
- **`<kind>`**: one of the 5 literal tokens below.
- **`<type>`**: a WGSL type token, copied verbatim from the adjacent real declaration — see the Types table's `ValueType` row for the exact set this crate recognizes.
- **`range(min, max)`**: optional. When present, `min`/`max` are parsed as `f64` and always take precedence over inference (see [`algorithm/001`](../algorithm/001_range_inference_heuristic.md)). When absent, the line's range is resolved by `infer_range` at discovery time.

Example, one of each kind:

```wgsl
//@ param: octaves argument u32 range(1, 8)
//@ param: enable_fog define bool
//@ param: amplitude uniform f32
//@ param: workgroup_x attribute u32
//@ param: albedo texture texture_2d
```

### Kinds

| Token | `ParameterKind` variant | Meaning |
|-------|--------------------------|---------|
| `argument` | `Argument` | A plain WGSL function argument. |
| `define` | `Define` | A compile-time `override`-style define directive. |
| `uniform` | `Uniform` | A uniform-buffer field. |
| `attribute` | `Attribute` | A vertex-stage attribute. |
| `texture` | `Texture` | A bound texture. `infer_range` always returns `None` for this kind, regardless of `<type>` or `<name>` — see [`algorithm/001`](../algorithm/001_range_inference_heuristic.md). |

### Types

| Type | Shape | Purpose |
|------|-------|---------|
| `ParameterKind` | 5-variant enum (`Argument`, `Define`, `Uniform`, `Attribute`, `Texture`) | The `<kind>` token, parsed. |
| `ValueType` | 14-variant enum (`Bool`, `U32`, `I32`, `F32`, `Vec2F`/`Vec3F`/`Vec4F`, `Vec2I`/`Vec3I`/`Vec4I`, `Vec2U`/`Vec3U`/`Vec4U`, `Texture2d`) | The `<type>` token, parsed. Variant names mirror the WGSL type token exactly (`vec2f` → `Vec2F`), except `texture_2d` → `Texture2d`. |
| `RangeSource` | 2-variant enum (`Declared`, `Inferred`) | Whether a `Parameter`'s range came from the line's own `range(min, max)` clause or from `infer_range`. |
| `Range` | `{ min: f64, max: f64 }` | An inclusive numeric range. |
| `Parameter` | `{ name: String, kind: ParameterKind, value_type: ValueType, range: Option<(Range, RangeSource)> }` | One fully-parsed `//@ param:` line. `range` is `None` only when the line declared none AND `infer_range` also returned `None` (a `texture` kind or `bool` type with no declared range). |

### Operations

| Function | Conceptual Signature | Behavior |
|----------|----------------------|----------|
| `discover` | `(wgsl: &str) -> Vec<Parameter>` | Parses every `//@ param:` line in `wgsl`, in file order. Returns an empty `Vec` when none are present — this is not an error. Panics on a malformed line (wrong token count, unknown `<kind>` token, unknown `<type>` token, or a malformed `range(min, max)` clause), naming the offending line in the panic message. |
| `discover_chunk` | `(chunk: &shader_chunks_core::ChunkDescriptor) -> Vec<Parameter>` | Equivalent to `discover(chunk.wgsl)` — this crate's only dependency on `shader_chunks_core`; `discover` itself has none. Same panic contract as `discover`. |
| `infer_range` | `(kind: ParameterKind, value_type: ValueType, name: &str) -> Option<Range>` | The range-resolution heuristic `discover` calls when a line declares no `range(min, max)` clause. Full rule table in [`algorithm/001`](../algorithm/001_range_inference_heuristic.md). Never panics. |

### Error Handling

There is no `Result`-returning function in this crate's public API. Every parsing failure — malformed token count, an unrecognized `<kind>`/`<type>` token, or a malformed `range(min, max)` clause — is a panic, mirroring `shader_chunks_core::manifest_field`'s established panic-on-malformed-authored-content idiom: chunk manifests are trusted authored content written by the same developer building the shader, not adversarial input from an untrusted caller.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `ParameterKind`, `ValueType`, `RangeSource`, `Range`, `Parameter`, `discover`, `discover_chunk` — the entire public API this document describes |

### Tests

| File | Relationship |
|------|--------------|
| `tests/discovery_test.rs` | Exercises every kind, declared vs. inferred ranges, file-order multi-param handling, the empty-`Vec` case, every panic path, and the `discover_chunk` wrapper |
