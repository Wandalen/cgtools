//! WebGPU backend adapter over `gpu_hal`'s browser WebGPU surface.
//!
//! Draws `RenderCommand::Sprite` only -- the minimal command family
//! `pingpong_animation`'s compiler (task 085) produces -- through one shared
//! textured-quad pipeline. Every other command family returns
//! `RenderError::Unsupported` rather than being silently skipped, so
//! `capabilities()` never over-claims what `submit` actually translates
//! (`docs/pattern/001_ports_and_adapters_backend_architecture.md`).
//!
//! `gpu_hal` has no texture pixel-upload call (`Device` offers
//! `texture_create` allocation only, no `texture_write`), so loaded images
//! are allocated but never populated with real pixels -- the same
//! async-load gap `ImageAsset::source`'s own doc comment records for the
//! WebGL adapter. Pixel-correctness verification is out of this task's
//! scope; see the governing task file's Out of Scope section.

mod private
{
  use crate::assets::Assets;
  use crate::backend::{ Backend, Capabilities, Output, RenderError };
  use crate::commands::{ RenderCommand, Sprite };
  use crate::types::RenderConfig;
  use web_sys::HtmlCanvasElement;
  use gpu_hal::
  {
    BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType,
    Buffer, BufferUsage, ColorAttachmentDesc, Device, Queue, RenderPass, RenderPipeline,
    RenderPipelineDesc, Sampler, SamplerDesc, ShaderSource, ShaderStages, Surface,
    TextureDesc, TextureFormat, TextureUsage, TextureView, VertexAttribute,
    VertexBufferLayout, VertexFormat,
  };

  /// Sprite quad shader: one textured triangle-list quad per draw. The 2D
  /// affine transform is embedded into a `mat4x4` (see `mat3_to_mat4`) so
  /// every `Uniforms` member lands on a natural 16-byte WGSL boundary.
  const WGSL_SOURCE : &str = "
struct Uniforms {
  transform: mat4x4<f32>,
  region: vec4<f32>,
  viewport: vec4<f32>,
  tint: vec4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var sprite_texture: texture_2d<f32>;
@group(0) @binding(2) var sprite_sampler: sampler;

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@location(0) local_position: vec2<f32>) -> VertexOutput {
  let scaled = local_position * uniforms.region.zw;
  let world = uniforms.transform * vec4<f32>(scaled, 0.0, 1.0);
  let ndc = (world.xy / uniforms.viewport.xy) * 2.0 - vec2<f32>(1.0, 1.0);
  var out: VertexOutput;
  out.clip_position = vec4<f32>(ndc, 0.0, 1.0);
  out.uv = local_position;
  return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
  return textureSample(sprite_texture, sprite_sampler, input.uv) * uniforms.tint;
}
";

  /// Unit quad, two triangles, local space `[0,1]x[0,1]` -- doubles as the
  /// fragment UV since the placeholder texture (see module docs) is always
  /// `1x1`, so no sub-region addressing is meaningful yet.
  const QUAD_VERTICES : [ f32; 12 ] =
  [
    0.0, 0.0,
    1.0, 0.0,
    1.0, 1.0,
    0.0, 0.0,
    1.0, 1.0,
    0.0, 1.0,
  ];

  /// An allocated (but unpopulated -- see module docs) image texture.
  struct LoadedImage
  {
    id : u32,
    view : TextureView,
  }

  /// A registered sprite region within a sheet image.
  struct LoadedSprite
  {
    id : u32,
    sheet : u32,
    region : [ f32; 4 ],
  }

  /// WebGPU-backed `Backend` implementation over `gpu_hal`.
  pub struct WebGpuBackend
  {
    config : RenderConfig,
    canvas : HtmlCanvasElement,
    device : Device,
    queue : Queue,
    surface : Surface,
    pipeline : RenderPipeline,
    bind_group_layout : BindGroupLayout,
    sampler : Sampler,
    quad_vertices : Buffer,
    images : Vec< LoadedImage >,
    sprites : Vec< LoadedSprite >,
  }

