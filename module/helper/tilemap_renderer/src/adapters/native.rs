//! Native GPU backend over `gpu_hal`'s offscreen `wgpu` surface.
//!
//! Renders into an off-screen texture and returns pixel bytes via
//! [`Backend::output`](crate::backend::Backend::output) -- there is no on-screen presentation. Draws
//! `RenderCommand::Clear` and `RenderCommand::Sprite` only, the same
//! minimal command family the WebGPU adapter translates; every other
//! command family returns `RenderError::Unsupported` so `capabilities()`
//! never over-claims what `submit` actually does
//! (`docs/pattern/001_ports_and_adapters_backend_architecture.md`).

mod private
{
  use crate::assets::{ Assets, ImageSource, to_rgba8 };
  use crate::backend::{ Backend, Bitmap, Capabilities, Output, RenderError };
  use crate::commands::{ RenderCommand, Sprite };
  use crate::types::{ RenderConfig, Transform };
  use gpu_hal::
  {
    BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferUsage,
    ColorAttachmentDesc, Device, IndexFormat, Queue, RenderPass, RenderPipeline,
    RenderPipelineDesc, Sampler, SamplerDesc, ShaderSource, ShaderStages, Surface, Texture,
    TextureDesc, TextureFormat, TextureUsage, VertexAttribute, VertexBufferLayout, VertexFormat,
  };

  /// Sprite quad shader: vertex positions and UVs arrive pre-transformed
  /// from the CPU (`Transform::to_mat3` plus a pixel-to-NDC projection), so
  /// the shader itself has no matrix math at all.
  const SPRITE_WGSL : &str = "
@group( 0 ) @binding( 0 ) var< uniform > tint : vec4f;
@group( 0 ) @binding( 1 ) var tex : texture_2d< f32 >;
@group( 0 ) @binding( 2 ) var samp : sampler;

struct VsOut
{
  @builtin( position ) clip_position : vec4f,
  @location( 0 ) uv : vec2f,
}

@vertex
fn vs_main( @location( 0 ) position : vec2f, @location( 1 ) uv : vec2f ) -> VsOut
{
  var out : VsOut;
  out.clip_position = vec4f( position, 0.0, 1.0 );
  out.uv = uv;
  return out;
}

@fragment
fn fs_main( in : VsOut ) -> @location( 0 ) vec4f
{
  return textureSample( tex, samp, in.uv ) * tint;
}
";

  /// Shared index buffer content for one quad: two CCW triangles over 4
  /// vertices (0..3).
  const QUAD_INDICES : [ u32; 6 ] = [ 0, 1, 2, 0, 2, 3 ];

  /// GPU handles rebuilt wholesale on construction and on every `resize` --
  /// the offscreen surface has no in-place resize, so nothing here
  /// survives one.
  struct GpuState
  {
    device : Device,
    queue : Queue,
    surface : Surface,
    sampler : Sampler,
    quad_indices : Buffer,
    bind_group_layout : BindGroupLayout,
    pipeline : RenderPipeline,
  }

  /// One uploaded sheet image: its GPU texture plus the pixel dimensions
  /// needed to normalize a `SpriteAsset::region` (pixels) into UVs (`0..1`).
  struct LoadedImage
  {
    id : u32,
    texture : Texture,
    width : u32,
    height : u32,
  }

  /// A registered sprite region within a sheet image.
  struct LoadedSprite
  {
    id : u32,
    sheet : u32,
    region : [ f32; 4 ],
  }

  /// Offscreen GPU backend over `gpu_hal`'s native `wgpu` surface.
  pub struct NativeBackend
  {
    gpu : GpuState,
    config : RenderConfig,
    images : Vec< LoadedImage >,
    sprites : Vec< LoadedSprite >,
  }

  impl NativeBackend
  {
    /// Constructs an offscreen native backend sized per `config`, via the
    /// unmodified `gpu_hal::Device::new_native`.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::BackendError`] if no native `wgpu` adapter is
    /// available, or if any fixed GPU resource (sampler, pipeline, index
    /// buffer) fails to build.
    pub fn new( config : RenderConfig ) -> Result< Self, RenderError >
    {
      let gpu = gpu_state_build( config.width, config.height )?;
      Ok( Self { gpu, config, images : Vec::new(), sprites : Vec::new() } )
    }

