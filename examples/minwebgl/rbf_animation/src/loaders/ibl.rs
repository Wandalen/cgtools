use minwebgl as gl;
use renderer::webgl::IBL;
use super::hdr_texture;

pub async fn load( gl : &gl::WebGl2RenderingContext, path : &str ) -> IBL
{
  let load_cube = async | name, mip_level, texture : Option< &gl::web_sys::WebGlTexture > |
  {
    let file_path = format!( "{}/{}.hdr", path, name );
    hdr_texture::load_to_mip_cube( gl, texture, mip_level, &file_path ).await;
  };

  let load_d2 = async | name, mip_level, texture : Option< &gl::web_sys::WebGlTexture > |
  {
    let file_path = format!( "{}/{}.hdr", path, name );
    hdr_texture::load_to_mip_d2( gl, texture, mip_level, &file_path ).await;
  };
  
  let diffuse_texture = gl.create_texture();
  let specular_1_texture = gl.create_texture();
  let specular_2_texture = gl.create_texture();

  load_cube( "diffuse", 0, diffuse_texture.as_ref() ).await;
  load_cube( "specular_1_0", 0, specular_1_texture.as_ref() ).await;
  load_cube( "specular_1_1", 1, specular_1_texture.as_ref() ).await;
  load_cube( "specular_1_2", 2, specular_1_texture.as_ref() ).await;
  load_cube( "specular_1_3", 3, specular_1_texture.as_ref() ).await;
  load_cube( "specular_1_4", 4, specular_1_texture.as_ref() ).await;
  load_cube( "specular_1_5", 5, specular_1_texture.as_ref() ).await;
  load_cube( "specular_1_6", 6, specular_1_texture.as_ref() ).await;
  load_cube( "specular_1_7", 7, specular_1_texture.as_ref() ).await;
  load_cube( "specular_1_8", 8, specular_1_texture.as_ref() ).await;
  load_cube( "specular_1_9", 9, specular_1_texture.as_ref() ).await;
  load_d2( "specular_2", 0, specular_2_texture.as_ref() ).await;

  gl.bind_texture( gl::TEXTURE_CUBE_MAP, specular_1_texture.as_ref() );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32 );
  gl.bind_texture( gl::TEXTURE_CUBE_MAP, None );

  IBL
  {
    diffuse_texture,
    specular_1_texture,
    specular_2_texture
  }
}