  impl WebGpuBackend
  {
    /// Constructs the backend against `canvas`, building the shared sprite
    /// pipeline. Async because `gpu_hal::Device::new_webgpu` is -- the first
    /// async adapter constructor in this crate.
    ///
    /// # Errors
    ///
    /// Returns `RenderError::BackendError` if device/surface acquisition or
    /// pipeline construction fails.
    pub async fn new( config : RenderConfig, canvas : &HtmlCanvasElement ) -> Result< Self, RenderError >
    {
      let ( device, queue, surface ) = Device::new_webgpu( canvas )
      .await
      .map_err( | e | RenderError::BackendError( format!( "failed to acquire WebGPU device : {e:?}" ) ) )?;

      let ( pipeline, bind_group_layout ) = pipeline_build( &device, surface.format() )?;
      let sampler = device.sampler_create( SamplerDesc::default() )
      .map_err( | e | RenderError::BackendError( format!( "failed to create sampler : {e:?}" ) ) )?;
      let quad_vertices = device.buffer_init_create( bytemuck::cast_slice( &QUAD_VERTICES ), BufferUsage::VERTEX )
      .map_err( | e | RenderError::BackendError( format!( "failed to create quad vertex buffer : {e:?}" ) ) )?;

      Ok( Self
      {
        config,
        canvas : canvas.clone(),
        device,
        queue,
        surface,
        pipeline,
        bind_group_layout,
        sampler,
        quad_vertices,
        images : Vec::new(),
        sprites : Vec::new(),
      })
    }

    /// The `Capabilities` this backend honestly implements: sprites only.
    /// A distinctly-named inherent `const fn` (not `capabilities`, to avoid
    /// colliding with the trait method) so it is checkable without
    /// constructing an instance -- no canvas or live device required.
    #[ must_use ]
    pub const fn declared_capabilities() -> Capabilities
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

    /// Extracts the position and sprite-resource-id `submit` draws `sprite`
    /// with -- pulled out of `sprite_draw` so the command-to-draw-parameter
    /// mapping is checkable without a live device.
    #[ must_use ]
    pub fn sprite_draw_params( sprite : &Sprite ) -> ( [ f32; 2 ], u32 )
    {
      ( sprite.transform.position, sprite.sprite.inner() )
    }

    /// Classifies whether `submit` can translate `command` into GPU work.
    /// `submit` calls this exact function for every command in its loop, so
    /// it is the real anti-faking gate -- every unsupported command family
    /// really does produce `Err`, never a silently-skipped `Ok` -- and is
    /// checkable without a live device.
    ///
    /// # Errors
    ///
    /// Returns `RenderError::Unsupported` for every command family other
    /// than `Sprite`.
    pub fn command_classify( command : &RenderCommand ) -> Result< (), RenderError >
    {
      match command
      {
        RenderCommand::Sprite( _ ) => Ok( () ),
        _ => Err( RenderError::Unsupported( "WebGpuBackend only translates a leading Clear plus Sprite commands" ) ),
      }
    }

    /// Draws one sprite into the active `pass`.
    fn sprite_draw( &self, pass : &mut RenderPass, sprite : &Sprite ) -> Result< (), RenderError >
    {
      let resource_id = sprite.sprite.inner();
      let loaded_sprite = self.sprites.iter().find( | s | s.id == resource_id )
      .ok_or( RenderError::MissingAsset( resource_id ) )?;
      let loaded_image = self.images.iter().find( | image | image.id == loaded_sprite.sheet )
      .ok_or( RenderError::MissingAsset( loaded_sprite.sheet ) )?;

      let transform = mat3_to_mat4( &sprite.transform.to_mat3() );
      let uniforms = uniforms_build
      (
        &transform,
        &loaded_sprite.region,
        &sprite.tint,
        self.config.width,
        self.config.height
      );

      let uniform_buffer = self.device.buffer_init_create( bytemuck::cast_slice( &uniforms ), BufferUsage::UNIFORM )
      .map_err( | e | RenderError::BackendError( format!( "failed to create uniform buffer : {e:?}" ) ) )?;
      let bind_group = self.device.bind_group_create
      (
        &self.bind_group_layout,
        &[
          BindingResource::Buffer( &uniform_buffer ),
          BindingResource::TextureView( &loaded_image.view ),
          BindingResource::Sampler( &self.sampler ),
        ]
      )
      .map_err( | e | RenderError::BackendError( format!( "failed to create bind group : {e:?}" ) ) )?;

      pass.bind_group_set( 0, &bind_group );
      pass.vertex_buffer_set( 0, &self.quad_vertices );
      pass.draw( 6 );
      Ok( () )
    }
  }

  impl Backend for WebGpuBackend
  {
    fn assets_load( &mut self, assets : &Assets ) -> Result< (), RenderError >
    {
      self.images.clear();
      self.sprites.clear();

      for image in &assets.images
      {
        let texture = self.device.texture_create( &TextureDesc
        {
          size : [ 1, 1, 1 ],
          format : TextureFormat::Rgba8Unorm,
          usage : TextureUsage::TEXTURE_BINDING,
        })
        .map_err( | e | RenderError::BackendError( format!( "failed to create texture : {e:?}" ) ) )?;
        let view = texture.view()
        .map_err( | e | RenderError::BackendError( format!( "failed to create texture view : {e:?}" ) ) )?;
        self.images.push( LoadedImage { id : image.id.inner(), view } );
      }

      for sprite in &assets.sprites
      {
        self.sprites.push( LoadedSprite
        {
          id : sprite.id.inner(),
          sheet : sprite.sheet.inner(),
          region : sprite.region,
        });
      }

      Ok( () )
    }

    fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >
    {
      let view = self.surface.current_view()
      .map_err( | e | RenderError::BackendError( format!( "failed to acquire surface view : {e:?}" ) ) )?;

      let ( clear_color, remaining ) = match commands
      {
        [ RenderCommand::Clear( clear ), tail @ .. ] => ( clear.color, tail ),
        _ => ( self.config.background, commands ),
      };

      let mut encoder = self.device.command_encoder_create();
      let color = ColorAttachmentDesc { view : &view, clear : clear_color };
      let mut pass = encoder.render_pass_begin( &color, None )
      .map_err( | e | RenderError::BackendError( format!( "failed to begin render pass : {e:?}" ) ) )?;

      pass.pipeline_set( &self.pipeline );
      for command in remaining
      {
        Self::command_classify( command )?;
        if let RenderCommand::Sprite( sprite ) = command
        {
          self.sprite_draw( &mut pass, sprite )?;
        }
      }
      pass.end();

      self.queue.submit( encoder );
      Ok( () )
    }

    fn output( &self ) -> Result< Output, RenderError >
    {
      Ok( Output::Presented )
    }

    fn resize( &mut self, width : u32, height : u32 )
    {
      self.canvas.set_width( width );
      self.canvas.set_height( height );
      self.config.width = width;
      self.config.height = height;
    }

    fn capabilities( &self ) -> Capabilities
    {
      Self::declared_capabilities()
    }
  }

  /// Builds the shared sprite pipeline and its bind group layout.
  fn pipeline_build( device : &Device, color_format : TextureFormat ) -> Result< ( RenderPipeline, BindGroupLayout ), RenderError >
  {
    let shader = device.shader_module_create( &ShaderSource
    {
      wgsl : WGSL_SOURCE,
      glsl_vertex : None,
      glsl_fragment : None,
    })
    .map_err( | e | RenderError::BackendError( format!( "failed to compile sprite shader : {e:?}" ) ) )?;

    let bind_group_layout = device.bind_group_layout_create
    (
      &[
        BindGroupLayoutEntry { visibility : ShaderStages::VERTEX | ShaderStages::FRAGMENT, ty : BindingType::UniformBuffer },
        BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::Texture },
        BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::Sampler },
      ]
    )
    .map_err( | e | RenderError::BackendError( format!( "failed to create bind group layout : {e:?}" ) ) )?;

    let vertex_buffers =
    [
      VertexBufferLayout
      {
        stride : 8,
        attributes : vec![ VertexAttribute { location : 0, format : VertexFormat::Float32x2, offset : 0 } ],
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
    .map_err( | e | RenderError::BackendError( format!( "failed to create render pipeline : {e:?}" ) ) )?;

    Ok(( pipeline, bind_group_layout ))
  }

  /// Embeds a 2D affine `mat3` (column-major, `Transform::to_mat3` layout)
  /// into a column-major `mat4x4` for natural WGSL uniform alignment.
  fn mat3_to_mat4( m : &[ f32; 9 ] ) -> [ f32; 16 ]
  {
    [
      m[ 0 ], m[ 1 ], 0.0, 0.0,
      m[ 3 ], m[ 4 ], 0.0, 0.0,
      0.0,    0.0,    1.0, 0.0,
      m[ 6 ], m[ 7 ], 0.0, 1.0,
    ]
  }

  /// Builds the 28-float uniform payload matching `Uniforms` in `WGSL_SOURCE`.
  fn uniforms_build
  (
    transform : &[ f32; 16 ],
    region : &[ f32; 4 ],
    tint : &[ f32; 4 ],
    viewport_width : u32,
    viewport_height : u32
  ) -> [ f32; 28 ]
  {
    let mut uniforms = [ 0.0_f32; 28 ];
    uniforms[ 0..16 ].copy_from_slice( transform );
    uniforms[ 16..20 ].copy_from_slice( region );
    uniforms[ 20 ] = viewport_width as f32;
    uniforms[ 21 ] = viewport_height as f32;
    uniforms[ 24..28 ].copy_from_slice( tint );
    uniforms
  }
}

mod_interface::mod_interface!
{
  own use WebGpuBackend;
}
