use minwebgl::{ self as gl };
use gl::GL;

use crate::sampler::Sampler;

pub struct Texture
{
  source : gl::web_sys::WebGlTexture,
  sampler : Sampler
}

impl Texture
{
  pub fn new
  ( 
    images : &[ gl::web_sys::WebGlTexture ], 
    t : gltf::Texture, 
  ) -> Self
  {
    let source = images[ t.source().index() ].clone();
    let sampler = Sampler::new( &t.sampler() );

    Self
    {
      source,
      sampler
    }
  }

  pub fn apply( &self, gl : &gl::WebGl2RenderingContext )
  {
    self.bind( gl );
    self.sampler.apply( gl );
  }

  pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
  {
    gl.bind_texture( gl::TEXTURE_2D, Some( &self.source ) );
  }
}