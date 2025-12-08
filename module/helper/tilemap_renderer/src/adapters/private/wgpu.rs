//! This module provides a `wgpu`-based rendering engine specifically designed for a 2D tile-based system.
//! It encapsulates the `wgpu` context, rendering pipelines, and resource management for textures and geometry,
//! executing a command-based rendering workflow.

use minwgpu::{ buffer, context, helper, texture };
use crate::{ commands, ports };

/// The offscreen renderer struct that encapsulates the `wgpu` context and for simple tile rendering.
///
/// # Limitations
///
/// Resolution of result image should be multiple of 256. So that width and height defined in a providing `RenderContext`
/// should be multiples of 256.
///
/// # Performance
///
/// This renderer is sutied for offscreen rendering and not intended for realtime usage.
///
/// # Example
///
/// ```rust
/// use tilemap_renderer::adapters::WGPUTileRenderer;
/// use tilemap_renderer::{ commands, ports::RenderContext };
/// use commands::{ Geometry2DCommand, Point2D, RenderCommand, Transform2D };
///
/// let mut renderer = WGPUTileRenderer::new
/// (
///   wgpu::Backends::PRIMARY,
///   RenderContext::new( 256, 256, [ 0.0; 4 ], true, Point2D::new( 0.0, 0.0 ), 1.0 )
/// ).unwrap();
///
/// let line = &[ 0.0_f32, 0.0, 1.0, 1.0 ];
/// renderer.geometry2d_load( line, 0 );
/// let res = renderer.commands_execute
/// (
///   &[
///     RenderCommand::Geometry2DCommand
///     (
///       Geometry2DCommand
///       {
///         id : 0,
///         transform : Transform2D::default(),
///         color : [ 1.0; 3 ],
///         mode : commands::GeometryMode::Lines
///       }
///     )
///   ]
/// );
/// ```
#[ derive( Debug ) ]
pub struct WGPUTileRenderer
{
  context : context::Context,
  texture_bindgroup_layout : wgpu::BindGroupLayout,
  texture_sampler : wgpu::Sampler,
  textures : rustc_hash::FxHashMap< u32, ( texture::Texture, wgpu::BindGroup ) >,
  geometry2d : rustc_hash::FxHashMap< u32, ( wgpu::Buffer, u32 ) >,
  geometry2d_pipeline : wgpu::RenderPipeline,
  line2d_pipeline : wgpu::RenderPipeline,
  sprite_pipeline : wgpu::RenderPipeline,
  render_context : ports::RenderContext,
}

impl WGPUTileRenderer
{
  const GEOMETRY2D_ATTRIBUTES : &[ wgpu::VertexAttribute ] = &[ helper::attr( wgpu::VertexFormat::Float32x2, 0, 0 ) ];

