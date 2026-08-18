//! Subprocess regression coverage for `shader_chunks_query`'s duplicated-
//! named-argument handling (see [`shader_chunks_cli_core::arg_string_checked`]/
//! `arg_bool_checked`) -- BUG-283's fix pattern applied to this crate's own
//! call sites (BUG-285). No other CLI behavior is covered here; the
//! `list`/`get`/`tags`/`tree` query engine itself is exercised through
//! `shader_chunks_query_core`'s own test suite.

use assert_cmd::Command;

// test_kind: bug_reproducer(BUG-285)
/// ## Root Cause
/// `query_params_from` read every named argument through
/// `shader_chunks_cli_core::arg_string`, whose catch-all `_ => None` arm
/// cannot distinguish "argument absent" from "argument supplied more than
/// once" -- `unilang` binds a repeated named key to `Value::List`
/// regardless of the argument's own declared `multiple` attribute (`pattern`
/// is a plain, non-`multiple` `Kind::String`), so `Value::List` fell through
/// the same arm as "absent" and `params.pattern` silently kept its empty
/// default, matching every chunk instead of failing.
/// ## Why Not Caught
/// No existing test in this crate or `shader_chunks_query_core` ever passed
/// a named `key::value` argument twice in one invocation -- every prior
/// test either omitted a parameter or supplied it exactly once, so the
/// `Value::List`-from-duplication shape was never exercised.
/// ## Fix Applied
/// `query_params_from` and `cmd_tree`'s routine now read every named
/// argument through `arg_string_checked`/`arg_bool_checked`
/// (`shader_chunks_query/src/lib.rs`), which returns a loud
/// `ValidationRuleFailed` naming the key and its repeat count instead of
/// silently falling back to the default.
/// ## Prevention
/// This subprocess test locks in the loud-failure behavior for `pattern`;
/// sibling crates `shader_chunks_render`/`shader_chunks_preview` gained
/// their own matching regression tests in the same fix (BUG-285).
/// ## Pitfall
/// `arg_usize` (`limit`/`offset`/`width` here) has the identical catch-all
/// shape and remains unfixed -- a duplicated `limit::1 limit::2` still
/// silently falls back to `0` rather than erroring; a known, not a
/// forgotten, gap (documented alongside BUG-283 and this fix's own source
/// comment).
#[ test ]
fn subprocess_list_with_duplicated_pattern_fails_loudly_instead_of_matching_everything()
{
  let output = Command::cargo_bin( "shader_chunks_query" ).expect( "binary builds" )
  .args( [ "list", "pattern::fbm3", "pattern::hash21" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ), "stdout: {}", String::from_utf8_lossy( &output.stdout ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "`pattern` was given 2 times" ), "stderr: {stderr}" );
}

#[ test ]
fn subprocess_tree_with_duplicated_reverse_fails_loudly()
{
  let output = Command::cargo_bin( "shader_chunks_query" ).expect( "binary builds" )
  .args( [ "tree", "fbm3", "reverse::1", "reverse::0" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ), "stdout: {}", String::from_utf8_lossy( &output.stdout ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "`reverse` was given 2 times" ), "stderr: {stderr}" );
}

#[ test ]
fn subprocess_list_with_single_pattern_still_succeeds()
{
  // Guards against an over-broad fix: a single occurrence of a named
  // argument must keep working exactly as before.
  let output = Command::cargo_bin( "shader_chunks_query" ).expect( "binary builds" )
  .args( [ "list", "pattern::fbm3" ] )
  .output()
  .expect( "runs" );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!( stdout.contains( "fbm3" ), "stdout: {stdout}" );
}
