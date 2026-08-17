# shader_chunks_query_core

**Keywords:** WGSL, Shader Composition, Query Engine, Filtering, Rendering

Query engine over [`shader_chunks_core`](../shader_chunks_core/readme.md)'s
bundled WGSL chunks: one filter/project/sort/page/render pipeline behind
both the [`shader_chunks_query`](../shader_chunks_query/readme.md) CLI's
`list` and `get` commands. Each public function here takes already-parsed
arguments and returns the exact string the CLI prints — keeping rendering
inside these functions, rather than in the CLI wiring layer, is what makes
direct-call testing possible: `shader_chunks_query`'s own test tier needs
no subprocess to assert on output content (see its `tests/`).

**Pipeline order** (applied by [`chunks_query`]):

```text
select (names:: or every bundled chunk)
  -> filter (pattern, tag, stage, depends_on, exports, roots, leaves)
  -> count short-circuit (returns early if count:: is set)
  -> sort + order
  -> offset + limit
  -> render (fields:: projection, then format::)
```

[`QueryParams`] carries the full 19-parameter surface as one struct;
[`QueryParams::list_defaults`]/[`QueryParams::get_defaults`] are the only
difference between `list` and `get` — same engine, different starting
values (overview columns + plain table vs. detail columns + expanded
records). Every enum-shaped parameter (`TagsMode`, `SortKey`, `SortOrder`,
`OutputFormat`) parses from its `key::value` spelling via `FromStr`,
rejecting an unrecognized value as `QueryError::InvalidParam` rather than
silently falling back to a default.

## Usage

```rust
use shader_chunks_query_core::{ QueryParams, chunks_query, tags_list, chunk_tree };

// list-equivalent: every chunk, overview columns, plain table
let table = chunks_query( &QueryParams::list_defaults() ).unwrap();

// get-equivalent, narrowed to one chunk
let mut params = QueryParams::get_defaults();
params.names = vec![ "fbm3".to_string() ];
let record = chunks_query( &params ).unwrap();

let tags = tags_list().unwrap();          // every group:tag pair and its chunk(s)
let forest = chunk_tree( None, false ).unwrap();       // dependency forest of every root chunk
let dependents = chunk_tree( None, true ).unwrap();     // reverse forest: what depends on each leaf chunk
```

## Errors

[`QueryError`] covers every failure mode a query can hit: `UnknownChunk`
(a `names`/`depends_on` entry not in [`shader_chunks_core::CHUNKS`]),
`UnknownField` (a `fields::` entry outside [`QUERY_FIELDS`]),
`InvalidParam` (a bad enum spelling), and `Render` (a `data_fmt`
formatter failure). `QueryError::exit_code()` maps the first three to `1`
(caller-fixable by passing different arguments) and `Render` to `2`
(internal) — the same split [`shader_chunks_query`](../shader_chunks_query/readme.md)'s
own CLI error mapping reuses verbatim.