    /// Draws one sprite into the active `pass`, sampling its sheet region.
    fn sprite_draw( &self, pass : &mut RenderPass, sprite : &Sprite ) -> Result< (), RenderError >
    {
      let resource_id = sprite.sprite.inner();
      let loaded_sprite = self.sprites.iter().find( | s | s.id == resource_id )
      .ok_or( RenderError::MissingAsset( resource_id ) )?;
      let image = self.images.iter().find( | i | i.id == loaded_sprite.sheet )
      .ok_or( RenderError::MissingAsset( loaded_sprite.sheet ) )?;

      let view = image.texture.view()
      .map_err( | e | RenderError::BackendError( e.to_string() ) )?;
      let vertices = quad_vertices( &sprite.transform, &self.config, &loaded_sprite.region, image.width, image.height );
      let vertex_buffer = self.gpu.device.buffer_init_create( &f32_bytes( &vertices ), BufferUsage::VERTEX )
      .map_err( | e | RenderError::BackendError( e.to_string() ) )?;
      let tint_buffer = self.gpu.device.buffer_init_create( &f32_bytes( &sprite.tint ), BufferUsage::UNIFORM )
      .map_err( | e | RenderError::BackendError( e.to_string() ) )?;
      let bind_group = self.gpu.device.bind_group_create
      (
        &self.gpu.bind_group_layout,
        &[
          BindingResource::Buffer( &tint_buffer ),
          BindingResource::TextureView( &view ),
          BindingResource::Sampler( &self.gpu.sampler ),
        ]
      )
      .map_err( | e | RenderError::BackendError( e.to_string() ) )?;

      pass.pipeline_set( &self.gpu.pipeline );
      pass.bind_group_set( 0, &bind_group );
      pass.vertex_buffer_set( 0, &vertex_buffer );
      pass.index_buffer_set( &self.gpu.quad_indices, IndexFormat::Uint32 );
      pass.draw_indexed( 6 );
      Ok( () )
    }
  }

  impl Backend for NativeBackend
  {
    fn assets_load( &mut self, assets : &Assets ) -> Result< (), RenderError >
    {
      self.images.clear();
      self.sprites.clear();

      for image in &assets.images
      {
        let ImageSource::Bitmap { bytes, width, height, format } = &image.source
        else
        {
          continue;
        };
        let rgba = to_rgba8( bytes, *format );
        let texture = self.gpu.device.texture_create( &TextureDesc
        {
          size : [ *width, *height, 1 ],
          format : TextureFormat::Rgba8Unorm,
          usage : TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST,
        })
        .map_err( | e | RenderError::BackendError( e.to_string() ) )?;
        self.gpu.queue.texture_write( &texture, &rgba )
        .map_err( | e | RenderError::BackendError( e.to_string() ) )?;
        self.images.push( LoadedImage { id : image.id.inner(), texture, width : *width, height : *height } );
      }

      for sprite in &assets.sprites
      {
        self.sprites.push( LoadedSprite { id : sprite.id.inner(), sheet : sprite.sheet.inner(), region : sprite.region } );
      }

      Ok( () )
    }

    fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >
    {
      let ( clear, remaining ) = match commands
      {
        [ RenderCommand::Clear( clear ), tail @ .. ] => ( clear.color, tail ),
        _ => ( self.config.background, commands ),
      };

      let view = self.gpu.surface.current_view()
      .map_err( | e | RenderError::BackendError( e.to_string() ) )?;
      let mut encoder = self.gpu.device.command_encoder_create();
      let mut pass = encoder.render_pass_begin( &ColorAttachmentDesc { view : &view, clear }, None )
      .map_err( | e | RenderError::BackendError( e.to_string() ) )?;

      pass.pipeline_set( &self.gpu.pipeline );
      for command in remaining
      {
        match command
        {
          RenderCommand::Sprite( sprite ) => self.sprite_draw( &mut pass, sprite )?,
          _ => return Err( RenderError::Unsupported( "NativeBackend only translates a leading Clear plus Sprite commands" ) ),
        }
      }
      pass.end();

      self.gpu.queue.submit( encoder );
      Ok( () )
    }

    fn output( &self ) -> Result< Output, RenderError >
    {
      let bytes = self.gpu.surface.pixels_read( &self.gpu.device, &self.gpu.queue )
      .map_err( | e | RenderError::BackendError( e.to_string() ) )?;
      Ok( Output::Bitmap( Bitmap { bytes, width : self.config.width, height : self.config.height, channels : 4 } ) )
    }

    fn resize( &mut self, width : u32, height : u32 )
    {
      self.gpu = gpu_state_build( width, height )
      .expect( "NativeBackend::resize : failed to rebuild the offscreen gpu_hal surface" );
      self.config.width = width;
      self.config.height = height;
      self.images.clear();
      self.sprites.clear();
    }

    fn capabilities( &self ) -> Capabilities
    {
      Capabilities
      {
        paths : false,
        text : false,
        meshes : false,
        sprites : true,
        batches : false,
        gradients : false,
        patterns : false,
        clip_masks : false,
        effects : false,
        blend_modes : false,
        supported_blend_modes : &[],
        text_on_path : false,
        max_texture_size : 8192,
      }
    }
  }

