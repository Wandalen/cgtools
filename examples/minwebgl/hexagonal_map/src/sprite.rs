use minwebgl as gl;
use gl::{ U32x2, GL };
use web_sys::WebGlTexture;

pub struct Sprite
{
  size : U32x2,
  image : WebGlTexture,
  width_over_height : f32,
}

impl Sprite
{
  pub fn new( size : U32x2, image : WebGlTexture ) -> Self
  {
    let width_over_height = size[ 0 ] as f32 / size[ 1 ] as f32;
    Self { size, image, width_over_height }
  }

  pub fn bind( &self, gl : &GL )
  {
    gl.bind_texture( GL::TEXTURE_2D, Some( &self.image ) );
  }

  pub fn draw( &self, gl : &GL )
  {
    gl.draw_arrays( GL::TRIANGLE_STRIP, 0, 4 );
  }
}

pub fn sprite_shader( gl : &GL ) -> Result< gl::shader::Program, minwebgl::WebglError >
{
  let vert = include_str!( "../shaders/sprite.vert" );
  let frag = include_str!( "../shaders/sprite.frag" );
  gl::shader::Program::new( gl.clone(), vert, frag )
}
