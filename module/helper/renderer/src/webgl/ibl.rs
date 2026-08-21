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

  // Test placement: constructing an `IBL` with a real, self-freeing `gl` field requires the
  // `pub(crate)` `gl` field directly ( no public constructor populates it -- the only two real
  // writers, `loaders::ibl::load`/`loaders::pmrem::generate`, both need real HDR assets or a
  // full PMREM pipeline run, too heavy for a fast unit test ), and asserting a `Clone`'s `gl`
  // is `None` needs the same private access. Only a test nested inside `mod private` can do
  // either. See `rulebook.md § Test placement`.
  #[ cfg( all( test, target_arch = "wasm32" ) ) ]
  mod tests
  {
    use super::*;

    fn gl_init() -> gl::WebGl2RenderingContext
    {
      gl::browser::setup( gl::browser::Config::default() );
      let options = gl::context::ContextOptions::default();
      let canvas = gl::canvas::make().unwrap();
      gl::context::from_canvas_with( &canvas, options ).unwrap()
    }

    // Test pitfall (not a production bug): `gl.create_texture()` alone allocates a texture
    // *name*, but per the WebGL/OpenGL ES spec, `isTexture` only recognizes an object once it
    // has been bound at least once via `bindTexture` -- an unbound name is correctly reported
    // as "not a texture" even though `create_texture` succeeded. Every real allocation path in
    // this crate binds before use ( `gl::TEXTURE_2D` is fine here regardless of the texture's
    // real target -- only "has this name ever been bound" affects `isTexture` ), so this helper
    // does the same one-time bind to make the constructed `IBL` observably real for the tests
    // below, matching what `loaders::ibl`/`loaders::pmrem` always do before this code ever runs.
    fn ibl_with_real_textures( gl : &gl::WebGl2RenderingContext ) -> IBL
    {
      let diffuse_texture = gl.create_texture();
      let specular_1_texture = gl.create_texture();
      let specular_2_texture = gl.create_texture();
      for texture in [ &diffuse_texture, &specular_1_texture, &specular_2_texture ]
      {
        gl.bind_texture( gl::TEXTURE_2D, texture.as_ref() );
      }
      gl.bind_texture( gl::TEXTURE_2D, None );

      IBL
      {
        diffuse_texture,
        specular_1_texture,
        specular_2_texture,
        num_mips : 10,
        gl : Some( gl.clone() ),
      }
    }

    /// ## Root Cause
    /// `IBL` allocated three cubemap/2D textures ( via its loaders ) but had no way to free
    /// them -- `Renderer::ibl_set` replacing an already-set `self.ibl` ( e.g. an application
    /// swapping environment maps at runtime ) silently leaked the previous `IBL`'s three
    /// textures every time, and nothing freed them even when the owning `Renderer` was dropped.
    ///
    /// ## Why Not Caught
    /// `webgl/ibl.rs`'s existing test only covers `ibl_texture_parameters_apply`'s mip-range
    /// targeting -- no test previously constructed-then-dropped an `IBL` to check for leaks.
    ///
    /// ## Fix Applied
    /// Added a `pub(crate) gl : Option< gl::WebGl2RenderingContext >` field ( populated by both
    /// loaders ) and `impl Drop for IBL`, deleting all three textures when `gl` is populated.
    ///
    /// ## Prevention
    /// Constructs an `IBL` directly via struct literal with all three textures pre-populated
    /// and `gl` set, then asserts all three handles are freed after drop.
    ///
    /// ## Pitfall
    /// `IBL` is a plain "loose bag of `pub` shared texture handles" with no allocation-time
    /// hook of its own ( unlike `TransformsData`/`DisplacementsData`, which allocate inside
    /// their own `upload()` method ) -- its textures are always populated by an external loader
    /// function, which is easy to overlook when auditing "does this type manage its own
    /// cleanup?", since the type itself never calls `gl.create_texture()`.
    // test_kind: bug_reproducer(BUG-440)
    #[ wasm_bindgen_test::wasm_bindgen_test ]
    fn ibl_drop_frees_all_three_textures_when_gl_populated()
    {
      let gl = gl_init();
      let ibl = ibl_with_real_textures( &gl );

      let diffuse = ibl.diffuse_texture.clone();
      let specular_1 = ibl.specular_1_texture.clone();
      let specular_2 = ibl.specular_2_texture.clone();
      assert!( gl.is_texture( diffuse.as_ref() ) );
      assert!( gl.is_texture( specular_1.as_ref() ) );
      assert!( gl.is_texture( specular_2.as_ref() ) );

      drop( ibl );

      assert!( !gl.is_texture( diffuse.as_ref() ), "IBL::drop must delete diffuse_texture" );
      assert!( !gl.is_texture( specular_1.as_ref() ), "IBL::drop must delete specular_1_texture" );
      assert!( !gl.is_texture( specular_2.as_ref() ), "IBL::drop must delete specular_2_texture" );
    }

    /// ## Root Cause
    /// `IBL` previously derived `Clone`, which would have copied the texture handles by
    /// reference ( aliasing the same GPU textures across instances ) with no
    /// reallocation-on-clone mechanism -- adding `Drop` on top of that derive would let either
    /// copy free textures the other still relies on, a double-free/dangling-handle risk.
    ///
    /// ## Why Not Caught
    /// N/A -- this is a safety margin added alongside the BUG-440 fix itself, not a
    /// previously-observed failure ( no caller in this workspace clones an `IBL` today,
    /// confirmed by grep across `module/` and `examples/` ).
    ///
    /// ## Fix Applied
    /// Replaced `#[ derive( Clone ) ]` with a manual `impl Clone` that copies the three texture
    /// handles field-for-field but always resets the clone's `gl` to `None` -- only the
    /// original loader-populated instance ever frees; every `Clone` is a permanently
    /// non-owning view.
    ///
    /// ## Prevention
    /// This test clones a real, `gl`-populated `IBL`, asserts the clone's textures alias the
    /// same GL objects ( same handle values, both still valid ), drops the CLONE first, and
    /// asserts the original's textures remain valid ( the clone's `Drop` was a no-op since its
    /// `gl` is `None` ) -- proving the clone cannot double-free the original's textures.
    ///
    /// ## Pitfall
    /// `#[ derive( Clone ) ]` on a struct holding shared GPU handles is only safe if the type
    /// either deep-copies the underlying resource or guarantees at most one copy ever frees it
    /// -- a derived field-for-field clone of a `gl`-populated `Drop` type silently creates two
    /// owners of the same GPU object.
    // test_kind: bug_reproducer(BUG-440)
    #[ wasm_bindgen_test::wasm_bindgen_test ]
    fn ibl_clone_does_not_double_free_original_textures()
    {
      let gl = gl_init();
      let original = ibl_with_real_textures( &gl );
      let diffuse = original.diffuse_texture.clone();

      let clone = original.clone();
      assert!( clone.gl.is_none(), "a Clone of IBL must never take on ownership ( gl must be None )" );
      assert!( gl.is_texture( clone.diffuse_texture.as_ref() ), "Clone must still alias a live GL texture object" );

      drop( clone );
      assert!( gl.is_texture( diffuse.as_ref() ), "dropping a non-owning Clone must not free the original's texture" );

      drop( original );
      assert!( !gl.is_texture( diffuse.as_ref() ), "dropping the original ( gl-populated ) IBL must still free its texture" );
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
