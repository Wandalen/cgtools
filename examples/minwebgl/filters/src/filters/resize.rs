#![ allow( clippy::unused_self ) ]

use super::*;
use std::marker::PhantomData;
use serde::{ Serialize, Deserialize };

#[ derive( Clone ) ]
pub struct Bilinear;
#[ derive( Clone ) ]
pub struct Nearest;

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Resize< T >
{
  pub scale : f32,
  _marker : std::marker::PhantomData< T >
}

impl< T > Resize< T >
{
  pub fn new( scale : f32, _impl : T ) -> Self
  {
    Self { scale, _marker: PhantomData }
  }

  fn glsl_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform vec2 u_resize_scale;

    void main()
    {
      vec2 tc = vec2( v_tex_coord.x / u_resize_scale.x, v_tex_coord.y / u_resize_scale.y );
      if ( tc.x > 1.0 || tc.y > 1.0 )
      {
        discard;
      }
      frag_color = texture( u_image, tc );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer, filter : u32 )
  {
    let gl = renderer.gl();

    let scale_location = gl.get_uniform_location( renderer.get_program(), "u_resize_scale" );
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, scale_location, [ self.scale, self.scale ].as_slice() ).unwrap();

    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, renderer.get_image_texture() );

    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, filter as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, filter as i32 );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    gl.viewport( 0, 0, gl.drawing_buffer_width(), gl.drawing_buffer_height() );

    // Clear with transparent color before drawing resized image
    gl.clear_color( 0.0, 0.0, 0.0, 0.0 );
    gl.clear( GL::COLOR_BUFFER_BIT );

    renderer.draw();
  }
}

impl Filter for Resize< Bilinear >
{
  fn glsl_fragment_source( &self ) -> String
  {
    self.glsl_source()

  }
  fn draw( &self, renderer : &impl FilterRenderer )
  {
    self.draw( renderer, GL::LINEAR );
  }
}

impl Filter for Resize< Nearest >
{
  fn glsl_fragment_source( &self ) -> String
  {
    self.glsl_source()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    self.draw( renderer, GL::NEAREST );
  }
}
