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

pub const HASH21 : &str = include_str!( "../shaders/chunks/hash21.wgsl" );
pub const VALUE_NOISE : &str = include_str!( "../shaders/chunks/value_noise.wgsl" );
pub const FBM3 : &str = include_str!( "../shaders/chunks/fbm3.wgsl" );
pub const FULLSCREEN_TRIANGLE : &str = include_str!( "../shaders/chunks/fullscreen_triangle.wgsl" );

// A chunk's `//@ key: value` header lines are its manifest; this reads only
// the two fields the composer actually needs ( `name` and `depends_on` ) via
// a first-match line scan, not a general comment parser. `description`/
// `export`/`stage` stay human- and future-tool-facing, unread by this
// example's own composition logic — the tests module below cross-checks
// `export` against the real WGSL body anyway, so it can't silently drift.

fn manifest_field<'a>( wgsl : &'a str, key : &str ) -> &'a str
{
  let prefix = format!( "//@ {key}:" );
  wgsl.lines()
  .find_map( | line | line.strip_prefix( prefix.as_str() ) )
  .unwrap_or_else( || panic!( "chunk missing required `//@ {key}:` header line:\n{wgsl}" ) )
  .trim()
}

fn parse_name( wgsl : &str ) -> &str
{
  manifest_field( wgsl, "name" )
}

fn parse_depends_on( wgsl : &str ) -> Vec< &str >
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
/// and concatenates their WGSL bodies in that order. Panics, naming the
/// offending chunk, on a dependency cycle or a `depends_on` entry not
/// present in `chunks` — both are authoring mistakes in a chunk's header
/// ( or in the set passed to `compose` ), not states a correctly-authored
/// composition can reach.
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

#[ cfg( test ) ]
mod tests
{
  use super::*;

  const ALL_CHUNKS : &[ &str ] = &[ HASH21, VALUE_NOISE, FBM3, FULLSCREEN_TRIANGLE ];

  /// Test-only: collects every `//@ key: value` line in `wgsl`, in file
  /// order. Unlike [`super::manifest_field`], this is never needed by
  /// `compose()` — only `export` repeats per chunk, and only the test below
  /// cares about it — so it lives here rather than the main module, where a
  /// fn unused outside tests would be dead code in a non-test build.
  fn manifest_fields<'a>( wgsl : &'a str, key : &str ) -> Vec< &'a str >
  {
    let prefix = format!( "//@ {key}:" );
    wgsl.lines()
    .filter_map( | line | line.strip_prefix( prefix.as_str() ) )
    .map( str::trim )
    .collect()
  }

  /// Test-only: pulls the declared symbol name out of an `export` line's
  /// WGSL signature ( `"fn hash21(p: vec2f) -> f32"` -> `"hash21"`,
  /// `"struct VertexOutput { .. }"` -> `"VertexOutput"` ).
  fn exported_name( signature : &str ) -> &str
  {
    signature.split_whitespace().nth( 1 ).unwrap_or( signature )
    .split( '(' ).next().unwrap_or( signature )
  }

  #[ test ]
  fn depends_on_covers_every_actual_wgsl_call_to_another_chunk()
  {
    for &chunk in ALL_CHUNKS
    {
      let name = parse_name( chunk );
      let declared = parse_depends_on( chunk );
      for &other in ALL_CHUNKS
      {
        let other_name = parse_name( other );
        if other_name == name
        {
          continue;
        }
        let calls_it = chunk.contains( &format!( "{other_name}(" ) );
        let declares_it = declared.contains( &other_name );
        assert_eq!
        (
          calls_it, declares_it,
          "chunk `{name}`: actual wgsl call to `{other_name}` = {calls_it}, but depends_on lists it = {declares_it}"
        );
      }
    }
  }

  #[ test ]
  fn export_names_match_a_real_declaration_in_the_wgsl_body()
  {
    for &chunk in ALL_CHUNKS
    {
      for signature in manifest_fields( chunk, "export" )
      {
        let name = exported_name( signature );
        let declared = chunk.contains( &format!( "fn {name}(" ) ) || chunk.contains( &format!( "struct {name}" ) );
        assert!( declared, "chunk declares export `{signature}` but no `fn {name}(` or `struct {name}` found in its body" );
      }
    }
  }

  #[ test ]
  fn compose_orders_dependencies_before_dependents_regardless_of_input_order()
  {
    let composed = compose( &[ FBM3, FULLSCREEN_TRIANGLE, VALUE_NOISE, HASH21 ] );
    let hash21_pos = composed.find( "fn hash21" ).expect( "hash21 present" );
    let value_noise_pos = composed.find( "fn value_noise" ).expect( "value_noise present" );
    let fbm3_pos = composed.find( "fn fbm3" ).expect( "fbm3 present" );
    assert!( hash21_pos < value_noise_pos, "hash21 must precede value_noise" );
    assert!( value_noise_pos < fbm3_pos, "value_noise must precede fbm3" );
  }

  #[ test ]
  #[ should_panic( expected = "was not passed to compose" ) ]
  fn compose_panics_on_missing_dependency()
  {
    compose( &[ VALUE_NOISE, FBM3 ] );
  }

  #[ test ]
  #[ should_panic( expected = "cyclic shader-chunk dependency" ) ]
  fn compose_panics_on_cyclic_dependency()
  {
    const A : &str = "//@ name: a\n//@ depends_on: b\nfn a() {}";
    const B : &str = "//@ name: b\n//@ depends_on: a\nfn b() {}";
    compose( &[ A, B ] );
  }

  #[ test ]
  fn parse_depends_on_handles_empty_value()
  {
    assert_eq!( parse_depends_on( "//@ name: x\n//@ depends_on:\n" ), Vec::< &str >::new() );
  }

  #[ test ]
  fn parse_depends_on_handles_multiple_entries()
  {
    assert_eq!( parse_depends_on( "//@ depends_on: a, b\n" ), vec![ "a", "b" ] );
  }
}
