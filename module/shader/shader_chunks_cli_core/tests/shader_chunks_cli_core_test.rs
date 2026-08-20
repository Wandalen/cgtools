//! Direct-call tests for `shader_chunks_cli_core`'s shared wiring layer —
//! no subprocess; each utility's own CLI crate exercises this layer
//! end-to-end through its own commands (see e.g.
//! `shader_chunks_preview/tests/preview_cli_test.rs`). This file covers
//! the layer's own cross-cutting invariants: [`registry_build`] must reject
//! two aggregated utilities declaring the same command name loudly, never
//! silently let the second shadow the first; and [`arg_usize_checked`]
//! (BUG-295) must reject a duplicated integer-valued named argument loudly
//! rather than silently defaulting to `0` -- covered here, via a real
//! `Pipeline` dispatch against a throwaway command, rather than through a
//! consuming crate's own subprocess test, because none of this crate
//! family's currently `arg_usize`-consuming commands (`shader_chunks_query`)
//! fall within this bug-hunt's assigned scope.

use unilang::prelude::*;
use shader_chunks_cli_core::{ CommandSet, arg_usize_checked, named_arg, registry_build, text_output };

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

/// A `( CommandDefinition, CommandRoutine )` pair carrying one optional
/// integer-valued named argument `n` ( default `"0"` ), whose routine calls
/// [`arg_usize_checked`] and echoes the result as text output — real
/// `unilang` dispatch through [`registry_build`] + `Pipeline`, not a
/// hand-built `VerifiedCommand` ( no code in this crate family constructs
/// one directly ; every extractor is exercised through real argv parsing ).
fn int_arg_command( name : &str ) -> ( CommandDefinition, CommandRoutine )
{
  let def = CommandDefinition::former()
  .name( name.to_string() )
  .namespace( String::new() )
  .description( "dummy int-arg command".to_string() )
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
  .arguments( vec![ named_arg( "n", Kind::Integer, "test integer", Some( "0".to_string() ) ) ] )
  .end();

  let routine : CommandRoutine = Box::new( | cmd, _ctx |
  {
    let n = arg_usize_checked( &cmd, "n" )?;
    Ok( text_output( n.to_string() ) )
  });
  ( def, routine )
}

// test_kind: bug_reproducer(BUG-295)
/// ## Root Cause
/// `arg_usize`'s catch-all `_ => Ok( 0 )` arm cannot tell "argument absent"
/// apart from "argument supplied twice" -- `unilang` binds ANY repeated
/// named argument to `Value::List` regardless of the argument's own
/// `multiple` attribute (same underlying mechanism as BUG-283/BUG-285),
/// so a duplicated integer-valued key silently resolved to `0` instead of
/// erroring.
/// ## Why Not Caught
/// BUG-283 and BUG-285 fixed every `arg_string`/`arg_bool` call site they
/// touched (`shader_chunks_compose`, `shader_chunks_query`,
/// `shader_chunks_preview`) but explicitly left `arg_usize` itself
/// unfixed, disclosed by name in both fixes' own Pitfall comments
/// (`shader_chunks_query/src/lib.rs`,
/// `shader_chunks_preview/tests/preview_cli_test.rs`) as "a known, not a
/// forgotten, gap" -- no test ever exercised a duplicated integer-valued
/// argument because the fix that would have added one hadn't landed yet.
/// ## Fix Applied
/// Added `arg_usize_checked` (`shader_chunks_cli_core/src/lib.rs`),
/// mirroring `arg_bool_checked`'s exact shape: an explicit `Value::List`
/// arm returning a loud `ValidationRuleFailed` naming the key and its
/// repeat count, instead of falling through to the default-`0` catch-all.
/// `arg_usize` itself is left as-is (additive fix, not a breaking
/// in-place change, matching how `arg_string`/`arg_bool` were handled).
/// ## Prevention
/// `shader_chunks_query`'s 3 existing `arg_usize` call sites (`limit`,
/// `offset`, `width`) remain exposed to this defect until swapped to
/// `arg_usize_checked` -- out of this bug-hunt's assigned scope
/// (`shader_chunks_cli_core`/`shader_chunks_preview`/
/// `shader_chunks_preview_web`), left as a disclosed follow-up rather than
/// silently fixed in a crate outside scope.
/// ## Pitfall
/// A `_checked` sibling only closes the gap for callers that actually
/// switch to it -- adding the function is necessary but not sufficient;
/// every existing unchecked call site remains vulnerable until migrated,
/// same lesson BUG-283's original fix already left on the table for
/// `arg_string`/`arg_bool`.
#[ test ]
fn arg_usize_checked_fails_loudly_on_duplicated_value_instead_of_defaulting_to_zero()
{
  let commands : CommandSet = vec![ int_arg_command( ".inttest" ) ];
  let registry = registry_build( commands );
  let pipeline = Pipeline::new( registry );

  let ok_result = pipeline.process_command_from_argv_simple
  ( &[ ".inttest".to_string(), "n::5".to_string() ] );
  assert!( ok_result.success, "a single `n::5` must succeed: {:?}", ok_result.error );
  assert_eq!( ok_result.outputs[ 0 ].content, "5", "single value must round-trip through arg_usize_checked" );

  let dup_result = pipeline.process_command_from_argv_simple
  ( &[ ".inttest".to_string(), "n::5".to_string(), "n::10".to_string() ] );
  assert!( !dup_result.success, "a duplicated `n::5 n::10` must fail loudly, not silently default to 0" );
  let error = dup_result.error.expect( "a failed dispatch carries an error message" );
  assert!
  (
    error.contains( "`n`" ) && error.contains( "2 times" ),
    "error should name the duplicated key and its repeat count, got: {error}"
  );
}
