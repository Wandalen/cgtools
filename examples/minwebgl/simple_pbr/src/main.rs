//! # Uniforms And Animation Example with UBOs
//!
//! This program demonstrates how to render a triangle in the middle of the screen using WebGL in Rust. It utilizes shaders with Uniform Block Objects (UBOs) to manage uniforms efficiently.

use minwebgl as gl;
use gl::{ GL };

fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shader source code
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;

  let resolution_loc = gl.get_uniform_location( &program, "u_resolution" );
  let metallic_loc = gl.get_uniform_location( &program, "u_metallic" );
  let roughness_loc = gl.get_uniform_location( &program, "u_roughness" );
  let reflactance_loc = gl.get_uniform_location( &program, "u_reflectance" );
  let base_color_loc = gl.get_uniform_location( &program, "u_base_color" );
  let time_loc = gl.get_uniform_location( &program, "u_time" );

  gl.uniform1f( metallic_loc.as_ref(), 0.0 );
  gl.uniform1f( roughness_loc.as_ref(), 0.5 ); // 0.027 - minimum value;
  gl.uniform1f( reflactance_loc.as_ref(), 2.0 );
  // gl.uniform3f( base_color_loc.as_ref(), 0.562, 0.565, 0.578 ); // iron
  // gl.uniform3f( base_color_loc.as_ref(), 1.022, 0.782, 0.344 ); // gold
  gl.uniform3f( base_color_loc.as_ref(), 0.673, 0.637, 0.585 ); // platinum


  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      gl.uniform1f( time_loc.as_ref(), t as f32 );
      gl.uniform2f( resolution_loc.as_ref(), width, height );
      // Draw points
      gl.draw_arrays( GL::TRIANGLE_STRIP, 0, 4 );
      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );
  Ok( () )
}

fn main()
{
  app_run().unwrap();
}

#[ cfg( test ) ]
mod tests
{
  /// ## Root Cause
  /// The `i == 0` branch of `shader.frag`'s light-rotation animation used `lightDir[ i ].xy *=
  /// rot( time ) * lightDir[ i ].xy` -- a compound multiply-assign -- while the
  /// structurally-parallel `i == 1` and `i == 2` branches both use a plain assignment
  /// ( `lightDir[ i ].xz = rot( time ) * lightDir[ i ].xz`, `lightDir[ i ].yz = rot( time ) *
  /// lightDir[ i ].yz` ). `*=` computes `new = old * ( rotation_matrix * old )` -- an
  /// element-wise self-multiplication by the rotated vector -- instead of `new = rotation_matrix *
  /// old`, so light 0 never actually rotates; its direction is corrupted every frame instead ( the
  /// result isn't even magnitude-preserving, unlike a real rotation ).
  ///
  /// ## Why Not Caught
  /// This crate has no test file -- it's a `fn main()`-only WebGL demo binary verified by eye in
  /// a browser. With three animated, differently-colored lights orbiting on different axes, one
  /// light moving on a wrong, non-rotating trajectory is easy to mistake for an intentional,
  /// idiosyncratic animation path rather than a broken one, especially since it still visibly
  /// changes every frame ( `time` still varies the corrupted result ) rather than freezing.
  ///
  /// ## Fix Applied
  /// Changed the `i == 0` branch's `*=` to `=` in `shader.frag`, matching the other two branches
  /// exactly.
  ///
  /// ## Prevention
  /// `test_light_rotation_branches_all_use_plain_assignment` parses the shader source and
  /// asserts the `i==0` branch uses plain assignment and not compound-multiply-assign, rather
  /// than only checking that the shader compiles.
  ///
  /// ## Pitfall
  /// Three structurally-parallel branches sharing the same right-hand-side expression shape are
  /// exactly where a single stray compound-assignment operator hides -- a reviewer's eye tends
  /// to pattern-match the overall shape of each branch and can skip over a one-character
  /// operator difference. Diff sibling branches token-by-token, not just by silhouette.
  // Fix(BUG-XXX-E): reproducer for the light-0 rotation branch using `*=` instead of `=`.
  // test_kind: bug_reproducer(BUG-XXX-E)
  #[ test ]
  fn test_light_rotation_branches_all_use_plain_assignment()
  {
    let shader_src = include_str!( "../shaders/shader.frag" );

    assert!
    (
      shader_src.contains( "lightDir[ i ].xy = rot( time ) * lightDir[ i ].xy;" ),
      "the i==0 light-rotation branch must plainly assign the rotated vector, matching the i==1/i==2 branches"
    );
    assert!
    (
      !shader_src.contains( "lightDir[ i ].xy *= rot( time ) * lightDir[ i ].xy;" ),
      "the i==0 light-rotation branch must not compound-multiply-assign -- that corrupts the vector instead of rotating it"
    );
  }

  /// Confirms the other two branches ( already correct before this fix ) stay in their plain
  /// assignment form, so this suite can't be satisfied by "fixing" i==0 into matching a
  /// compound-assign bug moved elsewhere instead of actually fixing it.
  #[ test ]
  fn test_sibling_light_rotation_branches_use_plain_assignment()
  {
    let shader_src = include_str!( "../shaders/shader.frag" );

    assert!( shader_src.contains( "lightDir[ i ].xz = rot( time ) * lightDir[ i ].xz;" ) );
    assert!( shader_src.contains( "lightDir[ i ].yz = rot( time ) * lightDir[ i ].yz;" ) );
  }
}
