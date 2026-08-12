//! Generic, manifest-driven shader-chunk composer. Each chunk under
//! `shaders/chunks/` is exactly one WGSL function (or, for the vertex stage,
//! one entry point plus the struct type it returns) with its manifest
//! embedded as a leading `//@`-prefixed comment block — the same
//! machine-parsable-attribute convention this ecosystem's playbooks use
//! ( see `playbook.rulebook.md § Structure : Attribute Block` ), with `//`
//! standing in for `#` since that is WGSL's line-comment token. [`compose`]
//! reads every chunk's `//@ depends_on:` line, topologically sorts the given
//! chunks, and concatenates their WGSL bodies dependency-before-dependent —
//! regardless of the order they were passed in — panicking immediately, and
//! naming the offending chunk, on a cyclic or unresolvable dependency.
//!
//! This module intentionally has no Rust reimplementation of any chunk's
//! math ( no `hash21`/`value_noise`/`fbm3` Rust ports ). Chunks are a
//! shader-side concept only; the manifest, not a parallel Rust body, is what
//! makes a chunk's interface legible — and since the manifest now lives
//! inside the same file as the code it describes, there is only ever one
//! file to open, not two.

/// `hash21` chunk: 2D-point -> pseudo-random scalar hash.
pub const HASH21 : &str = include_str!( "../shaders/chunks/hash21.wgsl" );
/// `value_noise` chunk: bilinear value noise over [`HASH21`].
pub const VALUE_NOISE : &str = include_str!( "../shaders/chunks/value_noise.wgsl" );
/// `fbm3` chunk: three-octave fractal Brownian motion over [`VALUE_NOISE`].
pub const FBM3 : &str = include_str!( "../shaders/chunks/fbm3.wgsl" );
/// `fullscreen_triangle` chunk: vertex entry point emitting one screen-covering triangle.
pub const FULLSCREEN_TRIANGLE : &str = include_str!( "../shaders/chunks/fullscreen_triangle.wgsl" );

// A chunk's `//@ key: value` header lines are its manifest; this reads only
// the two fields the composer actually needs ( `name` and `depends_on` ) via
// a first-match line scan, not a general comment parser. `description`/
// `export`/`stage` stay human- and future-tool-facing, unread by this
// example's own composition logic — the integration tests
// ( `tests/shader_chunks_test.rs` ) cross-check `export` against the real
// WGSL body anyway, so it can't silently drift.

fn manifest_field<'a>( wgsl : &'a str, key : &str ) -> &'a str
{
  let prefix = format!( "//@ {key}:" );
  wgsl.lines()
  .find_map( | line | line.strip_prefix( prefix.as_str() ) )
  .unwrap_or_else( || panic!( "chunk missing required `//@ {key}:` header line:\n{wgsl}" ) )
  .trim()
}

/// Reads the chunk's `//@ name:` manifest line.
///
/// # Panics
///
/// Panics if the chunk has no `//@ name:` header line.
#[ must_use ]
pub fn parse_name( wgsl : &str ) -> &str
{
  manifest_field( wgsl, "name" )
}

/// Reads the chunk's `//@ depends_on:` manifest line as a list of chunk
/// names ( empty when the value is empty ).
///
/// # Panics
///
/// Panics if the chunk has no `//@ depends_on:` header line.
#[ must_use ]
pub fn parse_depends_on( wgsl : &str ) -> Vec< &str >
{
  let raw = manifest_field( wgsl, "depends_on" );
  if raw.is_empty()
  {
    return Vec::new();
  }
  raw.split( ',' ).map( str::trim ).collect()
}

struct ParsedChunk< 'a >
{
  name : &'a str,
  depends_on : Vec< &'a str >,
  wgsl : &'a str,
}

/// Topologically sorts `chunks` by each one's header-declared `depends_on`
/// and concatenates their WGSL bodies in that order, dependency before
/// dependent, regardless of the order they were passed in.
///
/// # Panics
///
/// Panics, naming the offending chunk, on a dependency cycle or a
/// `depends_on` entry not present in `chunks` — both are authoring mistakes
/// in a chunk's header ( or in the set passed to `compose` ), not states a
/// correctly-authored composition can reach.
#[ must_use ]
pub fn compose( chunks : &[ &str ] ) -> String
{
  let entries : Vec< ParsedChunk< '_ > > = chunks.iter()
  .map( | &wgsl | ParsedChunk { name : parse_name( wgsl ), depends_on : parse_depends_on( wgsl ), wgsl } )
  .collect();

  let mut ordered_names : Vec< &str > = Vec::with_capacity( entries.len() );
  let mut visiting : Vec< &str > = Vec::new();

  for entry in &entries
  {
    visit( entry.name, &entries, &mut visiting, &mut ordered_names );
  }

  ordered_names.iter()
  .map( | name | entries.iter().find( | e | e.name == *name ).expect( "just visited" ).wgsl )
  .collect::< Vec< _ > >()
  .join( "\n\n" )
}

fn visit<'a>
(
  name : &'a str,
  entries : &[ ParsedChunk< 'a > ],
  visiting : &mut Vec< &'a str >,
  ordered_names : &mut Vec< &'a str >,
)
{
  if ordered_names.contains( &name )
  {
    return;
  }
  assert!( !visiting.contains( &name ), "cyclic shader-chunk dependency: {visiting:?} -> {name}" );

  let entry = entries.iter().find( | e | e.name == name )
  .unwrap_or_else( || panic!( "chunk `{name}` is depended on but was not passed to compose()" ) );

  visiting.push( name );
  for &dep in &entry.depends_on
  {
    visit( dep, entries, visiting, ordered_names );
  }
  visiting.pop();
  ordered_names.push( name );
}
