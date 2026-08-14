mod private
{
  use minwebgpu as gl;
  use crate::webgpu::
  {
    GpuContext,
    Geometry,
    Lights,
    LightsRaw,
    MaterialBinding,
    PbrMaterial,
    MaterialRaw
  };
  use gpu_hal::
  {
    Error,
    Device,
    Buffer,
    BufferUsage,
    TextureDesc,
    TextureFormat,
    TextureUsage,
    TextureView,
    SamplerDesc,
    FilterMode,
    AddressMode,
    Sampler,
    ShaderSource,
    ShaderStages,
    BindingType,
    BindGroupLayoutEntry,
    BindGroupLayout,
    BindGroup,
    BindingResource,
    RenderPipeline,
    RenderPipelineDesc,
    DepthState,
    ColorAttachmentDesc,
    DepthAttachmentDesc,
    IndexFormat
  };

  /// GPU layout of `CameraUniform` in `shaders/main.wgsl`.
  #[ repr( C ) ]
  #[ derive( Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable ) ]
  pub struct CameraRaw
  {
    /// Column-major view matrix.
    pub view_matrix : [ f32; 16 ],
    /// Column-major projection matrix ( 0..1 depth range ).
    pub projection_matrix : [ f32; 16 ],
    /// xyz — world-space eye position; w — exposure ( applied as `exp2` ).
    pub position_exposure : [ f32; 4 ]
  }

  /// GPU layout of `ModelUniform` in `shaders/main.wgsl`.
  ///
  /// `normal_matrix` is a `mat3x3f` in uniform space: three columns, each
  /// padded to vec4 stride.
  #[ repr( C ) ]
  #[ derive( Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable ) ]
  pub struct ModelRaw
  {
    /// Column-major world matrix.
    pub world_matrix : [ f32; 16 ],
    /// Column-major inverse-transpose of the world rotation-scale block.
    pub normal_matrix : [ f32; 12 ]
  }

  /// Per-frame camera parameters for `WebGpuRenderer::render`.
  ///
  /// The projection's clip-space depth range must match the backend's
  /// `Device::depth_range` : 0..1 on WebGPU
  /// ( e.g. `gl::math::mat3x3h::perspective_rh` ), -1..1 on WebGL
  /// ( `perspective_rh_gl` ).
  #[ derive( Debug, Clone, Copy, Default ) ]
  pub struct Frame
  {
    /// View matrix.
    pub view_matrix : gl::math::F32x4x4,
    /// Projection matrix ( 0..1 depth range ).
    pub projection_matrix : gl::math::F32x4x4,
    /// World-space eye position.
    pub eye : gl::math::F32x3,
    /// Exposure in stops ( `exp2` applied in the shader ).
    pub exposure : f32
  }

  /// One draw of a geometry with a material at a world transform.
  pub struct RenderItem
  {
    /// Mesh to draw.
    pub geometry : Geometry,
    /// Material binding for group 1.
    pub material : MaterialBinding,
    /// World transform; uploaded ( with its normal matrix ) every frame.
    pub world_matrix : gl::math::F32x4x4,
    model_buffer : Buffer,
    model_bind_group : BindGroup
  }

  /// Opaque PBR renderer: an HDR opaque pass followed by an ACES tone mapping
  /// pass onto the canvas.
  pub struct WebGpuRenderer
  {
    opaque_pipeline : RenderPipeline,
    tonemap_pipeline : RenderPipeline,
    material_layout : BindGroupLayout,
    model_layout : BindGroupLayout,
    frame_bind_group : BindGroup,
    tonemap_bind_group : BindGroup,
    camera_buffer : Buffer,
    lights_buffer : Buffer,
    hdr_view : TextureView,
    depth_view : TextureView,
    dummy_texture_view : TextureView,
    default_sampler : Sampler
  }

  /// Creates the HDR color and depth render targets sized to
  /// `width` x `height`, returning their views.
  fn frame_targets_create( device : &Device, width : u32, height : u32 )
  -> Result< ( TextureView, TextureView ), Error >
  {
    let hdr_texture = device.texture_create
    (
      &TextureDesc
      {
        size : [ width, height, 1 ],
        format : TextureFormat::Rgba16Float,
        usage : TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING
      }
    )?;
    let hdr_view = hdr_texture.view()?;

    let depth_texture = device.texture_create
    (
      &TextureDesc
      {
        size : [ width, height, 1 ],
        format : TextureFormat::Depth24Plus,
        usage : TextureUsage::RENDER_ATTACHMENT
      }
    )?;
    let depth_view = depth_texture.view()?;

    Ok( ( hdr_view, depth_view ) )
  }

  /// Creates the stand-ins bound when a material lacks a texture : a 1x1
  /// dummy texture view and a linear-filtering repeat sampler.
  fn material_defaults_create( device : &Device ) -> Result< ( TextureView, Sampler ), Error >
  {
    // 1x1 stand-in for absent material textures. Its contents are never
    // read: the shader samples a slot only when its flag bit is set.
    let dummy_texture = device.texture_create
    (
      &TextureDesc
      {
        size : [ 1, 1, 1 ],
        format : TextureFormat::Rgba8Unorm,
        usage : TextureUsage::TEXTURE_BINDING
      }
    )?;
    let dummy_texture_view = dummy_texture.view()?;

    let default_sampler = device.sampler_create
    (
      SamplerDesc { filter : FilterMode::Linear, address : AddressMode::Repeat }
    )?;

    Ok( ( dummy_texture_view, default_sampler ) )
  }

  /// Compiles the PBR shader and builds the opaque HDR pass pipeline over the
  /// frame/material/model bind group layouts.
  fn opaque_pipeline_create
  (
    device : &Device,
    frame_layout : &BindGroupLayout,
    material_layout : &BindGroupLayout,
    model_layout : &BindGroupLayout
  )
  -> Result< RenderPipeline, Error >
  {
    let main_shader = device.shader_module_create
    (
      &ShaderSource
      {
        wgsl : include_str!( "shaders/main.wgsl" ),
        glsl_vertex : Some( include_str!( "shaders/main.vert.glsl" ) ),
        glsl_fragment : Some( include_str!( "shaders/main.frag.glsl" ) )
      }
    )?;

    let vertex_layouts = Geometry::vertex_layouts();

    device.render_pipeline_create
    (
      &RenderPipelineDesc
      {
        shader : &main_shader,
        vertex_entry : "vs_main",
        fragment_entry : "fs_main",
        vertex_buffers : &vertex_layouts,
        bind_group_layouts : &[ frame_layout, material_layout, model_layout ],
        color_format : TextureFormat::Rgba16Float,
        depth : Some( DepthState { format : TextureFormat::Depth24Plus } ),
        cull_back : true
      }
    )
  }

  /// Compiles the ACES tone mapping shader and builds the fullscreen pass
  /// pipeline targeting `color_format` ( the surface's own format ).
  fn tonemap_pipeline_create
  (
    device : &Device,
    tonemap_layout : &BindGroupLayout,
    color_format : TextureFormat
  )
  -> Result< RenderPipeline, Error >
  {
    let tonemap_shader = device.shader_module_create
    (
      &ShaderSource
      {
        wgsl : include_str!( "shaders/tonemap.wgsl" ),
        glsl_vertex : Some( include_str!( "shaders/tonemap.vert.glsl" ) ),
        glsl_fragment : Some( include_str!( "shaders/tonemap.frag.glsl" ) )
      }
    )?;

    device.render_pipeline_create
    (
      &RenderPipelineDesc
      {
        shader : &tonemap_shader,
        vertex_entry : "vs_main",
        fragment_entry : "fs_main",
        vertex_buffers : &[],
        bind_group_layouts : &[ tonemap_layout ],
        color_format,
        depth : None,
        cull_back : false
      }
    )
  }

  impl WebGpuRenderer
  {
    /// Builds pipelines and frame targets sized to the context's current
    /// surface size.
    ///
    /// # Errors
    ///
    /// Returns an error when shader compilation, pipeline creation, or GPU
    /// resource allocation fails on the device.
    pub fn new( context : &GpuContext ) -> Result< Self, Error >
    {
      let device = &context.device;
      let [ width, height ] = context.size();

      // Group 0 — camera ( vertex + fragment ) and lights ( fragment ).
      let frame_layout = device.bind_group_layout_create
      (
        &[
          BindGroupLayoutEntry
          {
            visibility : ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty : BindingType::UniformBuffer
          },
          BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::UniformBuffer }
        ]
      )?;

      // Group 1 — material uniform + base color and metallic-roughness
      // texture/sampler pairs, all fragment-stage.
      let material_layout = device.bind_group_layout_create
      (
        &[
          BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::UniformBuffer },
          BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::Texture },
          BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::Sampler },
          BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::Texture },
          BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::Sampler }
        ]
      )?;

      // Group 2 — model uniform, vertex-stage.
      let model_layout = device.bind_group_layout_create
      (
        &[ BindGroupLayoutEntry { visibility : ShaderStages::VERTEX, ty : BindingType::UniformBuffer } ]
      )?;

      // Tone mapping group 0 — the HDR color target as a plain texture.
      let tonemap_layout = device.bind_group_layout_create
      (
        &[ BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::Texture } ]
      )?;

      let camera_buffer = device.buffer_create
      (
        core::mem::size_of::< CameraRaw >() as u64,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST
      )?;
      let lights_buffer = device.buffer_create
      (
        core::mem::size_of::< LightsRaw >() as u64,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST
      )?;

      let ( hdr_view, depth_view ) = frame_targets_create( device, width, height )?;
      let ( dummy_texture_view, default_sampler ) = material_defaults_create( device )?;

      let opaque_pipeline =
        opaque_pipeline_create( device, &frame_layout, &material_layout, &model_layout )?;
      let tonemap_pipeline =
        tonemap_pipeline_create( device, &tonemap_layout, context.surface.format() )?;

      let frame_bind_group = device.bind_group_create
      (
        &frame_layout,
        &[ BindingResource::Buffer( &camera_buffer ), BindingResource::Buffer( &lights_buffer ) ]
      )?;

      let tonemap_bind_group = device.bind_group_create
      (
        &tonemap_layout,
        &[ BindingResource::TextureView( &hdr_view ) ]
      )?;

      Ok
      (
        Self
        {
          opaque_pipeline,
          tonemap_pipeline,
          material_layout,
          model_layout,
          frame_bind_group,
          tonemap_bind_group,
          camera_buffer,
          lights_buffer,
          hdr_view,
          depth_view,
          dummy_texture_view,
          default_sampler
        }
      )
    }

    /// Uploads `material` into a fresh uniform buffer + bind group.
    ///
    /// Absent textures are bound to a 1x1 dummy; the shader samples a slot
    /// only when the corresponding flag bit is set.
    ///
    /// # Errors
    ///
    /// Returns an error when buffer allocation, the uniform upload, or bind-group
    /// creation fails.
    pub fn material_binding_create
    (
      &self,
      context : &GpuContext,
      material : &PbrMaterial
    ) -> Result< MaterialBinding, Error >
    {
      let buffer = context.device.buffer_create
      (
        core::mem::size_of::< MaterialRaw >() as u64,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST
      )?;
      context.queue.buffer_write( &buffer, bytemuck::bytes_of( &material.as_raw() ) )?;

      let base_color_view = material.base_color_texture.as_ref().unwrap_or( &self.dummy_texture_view );
      let mr_view = material.metallic_roughness_texture.as_ref().unwrap_or( &self.dummy_texture_view );

      let bind_group = context.device.bind_group_create
      (
        &self.material_layout,
        &[
          BindingResource::Buffer( &buffer ),
          BindingResource::TextureView( base_color_view ),
          BindingResource::Sampler( &self.default_sampler ),
          BindingResource::TextureView( mr_view ),
          BindingResource::Sampler( &self.default_sampler )
        ]
      )?;

      Ok( MaterialBinding { buffer, bind_group } )
    }

    /// Wraps a geometry + material into a draw item with its own model
    /// uniform. The model uniform is rewritten from `world_matrix` on every
    /// `render` call.
    ///
    /// # Errors
    ///
    /// Returns an error when the model uniform buffer or its bind group cannot be created.
    pub fn item_create
    (
      &self,
      context : &GpuContext,
      geometry : Geometry,
      material : MaterialBinding,
      world_matrix : gl::math::F32x4x4
    ) -> Result< RenderItem, Error >
    {
      let model_buffer = context.device.buffer_create
      (
        core::mem::size_of::< ModelRaw >() as u64,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST
      )?;
      let model_bind_group = context.device.bind_group_create
      (
        &self.model_layout,
        &[ BindingResource::Buffer( &model_buffer ) ]
      )?;

      Ok
      (
        RenderItem
        {
          geometry,
          material,
          world_matrix,
          model_buffer,
          model_bind_group
        }
      )
    }

    fn model_raw( world : &gl::math::F32x4x4 ) -> ModelRaw
    {
      let rotation_scale = world.truncate();
      // Singular world matrices fall back to the untransposed block — same
      // degenerate result the WebGL node path would produce lighting-wise.
      let normal = rotation_scale.inverse().map_or( rotation_scale, | m | m.transpose() );
      let n = normal.to_array();

      ModelRaw
      {
        world_matrix : world.to_array(),
        normal_matrix :
        [
          n[ 0 ], n[ 1 ], n[ 2 ], 0.0,
          n[ 3 ], n[ 4 ], n[ 5 ], 0.0,
          n[ 6 ], n[ 7 ], n[ 8 ], 0.0
        ]
      }
    }

    /// Renders `items`: the opaque pass into the HDR target, then the tone
    /// mapping pass onto the surface's current texture ( the canvas in the
    /// browser, the offscreen readback target natively ).
    ///
    /// # Errors
    ///
    /// Returns an error when a uniform upload fails, the surface's current texture
    /// cannot be acquired, or a render pass cannot be started.
    pub fn render
    (
      &self,
      context : &GpuContext,
      frame : &Frame,
      lights : &Lights,
      items : &[ RenderItem ]
    ) -> Result< (), Error >
    {
      let eye = frame.eye.to_array();
      let camera_raw = CameraRaw
      {
        view_matrix : frame.view_matrix.to_array(),
        projection_matrix : frame.projection_matrix.to_array(),
        position_exposure : [ eye[ 0 ], eye[ 1 ], eye[ 2 ], frame.exposure ]
      };
      context.queue.buffer_write( &self.camera_buffer, bytemuck::bytes_of( &camera_raw ) )?;
      context.queue.buffer_write( &self.lights_buffer, bytemuck::bytes_of( &lights.as_raw() ) )?;
      for item in items
      {
        context.queue.buffer_write( &item.model_buffer, bytemuck::bytes_of( &Self::model_raw( &item.world_matrix ) ) )?;
      }

      let canvas_view = context.surface.current_view()?;

      let mut encoder = context.device.command_encoder_create();

      {
        // Color clears to ( 0, 0, 0, 0 ) — alpha 0 marks background for the
        // tone mapping bypass — and depth clears to 1.0.
        let mut opaque_pass = encoder.render_pass_begin
        (
          &ColorAttachmentDesc { view : &self.hdr_view, clear : [ 0.0, 0.0, 0.0, 0.0 ] },
          Some( &DepthAttachmentDesc { view : &self.depth_view } )
        )?;

        opaque_pass.pipeline_set( &self.opaque_pipeline );
        opaque_pass.bind_group_set( 0, &self.frame_bind_group );

        for item in items
        {
          opaque_pass.bind_group_set( 1, &item.material.bind_group );
          opaque_pass.bind_group_set( 2, &item.model_bind_group );
          for ( slot, buffer ) in item.geometry.vertex_buffers.iter().enumerate()
          {
            opaque_pass.vertex_buffer_set( slot as u32, buffer );
          }
          match &item.geometry.index_buffer
          {
            Some( index_buffer ) =>
            {
              opaque_pass.index_buffer_set( index_buffer, IndexFormat::Uint32 );
              opaque_pass.draw_indexed( item.geometry.index_count );
            }
            None => opaque_pass.draw( item.geometry.vertex_count )
          }
        }

        opaque_pass.end();
      }

      {
        let mut tonemap_pass = encoder.render_pass_begin
        (
          &ColorAttachmentDesc { view : &canvas_view, clear : [ 0.0, 0.0, 0.0, 0.0 ] },
          None
        )?;

        tonemap_pass.pipeline_set( &self.tonemap_pipeline );
        tonemap_pass.bind_group_set( 0, &self.tonemap_bind_group );
        tonemap_pass.draw( 3 );
        tonemap_pass.end();
      }

      context.queue.submit( encoder );
      Ok( () )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    CameraRaw,
    ModelRaw,
    Frame,
    RenderItem,
    WebGpuRenderer
  };
}
