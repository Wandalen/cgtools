use super::*;
use serde::{ Serialize, Deserialize };

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Oil
{
  pub levels : i32,
  pub range : i32
}

impl Filter for Oil
{
  // The algorithm was taken from http://www.arahaya.com/imagefilters/
  fn glsl_fragment_source( &self ) -> String
  {
    format!
    (
      "#version 300 es
      #define OIL_LEVELS {}
      precision mediump float;

      in vec2 v_tex_coord;
      out vec4 frag_color;

      uniform sampler2D u_image;
      uniform ivec2 u_resolution;
      uniform vec2 u_texel_size;
      uniform int u_oil_range;

      void main()
      {{
        float rh[ OIL_LEVELS ];
        float gh[ OIL_LEVELS ];
        float bh[ OIL_LEVELS ];
        float rt[ OIL_LEVELS ];
        float gt[ OIL_LEVELS ];
        float bt[ OIL_LEVELS ];

        for( int i = 0; i < OIL_LEVELS; i++ )
        {{
          rh[ i ] = 0.0;
          gh[ i ] = 0.0;
          bh[ i ] = 0.0;
          rt[ i ] = 0.0;
          gt[ i ] = 0.0;
          bt[ i ] = 0.0;
        }}

        int width = u_resolution.x;
        int height = u_resolution.y;
        int x = int( gl_FragCoord.x );
        int y = int( gl_FragCoord.y );

        for( int row = -u_oil_range; row <= u_oil_range; row++ )
        {{
          int row_index = y + row;
          if( row_index < 0 || row_index >= height )
          {{
            continue;
          }}

          for( int col = -u_oil_range; col <= u_oil_range; col++ )
          {{
            int col_index = x + col;
            if( col_index < 0 || col_index >= width )
            {{
              continue;
            }}

            vec2 tex_coord = vec2( col_index, row_index ) * u_texel_size;
            vec3 source_rgb = texture( u_image, tex_coord ).rgb;
            ivec3 rgb_i = ( ivec3( source_rgb * 255.0 ) * OIL_LEVELS ) >> 8;

            rt[ rgb_i.r ] += source_rgb.r;
            gt[ rgb_i.g ] += source_rgb.g;
            bt[ rgb_i.b ] += source_rgb.b;

            rh[ rgb_i.r ] += 1.0;
            gh[ rgb_i.g ] += 1.0;
            bh[ rgb_i.b ] += 1.0;
          }}
        }}

        int r = 0;
        int g = 0;
        int b = 0;
        for( int i = 1; i < OIL_LEVELS; i += 1 )
        {{
          if( rh[ i ] > rh[ r ] )
          {{
            r = i;
          }}
          if( gh[ i ] > gh[ g ])
          {{
            g = i;
          }}
          if( bh[ i ] > bh[ b ] )
          {{
            b = i;
          }}
        }}

        float alpha = texture( u_image, v_tex_coord ).a;
        frag_color = vec4( rt[ r ] / rh[ r ], gt[ g ] / gh[ g ], bt[ b ] / bh[ b ], alpha );
      }}
      ",
      self.levels
    )
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let resolution_location = gl.get_uniform_location( renderer.get_program(), "u_resolution" );
    let texel_size_location = gl.get_uniform_location( renderer.get_program(), "u_texel_size" );
    let oil_range_location = gl.get_uniform_location( renderer.get_program(), "u_oil_range" );
    gl.use_program( Some( &renderer.get_program() ) );

    let resolution = [ gl.drawing_buffer_width(), gl.drawing_buffer_height() ];
    let texel_size = [ 1.0 / gl.drawing_buffer_width() as f32, 1.0 / gl.drawing_buffer_height() as f32 ];

    gl::uniform::upload( gl, resolution_location, resolution.as_slice() ).unwrap();
    gl::uniform::upload( gl, texel_size_location, texel_size.as_slice() ).unwrap();
    gl::uniform::upload( gl, oil_range_location, &self.range ).unwrap();

    default_render_pass( renderer );
  }
}
