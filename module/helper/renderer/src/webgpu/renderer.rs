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

  impl WebGpuRenderer
  {
    /// Builds pipelines and frame targets sized to the canvas' current size.
    pub fn new( context : &GpuContext ) -> Result< Self, Error >
    {
      let device = &context.device;
      let width = context.canvas.width();
      let height = context.canvas.height();

      // Group 0 — camera ( vertex + fragment ) and lights ( fragment ).
      let frame_layout = device.create_bind_group_layout
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
      let material_layout = device.create_bind_group_layout
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
      let model_layout = device.create_bind_group_layout
      (
        &[ BindGroupLayoutEntry { visibility : ShaderStages::VERTEX, ty : BindingType::UniformBuffer } ]
      )?;

      // Tone mapping group 0 — the HDR color target as a plain texture.
      let tonemap_layout = device.create_bind_group_layout
      (
        &[ BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::Texture } ]
      )?;

      let camera_buffer = device.create_buffer
      (
        core::mem::size_of::< CameraRaw >() as u64,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST
      )?;
      let lights_buffer = device.create_buffer
      (
        core::mem::size_of::< LightsRaw >() as u64,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST
      )?;

      let hdr_texture = device.create_texture
      (
        &TextureDesc
        {
          size : [ width, height, 1 ],
          format : TextureFormat::Rgba16Float,
          usage : TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING
        }
      )?;
      let hdr_view = hdr_texture.view()?;

      let depth_texture = device.create_texture
      (
        &TextureDesc
        {
          size : [ width, height, 1 ],
          format : TextureFormat::Depth24Plus,
          usage : TextureUsage::RENDER_ATTACHMENT
        }
      )?;
      let depth_view = depth_texture.view()?;

      // 1x1 stand-in for absent material textures. Its contents are never
      // read: the shader samples a slot only when its flag bit is set.
      let dummy_texture = device.create_texture
      (
        &TextureDesc
        {
          size : [ 1, 1, 1 ],
          format : TextureFormat::Rgba8Unorm,
          usage : TextureUsage::TEXTURE_BINDING
        }
      )?;
      let dummy_texture_view = dummy_texture.view()?;

      let default_sampler = device.create_sampler
      (
        SamplerDesc { filter : FilterMode::Linear, address : AddressMode::Repeat }
      )?;

      let main_shader = device.create_shader_module
      (
        &ShaderSource
        {
          wgsl : include_str!( "shaders/main.wgsl" ),
          glsl_vertex : Some( include_str!( "shaders/main.vert.glsl" ) ),
          glsl_fragment : Some( include_str!( "shaders/main.frag.glsl" ) )
        }
      )?;
      let tonemap_shader = device.create_shader_module
      (
        &ShaderSource
        {
          wgsl : include_str!( "shaders/tonemap.wgsl" ),
          glsl_vertex : Some( include_str!( "shaders/tonemap.vert.glsl" ) ),
          glsl_fragment : Some( include_str!( "shaders/tonemap.frag.glsl" ) )
        }
      )?;

      let vertex_layouts = Geometry::vertex_layouts();

      let opaque_pipeline = device.create_render_pipeline
      (
        &RenderPipelineDesc
        {
          shader : &main_shader,
          vertex_entry : "vs_main",
          fragment_entry : "fs_main",
          vertex_buffers : &vertex_layouts,
          bind_group_layouts : &[ &frame_layout, &material_layout, &model_layout ],
          color_format : TextureFormat::Rgba16Float,
          depth : Some( DepthState { format : TextureFormat::Depth24Plus } ),
          cull_back : true
        }
      )?;

      let tonemap_pipeline = device.create_render_pipeline
      (
        &RenderPipelineDesc
        {
          shader : &tonemap_shader,
          vertex_entry : "vs_main",
          fragment_entry : "fs_main",
          vertex_buffers : &[],
          bind_group_layouts : &[ &tonemap_layout ],
          color_format : context.surface.format(),
          depth : None,
          cull_back : false
        }
      )?;

      let frame_bind_group = device.create_bind_group
      (
        &frame_layout,
        &[ BindingResource::Buffer( &camera_buffer ), BindingResource::Buffer( &lights_buffer ) ]
      )?;

      let tonemap_bind_group = device.create_bind_group
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
    pub fn create_material_binding
    (
      &self,
      context : &GpuContext,
      material : &PbrMaterial
    ) -> Result< MaterialBinding, Error >
    {
      let buffer = context.device.create_buffer
      (
        core::mem::size_of::< MaterialRaw >() as u64,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST
      )?;
      context.queue.write_buffer( &buffer, bytemuck::bytes_of( &material.as_raw() ) )?;

      let base_color_view = material.base_color_texture.as_ref().unwrap_or( &self.dummy_texture_view );
      let mr_view = material.metallic_roughness_texture.as_ref().unwrap_or( &self.dummy_texture_view );

      let bind_group = context.device.create_bind_group
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
    pub fn create_item
    (
      &self,
      context : &GpuContext,
      geometry : Geometry,
      material : MaterialBinding,
      world_matrix : gl::math::F32x4x4
    ) -> Result< RenderItem, Error >
    {
      let model_buffer = context.device.create_buffer
      (
        core::mem::size_of::< ModelRaw >() as u64,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST
      )?;
      let model_bind_group = context.device.create_bind_group
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
    /// mapping pass onto the canvas' current texture.
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
      context.queue.write_buffer( &self.camera_buffer, bytemuck::bytes_of( &camera_raw ) )?;
      context.queue.write_buffer( &self.lights_buffer, bytemuck::bytes_of( &lights.as_raw() ) )?;
      for item in items
      {
        context.queue.write_buffer( &item.model_buffer, bytemuck::bytes_of( &Self::model_raw( &item.world_matrix ) ) )?;
      }

      let canvas_view = context.surface.current_view()?;

      let encoder = context.device.create_command_encoder();

      {
        // Color clears to ( 0, 0, 0, 0 ) — alpha 0 marks background for the
        // tone mapping bypass — and depth clears to 1.0.
        let opaque_pass = encoder.begin_render_pass
        (
          &ColorAttachmentDesc { view : &self.hdr_view, clear : [ 0.0, 0.0, 0.0, 0.0 ] },
          Some( &DepthAttachmentDesc { view : &self.depth_view } )
        )?;

        opaque_pass.set_pipeline( &self.opaque_pipeline );
        opaque_pass.set_bind_group( 0, &self.frame_bind_group );

        for item in items
        {
          opaque_pass.set_bind_group( 1, &item.material.bind_group );
          opaque_pass.set_bind_group( 2, &item.model_bind_group );
          for ( slot, buffer ) in item.geometry.vertex_buffers.iter().enumerate()
          {
            opaque_pass.set_vertex_buffer( slot as u32, buffer );
          }
          match &item.geometry.index_buffer
          {
            Some( index_buffer ) =>
            {
              opaque_pass.set_index_buffer( index_buffer, IndexFormat::Uint32 );
              opaque_pass.draw_indexed( item.geometry.index_count );
            }
            None => opaque_pass.draw( item.geometry.vertex_count )
          }
        }

        opaque_pass.end();
      }

      {
        let tonemap_pass = encoder.begin_render_pass
        (
          &ColorAttachmentDesc { view : &canvas_view, clear : [ 0.0, 0.0, 0.0, 0.0 ] },
          None
        )?;

        tonemap_pass.set_pipeline( &self.tonemap_pipeline );
        tonemap_pass.set_bind_group( 0, &self.tonemap_bind_group );
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
