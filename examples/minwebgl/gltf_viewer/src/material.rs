use std::collections::HashSet;

use minwebgl as gl;
use gl::GL;

use crate::texture::Texture;


#[ derive( Clone ) ]
pub struct Material
{

}

impl Material 
{
  pub async fn new< 'a >
  ( 
    m : gltf::Material< 'a >,
    textures : &[ Texture ]
  ) -> Self
  {
    let vs_defines = String::from( "#version 300 es\n" );
    let fs_defines = String::from( "#version 300 es\n" );

    let pbr = m.pbr_metallic_roughness();


    let result = Material{};
    return result;
  }
}