  /// Creates new renderer instance. `width` and `height` in the providing context must be multiples of 256.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the rendering backend cannot be initialized (e.g., no suitable adapter or device found).
  pub fn new( backends : wgpu::Backends, render_context : ports::RenderContext ) -> Result< Self, ports::RenderError >
  {
    let context = context::Context::builder()
    .backends( backends )
    .make_instance()
    .power_preference( wgpu::PowerPreference::HighPerformance )
    .request_adapter()
    .map_err( | e | ports::RenderError::InitializationFailed( format!( "{e}" ) ) )?
    .label( "device" )
    .required_features( wgpu::Features::PUSH_CONSTANTS )
    .required_limits( wgpu::Limits { max_push_constant_size : 44, ..Default::default() } )
    .finish_context()
    .map_err( | e | ports::RenderError::InitializationFailed( format!( "{e}" ) ) )?;

    let geom2d_shader = context.get_device().create_shader_module( wgpu::include_wgsl!( "../../../shaders/geom2d.wgsl" ) );
    let sprite_shader = context.get_device().create_shader_module( wgpu::include_wgsl!( "../../../shaders/sprite.wgsl" ) );

    let texture_bindgroup_layout = context.get_device().create_bind_group_layout
    (
      &wgpu::BindGroupLayoutDescriptor
      {
        label : Some( "texture_bindgroup_layout" ),
        entries :
        &[
          wgpu::BindGroupLayoutEntry
          {
            binding : 0,
            visibility : wgpu::ShaderStages::FRAGMENT,
            ty : wgpu::BindingType::Texture
            {
              multisampled : false,
              view_dimension : wgpu::TextureViewDimension::D2,
              sample_type : wgpu::TextureSampleType::Float { filterable : true },
            },
            count : None,
          },
          wgpu::BindGroupLayoutEntry
          {
            binding : 1,
            visibility : wgpu::ShaderStages::FRAGMENT,
            ty : wgpu::BindingType::Sampler( wgpu::SamplerBindingType::Filtering ),
            count : None,
          },
        ],
      }
    );

    let sampler = context.get_device().create_sampler
    (
      &wgpu::SamplerDescriptor
      {
        address_mode_u : wgpu::AddressMode::ClampToEdge,
        address_mode_v : wgpu::AddressMode::ClampToEdge,
        address_mode_w : wgpu::AddressMode::ClampToEdge,
        mag_filter : wgpu::FilterMode::Linear,
        min_filter : wgpu::FilterMode::Linear,
        mipmap_filter : wgpu::FilterMode::Nearest,
        ..Default::default()
      }
    );

    let geometry2d_layout = wgpu::VertexBufferLayout
    {
      array_stride : wgpu::VertexFormat::Float32x2.size(),
      step_mode : wgpu::VertexStepMode::Vertex,
      attributes : Self::GEOMETRY2D_ATTRIBUTES,
    };

    let geometry_pipeline_layout = context.get_device().create_pipeline_layout
    (
      &wgpu::PipelineLayoutDescriptor
      {
        label : Some( "geometry_pipeline" ),
        bind_group_layouts : &[],
        push_constant_ranges : &
        [
          wgpu::PushConstantRange { stages : wgpu::ShaderStages::VERTEX, range : 0..32 },
          wgpu::PushConstantRange { stages : wgpu::ShaderStages::FRAGMENT, range : 32..44 }
        ]
      }
    );

    let sprite_pipeline_layout = context.get_device().create_pipeline_layout
    (
      &wgpu::PipelineLayoutDescriptor
      {
        label : Some( "sprite_pipeline" ),
        bind_group_layouts : &[ &texture_bindgroup_layout ],
        push_constant_ranges : &[ wgpu::PushConstantRange { stages : wgpu::ShaderStages::VERTEX, range : 0..32 } ]
      }
    );

    let geometry2d_pipeline = context.get_device().create_render_pipeline
    (
      &wgpu::RenderPipelineDescriptor
      {
        label : Some( "geometry_pipeline" ),
        layout : Some( &geometry_pipeline_layout ),
        vertex : wgpu::VertexState
        {
          module : &geom2d_shader,
          entry_point : Some( "vs_main" ),
          compilation_options : wgpu::PipelineCompilationOptions::default(),
          buffers : &[ geometry2d_layout.clone() ]
        },
        primitive : wgpu::PrimitiveState::default(),
        depth_stencil : None,
        multisample : wgpu::MultisampleState::default(),
        fragment : Some
        (
          wgpu::FragmentState
          {
            module : &geom2d_shader,
            entry_point: Some( "fs_main" ),
            compilation_options : wgpu::PipelineCompilationOptions::default(),
            targets :
            &[
              Some
              (
                wgpu::ColorTargetState
                {
                  format : wgpu::TextureFormat::Rgba8UnormSrgb,
                  blend : Some( wgpu::BlendState::REPLACE ),
                  write_mask : wgpu::ColorWrites::ALL
                }
              )
            ]
          }
        ),
        multiview : None,
        cache : None
      }
    );

    let line2d_pipeline = context.get_device().create_render_pipeline
    (
      &wgpu::RenderPipelineDescriptor
      {
        label : Some( "line_pipeline" ),
        layout : Some( &geometry_pipeline_layout ),
        vertex : wgpu::VertexState
        {
          module : &geom2d_shader,
          entry_point : Some( "vs_main" ),
          compilation_options : wgpu::PipelineCompilationOptions::default(),
          buffers : &[ geometry2d_layout ]
        },
        primitive : wgpu::PrimitiveState
        {
          topology : wgpu::PrimitiveTopology::LineList,
          ..Default::default()
        },
        depth_stencil : None,
        multisample : wgpu::MultisampleState::default(),
        fragment : Some
        (
          wgpu::FragmentState
          {
            module : &geom2d_shader,
            entry_point: Some( "fs_main" ),
            compilation_options : wgpu::PipelineCompilationOptions::default(),
            targets :
            &[
              Some
              (
                wgpu::ColorTargetState
                {
                  format : wgpu::TextureFormat::Rgba8UnormSrgb,
                  blend : Some( wgpu::BlendState::REPLACE ),
                  write_mask : wgpu::ColorWrites::ALL
                }
              )
            ]
          }
        ),
        multiview : None,
        cache : None
      }
    );

    let sprite_pipeline = context.get_device().create_render_pipeline
    (
      &wgpu::RenderPipelineDescriptor
      {
        label : Some( "geometry_pipeline" ),
        layout : Some( &sprite_pipeline_layout ),
        vertex : wgpu::VertexState
        {
          module : &sprite_shader,
          entry_point : Some( "vs_main" ),
          compilation_options : wgpu::PipelineCompilationOptions::default(),
          buffers : &[]
        },
        primitive : wgpu::PrimitiveState
        {
          topology : wgpu::PrimitiveTopology::TriangleStrip,
          ..Default::default()
        },
        depth_stencil : None,
        multisample : wgpu::MultisampleState::default(),
        fragment : Some
        (
          wgpu::FragmentState
          {
            module : &sprite_shader,
            entry_point: Some( "fs_main" ),
            compilation_options : wgpu::PipelineCompilationOptions::default(),
            targets :
            &[
              Some
              (
                wgpu::ColorTargetState
                {
                  format : wgpu::TextureFormat::Rgba8UnormSrgb,
                  blend : Some( wgpu::BlendState::ALPHA_BLENDING ),
                  write_mask : wgpu::ColorWrites::ALL
                }
              )
            ]
          }
        ),
        multiview : None,
        cache : None
      }
    );

    Ok
    (
      Self
      {
        context,
        texture_bindgroup_layout,
        texture_sampler : sampler,
        textures : rustc_hash::FxHashMap::default(),
        geometry2d : rustc_hash::FxHashMap::default(),
        geometry2d_pipeline,
        line2d_pipeline,
        sprite_pipeline,
        render_context,
      }
    )
  }

