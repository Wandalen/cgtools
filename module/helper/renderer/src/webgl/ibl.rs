mod private
{
  use minwebgl as gl;

  /// Holds three precomputed textures used for Image-Based Lighting (IBL) calculations.
  ///
  /// According to:
  /// - <https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf>
  /// - <https://learnopengl.com/PBR/IBL/Diffuse-irradiance>
  #[ derive( Default ) ]
  pub struct IBL
  {
    /// The diffuse irradiance cubemap texture.
    pub diffuse_texture : Option< gl::web_sys::WebGlTexture >,
    /// The prefiltered specular environment map (cubemap) texture.
    pub specular_1_texture : Option< gl::web_sys::WebGlTexture >,
    /// The 2D lookup texture containing the BRDF (Bidirectional Reflectance Distribution Function) integration result.
    pub specular_2_texture : Option< gl::web_sys::WebGlTexture >,
    /// Number of mip levels in `specular_1_texture`. `max_lod = num_mips - 1`.
    pub num_mips : u32,
    /// GL context these textures were allocated from -- retained so `impl Drop` can free them.
    /// `None` for a default-constructed `IBL` ( owns nothing ) or for any `Clone` of another
    /// `IBL` -- see `impl Clone` below : a clone never takes on ownership of the texture
    /// handles it copies, so its own `Drop` is always a safe no-op. Only the instance a loader
    /// ( `loaders::ibl::load`, `loaders::pmrem::generate` ) populated directly ever frees.
    //
    // Fix(BUG-440): `pub(crate)`, not bare-private -- `loaders::ibl`/`loaders::pmrem` (the only
    // two call sites that ever populate this field with a real context) are sibling modules of
    // `webgl::ibl`, not descendants of its `mod private`, so a bare-private field is invisible
    // to them (E0451). `pub(crate)` keeps it invisible to external consumers of this crate
    // (the only visibility this field actually needs to lose) while letting same-crate loaders
    // construct a real, self-freeing `IBL` directly.
    // Root cause: initial field addition used bare-private without checking that its two real
    // writers live outside `mod private`'s own descendant-module boundary.
    // Pitfall: Rust module privacy is scoped to the defining module and its descendants only --
    // "internal-implementation-only" is not the same boundary as "this module and its
    // descendants"; a field meant to be crate-internal (not descendant-internal) needs
    // `pub(crate)`, not bare-private.
    pub(crate) gl : Option< gl::WebGl2RenderingContext >,
  }

  // Fix(BUG-440): `IBL` allocated three cubemap/2D textures ( via its loaders ) but had no
  // way to free them -- `Renderer::ibl_set` replacing an already-set `self.ibl` ( e.g. an
  // application swapping environment maps at runtime ) silently leaked the previous `IBL`'s
  // three textures every time, and nothing freed them even when the owning `Renderer` itself
  // was dropped.
  // Root cause: `IBL` had no `gl` field and no `impl Drop` -- it was a plain data bag with
  // `pub` texture fields, so nothing in the type itself was ever responsible for cleanup.
  // Pitfall: `IBL` previously derived `Clone`, which would have copied the texture handles
  // ( aliasing the same GPU textures across instances, with no reallocation-on-clone
  // mechanism like `TransformsData`/`DisplacementsData` have ) -- adding `Drop` on top of that
  // derive would let either copy free textures the other still relies on. The manual `impl
  // Clone` below keeps that field-for-field behavior but always resets `gl` to `None` on the
  // copy, so only the original loader-populated instance ever frees ; every `Clone` is a
  // non-owning view for as long as it exists. No caller in this workspace currently clones an
  // `IBL` ( verified by grep across `module/` and `examples/` ), so this is a documented
  // safety margin, not a fix for an observed bug.
  impl Clone for IBL
  {
    fn clone( &self ) -> Self
    {
      Self
      {
        diffuse_texture : self.diffuse_texture.clone(),
        specular_1_texture : self.specular_1_texture.clone(),
        specular_2_texture : self.specular_2_texture.clone(),
        num_mips : self.num_mips,
        gl : None,
      }
    }
  }

  impl Drop for IBL
  {
    fn drop( &mut self )
    {
      if let Some( ref gl ) = self.gl
      {
        gl.delete_texture( self.diffuse_texture.as_ref() );
        gl.delete_texture( self.specular_1_texture.as_ref() );
        gl.delete_texture( self.specular_2_texture.as_ref() );
      }
    }
  }

  impl IBL
  {
    /// Creates a new `IBL` instance with default (empty) texture options.
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Binds the IBL textures to specific texture units.
    ///
    /// * `gl`: The `WebGl2RenderingContext`.
    /// * `base_active_texture`: The starting texture unit index to which the diffuse texture will be bound.
    ///   Subsequent specular textures will be bound to the following units.
    pub fn bind( &self, gl : &gl::WebGl2RenderingContext, base_active_texture : u32 )
    {
      gl.active_texture( gl::TEXTURE0 + base_active_texture );
      gl.bind_texture( gl::TEXTURE_CUBE_MAP, self.diffuse_texture.as_ref() );

      gl.active_texture( gl::TEXTURE0 + base_active_texture + 1 );
      gl.bind_texture( gl::TEXTURE_CUBE_MAP, self.specular_1_texture.as_ref() );

      gl.active_texture( gl::TEXTURE0 + base_active_texture + 2 );
      gl.bind_texture( gl::TEXTURE_2D, self.specular_2_texture.as_ref() );
    }
  }

  /// Construction of, and inspection of, the owning form of `IBL` — reachable
  /// from `tests/` under `test_internals`.
  ///
  /// The three texture fields are already public, so a test could build an `IBL`
  /// by struct literal were it not for `gl`, which is `pub( crate )` and is the
  /// single field that decides whether this instance frees anything on drop.
  /// That is precisely what a teardown test needs to set and to read, and
  /// nothing outside this crate has any business doing either — hence a
  /// constructor plus a boolean rather than widening the field itself.
  #[ cfg( feature = "test_internals" ) ]
  impl IBL
  {
    /// An `IBL` that owns the given textures: its `Drop` will free all three.
    #[ doc( hidden ) ]
    #[ must_use ]
    pub fn new_owning_for_test
    (
      diffuse_texture : Option< gl::web_sys::WebGlTexture >,
      specular_1_texture : Option< gl::web_sys::WebGlTexture >,
      specular_2_texture : Option< gl::web_sys::WebGlTexture >,
      num_mips : u32,
      gl : &gl::WebGl2RenderingContext,
    ) -> Self
    {
      Self
      {
        diffuse_texture,
        specular_1_texture,
        specular_2_texture,
        num_mips,
        gl : Some( gl.clone() ),
      }
    }

    /// Whether this instance frees its textures on drop. False for a default
    /// `IBL` and for every `Clone`, which is what makes a clone's drop a no-op
    /// instead of a double free.
    #[ doc( hidden ) ]
    #[ must_use ]
    pub fn owns_textures_for_test( &self ) -> bool
    {
      self.gl.is_some()
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    IBL
  };
}
