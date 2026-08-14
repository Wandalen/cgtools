//! Verifies `scene_script::check_whole_ast_is_pure` rejects any function or
//! method call anywhere in a script's AST — top-level or nested inside a
//! `let` initializer, array element, object-map value, or a function body's
//! control-flow blocks — with no exception for operator calls. See
//! `docs/pattern/004_script_as_data.md` and
//! `docs/invariant/004_script_as_data_purity.md`.

use scene_script::{ engine_build, check_whole_ast_is_pure };

#[ test ]
fn accepts_a_pure_literal_document_mirroring_a_real_scene()
{
  // Mirrors examples/orrery/webgpu/scene/scene.rhai's own shape: `let`
  // bindings to plain map/array literals, then a trailing map literal
  // referencing them — zero calls anywhere.
  let engine = engine_build();
  let ast = engine.compile
  (
    "
    let star = #{ radius: 5.0 };
    let planets = [ #{ radius: 1.0, orbit_radius: 10.0 }, #{ radius: 0.5, orbit_radius: 20.0 } ];
    #{ star: star, planets: planets }
    "
  ).unwrap();

  check_whole_ast_is_pure( &ast ).unwrap();
}

#[ test ]
fn rejects_an_operator_call_nested_inside_a_map_value()
{
  // A bare `1 + 2` would be constant-folded away by the engine's default
  // `Simple` optimization level before the checker ever sees it — wrapping
  // one operand in a script-defined function call defeats that (`Simple`
  // never evaluates calls, only `Full` does — see
  // top_level_lint's `checker_rejects_a_top_level_if`), so the `+`
  // operator call survives as a real AST node. Proves no exception is
  // made for operator-desugared calls.
  let engine = engine_build();
  let ast = engine.compile( "fn one() { 1 } #{ x: one() + 2 }" ).unwrap();

  let violation = check_whole_ast_is_pure( &ast ).expect_err( "a nested operator call must be rejected" );
  assert_eq!( violation.name, "+" );
}

#[ test ]
fn rejects_a_named_call_nested_inside_an_array_element()
{
  let engine = engine_build();
  let ast = engine.compile( "fn compute( n ) { n } [ 1, compute( 2 ), 3 ]" ).unwrap();

  let violation = check_whole_ast_is_pure( &ast ).expect_err( "a nested named call must be rejected" );
  assert_eq!( violation.name, "compute" );
}

#[ test ]
fn rejects_a_method_call_nested_inside_a_map_value()
{
  let engine = engine_build();
  let ast = engine.compile( "#{ value: \"seed\".len() }" ).unwrap();

  let violation = check_whole_ast_is_pure( &ast ).expect_err( "a nested method call must be rejected" );
  assert_eq!( violation.name, "len" );
}

#[ test ]
fn rejects_a_call_two_blocks_deep_inside_a_function_body()
{
  // No top-level statement calls `main` — a trailing `main();` would
  // itself be a call and, since `AST::walk` visits top-level statements
  // before function bodies, would be found first and mask the
  // deeply-nested violation this test exists to prove recursion reaches.
  // `AST::walk` visits every `fn` definition's body regardless of whether
  // it is ever called, so omitting the call site still reaches `main`'s
  // body. `top_level_lint::check_top_level_is_declarative` would accept
  // this AST outright (only `fn` definitions exist; there are no
  // top-level statements at all to reject) — proving this checker catches
  // what that one structurally cannot see.
  //
  // The `if`'s condition reads `flag` (a bare parameter reference) rather
  // than a comparison like `i == 2`: `==` desugars to an operator call
  // the same way `+` does, and `Stmt::If::walk` visits the condition
  // before the branch body — a comparison there would itself be found
  // first and mask `trigger()`, the violation this test targets.
  let engine = engine_build();
  let ast = engine.compile
  (
    "
    fn trigger() {}

    fn main( flag )
    {
      for i in 0..3
      {
        if flag
        {
          trigger();
        }
      }
    }
    "
  ).unwrap();

  let violation = check_whole_ast_is_pure( &ast )
  .expect_err( "a call two control-flow blocks deep inside a function body must be rejected" );
  assert_eq!( violation.name, "trigger" );
}