  /// Creates a texture and loads `data` into it, storing it internally by the provided `id`.
  ///
  /// Data is expected to be in RGBA8 format. This `id` can be used to render the texture later.
  /// If a texture with the same `id` already exists, it is replaced.
  pub fn texture_load( &mut self, data : &[ u8 ], width : u32, height : u32, id : u32 )
  {
    let extent = wgpu::Extent3d
    {
      width,
      height,
      depth_or_array_layers : 1,
    };

    let texture = self.context.get_device().create_texture
    (
      &wgpu::TextureDescriptor
      {
        label : None,
        size : extent,
        mip_level_count : 1,
        sample_count : 1,
        dimension : wgpu::TextureDimension::D2,
        format : wgpu::TextureFormat::Rgba8UnormSrgb,
        usage : wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats : &[],
      }
    );

    self.context.get_queue().write_texture
    (
      wgpu::TexelCopyTextureInfo
      {
        texture: &texture,
        mip_level: 0,
        origin : wgpu::Origin3d::ZERO,
        aspect : wgpu::TextureAspect::All,
      },
      data,
      wgpu::TexelCopyBufferLayout
      {
        offset : 0,
        bytes_per_row : Some( 4 * width ),
        rows_per_image : Some( height ),
      },
      extent,
    );

    self.context.get_queue().submit( [] );

    let view = texture.create_view( &wgpu::wgt::TextureViewDescriptor::default() );
    let texture = texture::Texture::new( texture, extent, view, self.texture_sampler.clone() );

    let texture_bindgroup = self.context.get_device().create_bind_group
    (
      &wgpu::BindGroupDescriptor
      {
        label : None,
        layout : &self.texture_bindgroup_layout,
        entries :
        &[
          wgpu::BindGroupEntry
          {
            binding : 0,
            resource : wgpu::BindingResource::TextureView( &texture.view ),
          },
          wgpu::BindGroupEntry
          {
            binding : 1,
            resource : wgpu::BindingResource::Sampler( &self.texture_sampler.clone() ),
          },
        ],
      }
    );

    _ = self.textures.insert( id, ( texture, texture_bindgroup ) );
  }

