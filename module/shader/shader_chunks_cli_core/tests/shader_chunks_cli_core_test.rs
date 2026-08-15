//! Direct-call tests for `shader_chunks_cli_core`'s shared wiring layer —
//! no subprocess; each utility's own CLI crate exercises this layer
//! end-to-end through its own commands (see e.g.
//! `shader_chunks_preview/tests/preview_cli_test.rs`). This file covers
//! only the layer's own cross-cutting invariant: [`registry_build`] must
//! reject two aggregated utilities declaring the same command name loudly,
//! never silently let the second shadow the first.

use unilang::prelude::*;
use shader_chunks_cli_core::{ CommandSet, registry_build, text_output };

/// A minimal, well-formed `( CommandDefinition, CommandRoutine )` pair
/// carrying no arguments — enough to pass `registry_build`'s own
/// `register_with_routine` call, so a test can isolate the duplicate-name
/// assert from unrelated definition-validity failures.
fn dummy_command( name : &str ) -> ( CommandDefinition, CommandRoutine )
{
  let def = CommandDefinition::former()
  .name( name.to_string() )
  .namespace( String::new() )
  .description( "dummy".to_string() )
  .hint( "dummy" )
  .status( "stable" )
  .version( "1.0.0".to_string() )
  .aliases( vec![] )
  .tags( vec![] )
  .permissions( vec![] )
  .idempotent( true )
  .deprecation_message( String::new() )
  .http_method_hint( String::new() )
  .examples( vec![] )
  .arguments( vec![] )
  .end();

  let routine : CommandRoutine = Box::new( | _cmd, _ctx | Ok( text_output( String::new() ) ) );
  ( def, routine )
}

#[ test ]
fn registry_build_accepts_unique_command_names()
{
  let commands : CommandSet = vec![ dummy_command( ".alpha" ), dummy_command( ".beta" ) ];
  let registry = registry_build( commands );
  assert!( registry.commands().contains_key( ".alpha" ), "registry should contain `.alpha`" );
  assert!( registry.commands().contains_key( ".beta" ), "registry should contain `.beta`" );
}

#[ test ]
#[ should_panic( expected = "duplicate command name" ) ]
fn registry_build_panics_on_duplicate_command_name_across_utilities()
{
  // The scenario `registry_build`'s doc comment names explicitly: two
  // aggregated utilities independently declaring the same command name is
  // an integration mistake that must fail loudly at build time, never
  // silently let the second definition shadow the first.
  let commands : CommandSet = vec![ dummy_command( ".dup" ), dummy_command( ".dup" ) ];
  let _ = registry_build( commands );
}
