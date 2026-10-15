//! Cross-checks that every `uniform` declared in `shader.frag` has a matching
//! `gl.get_uniform_location( &program, "..." )` call in `main.rs`, and vice versa -- catches a
//! uniform being renamed on one side and not the other, which silently reads back `None`/no-ops
//! at runtime instead of failing to compile.

use std::collections::HashSet;

const SHADER_FRAG : &str = include_str!( "../shaders/shader.frag" );
const MAIN_RS : &str = include_str!( "../src/main.rs" );

/// Extracts every `uniform <type> <name>;` declaration from a GLSL source.
fn declared_uniforms( src : &str ) -> HashSet< String >
{
  src
  .lines()
  .filter_map( | line | line.trim().strip_prefix( "uniform " ) )
  .filter_map( | rest | rest.trim_end_matches( ';' ).split_whitespace().last() )
  .map( str::to_string )
  .collect()
}

/// Extracts every uniform name looked up via `get_uniform_location( &program, "name" )` in Rust.
fn queried_uniforms( src : &str ) -> HashSet< String >
{
  src
  .split( "get_uniform_location( &program, \"" )
  .skip( 1 )
  .filter_map( | s | s.split( '"' ).next() )
  .map( str::to_string )
  .collect()
}

#[ test ]
fn test_every_declared_uniform_is_queried_in_rust()
{
  let declared = declared_uniforms( SHADER_FRAG );
  assert!( !declared.is_empty(), "sanity: shader.frag should declare at least one uniform" );

  let queried = queried_uniforms( MAIN_RS );
  assert!( !queried.is_empty(), "sanity: main.rs should query at least one uniform location" );

  for name in &declared
  {
    assert!
    (
      queried.contains( name ),
      "shader.frag declares uniform \"{name}\" but main.rs never calls \
      get_uniform_location( &program, \"{name}\" ) for it — likely a stale/renamed uniform"
    );
  }
}

#[ test ]
fn test_every_queried_uniform_is_declared_in_shader()
{
  let declared = declared_uniforms( SHADER_FRAG );
  let queried = queried_uniforms( MAIN_RS );

  for name in &queried
  {
    assert!
    (
      declared.contains( name ),
      "main.rs queries uniform \"{name}\" but shader.frag does not declare it — likely a \
      stale/renamed uniform"
    );
  }
}