  /// Creates a vertex buffer, loads `data` into it, and stores it internally by the provided `id`.
  /// Expects an array of 2D `f32` points.
  ///
  /// The `id` can be used to render the geometry later.
  /// If geometry with the same `id` already exists, it is replaced.
  pub fn geometry2d_load( &mut self, data : &[ f32 ], id : u32 )
  {
    let buf = buffer::buffer( wgpu::BufferUsages::VERTEX )
    .data( data )
    .build( self.context.get_device() );
    let vertex_count = data.len() as u32 / 2;
    _ = self.geometry2d.insert( id, ( buf, vertex_count ) );
  }

  /// Executes a list of render commands and returns the resulting image.
  ///
  /// This function performs an off-screen render pass based on the provided commands
  /// and the current `RenderContext`, then copies the result from the GPU to a CPU buffer.
  ///
  /// Currently supports only `RenderCommand::Geometry2DCommand`, `RenderCommand::SpriteCommand`
  /// commands. In case of facing an unsupported command in the command buffer just ignores it.
  ///
  /// # Returns
  /// A `Vec<u8>` containing the RGBA8 pixel data of the rendered image.
  ///
  /// # Panics
  /// Panics if it fails to poll the GPU device, which may happen if rendering takes too long or fails.
  pub fn commands_execute( &self, commands : &[ commands::RenderCommand ] ) -> Vec< u8 >
  {
    let ctx = &self.render_context;
    let width = ctx.width;
    let height = ctx.height;
    let scale = ctx.viewport_scale;
    let camera_pos = [ ctx.viewport_offset.x, ctx.viewport_offset.y ];
    let aspect_scale = if width > height
    {
      [ scale, ( width as f32 / height as f32 ) * scale ]
    }
    else
    {
      [ ( height as f32 / width as f32 ) * scale, scale ]
    };

    let texture_extent = wgpu::Extent3d
    {
      width,
      height,
      depth_or_array_layers : 1,
    };
    let render_texture = self.context.get_device().create_texture
    (
      &wgpu::TextureDescriptor
      {
        label : Some( "render_texture" ),
        size : texture_extent,
        mip_level_count : 1,
        sample_count : 1,
        dimension : wgpu::TextureDimension::D2,
        format : wgpu::TextureFormat::Rgba8UnormSrgb,
        usage : wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats : &[],
      }
    );
    let rendertarget_view = render_texture.create_view( &wgpu::TextureViewDescriptor::default() );

    let bytes_per_pixel = 4;
    let buffer_size = bytes_per_pixel * width * height;
    let output_buffer_size = wgpu::BufferAddress::from( buffer_size );
    let output_buffer = buffer::buffer
    (
      wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ
    )
    .label( "output_buffer" )
    .size_from_value( output_buffer_size )
    .build( self.context.get_device() );

    let op = if ctx.clear_background
    {
      let [ r, g, b, a ] = ctx.background_color;
      wgpu::LoadOp::Clear( wgpu::Color { r : r as f64, g : g as f64, b : b as f64, a : a as f64 } )
    }
    else
    {
      wgpu::LoadOp::Load
    };

    let renderpass_desc = &wgpu::RenderPassDescriptor
    {
      label : Some( "sprite_render_pass" ),
      color_attachments :
      &[
        Some
        (
          wgpu::RenderPassColorAttachment
          {
            view : &rendertarget_view,
            resolve_target : None,
            ops : wgpu::Operations
            {
              load : op,
              store : wgpu::StoreOp::Store,
            },
            depth_slice : None,
          }
        )
      ],
      depth_stencil_attachment : None,
      timestamp_writes : None,
      occlusion_query_set : None,
    };

    let mut encoder = self.context.get_device()
    .create_command_encoder( &wgpu::CommandEncoderDescriptor { label : Some( "encoder" ) } );

    {
      let mut renderpass = encoder.begin_render_pass( &renderpass_desc );

      for command in commands
      {
        match command
        {
          commands::RenderCommand::Geometry2DCommand( command ) =>
          {
            self.geometry2d_draw( &mut renderpass, &command, camera_pos, aspect_scale )
          }
          commands::RenderCommand::SpriteCommand( command ) =>
          {
            self.sprite_draw( &mut renderpass, &command, camera_pos, aspect_scale );
          },
          commands::RenderCommand::Line( _ ) |
          commands::RenderCommand::Curve( _ ) |
          commands::RenderCommand::Text( _ ) |
          commands::RenderCommand::Tilemap( _ ) |
          commands::RenderCommand::ParticleEmitter( _ ) => {}
        }
      }
    }

    encoder.copy_texture_to_buffer
    (
      render_texture.as_image_copy(),
      wgpu::TexelCopyBufferInfo
      {
        buffer : &output_buffer,
        layout : wgpu::TexelCopyBufferLayout
        {
          offset : 0,
          bytes_per_row : Some( width * bytes_per_pixel ),
          rows_per_image : None
        }
      },
      texture_extent
    );

    self.context.get_queue().submit( Some( encoder.finish() ) );

    let buffer_slice = output_buffer.slice( .. );
    buffer_slice.map_async( wgpu::MapMode::Read, | _ | {} );
    self.context.get_device().poll( wgpu::PollType::Wait ).expect( "Failed to render an image" );

    let data = buffer_slice.get_mapped_range();
    data.to_owned()
  }

