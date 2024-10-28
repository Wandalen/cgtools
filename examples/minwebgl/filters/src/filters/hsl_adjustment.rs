use super::*;
use serde::{ Serialize, Deserialize };

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct HSLAdjustment
{
  pub hue : f32,
  pub saturation : f32,
  pub lightness : f32,
}

impl Filter for HSLAdjustment
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform vec3 u_hsl;

    vec3 rgb2hsl( vec3 color )
    {
      float max = max( color.r, max( color.g, color.b ) );
      float min = min( color.r, min( color.g, color.b ) );
      float lightness = ( max + min ) / 2.0;
      float hue = 0.0;
      float saturation = 0.0;

      if ( max != min )
      {
        float delta = max - min;
        saturation = lightness > 0.5 ? delta / ( 2.0 - max - min ) : delta / ( max + min );
        if ( max == color.r )
        {
          hue = ( color.g - color.b ) / delta + ( color.g < color.b ? 6.0 : 0.0 );
        }
        else if ( max == color.g )
        {
          hue = ( color.b - color.r ) / delta + 2.0;
        }
        else
        {
          hue = ( color.r - color.g ) / delta + 4.0;
        }
        hue /= 6.0;
      }

      return vec3( hue, saturation, lightness );
    }

    float hue2rgb( float p, float q, float t )
    {
      if ( t < 0.0 ) t += 1.0;
      if ( t > 1.0 ) t -= 1.0;
      if ( t < 1.0 / 6.0 ) return p + ( q - p ) * 6.0 * t;
      if ( t < 1.0 / 2.0 ) return q;
      if ( t < 2.0 / 3.0 ) return p + ( q - p ) * ( 2.0 / 3.0 - t ) * 6.0;
      return p;
    }

    vec3 hsl2rgb( vec3 hsl )
    {
      float hue = hsl.x;
      float saturation = hsl.y;
      float lightness = hsl.z;

      if ( saturation == 0.0 )
      {
        return vec3( lightness );
      }

      float q = lightness < 0.5 ? lightness * ( 1.0 + saturation ) : lightness + saturation - lightness * saturation;
      float p = 2.0 * lightness - q;

      float r = hue2rgb( p, q, hue + 1.0 / 3.0 );
      float g = hue2rgb( p, q, hue );
      float b = hue2rgb( p, q, hue - 1.0 / 3.0 );

      return vec3( r, g, b );
    }

    void main()
    {
      vec4 pixel = texture( u_image, v_tex_coord );
      vec3 hsl = rgb2hsl( pixel.rgb );
      hsl.x += u_hsl.x;
      hsl.y = clamp( hsl.y + u_hsl.y, 0.0, 1.0 );
      hsl.z = clamp( hsl.z + u_hsl.z, 0.0, 1.0 );
      vec3 rgb = hsl2rgb( hsl );
      frag_color = vec4( rgb, pixel.a );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let threshold_location = gl.get_uniform_location( renderer.get_program(), "u_hsl" );
    gl.use_program( Some( &renderer.get_program() ) );
    let hsl = [ self.hue, self.saturation, self.lightness ];
    gl::uniform::upload( gl, threshold_location, hsl.as_slice() ).unwrap();
    default_render_pass( renderer );
  }
}
