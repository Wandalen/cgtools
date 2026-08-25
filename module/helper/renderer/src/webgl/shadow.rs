//! Module

mod private
{
  use minwebgl as gl;
  use gl::{ GL, Program, math::mat3x3h };
  use web_sys::{ WebGlFramebuffer, WebGlTexture };
  use crate::webgl::Node;

  /// Shadow map for rendering depth from light's perspective
  #[ derive( Debug ) ]
  pub struct ShadowMap
  {
    framebuffer   : Option< WebGlFramebuffer >,
    depth_texture : Option< WebGlTexture >,
    program       : Program,
    resolution    : i32,
    gl            : GL,
  }

  impl ShadowMap
  {
    /// Creates shadow map with specified resolution
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if allocating the shadow-map GPU resources fails.
    pub fn new( gl : &GL, resolution : u32 ) -> Result< Self, gl::WebglError >
    {
      let resolution = resolution as i32;

      let depth_texture = gl.create_texture();
      gl.bind_texture( gl::TEXTURE_2D, depth_texture.as_ref() );
      gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::DEPTH_COMPONENT32F, resolution, resolution );
      gl::texture::d2::filter_nearest( gl );
      gl::texture::d2::wrap_clamp( gl );

      let framebuffer = gl.create_framebuffer();
      gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
      gl.framebuffer_texture_2d
      (
        gl::FRAMEBUFFER,
        gl::DEPTH_ATTACHMENT,
        gl::TEXTURE_2D,
        depth_texture.as_ref(),
        0
      );

      gl::drawbuffers::drawbuffers( gl, &[] );
      gl.read_buffer( gl::NONE );

      let status = gl.check_framebuffer_status( gl::FRAMEBUFFER );
      if status != gl::FRAMEBUFFER_COMPLETE
      {
        gl::browser::error!( "Framebuffer incomplete: {status:?}" );
      }

      gl.bind_framebuffer( gl::FRAMEBUFFER, None );

      let vertex = include_str!( "shaders/depth.vert" );
      let fragment = include_str!( "shaders/empty.frag" );
      let program = gl::Program::new( gl.clone(), vertex, fragment )?;

      Ok
      (
        Self
        {
          framebuffer,
          depth_texture,
          program,
          resolution,
          gl : gl.clone(),
        }
      )
    }

    /// Activates shadow map for depth rendering
    pub fn bind( &self )
    {
      self.gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
      self.program.activate();
      self.gl.enable( gl::DEPTH_TEST );
      self.gl.enable( gl::CULL_FACE );
      self.gl.cull_face( gl::FRONT );
      self.gl.viewport( 0, 0, self.resolution, self.resolution );
    }

    /// Sets model-view-projection matrix
    pub fn mvp_upload( &self, mvp : gl::F32x4x4 )
    {
      self.program.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
    }

    /// Returns depth texture for sampling
    pub fn depth_buffer( &self ) -> Option< &WebGlTexture >
    {
      self.depth_texture.as_ref()
    }

    /// Clears depth buffer
    pub fn clear( &self )
    {
      self.gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
      self.gl.clear( gl::DEPTH_BUFFER_BIT );
    }