  /// Rebuilds every GPU handle from scratch against a `width`x`height`
  /// offscreen surface -- shared by `new` and `resize` since nothing in
  /// `GpuState` survives a resize.
  fn gpu_state_build( width : u32, height : u32 ) -> Result< GpuState, RenderError >
  {
    let ( device, queue, surface ) = Device::new_native( width, height )
    .map_err( | e | RenderError::BackendError( e.to_string() ) )?;
    let sampler = device.sampler_create( SamplerDesc::default() )
    .map_err( | e | RenderError::BackendError( e.to_string() ) )?;
    let index_bytes : Vec< u8 > = QUAD_INDICES.iter().flat_map( | i | i.to_le_bytes() ).collect();
    let quad_indices = device.buffer_init_create( &index_bytes, BufferUsage::INDEX )
    .map_err( | e | RenderError::BackendError( e.to_string() ) )?;
    let ( pipeline, bind_group_layout ) = pipeline_build( &device, surface.format() )?;

    Ok( GpuState { device, queue, surface, sampler, quad_indices, bind_group_layout, pipeline } )
  }

  /// Builds the shared sprite pipeline and its bind group layout. Binding
  /// order (uniform, then texture, then sampler) keeps the texture entry
  /// immediately before the sampler entry -- load-bearing for the WebGL
  /// backend, which pairs a sampler with the nearest preceding texture
  /// entry (see `gpu_hal`'s own native-backend test).
  fn pipeline_build( device : &Device, color_format : TextureFormat ) -> Result< ( RenderPipeline, BindGroupLayout ), RenderError >
  {
    let shader = device.shader_module_create( &ShaderSource { wgsl : SPRITE_WGSL, glsl_vertex : None, glsl_fragment : None } )
    .map_err( | e | RenderError::BackendError( e.to_string() ) )?;

    let bind_group_layout = device.bind_group_layout_create
    (
      &[
        BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::UniformBuffer },
        BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::Texture },
        BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::Sampler },
      ]
    )
    .map_err( | e | RenderError::BackendError( e.to_string() ) )?;

    let vertex_buffers =
    [
      VertexBufferLayout
      {
        stride : 16,
        attributes : vec!
        [
          VertexAttribute { location : 0, format : VertexFormat::Float32x2, offset : 0 },
          VertexAttribute { location : 1, format : VertexFormat::Float32x2, offset : 8 },
        ],
      }
    ];

    let pipeline = device.render_pipeline_create( &RenderPipelineDesc
    {
      shader : &shader,
      vertex_entry : "vs_main",
      fragment_entry : "fs_main",
      vertex_buffers : &vertex_buffers,
      bind_group_layouts : &[ &bind_group_layout ],
      color_format,
      depth : None,
      cull_back : false,
    })
    .map_err( | e | RenderError::BackendError( e.to_string() ) )?;

    Ok(( pipeline, bind_group_layout ))
  }

  /// The 4 quad-corner vertices (position already in NDC, UV already
  /// mapped into the sheet's pixel space) for one sprite draw.
  fn quad_vertices( transform : &Transform, config : &RenderConfig, region : &[ f32; 4 ], sheet_width : u32, sheet_height : u32 ) -> [ f32; 16 ]
  {
    let m = transform.to_mat3();
    let w = config.width as f32;
    let h = config.height as f32;
    let sw = sheet_width as f32;
    let sh = sheet_height as f32;
    // Local quad corners in `[-0.5, 0.5]`; `fx`/`fy` is the same corner
    // mapped to `[0, 1]`. `fy` is flipped for the UV only: row 0 of the
    // source bytes is the image's top row, but `ly = 0.5` is the *top* of
    // a Y-up world-space quad.
    let corners = [ ( -0.5f32, -0.5f32 ), ( 0.5, -0.5 ), ( 0.5, 0.5 ), ( -0.5, 0.5 ) ];

    let mut out = [ 0.0f32; 16 ];
    for ( i, ( lx, ly ) ) in corners.into_iter().enumerate()
    {
      let world_x = m[ 0 ] * lx + m[ 3 ] * ly + m[ 6 ];
      let world_y = m[ 1 ] * lx + m[ 4 ] * ly + m[ 7 ];
      let ( fx, fy ) = ( lx + 0.5, 1.0 - ( ly + 0.5 ) );
      out[ i * 4 ] = ( world_x / w ) * 2.0 - 1.0;
      out[ i * 4 + 1 ] = ( world_y / h ) * 2.0 - 1.0;
      out[ i * 4 + 2 ] = ( region[ 0 ] + fx * region[ 2 ] ) / sw;
      out[ i * 4 + 3 ] = ( region[ 1 ] + fy * region[ 3 ] ) / sh;
    }
    out
  }

  /// Byte-reinterprets a `f32` slice as tightly-packed little-endian bytes,
  /// for `buffer_init_create` uploads -- avoids depending on `bytemuck`
  /// (not part of `adapter-native`'s dependency set).
  fn f32_bytes( floats : &[ f32 ] ) -> Vec< u8 >
  {
    floats.iter().flat_map( | f | f.to_le_bytes() ).collect()
  }
}

mod_interface::mod_interface!
{
  own use NativeBackend;
}