  fn geometry2d_draw
  (
    &self,
    renderpass : &mut wgpu::RenderPass< '_ >,
    command : &commands::Geometry2DCommand,
    camera_pos : [ f32; 2 ],
    aspect_scale : [ f32; 2 ]
  )
  {
    let pipeline = match command.mode
    {
      commands::GeometryMode::Triangles => &self.geometry2d_pipeline,
      commands::GeometryMode::Lines => &self.line2d_pipeline,
    };
    let ( buffer, vertex_count ) = &self.geometry2d[ &command.id ];
    renderpass.set_pipeline( pipeline );
    renderpass.set_vertex_buffer( 0, buffer.slice( .. ) );

    let [ pos_x, pos_y ] = command.transform.position;
    let [ cam_pos_x, cam_pos_y ] = camera_pos;

    let pc = PushConstant
    {
      aspect_scale,
      translation : [ pos_x + cam_pos_x, pos_y + cam_pos_y ],
      rotation_cos_sin : [ command.transform.rotation.cos(), command.transform.rotation.sin() ],
      scale : command.transform.scale,
    };
    renderpass.set_push_constants( wgpu::ShaderStages::VERTEX, 0, bytemuck::bytes_of( &pc ) );
    renderpass.set_push_constants( wgpu::ShaderStages::FRAGMENT, 32, bytemuck::bytes_of( &command.color ) );
    renderpass.draw( 0..*vertex_count, 0..1 );
  }

