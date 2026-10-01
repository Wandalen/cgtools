//! Pixel readback of `main.frag`'s tangent frame without vertex tangents ( `getTBN` ).
//!
//! The function is cut out of the shipped `main.frag` source and run on a full-screen quad whose
//! world position is its clip position ( z = 0, normal +Z ) and whose UVs are an affine map of it,
//! so every column of the frame has a known expected direction. glTF puts the UV origin at the
//! image's upper-left corner and defines tangent space as +X right and +Y up the image, and the
//! loader uploads images unflipped, so +Y is the direction in which v decreases.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;

  const MAIN_FRAGMENT_SHADER : &str = include_str!( "../src/webgl/shaders/main.frag" );

  const VERTEX_SHADER : &str = "#version 300 es
uniform mat3 uvMap;
out vec3 vPos;
out vec2 vUv;
void main()
{
  vec2 corners[ 6 ] = vec2[]( vec2( -1, -1 ), vec2( 1, -1 ), vec2( 1, 1 ), vec2( -1, -1 ), vec2( 1, 1 ), vec2( -1, 1 ) );
  vec2 p = corners[ gl_VertexID ];
  vPos = vec3( p, 0.0 );
  vUv = ( uvMap * vec3( p, 1.0 ) ).xy;
  gl_Position = vec4( p, 0.0, 1.0 );
}
";

  /// Tolerance for a decoded RGBA8 component ( one step is 2 / 255 ).
  const EPS : f32 = 0.02;

  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let options = gl::context::ContextOptions::default().antialias( false );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, options ).unwrap()
  }

  /// `main.frag`'s `getTBN` definition, from its signature to its closing brace.
  fn get_tbn_source() -> &'static str
  {
    let start = MAIN_FRAGMENT_SHADER.find( "mat3 getTBN(" ).expect( "main.frag defines getTBN" );
    let open = start + MAIN_FRAGMENT_SHADER[ start.. ].find( '{' ).expect( "getTBN has a body" );
    let mut depth = 0;
    for ( i, c ) in MAIN_FRAGMENT_SHADER[ open.. ].char_indices()
    {
      match c
      {
        '{' => depth += 1,
        '}' =>
        {
          depth -= 1;
          if depth == 0
          {
            return &MAIN_FRAGMENT_SHADER[ start..=open + i ];
          }
        },
        _ => {},
      }
    }
    panic!( "getTBN has no closing brace" );
  }

  /// Renders column `column` of `getTBN( +Z, pos, uv )` with `uv = uv_map * ( x, y, 1 )` and
  /// returns it decoded from the center pixel. `uv_map` is column-major, as GLSL reads it.
  fn frame_column( gl : &GL, uv_map : [ f32; 9 ], column : i32 ) -> [ f32; 3 ]
  {
    let fs = format!
    (
      "#version 300 es
precision highp float;
in vec3 vPos;
in vec2 vUv;
uniform int column;
out vec4 color;
{}
void main()
{{
  mat3 tbn = getTBN( vec3( 0.0, 0.0, 1.0 ), vPos, vUv );
  color = vec4( tbn[ column ] * 0.5 + 0.5, 1.0 );
}}
",
      get_tbn_source()
    );
    let program = gl::ProgramFromSources::new( VERTEX_SHADER, &fs )
    .compile_and_link( gl )
    .expect( "getTBN test program compiles" );
    gl.use_program( Some( &program ) );
    gl.uniform_matrix3fv_with_f32_array( gl.get_uniform_location( &program, "uvMap" ).as_ref(), false, &uv_map );
    gl.uniform1i( gl.get_uniform_location( &program, "column" ).as_ref(), column );

    let ( width, height ) = ( gl.drawing_buffer_width(), gl.drawing_buffer_height() );
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    gl.viewport( 0, 0, width, height );
    gl.draw_arrays( gl::TRIANGLES, 0, 6 );

    let mut pixel = [ 0_u8; 4 ];
    gl.read_pixels_with_opt_u8_array( width / 2, height / 2, 1, 1, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut pixel ) )
    .expect( "read_pixels succeeds" );
    gl.delete_program( Some( &program ) );
    [ 0, 1, 2 ].map( | i | f32::from( pixel[ i ] ) / 255.0 * 2.0 - 1.0 )
  }

  fn assert_near( actual : [ f32; 3 ], expected : [ f32; 3 ], what : &str )
  {
    let close = actual.iter().zip( expected ).all( | ( a, e ) | ( a - e ).abs() < EPS );
    assert!( close, "{what}: got {actual:?}, expected {expected:?}" );
  }

  /// With u = ( 1 + x ) / 2 and v = ( 1 - y ) / 2 the image is upright on the quad, so the
  /// bitangent must point up the image ( +Y ), as glTF's tangent space does: a frame whose
  /// bitangent follows increasing v inverts every normal map's green channel and mirrors the
  /// anisotropy direction.
  #[ wasm_bindgen_test ]
  fn derivative_frame_bitangent_points_up_the_image()
  {
    let gl = gl_init();
    let upright = [ 0.5, 0.0, 0.0, 0.0, -0.5, 0.0, 0.5, 0.5, 1.0 ];

    assert_near( frame_column( &gl, upright, 0 ), [ 1.0, 0.0, 0.0 ], "tangent" );
    assert_near( frame_column( &gl, upright, 1 ), [ 0.0, 1.0, 0.0 ], "bitangent" );
    assert_near( frame_column( &gl, upright, 2 ), [ 0.0, 0.0, 1.0 ], "normal" );
  }

  /// Mirroring u ( u = ( 1 - x ) / 2 ) flips the tangent and leaves the bitangent up the image:
  /// the frame follows mirrored UVs instead of assuming a right-handed layout.
  #[ wasm_bindgen_test ]
  fn derivative_frame_follows_mirrored_u()
  {
    let gl = gl_init();
    let mirrored = [ -0.5, 0.0, 0.0, 0.0, -0.5, 0.0, 0.5, 0.5, 1.0 ];

    assert_near( frame_column( &gl, mirrored, 0 ), [ -1.0, 0.0, 0.0 ], "tangent" );
    assert_near( frame_column( &gl, mirrored, 1 ), [ 0.0, 1.0, 0.0 ], "bitangent" );
  }

  /// With u = x and v = -2y, v has twice u's texel density. The frame must stay orthonormal:
  /// an anisotropy direction mapped through unequal columns turns 45° into 63.4°.
  #[ wasm_bindgen_test ]
  fn derivative_frame_is_orthonormal_with_unequal_uv_density()
  {
    let gl = gl_init();
    let stretched = [ 1.0, 0.0, 0.0, 0.0, -2.0, 0.0, 0.0, 0.0, 1.0 ];

    assert_near( frame_column( &gl, stretched, 0 ), [ 1.0, 0.0, 0.0 ], "tangent" );
    assert_near( frame_column( &gl, stretched, 1 ), [ 0.0, 1.0, 0.0 ], "bitangent" );
  }

  /// With u = x + y / 2 and v = -y, u increases along +X on the surface, while its gradient
  /// leans 26.6° towards +Y. The tangent must follow the surface direction, as a MikkTSpace
  /// tangent does, or even an anisotropy rotation of 0 renders at the wrong angle.
  #[ wasm_bindgen_test ]
  fn derivative_frame_tangent_follows_u_on_sheared_uvs()
  {
    let gl = gl_init();
    let sheared = [ 1.0, 0.0, 0.0, 0.5, -1.0, 0.0, 0.0, 0.0, 1.0 ];

    assert_near( frame_column( &gl, sheared, 0 ), [ 1.0, 0.0, 0.0 ], "tangent" );
    assert_near( frame_column( &gl, sheared, 1 ), [ 0.0, 1.0, 0.0 ], "bitangent" );
  }

  /// Where the UV set has no usable gradient there is no tangent direction, but the frame must
  /// still be finite: a mesh without TEXCOORD_0 reads a constant `vUv_0`, and palette-style UVs
  /// collapse onto a point or a line. A zero tangent turns the anisotropic highlight and the
  /// bent environment normal into NaN, even at strength 0.
  #[ wasm_bindgen_test ]
  fn derivative_frame_is_finite_without_a_uv_gradient()
  {
    let gl = gl_init();
    let constant = [ 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.3, 0.6, 1.0 ];
    let collinear = [ 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0 ];

    for ( uv_map, label ) in [ ( constant, "constant UVs" ), ( collinear, "UVs on a line" ) ]
    {
      let t = frame_column( &gl, uv_map, 0 );
      let b = frame_column( &gl, uv_map, 1 );
      let length = | v : [ f32; 3 ] | v.iter().map( | c | c * c ).sum::< f32 >().sqrt();
      assert!( ( length( t ) - 1.0 ).abs() < 2.0 * EPS, "{label}: tangent {t:?} must be a unit vector" );
      assert!( ( length( b ) - 1.0 ).abs() < 2.0 * EPS, "{label}: bitangent {b:?} must be a unit vector" );
      assert!( t[ 2 ].abs() < EPS && b[ 2 ].abs() < EPS, "{label}: frame {t:?} / {b:?} must lie around the normal" );
    }
  }
}
