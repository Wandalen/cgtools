use super::*;

pub struct Enrich;

impl Filter for Enrich
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
      precision mediump float;

      in vec2 v_tex_coord;
      out vec4 frag_color;

      uniform sampler2D u_image;
      uniform vec2 u_texel_size;

      vec3 apply_3x3_kernel( float kernel[ 9 ] )
      {
        const vec2 OFFSETS[] = vec2[]
        (
          vec2( -1.0,  1.0 ), vec2( 0.0,  1.0 ), vec2( 1.0,  1.0 ),
          vec2( -1.0,  0.0 ), vec2( 0.0,  0.0 ), vec2( 1.0,  0.0 ),
          vec2( -1.0, -1.0 ), vec2( 0.0, -1.0 ), vec2( 1.0, -1.0 )
        );

        vec4 sum = vec4( 0.0 );
        for ( int i = 0; i < 9; i++ )
        {
          vec2 offset = v_tex_coord + OFFSETS[ i ] * u_texel_size;
          vec4 pixel = texture( u_image, offset );
          sum += pixel * kernel[ i ];
        }

        return sum.rgb;
      }

      void main()
      {
        const float ENRICH_KERNEL[ 9 ] = float[ 9 ]
        (
           0.0, -2.0,  0.0,
          -2.0, 20.0, -2.0,
           0.0, -2.0,  0.0
        );

        float alpha = texture( u_image, v_tex_coord ).a;
        frag_color = vec4( apply_3x3_kernel( ENRICH_KERNEL ) / 10.0 - 0.15625, alpha );
      }
      ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let texel_size = [ 1.0 / gl.drawing_buffer_width() as f32, 1.0 / gl.drawing_buffer_height() as f32 ];
    let texel_size_location = gl.get_uniform_location( renderer.get_program(), "u_texel_size" );
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, texel_size_location, texel_size.as_slice() ).unwrap();
    default_render_pass( renderer );
  }
}
