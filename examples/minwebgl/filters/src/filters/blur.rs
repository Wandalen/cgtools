#![ allow( clippy::unused_self ) ]

use super::*;
use serde::{ Serialize, Deserialize };

#[ derive( Clone ) ]
pub struct Box;
#[ derive( Clone ) ]
pub struct Gaussian;
#[ derive( Clone ) ]
pub struct Stack;

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Blur< T >
{
  pub size : i32,
  _marker : std::marker::PhantomData< T >,
}

impl< T > Blur< T >
{
  pub fn new( size : i32, _impl : T ) -> Self
  {
    Self { size, _marker: std::marker::PhantomData }
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let texel_size = [ 1.0 / gl.drawing_buffer_width() as f32, 1.0 / gl.drawing_buffer_height() as f32 ];

    let direction_location = gl.get_uniform_location( renderer.get_program(), "u_direction" );
    let texel_size_location = gl.get_uniform_location( renderer.get_program(), "u_texel_size" );
    gl::uniform::upload( gl, texel_size_location, texel_size.as_slice() ).unwrap();
    gl::uniform::upload( gl, direction_location.clone(), [ 1.0, 0.0 ].as_slice() ).unwrap();

    gl.viewport( 0, 0, gl.drawing_buffer_width(), gl.drawing_buffer_height() );
    gl.bind_texture
    (
      GL::TEXTURE_2D,
      renderer.get_image_texture()
    );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR_MIPMAP_LINEAR as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32 );
    gl.bind_framebuffer
    (
      GL::FRAMEBUFFER,
      Some( renderer.get_framebuffer().framebuffer() )
    );
    renderer.draw();

    gl::uniform::upload( gl, direction_location, [ 0.0, 1.0 ].as_slice() ).unwrap();
    gl.bind_texture
    (
      GL::TEXTURE_2D,
      Some( renderer.get_framebuffer().color_attachment() )
    );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32 );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    renderer.draw();
  }
}

impl Filter for Blur< Box >
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform vec2 u_texel_size;
    uniform vec2 u_direction;
    uniform int u_box_size;

    void main()
    {
      vec4 sum = vec4( 0.0 );
      for ( int i = 0; i < u_box_size; i++ )
      {
        vec2 tc = v_tex_coord + u_direction * float( i - u_box_size / 2 ) * u_texel_size;
        sum += texture( u_image, tc );
      }

      frag_color = sum / float( u_box_size );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();

    let box_size_location = gl.get_uniform_location( renderer.get_program(), "u_box_size" );
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, box_size_location, &self.size ).unwrap();

    self.draw( renderer );
  }
}

impl Filter for Blur< Gaussian >
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform int u_sigma;
    uniform vec2 u_texel_size;
    uniform vec2 u_direction;

    float gaussian_kernel( int i, float sigma )
    {
      float coeff = 1.0 / ( sqrt( 2.0 * 3.14159265359 ) * sigma );
      float exponent = -float( i * i ) / ( 2.0 * sigma * sigma );
      return coeff * exp( exponent );
    }

    void main()
    {
      float alpha = texture( u_image, v_tex_coord ).a;
      vec3 sum = vec3( 0.0 );

      int kernel_size = u_sigma * 6 + 1;
      int half_size = kernel_size / 2;
      for ( int i = -half_size; i <= half_size; i++ )
      {
        vec2 tc = v_tex_coord + u_direction * float( i ) * u_texel_size;
        sum += gaussian_kernel( i, float( u_sigma ) ) * texture( u_image, tc ).rgb;
      }

      frag_color = vec4( sum, alpha );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();

    let sigma_location = gl.get_uniform_location( renderer.get_program(), "u_sigma" );
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, sigma_location, &self.size ).unwrap();

    self.draw( renderer );
  }
}

impl Filter for Blur< Stack >
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform int u_radius;
    uniform vec2 u_texel_size;
    uniform vec2 u_direction;

    void main()
    {
      vec4 sum = vec4( 0.0 );
      for ( int i = -u_radius; i <= u_radius; i++ )
      {
        vec2 tc = v_tex_coord + u_direction * float( i ) * u_texel_size;
        sum += texture( u_image, tc );
      }

      frag_color = sum / float( ( u_radius * 2 ) + 1 );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();

    let radius_location = gl.get_uniform_location( renderer.get_program(), "u_radius" );
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, radius_location, &self.size ).unwrap();

    self.draw( renderer );
  }
}
