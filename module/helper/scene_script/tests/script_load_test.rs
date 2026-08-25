//! Verifies `scene_script::script_as_glue_load`/`script_as_data_load` — the
//! production compile-and-lint entry points task 416 adds — actually reject
//! a lint-violating script at load time, before any evaluation happens, and
//! still accept a real well-formed script for each form. Distinct from
//! `purity_lint_test.rs` and `example_convention_test.rs`, which test the
//! two lint functions directly against an already-compiled `AST`; this file
//! tests the new wiring layer that compiles `source` and calls them.

use scene_script::{ engine_build, script_as_glue_load, script_as_data_load, ScriptLoadError };
use std::{ cell::RefCell, rc::Rc };

#[ test ]
fn script_as_glue_load_rejects_a_top_level_loop()
{
  let engine = engine_build();
  let err = script_as_glue_load( &engine, "let x = 1; for i in 0..3 { x += 1; }" )
  .expect_err( "a top-level `for` loop violates the script-as-glue convention" );

  assert!( matches!( err, ScriptLoadError::Lint( _ ) ), "expected a Lint rejection, got {err:?}" );
}

#[ test ]
fn script_as_data_load_rejects_an_impure_call()
{
  let engine = engine_build();
  let err = script_as_data_load( &engine, "fn one() { 1 } #{ x: one() }" )
  .expect_err( "a call anywhere in the AST violates the script-as-data purity convention" );

  assert!( matches!( err, ScriptLoadError::Lint( _ ) ), "expected a Lint rejection, got {err:?}" );
}

#[ test ]
fn script_as_glue_load_rejects_invalid_syntax()
{
  let engine = engine_build();
  let err = script_as_glue_load( &engine, "let x = ;" )
  .expect_err( "malformed Rhai syntax must fail to parse" );

  assert!( matches!( err, ScriptLoadError::Parse( _ ) ), "expected a Parse failure, got {err:?}" );
}

#[ test ]
fn script_as_data_load_rejects_invalid_syntax()
{
  let engine = engine_build();
  let err = script_as_data_load( &engine, "#{ x: " )
  .expect_err( "malformed Rhai syntax must fail to parse" );

  assert!( matches!( err, ScriptLoadError::Parse( _ ) ), "expected a Parse failure, got {err:?}" );
}

#[ test ]
fn script_as_glue_load_accepts_bindings_plus_trailing_main_call()
{
  let engine = engine_build();
  let ast = script_as_glue_load( &engine, "let x = 1; fn main( x ) { x + 1 } main( x )" ).unwrap();

  let result : i64 = engine.eval_ast( &ast ).unwrap();
  assert_eq!( result, 2 );
}

#[ test ]
fn script_as_data_load_accepts_a_pure_literal_document()
{
  let engine = engine_build();
  let ast = script_as_data_load( &engine, "let star = #{ radius: 5.0 }; star" ).unwrap();

  let _ : rhai::Dynamic = engine.eval_ast( &ast ).unwrap();
}

/// Proves the Test Matrix's own ordering guarantee ( T01: "Returns `Err`
/// before any engine evaluation happens" ) — not merely that the rejected
/// script never yields an `AST`, but that evaluation genuinely never ran. A
/// registered side-effecting function inside the rejected script would flip
/// `called` if `script_as_glue_load` evaluated anything internally instead
/// of only compiling and linting.
#[ test ]
fn script_as_glue_load_rejection_never_evaluates_the_script()
{
  let mut engine = engine_build();
  let called = Rc::new( RefCell::new( false ) );
  let called_sink = called.clone();
  engine.register_fn( "mark_called", move || { *called_sink.borrow_mut() = true; } );

  // The leading `mark_called()` is a non-trailing, non-`main` top-level
  // call -- exactly what the checker rejects. If evaluation ever ran
  // despite the rejection, this call would flip `called` to `true`.
  let err = script_as_glue_load( &engine, "mark_called(); let x = 1;" )
  .expect_err( "a non-trailing, non-main top-level call must be rejected" );

  assert!( matches!( err, ScriptLoadError::Lint( _ ) ) );
  assert!( !*called.borrow(), "script_as_glue_load must reject before evaluating anything" );
}
