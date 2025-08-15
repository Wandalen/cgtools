mod private
{
  use minwebgl as gl;
  use crate::webgl::IBL;
  use crate::webgl::loaders::hdr_texture;

  /// Asynchronously loads Image-Based Lighting (IBL) textures from a specified directory.
  ///
  /// This function loads a set of HDR textures, including a diffuse irradiance map (a cube map)
  /// and a specular pre-filtered environment map (a cube map with multiple mip levels).
  /// It also loads a 2D texture for specular BRDF lookup. The specular cube map's
  /// minification filter is set to `LINEAR_MIPMAP_LINEAR`.
  ///
  /// # Arguments
  ///
  /// * `gl` - The `WebGl2RenderingContext` used for creating and uploading textures.
  /// * `path` - The base path to the directory containing the IBL HDR files.
  ///
  /// # Returns
  ///
  /// An `IBL` struct containing the loaded WebGL textures.
  pub async fn load( gl : &gl::WebGl2RenderingContext, path : &str ) -> IBL
  {
    // Asynchronously loads an HDR image and uploads it to a single mipmap level of a WebGL cube map texture.
    let load_cube = async | name, mip_level, texture : Option< &gl::web_sys::WebGlTexture > |
    {
      let file_path = format!( "{}/{}.hdr", path, name );
      hdr_texture::load_to_mip_cube( gl, texture, mip_level, &file_path ).await;
    };

    // Asynchronously loads an HDR image and uploads it to a single mipmap level of a WebGL 2D texture.
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
}

crate::mod_interface!
{
  own use
  {
    load
  };
}
