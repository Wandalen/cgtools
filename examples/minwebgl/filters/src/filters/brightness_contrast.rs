use super::*;
use serde::{ Serialize, Deserialize };

pub struct Photoshop;
pub struct GIMP;

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct BrightnessContrast< T >
{
  pub brightness : f32,
  pub contrast : f32,
  _marker : std::marker::PhantomData< T >
}

impl< T > BrightnessContrast< T >
{
  pub fn new( brightness : f32, contrast : f32, _impl : T ) -> Self
  {
    Self { brightness, contrast, _marker: std::marker::PhantomData }
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let brightness_location = gl.get_uniform_location( renderer.get_program(), "u_brightness" );
    let contrast_location = gl.get_uniform_location( renderer.get_program(), "u_contrast" );
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, brightness_location, &self.brightness ).unwrap();
    gl::uniform::upload( gl, contrast_location, &self.contrast ).unwrap();
    default_render_pass( renderer );
  }
}

impl Filter for BrightnessContrast< Photoshop >
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform float u_brightness;
    uniform float u_contrast;

    void main()
    {
      vec4 pixel = texture( u_image, v_tex_coord );
      float brightness = ( u_brightness + 1.0 );
      float conrast = ( u_contrast + 1.0 );

      pixel.rgb *= brightness;
      pixel.rgb = ( pixel.rgb - vec3( 0.5 ) ) * conrast + 0.5;

      frag_color = pixel;
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    self.draw( renderer );
  }
}

impl Filter for BrightnessContrast< GIMP >
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform float u_brightness;
    uniform float u_contrast;

    void main()
    {
      const float PI4 = 0.78539816339;
      vec4 pixel = texture( u_image, v_tex_coord );
      float brightness = u_brightness / 100.0;
      float contrast = u_contrast * 0.99 / 100.0;
      contrast = tan( ( contrast + 1.0 ) * PI4 );
      float avg = 0.5;

      if ( brightness < 0.0 )
      {
        pixel.rgb *= 1.0 + brightness;
      }
      else if ( brightness > 0.0 )
      {
        pixel.rgb += ( ( vec3( 1.0 ) - pixel.rgb ) * brightness );
      }

      if ( contrast != 0.0 )
      {
        pixel.rgb = ( pixel.rgb - avg ) * contrast + avg;
      }

      frag_color = pixel;
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    self.draw( renderer );
  }
}

