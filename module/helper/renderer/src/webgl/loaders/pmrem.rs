mod private
{
  use minwebgl as gl;
  use gl::GL;
  use web_sys::{ WebGlFramebuffer, WebGlProgram, WebGlTexture, WebGlUniformLocation };
  use crate::webgl::
  {
    post_processing::VS_TRIANGLE,
    IBL
  };

  struct ShaderProg< 'a >
  {
    gl : &'a GL,
    program : WebGlProgram,
  }

  impl< 'a > ShaderProg< 'a >
  {
    fn compile( gl : &'a GL, fs_src : &str ) -> Result< Self, gl::WebglError >
    {
      let program = gl::ProgramFromSources::new( VS_TRIANGLE, fs_src ).compile_and_link( gl )?;
      Ok( Self { gl, program } )
    }

    fn bind( &self, gl : &GL )
    {
      gl.use_program( Some( &self.program ) );
    }

    fn loc( &self, gl : &GL, name : &str ) -> Option< WebGlUniformLocation >
    {
      gl.get_uniform_location( &self.program, name )
    }
  }

  // web_sys `WebGlProgram` has no `Drop` freeing the GPU program, so each compiled program
  // must be deleted explicitly. Owning the deletion here keeps `Programs::compile` leak-safe
  // even on partial construction: if a later shader fails to compile, the already-built
  // `ShaderProg` fields are dropped and their programs freed.
  impl Drop for ShaderProg< '_ >
  {
    fn drop( &mut self )
    {
      self.gl.delete_program( Some( &self.program ) );
    }
  }

  struct Programs< 'a >
  {
    equirect_to_cube : ShaderProg< 'a >,
    prefilter : ShaderProg< 'a >,
    irradiance : ShaderProg< 'a >,
    brdf_lut : ShaderProg< 'a >,
  }

  impl< 'a > Programs< 'a >
  {
    fn compile( gl : &'a GL ) -> Result< Self, gl::WebglError >
    {
      Ok( Self
      {
        equirect_to_cube : ShaderProg::compile( gl, include_str!( "../shaders/pmrem/equirect_to_cube.frag" ) )?,
        prefilter : ShaderProg::compile( gl, include_str!( "../shaders/pmrem/prefilter_specular.frag" ) )?,
        irradiance : ShaderProg::compile( gl, include_str!( "../shaders/pmrem/irradiance_convolution.frag" ) )?,
        brdf_lut : ShaderProg::compile( gl, include_str!( "../shaders/pmrem/brdf_integration.frag" ) )?,
      })
    }
  }

  /// RAII guard that deletes its `WebGlTexture` on drop unless [`TextureGuard::release`]d.
  ///
  /// web_sys texture handles have no `Drop` that frees the underlying GPU object, so an
  /// intermediate texture would leak whenever an `?` propagates an error before the texture
  /// can be handed to the caller. The guard frees it on every early return and is disarmed
  /// via `release` once ownership is transferred out on the success path.
  struct TextureGuard< 'a >
  {
    gl : &'a GL,
    texture : Option< WebGlTexture >,
  }

  impl< 'a > TextureGuard< 'a >
  {
    fn new( gl : &'a GL, texture : WebGlTexture ) -> Self
    {
      Self { gl, texture : Some( texture ) }
    }

    fn as_ref( &self ) -> &WebGlTexture
    {
      // The texture is present for the whole armed lifetime; `release` consumes the guard.
      self.texture.as_ref().unwrap()
    }

    /// Disarms the guard and returns ownership of the texture to the caller.
    fn release( mut self ) -> WebGlTexture
    {
      self.texture.take().unwrap()
    }
  }

  impl Drop for TextureGuard< '_ >
  {
    fn drop( &mut self )
    {
      if let Some( texture ) = self.texture.take()
      {
        self.gl.delete_texture( Some( &texture ) );
      }
    }
  }

  /// RAII guard that deletes its framebuffer on drop. Unlike textures the FBO never escapes
  /// `generate`, so there is no disarm path — it is always freed when `generate` returns.
  struct FramebufferGuard< 'a >
  {
    gl : &'a GL,
    framebuffer : WebGlFramebuffer,
  }

  impl< 'a > FramebufferGuard< 'a >
  {
    fn new( gl : &'a GL, framebuffer : WebGlFramebuffer ) -> Self
    {
      Self { gl, framebuffer }
    }

    fn as_ref( &self ) -> &WebGlFramebuffer
    {
      &self.framebuffer
    }
  }

  impl Drop for FramebufferGuard< '_ >
  {
    fn drop( &mut self )
    {
      self.gl.delete_framebuffer( Some( &self.framebuffer ) );
    }
  }

  fn allocate_cubemap( gl : &GL, size : u32, num_mips : i32 ) -> Result< WebGlTexture, gl::WebglError >
  {
    let texture = gl.create_texture().ok_or( gl::WebglError::FailedToAllocateResource( "PMREM cubemap" ) )?;
    gl.bind_texture( gl::TEXTURE_CUBE_MAP, Some( &texture ) );
    gl.tex_storage_2d( gl::TEXTURE_CUBE_MAP, num_mips, gl::RGBA16F, size as i32, size as i32 );
    gl::texture::cube::wrap_clamp( gl );
    Ok( texture )
  }

  fn render_to_cube_face
  (
    gl : &GL,
    texture : &WebGlTexture,
    face : u32,
    mip_level : i32,
    viewport_size : u32,
    face_loc : Option< &WebGlUniformLocation >
  )
  {
    gl.viewport( 0, 0, viewport_size as i32, viewport_size as i32 );
    gl.framebuffer_texture_2d
    (
      gl::FRAMEBUFFER,
      gl::COLOR_ATTACHMENT0,
      gl::TEXTURE_CUBE_MAP_POSITIVE_X + face,
      Some( texture ),
      mip_level
    );
    gl.uniform1i( face_loc, face as i32 );
    gl.clear( gl::COLOR_BUFFER_BIT );
    gl.draw_arrays( gl::TRIANGLES, 0, 3 );
  }

  /// Returns an error if the currently bound draw framebuffer is not complete.
  ///
  /// Logs the GL status code (mirroring the `shadow.rs` check) and fails loudly instead of
  /// letting an incomplete FBO silently render to undefined state — e.g. when
  /// `EXT_color_buffer_float` is unavailable so the RGBA16F attachment is not color-renderable,
  /// or after a context-loss recovery.
  fn check_framebuffer_complete( gl : &GL ) -> Result< (), gl::WebglError >
  {
    let status = gl.check_framebuffer_status( gl::FRAMEBUFFER );
    if status != gl::FRAMEBUFFER_COMPLETE
    {
      gl::browser::error!( "PMREM framebuffer incomplete: {:?}", status );
      return Err( gl::WebglError::Other( "PMREM framebuffer incomplete" ) );
    }
    Ok( () )
  }

  fn equirect_to_cubemap
  (
    gl : &GL,
    programs : &Programs< '_ >,
    equirect : &WebGlTexture,
    size : u32,
    num_mips : i32,
  ) -> Result< WebGlTexture, gl::WebglError >
  {
    let texture = TextureGuard::new( gl, allocate_cubemap( gl, size, num_mips )? );

    programs.equirect_to_cube.bind( gl );
    let face_loc = programs.equirect_to_cube.loc( gl, "face" );

    gl.active_texture( gl::TEXTURE0 );
    gl.bind_texture( gl::TEXTURE_2D, Some( equirect ) );
    gl.uniform1i( programs.equirect_to_cube.loc( gl, "equirectMap" ).as_ref(), 0 );

    for face in 0..6u32
    {
      render_to_cube_face( gl, texture.as_ref(), face, 0, size, face_loc.as_ref() );

      // The whole PMREM pipeline renders into one FBO with a single RGBA16F color attachment, so
      // completeness is governed by that format's renderability — identical for every face, mip
      // and pass. One check right after the first attachment therefore guards the entire pipeline;
      // bail out before the remaining faces and passes render into an unusable FBO.
      if face == 0
      {
        check_framebuffer_complete( gl )?;
      }
    }

    gl.bind_texture( gl::TEXTURE_CUBE_MAP, Some( texture.as_ref() ) );
    gl.generate_mipmap( gl::TEXTURE_CUBE_MAP );
    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32 );
    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );

    Ok( texture.release() )
  }

  fn prefilter_specular
  (
    gl : &GL,
    programs : &Programs< '_ >,
    source_cubemap : &WebGlTexture,
    size : u32,
    num_mips : u32,
  ) -> Result< WebGlTexture, gl::WebglError >
  {
    // Guarded because the `gl::uniform::upload` calls below can fail *after* allocation,
    // which would otherwise leak this cubemap before it is returned to the caller.
    let texture = TextureGuard::new( gl, allocate_cubemap( gl, size, num_mips as i32 )? );

    programs.prefilter.bind( gl );

    gl.active_texture( gl::TEXTURE0 );
    gl.bind_texture( gl::TEXTURE_CUBE_MAP, Some( source_cubemap ) );
    gl.uniform1i( programs.prefilter.loc( gl, "envMap" ).as_ref(), 0 );

    let face_loc = programs.prefilter.loc( gl, "face" );
    let roughness_loc = programs.prefilter.loc( gl, "roughness" );
    let resolution_loc = programs.prefilter.loc( gl, "resolution" );

    // `resolution` drives the LOD heuristic's `saTexel` (solid angle of a source texel), so it
    // must be the *source* cubemap's base resolution — constant across output mips, equal to
    // `size` — not the per-mip output size. Using the output size biases LOD selection toward
    // sharper source mips and adds noise on rough mips.
    gl::uniform::upload( gl, resolution_loc.clone(), &( size as f32 ) )?;

    for mip in 0..num_mips
    {
      let mip_size = size >> mip;
      // `num_mips` is always >= 1 (it is `log2(resolution) + 1`), so a single-mip cubemap
      // would make the denominator `num_mips - 1` zero and yield `0.0 / 0.0 = NaN`, which the
      // `roughness` uniform would propagate into the prefilter shader. A lone mip is the fully
      // sharp level, so its roughness is 0.
      let roughness = if num_mips > 1 { mip as f32 / ( num_mips - 1 ) as f32 } else { 0.0 };

      gl::uniform::upload( gl, roughness_loc.clone(), &roughness )?;

      for face_idx in 0..6u32
      {
        render_to_cube_face( gl, texture.as_ref(), face_idx, mip as i32, mip_size, face_loc.as_ref() );
      }
    }

    gl.bind_texture( gl::TEXTURE_CUBE_MAP, Some( texture.as_ref() ) );
    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32 );
    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );

    Ok( texture.release() )
  }

  fn convolve_irradiance
  (
    gl : &GL,
    programs : &Programs< '_ >,
    source_cubemap : &WebGlTexture,
  ) -> Result< WebGlTexture, gl::WebglError >
  {
    let irradiance_size = 64u32;
    let texture = allocate_cubemap( gl, irradiance_size, 1 )?;

    programs.irradiance.bind( gl );

    gl.active_texture( gl::TEXTURE0 );
    gl.bind_texture( gl::TEXTURE_CUBE_MAP, Some( source_cubemap ) );
    gl.uniform1i( programs.irradiance.loc( gl, "envMap" ).as_ref(), 0 );

    let face_loc = programs.irradiance.loc( gl, "face" );
    for face_idx in 0..6u32
    {
      render_to_cube_face( gl, &texture, face_idx, 0, irradiance_size, face_loc.as_ref() );
    }

    gl.bind_texture( gl::TEXTURE_CUBE_MAP, Some( &texture ) );
    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );

    Ok( texture )
  }

  fn integrate_brdf( gl : &GL, programs : &Programs< '_ > ) -> Result< WebGlTexture, gl::WebglError >
  {
    let lut_size = 512u32;
    let texture = gl.create_texture().ok_or( gl::WebglError::FailedToAllocateResource( "PMREM BRDF LUT" ) )?;
    gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
    gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA16F, lut_size as i32, lut_size as i32 );
    gl::texture::d2::filter_linear( gl );
    gl::texture::d2::wrap_clamp( gl );

    programs.brdf_lut.bind( gl );

    gl.viewport( 0, 0, lut_size as i32, lut_size as i32 );
    gl.framebuffer_texture_2d
    (
      gl::FRAMEBUFFER,
      gl::COLOR_ATTACHMENT0,
      gl::TEXTURE_2D,
      Some( &texture ),
      0
    );
    gl.clear( gl::COLOR_BUFFER_BIT );
    gl.draw_arrays( gl::TRIANGLES, 0, 3 );

    Ok( texture )
  }

  /// Generates a complete IBL set from an equirectangular HDR texture on the GPU.
  ///
  /// Converts the equirect map to cubemaps with GGX importance-sampled specular prefiltering,
  /// cosine-weighted diffuse irradiance convolution, and a split-sum BRDF integration LUT.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL2 rendering context.
  /// * `equirect_texture` - A 2D texture containing an equirectangular HDR image.
  /// * `cubemap_resolution` - Side length of the cubemap faces. Any value >= 1 is accepted:
  ///   WebGL2 fully supports non-power-of-two cubemaps, and the mip count adapts as
  ///   `floor(log2(resolution)) + 1` — exactly GL's maximum, so the per-mip viewport sizes
  ///   (`resolution >> mip`) always match the allocated storage. A power of two is *recommended*
  ///   (512 gives a clean 10-level chain) but not required.
  ///
  /// # GL state
  ///
  /// This is a one-shot loader that drives its own off-screen FBO passes. Following the
  /// renderer-wide convention (every pass sets the state it needs and does not restore it),
  /// it leaves global render state — `DEPTH_TEST`, `BLEND`, `CULL_FACE`, color mask,
  /// clear color, pixel store — modified on return. Framebuffer, texture and program
  /// bindings are reset to `None`. The caller is expected to (re)establish the state it
  /// needs before its next draw; the renderer's per-frame passes already do so.
  ///
  /// # Resource cleanup
  ///
  /// Intermediate GPU resources (the compiled programs, the off-screen FBO and the source
  /// cubemap) are freed on every exit path — success or `?`-propagated error — via RAII
  /// guards. Only the three output textures escape, wrapped in the returned [`IBL`].
  pub fn generate
  (
    gl : &gl::WebGl2RenderingContext,
    equirect_texture : &WebGlTexture,
    cubemap_resolution : u32
  ) -> Result< IBL, gl::WebglError >
  {
    let num_mips = ( cubemap_resolution as f32 ).log2() as u32 + 1;

    let programs = Programs::compile( gl )?;

    let fbo = FramebufferGuard::new
    (
      gl,
      gl.create_framebuffer().ok_or( gl::WebglError::FailedToAllocateResource( "PMREM FBO" ) )?
    );
    gl.bind_framebuffer( gl::FRAMEBUFFER, Some( fbo.as_ref() ) );
    gl::drawbuffers::drawbuffers( gl, &[ 0 ] );

    gl.disable( gl::DEPTH_TEST );
    gl.disable( gl::BLEND );
    gl.disable( gl::CULL_FACE );
    gl.color_mask( true, true, true, true );
    gl.clear_color( 0.0, 0.0, 0.0, 0.0 );
    gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 0 );

    let source_cubemap = TextureGuard::new
    (
      gl,
      equirect_to_cubemap( gl, &programs, equirect_texture, cubemap_resolution, num_mips as i32 )?
    );

    gl.bind_framebuffer( gl::FRAMEBUFFER, Some( fbo.as_ref() ) );
    let specular_texture = TextureGuard::new
    (
      gl,
      prefilter_specular( gl, &programs, source_cubemap.as_ref(), cubemap_resolution, num_mips )?
    );

    gl.bind_framebuffer( gl::FRAMEBUFFER, Some( fbo.as_ref() ) );
    let diffuse_texture = TextureGuard::new
    (
      gl,
      convolve_irradiance( gl, &programs, source_cubemap.as_ref() )?
    );

    gl.bind_framebuffer( gl::FRAMEBUFFER, Some( fbo.as_ref() ) );
    let brdf_lut = integrate_brdf( gl, &programs )?;

    // Every pass succeeded. Reset bindings, then hand the output textures to the caller by
    // disarming their guards. `source_cubemap`, `fbo` and `programs` are freed by their
    // guards / `Drop` impls when this scope ends.
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    gl.bind_texture( gl::TEXTURE_CUBE_MAP, None );
    gl.bind_texture( gl::TEXTURE_2D, None );
    gl.use_program( None );

    Ok
    (
      IBL
      {
        diffuse_texture : Some( diffuse_texture.release() ),
        specular_1_texture : Some( specular_texture.release() ),
        specular_2_texture : Some( brdf_lut ),
        num_mips,
      }
    )
  }
}

crate::mod_interface!
{
  own use
  {
    generate
  };
}
