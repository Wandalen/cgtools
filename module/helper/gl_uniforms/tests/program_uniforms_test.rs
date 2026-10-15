//! Live WebGL2-context tests for `ProgramUniforms` -- confirms `.upload()`/`.matrix_upload()`
//! complete without panicking against a real compiled+linked `WebGlProgram`, for both a uniform
//! name present in the shader and one absent from it. `gl.uniform*` treats a `None` location as
//! a silent no-op per the WebGL spec, so an absent name is not itself a failure condition -- see
//! `minwebgl::uniform::float32`'s `UniformUpload for f32` impl, which always returns `Ok(())`
//! regardless of whether a location was found.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Browser, not Node: every test here needs a real WebGL2 context.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;
  use gl_uniforms::ProgramUniforms;

  const VERTEX_SHADER : &str = "#version 300 es

void main()
{
  gl_Position = vec4( 0.0, 0.0, 0.0, 1.0 );
}
";

  // Every uniform below feeds `frag_color` so the compiler can't optimize its location away --
  // an unread uniform is free to come back with a `None` location on some drivers, which would
  // make the "found" and "absent" test cases below indistinguishable.
  const FRAGMENT_SHADER : &str = "#version 300 es
precision mediump float;

uniform float uScale;
uniform vec3 uColor;
uniform mat4 uMvp;

out vec4 frag_color;

void main()
{
  frag_color = vec4( uColor * uScale, 1.0 ) * uMvp[ 0 ][ 0 ];
}
";

  /// Creates a headless WebGL2 context and a linked program exposing `uScale` (float),
  /// `uColor` (vec3), and `uMvp` (mat4) -- enough surface to exercise both `upload` and
  /// `matrix_upload` against real GL uniform locations, plus a not-found lookup.
  fn program_make() -> ( GL, gl::WebGlProgram )
  {
    gl::browser::setup( gl::browser::Config::default() );
    let canvas = gl::canvas::make().unwrap();
    let gl_context = gl::context::from_canvas_with( &canvas, gl::context::ContextOptions::default() ).unwrap();
    let program = gl::ProgramFromSources::new( VERTEX_SHADER, FRAGMENT_SHADER )
    .compile_and_link( &gl_context )
    .expect( "fixture vertex/fragment shaders must compile and link" );
    gl_context.use_program( Some( &program ) );

    ( gl_context, program )
  }

  #[ wasm_bindgen_test ]
  fn upload_scalar_to_a_present_uniform_does_not_panic()
  {
    let ( gl_context, program ) = program_make();
    let uniforms = ProgramUniforms::new( &gl_context, &program );

    uniforms.upload( "uScale", &1.5f32 );
  }

  #[ wasm_bindgen_test ]
  fn upload_vector_to_a_present_uniform_does_not_panic()
  {
    let ( gl_context, program ) = program_make();
    let uniforms = ProgramUniforms::new( &gl_context, &program );
    let color : [ f32 ; 3 ] = [ 1.0, 0.5, 0.25 ];

    uniforms.upload( "uColor", &color );
  }

  #[ wasm_bindgen_test ]
  fn matrix_upload_to_a_present_uniform_does_not_panic()
  {
    let ( gl_context, program ) = program_make();
    let uniforms = ProgramUniforms::new( &gl_context, &program );
    let identity : [ f32 ; 16 ] =
    [
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0,
    ];

    uniforms.matrix_upload( "uMvp", &identity, true );
  }

  /// `get_uniform_location` returns `None` for a name absent from the linked program -- per the
  /// WebGL spec, every `gl.uniform*` call silently ignores a `None`/`null` location rather than
  /// erroring, so `upload` must complete (not panic) exactly like the present-uniform cases above.
  #[ wasm_bindgen_test ]
  fn upload_to_an_absent_uniform_name_does_not_panic()
  {
    let ( gl_context, program ) = program_make();
    let uniforms = ProgramUniforms::new( &gl_context, &program );

    uniforms.upload( "uDoesNotExist", &1.0f32 );
  }
}