  fn sprite_draw
  (
    &self,
    renderpass : &mut wgpu::RenderPass< '_ >,
    command : &commands::SpriteCommand,
    camera_pos : [ f32; 2 ],
    aspect_scale : [ f32; 2 ]
  )
  {
    let pipeline = &self.sprite_pipeline;
    let ( _, bindgroup ) = &self.textures[ &command.id ];
    renderpass.set_pipeline( pipeline );
    renderpass.set_bind_group( 0, bindgroup, &[] );

    let [ pos_x, pos_y ] = command.transform.position;
    let [ cam_pos_x, cam_pos_y ] = camera_pos;

    let pc = PushConstant
    {
      aspect_scale,
      translation : [ pos_x + cam_pos_x, pos_y + cam_pos_y ],
      rotation_cos_sin : [ command.transform.rotation.cos(), command.transform.rotation.sin() ],
      scale : command.transform.scale,
    };
    renderpass.set_push_constants( wgpu::ShaderStages::VERTEX, 0, bytemuck::bytes_of( &pc ) );
    renderpass.draw( 0..4, 0..1 );
  }

  /// Sets `RenderContext`. `width` and `height` must be multiples of 256.
  pub fn render_context_set( &mut self, render_context : ports::RenderContext )
  {
    self.render_context = render_context;
  }
}

#[ derive( Debug, Clone, Copy, Default, bytemuck::NoUninit ) ]
#[ repr( C ) ]
struct PushConstant
{
  aspect_scale : [ f32; 2 ],
  translation : [ f32; 2 ],
  rotation_cos_sin : [ f32; 2 ],
  scale : [ f32; 2 ],
}

#[ cfg( test ) ]
mod tests
{
  use crate::{ commands, ports::RenderContext };
  use commands::{ Geometry2DCommand, Point2D, RenderCommand, Transform2D };
  use super::*;

  #[ test ]
  fn test_context_creation()
  {
    let renderer = WGPUTileRenderer::new
    (
      wgpu::Backends::PRIMARY,
      RenderContext::new( 256, 256, [ 0.0; 4 ], true, Point2D::new( 0.0, 0.0 ), 1.0 )
    );
    assert!( renderer.is_ok() );
  }

  #[ test ]
  fn test_shader_compilation()
  {
    let context = context::Context::builder()
    .backends( wgpu::Backends::PRIMARY )
    .make_instance()
    .power_preference( wgpu::PowerPreference::HighPerformance )
    .request_adapter()
    .map_err( | e | ports::RenderError::InitializationFailed( format!( "{e}" ) ) ).unwrap()
    .label( "device" )
    .required_features( wgpu::Features::PUSH_CONSTANTS )
    .required_limits( wgpu::Limits { max_push_constant_size : 44, ..Default::default() } )
    .finish_context()
    .map_err( | e | ports::RenderError::InitializationFailed( format!( "{e}" ) ) ).unwrap();

    _ = context.get_device().create_shader_module( wgpu::include_wgsl!( "../../../shaders/geom2d.wgsl" ) );
    _ = context.get_device().create_shader_module( wgpu::include_wgsl!( "../../../shaders/sprite.wgsl" ) );
  }

  #[ test ]
  fn test_render_pass()
  {
    let mut renderer = WGPUTileRenderer::new
    (
      wgpu::Backends::PRIMARY,
      RenderContext::new( 256, 256, [ 0.0; 4 ], true, Point2D::new( 0.0, 0.0 ), 1.0 )
    ).unwrap();

    let line = &[ 0.0_f32, 0.0, 1.0, 1.0 ];

    renderer.geometry2d_load( line, 0 );

    _ = renderer.commands_execute
    (
      &[
        RenderCommand::Geometry2DCommand
        (
          Geometry2DCommand
          {
            id : 0,
            transform : Transform2D::default(),
            color: [ 1.0; 3 ],
            mode: commands::GeometryMode::Lines
          }
        )
      ]
    );
  }
}