    /// Renders shadow map from light's perspective
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if a node upload or draw call fails during the depth pass.
    pub fn render
    (
      &self,
      scene : &crate::webgl::Scene,
      mut light : Light
    ) -> Result< (), gl::WebglError >
    {
      self.bind();
      self.clear();

      // Recursively traverse scene and render all shadow-casting meshes
      scene.traverse
      (
        &mut | node |
        {
          let node = node.borrow();

          if let crate::webgl::Object3D::Mesh( mesh ) = &node.object
          {
            if !mesh.borrow().is_shadow_caster
            {
              return Ok( () );
            }

            let model = node.world_matrix_get();
            let mvp = light.view_projection() * model;
            self.mvp_upload( mvp );

            for primitive in &mesh.borrow().primitives
            {
              let primitive = primitive.borrow();
              primitive.geometry.borrow().bind( &self.gl );
              primitive.draw( &self.gl );
            }
          }

          Ok( () )
        }
      )?;

      // Fix(BUG-439): restore `cull_face` to the renderer-wide default ( BACK ) before
      // returning, so code drawing anything immediately after this shadow pass -- without
      // going through `Renderer::render()`'s own per-material `material_face_properties_enable`,
      // which always re-sets `cull_face` explicitly before every draw -- doesn't silently
      // inherit `bind()`'s FRONT-face culling.
      // Root cause: `bind()` sets `cull_face( FRONT )` ( a standard peter-panning mitigation
      // for depth-only passes ); `render()` already restored the framebuffer binding at its
      // end but left this piece of state untouched.
      // Pitfall: `CULL_FACE` enable/disable is deliberately left as `bind()` set it ( enabled )
      // -- restoring face *mode* to a sane default is enough to prevent silently-wrong culling;
      // whether culling is enabled at all is the next draw call's own responsibility, same as
      // for every material-driven draw in `Renderer::opaque_draw`. The viewport `bind()` sets
      // ( `resolution x resolution` ) is deliberately left unrestored too -- there is no single
      // correct default to restore it to from this scope ( the real render target's size isn't
      // known here ); callers relying on a specific viewport must set it themselves before
      // their next draw, same as any other GL viewport consumer.
      self.gl.cull_face( gl::BACK );
      self.gl.bind_framebuffer( gl::FRAMEBUFFER, None );

      Ok( () )
    }
  }

  impl Drop for ShadowMap
  {
    fn drop( &mut self )
    {
      self.gl.delete_framebuffer( self.framebuffer.as_ref() );
      _ = self.framebuffer.take();
      self.gl.delete_texture( self.depth_texture.as_ref() );
      _ = self.depth_texture.take();
    }
  }

  /// Bakes PCSS shadows into lightmap textures
  #[ derive( Debug ) ]
  pub struct ShadowBaker
  {
    framebuffer : Option< WebGlFramebuffer >,
    program     : Program,
    gl          : GL,
  }

  impl ShadowBaker
  {
    /// Creates shadow baker
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if allocating the baker's GPU resources fails.
    pub fn new( gl : &GL ) -> Result< Self, gl::WebglError >
    {
      let framebuffer = gl.create_framebuffer();

      let vertex = include_str!( "shaders/bake.vert" );
      let fragment = include_str!( "shaders/bake.frag" );
      let program = gl::Program::new( gl.clone(), vertex, fragment )?;

      Ok
      (
        Self
        {
          framebuffer,
          program,
          gl : gl.clone(),
        }
      )
    }

    /// Sets target lightmap texture and dimensions
    fn target_set( &self, texture : Option< &WebGlTexture > )
    {
      self.gl.bind_framebuffer( gl::FRAMEBUFFER, self.framebuffer.as_ref() );
      self.gl.framebuffer_texture_2d
      (
        gl::FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D,
        texture,
        0
      );

      // Check framebuffer completeness
      let status = self.gl.check_framebuffer_status( gl::FRAMEBUFFER );
      if status != gl::FRAMEBUFFER_COMPLETE
      {
        gl::browser::error!( "Shadow baker framebuffer incomplete: {status:?}" );
      }
    }

    /// Activates baker for rendering
    fn bind( &self, width: i32, height : i32 )
    {
      self.program.activate();
      self.gl.bind_framebuffer( gl::FRAMEBUFFER, self.framebuffer.as_ref() );
      self.gl.viewport( 0, 0, width, height );
      self.gl.disable( gl::DEPTH_TEST );
      self.gl.disable( gl::CULL_FACE );
    }

    /// Sets model matrix for geometry
    fn model_upload( &self, model : gl::F32x4x4 )
    {
      self.program.uniform_matrix_upload( "u_model", model.raw_slice(), true );
    }

    /// Binds shadow map for sampling
    fn shadowmap_set( &self, shadowmap : Option< &WebGlTexture > )
    {
      self.gl.active_texture( gl::TEXTURE0 );
      self.gl.bind_texture( gl::TEXTURE_2D, shadowmap );
    }

    /// Uploads light parameters to shader
    fn light_upload( &self, light : &mut Light )
    {
      let light_vp = light.view_projection();
      self.program.uniform_matrix_upload( "u_light_view_projection", light_vp.raw_slice(), true );

      let light_dir = light.direction();
      self.program.uniform_upload( "u_light_dir", light_dir.as_slice() );

      let light_pos = light.position();
      self.program.uniform_upload( "u_light_position", light_pos.as_slice() );

      let is_ortho = i32::from(light.is_orthographic());
      self.program.uniform_upload( "u_is_orthographic", &is_ortho );

      let light_size = light.size();
      self.program.uniform_upload( "u_light_size", &light_size );

      let ( near, far ) = light.near_far_planes();

      self.program.uniform_upload( "u_near", &near );
      self.program.uniform_upload( "u_far", &far );
    }

    /// Bakes shadows into lightmaps via two-pass rendering: depth map, then PCSS lightmap baking
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if a pass, upload, or draw fails during either baking pass.
    pub fn soft_shadow_render
    (
      &self,
      node : &Node,
      target : Option< &WebGlTexture >,
      width: u32,
      height : u32,
      shadowmap : &ShadowMap,
      mut light : Light,
    ) -> Result< (), gl::WebglError >
    {
      self.bind( width as i32, height as i32 );
      self.target_set( target );
      self.light_upload( &mut light );
      self.shadowmap_set( shadowmap.depth_buffer() );
      let model = node.world_matrix_get();
      self.model_upload( model );

      if let crate::webgl::Object3D::Mesh( mesh ) = &node.object
      {
        for primitive in &mesh.borrow().primitives
        {
          let primitive_ref = primitive.borrow_mut();
          primitive_ref.geometry.borrow().bind( &self.gl );
          primitive_ref.draw( &self.gl );
        }
      }

      Ok( () )
    }
  }

  // Fix(BUG-432): `ShadowBaker` created a `WebGlFramebuffer` in `new` but never deleted it --
  // every `ShadowBaker` construct/drop cycle (e.g. a scene reload that rebuilds the lightmap
  // baking pipeline) permanently leaked one framebuffer object for the lifetime of the GL
  // context, with no way for a caller to reclaim it short of losing the whole context.
  // Root cause: unlike `ShadowMap` right above (which already has `impl Drop` deleting both
  // its `framebuffer` and `depth_texture`), `ShadowBaker` was never given a matching `Drop`
  // impl when it was added -- the GPU handle wrapper types (`Option< WebGlTexture >` etc.) are
  // just JS-object handles; dropping the Rust value does not call `gl.delete*` for you.
  // Pitfall: adding a new GL-resource-owning struct next to an existing one that already has
  // `impl Drop` is easy to do without copying that pattern over -- the struct compiles and
  // runs identically either way, so nothing short of a GPU-memory audit surfaces the leak.
  impl Drop for ShadowBaker
  {
    fn drop( &mut self )
    {
      self.gl.delete_framebuffer( self.framebuffer.as_ref() );
      _ = self.framebuffer.take();
    }
  }

  /// Light source for shadow casting
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Light
  {
    position        : gl::F32x3,
    direction       : gl::F32x3,
    projection      : gl::F32x4x4,
    size            : f32,
    view_projection : Option< gl::F32x4x4 >,
  }

  impl Light
  {
    /// Creates light with position, direction, projection, and size
    #[ must_use ]
    pub fn new
    (
      position : gl::F32x3,
      direction : gl::F32x3,
      projection : gl::F32x4x4,
      size : f32
    ) -> Self
    {
      Self
      {
        position,
        direction : direction.normalize(),
        projection,
        size,
        view_projection : None,
      }
    }

    /// Returns light size (controls shadow softness)
    #[ must_use ]
    pub fn size( &self ) -> f32
    {
      self.size
    }

    /// Extracts near and far planes from projection matrix
    #[ must_use ]
    pub fn near_far_planes( &self ) -> ( f32, f32 )
    {
      let m = self.projection.raw_slice();
      let m10 = m[ 10 ];  // [2][2] in column-major
      let m14 = m[ 14 ];  // [3][2] in column-major

      if self.is_orthographic()
      {
        // Orthographic projection: m[15] = 1.0
        // m[10] = -2 / (far - near)
        // m[14] = -(far + near) / (far - near)
        // Solving:
        //   far = (m[14] - 1) / m[10]
        //   near = (1 + m[14]) / m[10]
        let far = ( m14 - 1.0 ) / m10;
        let near = ( 1.0 + m14 ) / m10;
        ( near, far )
      }
      else
      {
        // Perspective projection: m[15] = 0.0
        // m[10] = -(far + near) / (far - near)
        // m[14] = -2 * far * near / (far - near)
        // Solving:
        //   near = m[14] / (m[10] - 1)
        //   far = m[14] / (m[10] + 1)
        let near = m14 / ( m10 - 1.0 );
        let far = m14 / ( m10 + 1.0 );
        ( near, far )
      }
    }

    /// Returns light position
    #[ must_use ]
    pub fn position( &self ) -> gl::F32x3
    {
      self.position
    }

    /// Returns light direction
    #[ must_use ]
    pub fn direction( &self ) -> gl::F32x3
    {
      self.direction
    }

    /// Returns projection matrix
    #[ must_use ]
    pub fn projection( &self ) -> gl::F32x4x4
    {
      self.projection
    }

    /// Returns true if using orthographic projection (checks `matrix[3][3] == 1.0`)
    #[ must_use ]
    pub fn is_orthographic( &self ) -> bool
    {
      let m = self.projection.raw_slice();
      let w_component = m[ 15 ]; // [3][3] in column-major order
      ( w_component - 1.0 ).abs() < 0.01
    }

    /// Returns cached view-projection matrix
    pub fn view_projection( &mut self ) -> gl::F32x4x4
    {
      if let Some( mvp ) = self.view_projection
      {
        mvp
      }
      else
      {
        let view = mat3x3h::look_to_rh( self.position(), self.direction, gl::F32x3::Y );
        let view_projection = self.projection * view;
        self.view_projection = Some( view_projection );

        view_projection
      }
    }
  }

  impl From< crate::webgl::SpotLight > for Light
  {
    fn from( spot : crate::webgl::SpotLight ) -> Self
    {
      // Use outer cone angle for FOV
      let fov = spot.outer_cone_angle * 2.0;
      let near = 0.1;
      let far = spot.range;

      // Light size affects shadow softness - derive from cone angle
      // Smaller angles = tighter beam = smaller physical size
      let radius = spot.outer_cone_angle * 2.0;
      let max_radius = 135.0_f32.to_radians();

      // Fix(BUG-175): `.min( 0.01 )` on the last line made this a ceiling, not a floor -- since
      // the preceding `( radius / max_radius ).min( 1.0 ) * 1.7` term is >= 0.01 for every
      // `outer_cone_angle` above ~0.4 degrees ( i.e. every realistic spot light ), `.min` always
      // picked the constant 0.01 and the entire angle-dependent scaling above it was dead code --
      // every spot light baked identically soft shadows regardless of cone angle.
      // Root cause: `.min` used where a lower-bound floor ( `.max` ) was intended.
      // Pitfall: a `.min( FLOOR )`/`.max( FLOOR )` mixup silently reads as a working line -- it
      // still compiles and always returns *a* value in range, it just discards a preceding
      // computation. Check which direction the clamp actually needs before trusting it compiled.
      let light_size = ( ( radius / max_radius ).min( 1.0 ) * 1.7 ).max( 0.01 );

      let projection = gl::math::mat3x3h::perspective_rh_gl( fov, 1.0, near, far );

      Self::new( spot.position, spot.direction, projection, light_size )
    }
  }

  // Test placement: both tests below need private-field/handle access (`ShadowBaker::framebuffer`
  // to capture the handle before `drop`; `ShadowMap`'s `cull_face` assertion needs nothing
  // private, but is kept alongside the other GPU-teardown-hygiene test in this file for
  // discoverability) -- see `rulebook.md § Test placement`. Live-GL-context tests, so wasm32-only.
  #[ cfg( all( test, target_arch = "wasm32" ) ) ]
  mod tests
  {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    fn gl_init() -> GL
    {
      gl::browser::setup( gl::browser::Config::default() );
      let options = gl::context::ContextOptions::default();
      let canvas = gl::canvas::make().unwrap();
      gl::context::from_canvas_with( &canvas, options ).unwrap()
    }

    fn spot_light_make() -> crate::webgl::SpotLight
    {
      crate::webgl::SpotLight
      {
        position : gl::F32x3::from_array( [ 0.0, 5.0, 0.0 ] ),
        direction : gl::F32x3::from_array( [ 0.0, -1.0, 0.0 ] ),
        color : gl::F32x3::from_array( [ 1.0, 1.0, 1.0 ] ),
        strength : 1.0,
        range : 10.0,
        inner_cone_angle : 0.1,
        outer_cone_angle : 0.5,
        use_light_map : false,
      }
    }

    /// ## Root Cause
    /// `ShadowBaker` allocated a `WebGlFramebuffer` in `new` but had no `impl Drop` -- every
    /// construct/drop cycle permanently leaked one framebuffer for the GL context's lifetime.
    ///
    /// ## Why Not Caught
    /// `ShadowBaker` had zero prior test coverage of any kind -- nothing exercised its
    /// construction or destruction, so a missing `Drop` impl produced no observable failure.
    ///
    /// ## Fix Applied
    /// Added `impl Drop for ShadowBaker`, calling `gl.delete_framebuffer` on `self.framebuffer`
    /// ( matching the sibling `ShadowMap`'s pre-existing `impl Drop`, right above it in this
    /// file, which this new impl was modeled on ).
    ///
    /// ## Prevention
    /// This test captures a clone of the private `framebuffer` handle before drop ( `mod tests`
    /// is a descendant of `mod private`, so the field is directly visible here ), then asserts
    /// `gl.is_framebuffer` flips from `true` to `false` once the `ShadowBaker` is dropped --
    /// the same deterministic existence-check pattern used by this crate's other GPU-teardown
    /// tests ( see `gpu_resource_leak_test` siblings in `gbuffer.rs`, `unreal_bloom.rs`, etc. ).
    ///
    /// ## Pitfall
    /// A GPU handle wrapper ( `Option< WebGlFramebuffer >` ) is just a JS-object reference --
    /// letting the Rust value go out of scope does not call `gl.delete*` for you; only an
    /// explicit delete call ( here, via `impl Drop` ) reclaims the actual GPU-side allocation.
    // test_kind: bug_reproducer(BUG-432)
    #[ wasm_bindgen_test ]
    fn shadow_baker_drop_frees_framebuffer()
    {
      let gl = gl_init();
      let baker = ShadowBaker::new( &gl ).expect( "ShadowBaker::new should succeed on a valid context" );

      let framebuffer = baker.framebuffer.clone();
      // Test pitfall (not a production bug): `ShadowBaker::new` calls `create_framebuffer()`
      // but only ever binds it later, inside `target_set` ( called from `soft_shadow_render`,
      // not from `new` ) -- an unbound name is correctly reported as "not a framebuffer" by
      // `isFramebuffer` per the WebGL/OpenGL ES spec until bound at least once. This one-time
      // bind/unbind reproduces that precondition without needing a real `target_set` call
      // ( which additionally attaches a texture and checks completeness -- more than this test
      // needs ).
      gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );
      assert!( gl.is_framebuffer( framebuffer.as_ref() ), "framebuffer must be a live GL object right after construction" );

      drop( baker );

      assert!( !gl.is_framebuffer( framebuffer.as_ref() ), "ShadowBaker::drop must delete its framebuffer" );
    }

    /// ## Root Cause
    /// `ShadowMap::bind()` sets `cull_face( FRONT )` as a peter-panning mitigation for the
    /// depth-only shadow pass. `render()` previously restored the framebuffer binding at its
    /// end but left `cull_face` at `FRONT`, so any subsequent draw call issued without going
    /// through `Renderer::render()`'s own per-material face-property setup would silently
    /// inherit front-face culling.
    ///
    /// ## Why Not Caught
    /// Existing `ShadowMap::render` tests ( `fbo_pass_cycle_test.rs` ) only assert `Result::is_ok`
    /// -- they never inspect GL state left behind after the call returns.
    ///
    /// ## Fix Applied
    /// `render()` now calls `self.gl.cull_face( gl::BACK )` immediately before returning,
    /// restoring the renderer-wide default face mode ( `CULL_FACE` enable state and the
    /// viewport are deliberately left unrestored -- see the `Fix(BUG-439)` comment above for
    /// why neither has a single correct default from this scope ).
    ///
    /// ## Prevention
    /// This test reads back `gl::CULL_FACE_MODE` via `gl.get_parameter` after a real
    /// `ShadowMap::render` call on an empty scene, asserting it is `gl::BACK` -- the general,
    /// state-agnostic invariant the fix restores, not a pinned per-scene expectation.
    ///
    /// ## Pitfall
    /// GL state ( as opposed to GL objects/resources ) has no "drop" mechanism at all -- a pass
    /// that mutates global context state ( `cull_face`, blend mode, depth func, etc. ) must
    /// explicitly restore whatever contract it promises callers, since nothing in the type
    /// system enforces symmetric enable/restore the way `Drop` does for owned GPU objects.
    // test_kind: bug_reproducer(BUG-439)
    #[ wasm_bindgen_test ]
    fn shadow_map_render_restores_cull_face_to_back()
    {
      let gl = gl_init();
      let shadow_map = ShadowMap::new( &gl, 64 ).expect( "ShadowMap::new should succeed on a valid context" );
      let scene = crate::webgl::Scene::new();
      let light = Light::from( spot_light_make() );

      let result = shadow_map.render( &scene, light );
      assert!( result.is_ok(), "ShadowMap::render should succeed on an empty scene -- got {:?}", result.err() );

      let mode = gl.get_parameter( gl::CULL_FACE_MODE ).expect( "CULL_FACE_MODE must be readable" );
      let mode = mode.as_f64().expect( "CULL_FACE_MODE must be numeric" ) as u32;
      assert_eq!( mode, gl::BACK, "ShadowMap::render must restore cull_face to BACK before returning" );
    }
  }
}

crate::mod_interface!
{
  own use
  {
    ShadowBaker,
    ShadowMap,
    Light,
  };
}
