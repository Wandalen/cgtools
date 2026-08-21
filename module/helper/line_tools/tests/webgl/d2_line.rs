//! Structural regression coverage for UX/DX #4: `d2::Line::mesh_update`/`draw` used to panic via
//! `.expect( "Mesh has not been created yet" )` when called before `mesh_create`, while
//! `d3::Line`'s equivalents (via `impl_basic_line!`) already returned `Err` gracefully for the
//! identical condition. No live `WebGl2RenderingContext` test infrastructure exists in this
//! crate (no `wasm-bindgen-test` dev-dependency -- same limitation this crate's other
//! GL-context-dependent fixes document, e.g. `canvas_renderer`'s BUG-342/BUG-493 tests), so this
//! is a structural/source-inspection check instead, mirroring that established technique.

/// Extracts a function's body verbatim from `src/d2/line.rs`, via brace counting from the given
/// signature fragment's opening `{` to its matching closing `}`.
fn d2_line_fn_body( signature_fragment : &str ) -> &'static str
{
  const SRC : &str = include_str!( "../../src/d2/line.rs" );

  let sig_pos = SRC.find( signature_fragment )
  .unwrap_or_else( || panic!( "`{signature_fragment}` not found in src/d2/line.rs -- has this function been renamed or moved?" ) );
  let after_sig = &SRC[ sig_pos.. ];
  let open_brace = after_sig.find( '{' )
  .unwrap_or_else( || panic!( "no opening brace found after `{signature_fragment}`" ) );

  let mut depth = 0_i32;
  let mut close = None;
  for ( i, ch ) in after_sig[ open_brace.. ].char_indices()
  {
    match ch
    {
      '{' => depth += 1,
      '}' =>
      {
        depth -= 1;
        if depth == 0
        {
          close = Some( open_brace + i + 1 );
          break;
        }
      },
      _ => {}
    }
  }
  let close = close.unwrap_or_else( || panic!( "unbalanced braces while scanning `{signature_fragment}`'s body" ) );

  &after_sig[ ..close ]
}

/// ## Root Cause
/// `d2::Line::mesh_update`/`draw` accessed `self.mesh` via `.as_mut().expect( "Mesh has not
/// been created yet" )`, panicking if called before `mesh_create` -- while `d3::Line`'s
/// equivalents (via `impl_basic_line!`'s `mesh_get`/`mesh_get_mut`) already returned
/// `Err( WebglError )` for the identical precondition.
/// ## Why Not Caught
/// No existing test exercises `d2::Line` at all (confirmed: zero matches for `d2::Line`/`d2::line`
/// across this crate's `tests/` directory prior to this fix).
/// ## Fix Applied
/// Both call sites now use `.ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?`
/// instead of `.expect( .. )`, matching `d3::Line`'s established convention. `mesh_get`/
/// `mesh_get_mut` were deliberately left unchanged (still panicking, bare references) -- see
/// this crate's own judgment-call note for why: 2 out-of-scope example files
/// (`examples/minwebgl/2d_line`, `examples/minwebgl/space_partition`) call `.mesh_get_mut()` in
/// direct-chain style incompatible with a `Result`-returning signature.
/// ## Prevention
/// Structural/source-inspection test (no live GL context available -- see module doc): asserts
/// neither function's body contains the panicking `.expect( "Mesh has not been created yet" )`
/// call, and that both instead contain the `Result`-returning `.ok_or( .. )?` form.
/// ## Pitfall
/// A structural check proves only that the *text* of the fix is present, not that every code
/// path reaches it -- read the diff, not just this test's PASS, when touching these functions
/// again.
#[ test ]
fn mesh_update_and_draw_return_err_instead_of_panicking_when_mesh_not_created()
{
  for ( signature, name ) in
  [
    ( "pub fn mesh_update( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >", "mesh_update" ),
    ( "pub fn draw( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >", "draw" ),
  ]
  {
    let body = d2_line_fn_body( signature );

    assert!
    (
      !body.contains( ".expect( \"Mesh has not been created yet\" )" ),
      "d2::Line::{name}() must not panic via `.expect( \"Mesh has not been created yet\" )` -- \
      found the panicking form still present in its body"
    );
    assert!
    (
      body.contains( ".ok_or( gl::WebglError::Other( \"Mesh has not been created yet\" ) )?" ),
      "d2::Line::{name}() must return `Err` via `.ok_or( gl::WebglError::Other( .. ) )?` instead \
      of panicking when `self.mesh` is `None` -- expected form not found in its body"
    );
  }
}
