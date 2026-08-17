mod private
{
  use minwebgl as gl;
  use crate::webgl::IBL;
  use crate::webgl::loaders::hdr_texture;
  use std::ops::Range;

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
  pub async fn load( gl : &gl::WebGl2RenderingContext, path : &str, mip_range : Option< Range<u32> > ) -> IBL
  {
    // Asynchronously loads an HDR image and uploads it to a single mipmap level of a WebGL cube map texture.
    let load_cube = async | name, mip_level, texture : Option< &gl::web_sys::WebGlTexture > |
    {
      let file_path = format!( "{path}/{name}.hdr" );
      hdr_texture::load_to_mip_cube( gl, texture, mip_level, &file_path ).await;
    };

    // Asynchronously loads an HDR image and uploads it to a single mipmap level of a WebGL 2D texture.
    let load_d2 = async | name, mip_level, texture : Option< &gl::web_sys::WebGlTexture > |
    {
      let file_path = format!( "{path}/{name}.hdr" );
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

    ibl_texture_parameters_apply
    (
      gl,
      specular_1_texture.as_ref(),
      specular_2_texture.as_ref(),
      diffuse_texture.as_ref(),
      mip_range
    );

    IBL
    {
      diffuse_texture,
      specular_1_texture,
      specular_2_texture,
      num_mips : 10,
    }
  }

  /// Applies texture filtering parameters to the three IBL textures, plus the specular
  /// pre-filtered environment map's mip-range clamp. Pulled out of `load` so the filter/
  /// mip-range wiring is unit-testable independent of any HDR file I/O.
  ///
  /// `mip_range`, when present, is applied to `specular_1_texture` -- the only one of the three
  /// IBL textures with a real multi-level mip chain ( see [`IBL`]'s own `num_mips` doc comment:
  /// "Number of mip levels in specular_1_texture" ). `diffuse_texture` and `specular_2_texture`
  /// each carry exactly one level, so a base/max mip-level clamp on either of them would be
  /// meaningless.
  pub fn ibl_texture_parameters_apply
  (
    gl : &gl::WebGl2RenderingContext,
    specular_1_texture : Option< &gl::web_sys::WebGlTexture >,
    specular_2_texture : Option< &gl::web_sys::WebGlTexture >,
    diffuse_texture : Option< &gl::web_sys::WebGlTexture >,
    mip_range : Option< Range< u32 > >
  )
  {
    gl.bind_texture( gl::TEXTURE_CUBE_MAP, specular_1_texture );
    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32 );
    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );
    // Fix(BUG-260): `mip_range` was previously applied after an intervening rebind to
    // `diffuse_texture`, so `TEXTURE_BASE_LEVEL`/`TEXTURE_MAX_LEVEL` silently landed on the
    // wrong texture -- `diffuse_texture` has exactly one mip level ( clamping its range is
    // meaningless ), while `specular_1_texture` ( the texture actually carrying the 10-level
    // chain `IBL::num_mips` documents ) never received the clamp at all.
    // Root cause: the filter-setup block bound 3 different textures to the single global
    // `TEXTURE_CUBE_MAP` binding point in sequence, and the `mip_range` block sat after the
    // *last* rebind ( to `diffuse_texture` ) instead of staying adjacent to the bind of the
    // texture it was actually meant to configure ( `specular_1_texture`, bound first ).
    // Pitfall: WebGL's `bind_texture`/`tex_parameteri` pair operates on whichever texture is
    // *currently* bound to the target -- any `tex_parameteri` call must stay textually adjacent
    // to the `bind_texture` call for the texture it is meant to configure, especially once more
    // than one texture shares the same binding point within one function.
    if let Some( mip_range ) = mip_range
    {
      gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_BASE_LEVEL, mip_range.start as i32 );
      gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAX_LEVEL, mip_range.end as i32 );
    }

    gl.bind_texture( gl::TEXTURE_2D, specular_2_texture );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );

    gl.bind_texture( gl::TEXTURE_CUBE_MAP, diffuse_texture );
    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );

    gl.bind_texture( gl::TEXTURE_CUBE_MAP, None );
  }
}

crate::mod_interface!
{
  own use
  {
    load,
    ibl_texture_parameters_apply
  };
}
