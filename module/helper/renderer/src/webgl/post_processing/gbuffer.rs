mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use rustc_hash::{ FxHashMap, FxHashSet };
  use minwebgl as gl;
  use web_sys::{ WebGlTexture, WebGlBuffer, WebGlFramebuffer, WebGlRenderbuffer, WebGlUniformLocation, WebGlVertexArrayObject, WebGlProgram };
  use gl::{ F32x4, GL, VectorDataType, drawbuffers::drawbuffers };
  use crate::webgl::
  {
    AttributeInfo,
    Camera,
    Material,
    Node,
    Object3D,
    Scene,
    ShaderProgram,
    material::PbrMaterial,
    ProgramInfo
  };
  use crate::webgl::impl_locations;

  /// The source code for the gbuffer vertex shader.
  const GBUFFER_VERTEX_SHADER : &str = include_str!( "../shaders/post_processing/gbuffer.vert" );
  /// The source code for the gbuffer fragment shader.
  const GBUFFER_FRAGMENT_SHADER : &str = include_str!( "../shaders/post_processing/gbuffer.frag" );

  /// Every G-buffer attachment, in color-attachment order.
  pub const ALL : [ GBufferAttachment; 7 ] = [
    GBufferAttachment::Position,
    GBufferAttachment::Color,
    GBufferAttachment::Uv1,
    GBufferAttachment::Albedo,
    GBufferAttachment::Normal,
    GBufferAttachment::PbrInfo,
    GBufferAttachment::ObjectColor
  ];

  // A public struct for a Geometry Buffer (GBuffer) shader.
  impl_locations!
  (
    GBufferShader,
    "worldMatrix",
    "viewMatrix",
    "projectionMatrix",
    "normalMatrix",
    "near_far",
    "albedoTexture",
    "objectId",
    "materialId",
    "objectColor"
  );

  /// Identifies one color attachment ( render target ) written by the geometry pass.
  #[ derive( Debug, Copy, Clone, Eq, PartialEq, Hash ) ]
  pub enum GBufferAttachment
  {
    /// World-space fragment position.
    Position,
    /// Interpolated vertex color.
    Color,
    /// UV coordinates ( channel 1 ).
    Uv1,
    /// Sampled albedo ( base color ).
    Albedo,
    /// Surface normal.
    Normal,
    /// Packed PBR material parameters.
    PbrInfo,
    /// Per-object color supplied at render time ( e.g. for object id / picking ).
    ObjectColor
  }

  impl GBufferAttachment
  {
    /// Builds the vertex-attribute descriptor(s) this attachment needs, pairing each with a
    /// buffer from `buffers` in slot order. Returns an empty `Vec` if `buffers` is empty or if
    /// this attachment ( e.g. [`GBufferAttachment::Albedo`] ) has no dedicated vertex attribute.
    ///
    /// # Panics
    ///
    /// Panics if `buffers` is non-empty but has fewer entries than this attachment needs.
    #[ must_use ]
    pub fn attribute_info( self, buffers : &[ web_sys::WebGlBuffer ] ) -> Vec< AttributeInfo >
    {
      if buffers.is_empty()
      {
        return vec![];
      }

      // Each attachment's vertex attribute is described once via the cross-backend
      // `mingl::VertexAttribute` ( location + vector shape + offset ), paired with the
      // WebGL-only `normalized` flag that type doesn't model. Bridged down to
      // `BufferDescriptor` below since `AttributeInfo.descriptor` is WebGL-specific.
      let descriptors : Vec< ( mingl::VertexAttribute, bool ) > = match self
      {
        GBufferAttachment::Position =>
        vec![ ( mingl::VertexAttribute::new( 0, VectorDataType::new( mingl::DataType::F32, 3, 1 ), 0 ), false ) ],
        GBufferAttachment::Color =>
        vec![ ( mingl::VertexAttribute::new( 1, VectorDataType::new( mingl::DataType::F32, 4, 1 ), 0 ), true ) ],
        GBufferAttachment::Normal =>
        vec![ ( mingl::VertexAttribute::new( 2, VectorDataType::new( mingl::DataType::F32, 3, 1 ), 0 ), true ) ],
        GBufferAttachment::Uv1 =>
        vec![ ( mingl::VertexAttribute::new( 3, VectorDataType::new( mingl::DataType::F32, 2, 1 ), 0 ), true ) ],
        _ => vec![]
      };

      let mut attribute_infos = vec![];

      for ( i, ( attr, normalized ) ) in descriptors.into_iter().enumerate()
      {
        let descriptor = gl::BufferDescriptor::from_vector( attr.vector )
        .offset( attr.offset )
        .stride( 0 )
        .normalized( normalized );

        let a = AttributeInfo
        {
          slot : attr.location,
          buffer : buffers.get( i ).expect( "Some GbufferAttachment hasn't enough buffers" ).clone(),
          descriptor,
          bounding_box : gl::geometry::BoundingBox::default()
        };

        attribute_infos.push( a );
      }

      attribute_infos
    }

    /// The fragment-shader `#define` name identifying this attachment ( see [`into_defines`] ).
    #[ must_use ]
    pub fn define_const( self ) -> String
    {
      match self
      {
        GBufferAttachment::Position => "POSITION",
        GBufferAttachment::Color => "COLOR",
        GBufferAttachment::Uv1 => "UV_1",
        GBufferAttachment::Albedo => "ALBEDO",
        GBufferAttachment::Normal => "NORMAL",
        GBufferAttachment::PbrInfo => "PBR_INFO",
        GBufferAttachment::ObjectColor => "OBJECT_COLOR",
      }
      .to_string()
    }
  }

  fn into_defines( attachments : &FxHashSet< GBufferAttachment > ) -> String
  {
    let mut defines = String::new();

    for attachment in attachments
    {
      defines = format!( "{defines} #define {}\n", attachment.define_const() );
    }

    defines
  }

  /// Binds a texture to a texture unit and uploads its location to a uniform.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL2 rendering context.
  /// * `texture` - The texture to bind.
  /// * `location` - The uniform location in the shader for the sampler.
  /// * `slot` - The texture unit to bind to ( e.g., `GL::TEXTURE0` ).
  fn texture_upload
  (
    gl : &gl::WebGl2RenderingContext,
    texture : &WebGlTexture,
    location : &WebGlUniformLocation,
    slot : u32,
  )
  {
    gl.active_texture( slot );
    gl.bind_texture( GL::TEXTURE_2D, Some( texture ) );
    // Tell the sampler uniform in the shader which texture unit to use ( 0 for GL_TEXTURE0, 1 for GL_TEXTURE1, etc. )
    gl.uniform1i( Some( location ), ( slot - GL::TEXTURE0 ) as i32 );
  }

  fn camera_upload
  (
    gl : &gl::WebGl2RenderingContext,
    camera : &Camera,
    locations : &FxHashMap< String, Option< WebGlUniformLocation > >
  )
  {
    camera.upload( gl, locations );

    let [ near, far ] = camera.near_far_get().0;

    gl::uniform::upload
    (
      gl,
      locations.get( "near_far" ).unwrap().clone(),
      &[ near, far ]
    ).unwrap();
  }

  /// Geometry pass : a multi-render-target framebuffer and the shader that fills it.
  pub struct GBuffer
  {
    shader_program : GBufferShader,
    attachment_buffers: FxHashMap< GBufferAttachment, Vec< WebGlBuffer > >,
    vao : WebGlVertexArrayObject,
    width : u32,
    height : u32,
    framebuffer : WebGlFramebuffer,
    depthbuffer : WebGlRenderbuffer,
    textures: FxHashMap< String, WebGlTexture >,
    color_attachments : Vec< u32 >,
    gl : GL,
  }

  impl GBuffer
  {
    /// Creates a new `GBuffer` instance.
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if shader compilation/linking or G-buffer texture/framebuffer creation fails.
    pub fn new
    (
      gl : &gl::WebGl2RenderingContext,
      width : u32,
      height : u32,
      attachment_buffers: FxHashMap< GBufferAttachment, Vec< WebGlBuffer > >
    ) -> Result< Self, gl::WebglError >
    {
      let attachments_set = attachment_buffers.keys().copied()
      .collect::< FxHashSet< _ > >();
      let defines = into_defines( &attachments_set );
      let program = gl::ProgramFromSources::new
      (
        &format!( "#version 300 es\n{defines}\n{GBUFFER_VERTEX_SHADER}" ),
        &format!( "#version 300 es\n{defines}\n{GBUFFER_FRAGMENT_SHADER}" ),
      ).compile_and_link( gl )?;
      let shader_program = GBufferShader::new( gl, &program );

      let vao = gl.create_vertex_array().ok_or( gl::WebglError::FailedToAllocateResource( "VAO" ) )?;
      gl.bind_vertex_array( Some( &vao ) );

      for ( attachment, buffers ) in &attachment_buffers
      {
        for attribute_info in attachment.attribute_info( buffers )
        {
          attribute_info.upload( gl )?;
        }
      }

      let mut textures = FxHashMap::default();

      let framebuffer = gl.create_framebuffer().ok_or( gl::WebglError::FailedToAllocateResource( "Framebuffer" ) )?;
      gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
      gl.viewport( 0, 0, width as i32, height as i32 );

      let mut color_attachments = vec![];

      let mut setup_texture = | gb_attachment : &GBufferAttachment, attachment, internal_format, filter, wrap |
      {
        let texture = gl.create_texture().ok_or( gl::WebglError::FailedToAllocateResource( "Texture" ) )?;
        gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
        gl.tex_storage_2d( GL::TEXTURE_2D, 1, internal_format, width as i32, height as i32 );
        gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, filter as i32 );
        gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, wrap as i32 );
        gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, wrap as i32 );
        gl.framebuffer_texture_2d( GL::FRAMEBUFFER, attachment, GL::TEXTURE_2D, Some( &texture ), 0 );
        if attachment != GL::DEPTH_ATTACHMENT
        {
          color_attachments.push( attachment - GL::COLOR_ATTACHMENT0 );
        }
        textures.insert( gb_attachment.define_const(), texture );
        Ok::< (), gl::WebglError >( () )
      };

      for attachment in attachment_buffers.keys()
      {
        match attachment
        {
          GBufferAttachment::Position => setup_texture( attachment, GL::COLOR_ATTACHMENT0, gl::RGBA16F, GL::NEAREST, GL::CLAMP_TO_EDGE )?,
          GBufferAttachment::Albedo => setup_texture( attachment, GL::COLOR_ATTACHMENT1, gl::RGBA8, GL::NEAREST, GL::CLAMP_TO_EDGE )?,
          GBufferAttachment::Normal => setup_texture( attachment, GL::COLOR_ATTACHMENT2, gl::RGBA16F, GL::NEAREST, GL::CLAMP_TO_EDGE )?,
          GBufferAttachment::PbrInfo => setup_texture( attachment, GL::COLOR_ATTACHMENT3, gl::RGBA8, GL::NEAREST, GL::CLAMP_TO_EDGE )?,
          GBufferAttachment::ObjectColor => setup_texture( attachment, GL::COLOR_ATTACHMENT4, gl::RGBA16F, GL::NEAREST, GL::CLAMP_TO_EDGE )?,
          _ => ()
        }
      }

      let depthbuffer = gl.create_renderbuffer().ok_or( gl::WebglError::FailedToAllocateResource( "Renderbuffer" ) )?;
      gl.bind_renderbuffer( GL::RENDERBUFFER, Some( &depthbuffer ) );
      gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT24, width as i32, height as i32 );
      gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, Some( &depthbuffer ) );

      gl.bind_vertex_array( None );
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );

      let gbuffer = Self
      {
        shader_program,
        attachment_buffers,
        vao,
        width,
        height,
        framebuffer,
        depthbuffer,
        textures,
        color_attachments,
        gl : gl.clone(),
      };

      Ok( gbuffer )
    }

    /// Binds the gbuffer's program, VAO, framebuffer and set drawbuffers
    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.shader_program.bind( gl );
      gl.bind_vertex_array( Some( &self.vao ) );
      gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );
      gl.viewport( 0, 0, self.width as i32, self.height as i32 );
      drawbuffers( gl, &self.color_attachments );
    }

    /// Returns the texture backing `attachment`, if that attachment exists.
    #[ must_use ]
    pub fn texture( &self, attachment : GBufferAttachment ) -> Option< WebGlTexture >
    {
      self.textures.get( &attachment.define_const() ).cloned()
    }

    /// Runs the geometry pass over `scene`, filling every attachment texture.
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if a scene upload or draw call fails.
    ///
    /// # Panics
    ///
    /// Panics if the G-buffer shader misses one of its fixed uniforms
    /// ( `albedoTexture`, `objectId`, `materialId`, `objectColor` ) or an object-id upload fails.
    pub fn render
    (
      &mut self,
      gl : &gl::WebGl2RenderingContext,
      scene : &mut Scene,
      object_colors: Option< &[ F32x4 ] >,
      camera : &Camera
    ) -> Result< (), gl::WebglError >
    {
      self.bind( gl );

      let locations = self.shader_program.locations();

      gl.enable( gl::DEPTH_TEST );
      gl.disable( gl::BLEND );
      gl.depth_mask( true );
      gl.front_face( gl::CCW );
      gl.cull_face( gl::BACK );
      gl.depth_func( gl::LESS );
      gl.clear_depth( 1.0 );
      gl.clear( GL::DEPTH_BUFFER_BIT );

      gl.clear_bufferfv_with_f32_array( gl::COLOR, 0, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 1, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 2, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 3, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 4, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );

      camera_upload( gl, camera, locations );

      let albedo_texture_loc = &self.shader_program.locations()
      .get( "albedoTexture" ).unwrap().clone().unwrap();

      let object_id_loc = &self.shader_program.locations()
      .get( "objectId" ).unwrap().clone();

      let material_id_loc = &self.shader_program.locations()
      .get( "materialId" ).unwrap().clone();

      let object_color_loc = &self.shader_program.locations()
      .get( "objectColor" ).unwrap().clone();

      let object_id = Rc::new( RefCell::new( 1_u32 ) );

      // Define a closure to handle the drawing of each node in the scene.
      let mut draw_node =
      |
        node : Rc< RefCell< Node > >
      | -> Result< (), gl::WebglError >
      {
        // If the node contains a mesh...
        if let Object3D::Mesh( ref mesh ) = node.borrow().object
        {
          if self.attachment_buffers.contains_key( &GBufferAttachment::PbrInfo )
          {
            gl::uniform::upload( gl, object_id_loc.clone(), &*object_id.borrow() ).unwrap();
          }

          if self.attachment_buffers.contains_key( &GBufferAttachment::ObjectColor )
          {
            let object_color = if let Some( oc ) = object_colors
            {
              ( oc.get( ( *object_id.borrow() - 1 ) as usize ) ).copied().unwrap_or( F32x4::default() )
            }
            else
            {
              F32x4::default()
            };
            gl::uniform::upload( gl, object_color_loc.clone(), object_color.as_slice() ).unwrap();
          }

          // Iterate over each primitive in the mesh.
          for primitive_rc in &mesh.borrow().primitives
          {
            let primitive = primitive_rc.borrow();
            let material = primitive.material.borrow();
            let material = ( material.as_ref() as &dyn std::any::Any  ).downcast_ref::< PbrMaterial >().expect( "GBuffer only supports PbrMaterial" );

            if self.attachment_buffers.contains_key( &GBufferAttachment::Albedo )
            && self.attachment_buffers.contains_key( &GBufferAttachment::PbrInfo )
            {
              let albedo_texture = material.base_color_texture()
              .and_then(| t | t.texture.borrow().source.clone());

              if let Some( albedo_texture ) = albedo_texture
              {
                texture_upload( gl, &albedo_texture, albedo_texture_loc, GL::TEXTURE0 );
              }
            }

            if self.attachment_buffers.contains_key( &GBufferAttachment::PbrInfo )
            {
              let material_id = &material.id().to_fields_le().0;
              gl::uniform::upload( gl, material_id_loc.clone(), material_id ).unwrap();
            }

            camera_upload( gl, camera, locations );
            node.borrow().upload( gl, locations );
            primitive.geometry.borrow().bind( gl );
            primitive.draw( gl );
          }

          *object_id.borrow_mut() += 1;
        }

        Ok( () )
      };

      // Traverse the scene and draw all opaque objects.
      scene.traverse( &mut draw_node )?;

      Ok( () )
    }
  }

  // Fix(BUG-433): `GBuffer::new` created a depth `WebGlRenderbuffer` ( local `depthbuffer`
  // binding ) but never stored it on the struct, so nothing could ever delete it -- every
  // `GBuffer` construct/drop cycle ( e.g. a canvas resize that rebuilds the geometry pass at a
  // new resolution ) permanently leaked one renderbuffer, plus the VAO, the color framebuffer,
  // and every attachment texture, none of which had a matching `gl.delete*` call either.
  // Root cause: `GBuffer` never had an `impl Drop` at all -- the local `depthbuffer` variable
  // was dropped as a plain Rust value at the end of `new`'s scope save for the one field it got
  // assigned to, and the struct itself carried no cleanup path for any of its five owned GL
  // object families.
  // Pitfall: a GPU handle wrapper ( `Option< WebGlTexture >`, `WebGlFramebuffer`,
  // `WebGlRenderbuffer`, `WebGlVertexArrayObject` ) is just a JS-object reference -- letting the
  // Rust value go out of scope does not call `gl.delete*` for you; only an explicit delete call
  // (here, via `impl Drop`) reclaims the actual GPU-side allocation.
  impl Drop for GBuffer
  {
    fn drop( &mut self )
    {
      self.gl.delete_vertex_array( Some( &self.vao ) );
      self.gl.delete_framebuffer( Some( &self.framebuffer ) );
      self.gl.delete_renderbuffer( Some( &self.depthbuffer ) );
      for texture in self.textures.values()
      {
        self.gl.delete_texture( Some( texture ) );
      }
    }
  }

  // Test placement: verifying `impl Drop` actually deleted `vao`/`framebuffer`/`depthbuffer`/
  // `textures` needs the pre-drop handles, and all four fields are private -- only a test
  // nested inside `mod private` can read them. See `rulebook.md § Test placement`.
  #[ cfg( all( test, target_arch = "wasm32" ) ) ]
  mod tests
  {
    use super::*;

    fn gl_init() -> GL
    {
      gl::browser::setup( gl::browser::Config::default() );
      let options = gl::context::ContextOptions::default();
      let canvas = gl::canvas::make().unwrap();
      gl::context::from_canvas_with( &canvas, options ).unwrap()
    }

    /// ## Root Cause
    /// `GBuffer::new` created a depth `WebGlRenderbuffer` ( local `depthbuffer` binding ) but
    /// never stored it on the struct, so nothing could ever delete it -- and the struct had no
    /// `impl Drop` at all, so the VAO, color framebuffer, and every attachment texture leaked
    /// too on every construct/drop cycle ( e.g. a canvas resize rebuilding the geometry pass ).
    ///
    /// ## Why Not Caught
    /// `webgl/gbuffer.rs`'s existing test only covers `GBufferAttachment::define_const`/
    /// `attribute_info` mapping -- no test previously constructed or dropped a real `GBuffer`.
    ///
    /// ## Fix Applied
    /// `depthbuffer` is now a stored field, and `impl Drop for GBuffer` deletes the VAO,
    /// framebuffer, depthbuffer, and every texture in `textures`.
    ///
    /// ## Prevention
    /// This test captures clones of all four handle families from the private fields before
    /// drop, then asserts each `gl.is_*` check flips from `true` to `false` afterward -- the
    /// same deterministic existence-check pattern used by this crate's other GPU-teardown
    /// reproducer tests ( `shadow.rs`, `unreal_bloom.rs`, `wide_outline.rs`, `skeleton.rs` ).
    ///
    /// ## Pitfall
    /// A local variable holding a GPU handle wrapper going out of scope without ever being
    /// stored on the struct is doubly invisible -- neither a compiler warning nor a runtime
    /// signal indicates the allocation was never reachable for cleanup in the first place.
    // test_kind: bug_reproducer(BUG-433)
    #[ wasm_bindgen_test::wasm_bindgen_test ]
    fn gbuffer_drop_frees_vao_framebuffer_depthbuffer_and_textures()
    {
      let gl = gl_init();

      let mut attachment_buffers : FxHashMap< GBufferAttachment, Vec< WebGlBuffer > > = FxHashMap::default();
      attachment_buffers.insert( GBufferAttachment::Albedo, vec![] );
      attachment_buffers.insert( GBufferAttachment::PbrInfo, vec![] );
      attachment_buffers.insert( GBufferAttachment::Uv1, vec![] );

      let gbuffer = GBuffer::new( &gl, 64, 64, attachment_buffers )
      .expect( "GBuffer::new should succeed on a valid context with a minimal attachment set" );

      let vao = gbuffer.vao.clone();
      let framebuffer = gbuffer.framebuffer.clone();
      let depthbuffer = gbuffer.depthbuffer.clone();
      let textures : Vec< WebGlTexture > = gbuffer.textures.values().cloned().collect();
      assert!( !textures.is_empty(), "minimal attachment set must still allocate at least one texture" );

      assert!( gl.is_vertex_array( Some( &vao ) ) );
      assert!( gl.is_framebuffer( Some( &framebuffer ) ) );
      assert!( gl.is_renderbuffer( Some( &depthbuffer ) ) );
      for texture in &textures
      {
        assert!( gl.is_texture( Some( texture ) ) );
      }

      drop( gbuffer );

      assert!( !gl.is_vertex_array( Some( &vao ) ), "GBuffer::drop must delete its VAO" );
      assert!( !gl.is_framebuffer( Some( &framebuffer ) ), "GBuffer::drop must delete its framebuffer" );
      assert!( !gl.is_renderbuffer( Some( &depthbuffer ) ), "GBuffer::drop must delete its depthbuffer" );
      for texture in &textures
      {
        assert!( !gl.is_texture( Some( texture ) ), "GBuffer::drop must delete every attachment texture" );
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    GBuffer,
    GBufferAttachment,
    ALL
  };
}
