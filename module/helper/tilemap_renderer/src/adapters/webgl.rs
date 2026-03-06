//! WebGL backend adapter.
//!
//! Hardware-accelerated 2D rendering via WebGL2 (wasm32 target).
//! Uses `minwebgl` for GL calls. Quad vertices are generated in
//! the vertex shader via `gl_VertexID` — no quad VAO needed.

use minwebgl as gl;
use nohash_hasher::IntMap;
use crate::assets::Assets;
use crate::backend::*;
use crate::commands::*;
use crate::types::*;

// ============================================================================
// GPU resource handles
// ============================================================================

/// Manages GPU-side resources: textures and geometry buffers.
struct GpuResources
{
  textures : IntMap< ResourceId< asset::Image >, GpuTexture >,
  geometries : IntMap< ResourceId< asset::Geometry >, GpuGeometry >,
  batches : IntMap< ResourceId< Batch >, GpuBatch >,
}

struct GpuTexture
{
  texture : web_sys::WebGlTexture,
  width : u32,
  height : u32,
  filter : SamplerFilter,
}

struct GpuGeometry
{
  vao : web_sys::WebGlVertexArrayObject,
  vertex_count : u32,
  index_count : Option< u32 >,
}

/// Persistent batch — sprite or mesh.
enum GpuBatch
{
  Sprite
  {
    instance_buffer : web_sys::WebGlBuffer,
    sheet : web_sys::WebGlTexture,
    instance_count : u32,
    blend : BlendMode,
  },
  Mesh
  {
    instance_buffer : web_sys::WebGlBuffer,
    geometry : web_sys::WebGlVertexArrayObject,
    topology : u32,
    instance_count : u32,
    blend : BlendMode,
  },
}

impl GpuResources
{
  fn new() -> Self
  {
    Self
    {
      textures : IntMap::default(),
      geometries : IntMap::default(),
      batches : IntMap::default(),
    }
  }

  fn texture( &self, id : ResourceId< asset::Image > ) -> Option< &GpuTexture >
  {
    self.textures.get( &id )
  }

  fn geometry( &self, id : ResourceId< asset::Geometry > ) -> Option< &GpuGeometry >
  {
    self.geometries.get( &id )
  }

  fn batch( &self, id : ResourceId< Batch > ) -> Option< &GpuBatch >
  {
    self.batches.get( &id )
  }

  fn store_texture( &mut self, id : ResourceId< asset::Image >, tex : GpuTexture )
  {
    self.textures.insert( id, tex );
  }

  fn store_geometry( &mut self, id : ResourceId< asset::Geometry >, geom : GpuGeometry )
  {
    self.geometries.insert( id, geom );
  }

  fn store_batch( &mut self, id : ResourceId< Batch >, batch : GpuBatch )
  {
    self.batches.insert( id, batch );
  }
}

// ============================================================================
// Sprite renderer
// ============================================================================

/// Handles single sprite draws and sprite batch instancing.
/// Quad is generated in vertex shader from `gl_VertexID` (triangle strip, 4 vertices).
struct SpriteRenderer
{
  program : gl::Program,
}

impl SpriteRenderer
{
  fn new( gl : &gl::GL ) -> Result< Self, gl::WebglError >
  {
    let program = gl::Program::new
    (
      gl.clone(),
      include_str!( "shaders/sprite.vert" ),
      include_str!( "shaders/sprite.frag" ),
    )?;
    Ok( Self { program } )
  }

  /// Draw a single sprite as a textured quad (triangle strip, 4 vertices from gl_VertexID).
  fn draw( &self, gl : &gl::GL, transform : &[ f32; 9 ], uv_rect : &[ f32; 4 ], tint : &[ f32; 4 ], viewport : &[ f32; 2 ] )
  {
    self.program.activate();
    self.program.uniform_upload( "u_transform", transform );
    self.program.uniform_upload( "u_uv_rect", uv_rect );
    self.program.uniform_upload( "u_tint", tint );
    self.program.uniform_upload( "u_viewport", viewport );
    gl.draw_arrays( gl::GL::TRIANGLE_STRIP, 0, 4 );
  }

  /// Draw an instanced sprite batch.
  fn draw_batch( &self, _gl : &gl::GL, _batch : &GpuBatch )
  {
    // TODO:
    // 1. Bind sheet texture
    // 2. Bind instance buffer, set per-instance attribs with divisor
    // 3. gl.draw_arrays_instanced( TRIANGLE_STRIP, 0, 4, instance_count )
  }
}

// ============================================================================
// Mesh renderer
// ============================================================================

/// Handles single mesh draws and mesh batch instancing.
struct MeshRenderer
{
  program : gl::Program,
}

impl MeshRenderer
{
  fn new( gl : &gl::GL ) -> Result< Self, gl::WebglError >
  {
    let program = gl::Program::new
    (
      gl.clone(),
      include_str!( "shaders/mesh.vert" ),
      include_str!( "shaders/mesh.frag" ),
    )?;
    Ok( Self { program } )
  }

  /// Draw a single mesh.
  fn draw( &self, gl : &gl::GL, geom : &GpuGeometry, transform : &[ f32; 9 ], color : &[ f32; 4 ], topology : u32, viewport : &[ f32; 2 ] )
  {
    self.program.activate();
    self.program.uniform_upload( "u_transform", transform );
    self.program.uniform_upload( "u_color", color );
    self.program.uniform_upload( "u_viewport", viewport );

    gl.bind_vertex_array( Some( &geom.vao ) );

    if let Some( count ) = geom.index_count
    {
      gl.draw_elements_with_i32( topology, count as i32, gl::GL::UNSIGNED_SHORT, 0 );
    }
    else
    {
      gl.draw_arrays( topology, 0, geom.vertex_count as i32 );
    }

    gl.bind_vertex_array( None );
  }

  /// Draw an instanced mesh batch.
  fn draw_batch( &self, _gl : &gl::GL, _batch : &GpuBatch )
  {
    // TODO:
    // 1. Bind geometry VAO
    // 2. Bind instance buffer, set per-instance attribs with divisor
    // 3. gl.draw_arrays_instanced / draw_elements_instanced
  }
}

// ============================================================================
// Backend struct
// ============================================================================

/// WebGL renderer backend.
///
/// ```ignore
/// let config = RenderConfig { width: 800, height: 600, ..Default::default() };
/// let gl_ctx = minwebgl::context::from_canvas( &canvas )?;
/// let mut backend = WebGlBackend::new( config, gl_ctx )?;
/// backend.load_assets( &assets )?;
/// backend.submit( &commands )?;
/// ```
pub struct WebGlBackend
{
  config : RenderConfig,
  gl : gl::GL,
  resources : GpuResources,
  sprite : SpriteRenderer,
  mesh : MeshRenderer,

  // -- streaming state --
  path_active : bool,
  path_vertices : Vec< f32 >,
  path_style : Option< BeginPath >,
  text_active : bool,
  text_cursor : [ f32; 2 ],
  text_style : Option< BeginText >,
  instance_buffer_data : Vec< f32 >,
  recording_batch : Option< ResourceId< Batch > >,
}

impl WebGlBackend
{
  /// Creates a new WebGL backend.
  ///
  /// # Errors
  /// Returns error if shader compilation fails.
  pub fn new( config : RenderConfig, gl : gl::GL ) -> Result< Self, RenderError >
  {
    let map_err = | e : gl::WebglError | RenderError::BackendError( format!( "{e:?}" ) );

    let sprite = SpriteRenderer::new( &gl ).map_err( map_err )?;
    let mesh = MeshRenderer::new( &gl ).map_err( map_err )?;

    // Initial GL state
    gl.viewport( 0, 0, config.width as i32, config.height as i32 );
    gl.enable( gl::GL::BLEND );
    gl.blend_func( gl::GL::SRC_ALPHA, gl::GL::ONE_MINUS_SRC_ALPHA );

    if !matches!( config.antialias, Antialias::None )
    {
      gl.enable( gl::GL::SAMPLE_COVERAGE );
    }

    Ok( Self
    {
      config,
      gl,
      resources : GpuResources::new(),
      sprite,
      mesh,
      path_active : false,
      path_vertices : Vec::new(),
      path_style : None,
      text_active : false,
      text_cursor : [ 0.0, 0.0 ],
      text_style : None,
      instance_buffer_data : Vec::new(),
      recording_batch : None,
    })
  }

  fn viewport_size( &self ) -> [ f32; 2 ]
  {
    [ self.config.width as f32, self.config.height as f32 ]
  }

  // ---- Blend ----

  fn apply_blend( &self, blend : &BlendMode )
  {
    let gl = &self.gl;
    match blend
    {
      BlendMode::Normal => gl.blend_func( gl::GL::SRC_ALPHA, gl::GL::ONE_MINUS_SRC_ALPHA ),
      BlendMode::Add => gl.blend_func( gl::GL::SRC_ALPHA, gl::GL::ONE ),
      BlendMode::Multiply => gl.blend_func( gl::GL::DST_COLOR, gl::GL::ONE_MINUS_SRC_ALPHA ),
      BlendMode::Screen => gl.blend_func( gl::GL::ONE, gl::GL::ONE_MINUS_SRC_COLOR ),
      BlendMode::Overlay => gl.blend_func( gl::GL::SRC_ALPHA, gl::GL::ONE_MINUS_SRC_ALPHA ),
    }
  }

  // ---- Flush ----

  fn flush_path( &mut self )
  {
    let Some( _style ) = self.path_style.take() else { return };
    // TODO: tessellate → draw with mesh renderer
    self.path_vertices.clear();
  }

  fn flush_text( &mut self )
  {
    let Some( _style ) = self.text_style.take() else { return };
    // TODO: glyph atlas → draw quads with sprite renderer
    self.text_cursor = [ 0.0, 0.0 ];
  }
}

// ============================================================================
// Shared utilities
// ============================================================================

/// Builds a column-major 3x3 affine matrix from Transform.
fn transform_to_mat3( t : &Transform ) -> [ f32; 9 ]
{
  let cos_r = t.rotation.cos();
  let sin_r = t.rotation.sin();
  let sx = t.scale[ 0 ];
  let sy = t.scale[ 1 ];
  let skx = t.skew[ 0 ].tan();
  let sky = t.skew[ 1 ].tan();

  let m00 = ( cos_r + sin_r * sky ) * sx;
  let m10 = ( sin_r - cos_r * sky ) * sx;
  let m01 = ( cos_r * skx - sin_r ) * sy;
  let m11 = ( sin_r * skx + cos_r ) * sy;

  [
    m00,                m10,                0.0,
    m01,                m11,                0.0,
    t.position[ 0 ],   t.position[ 1 ],    1.0,
  ]
}

fn topology_to_gl( t : &Topology ) -> u32
{
  match t
  {
    Topology::TriangleList => gl::GL::TRIANGLES,
    Topology::TriangleStrip => gl::GL::TRIANGLE_STRIP,
    Topology::LineList => gl::GL::LINES,
    Topology::LineStrip => gl::GL::LINE_STRIP,
  }
}

fn apply_texture_filter( gl : &gl::GL, filter : &SamplerFilter )
{
  let f = match filter
  {
    SamplerFilter::Nearest => gl::GL::NEAREST as i32,
    SamplerFilter::Linear => gl::GL::LINEAR as i32,
  };
  gl.tex_parameteri( gl::GL::TEXTURE_2D, gl::GL::TEXTURE_MIN_FILTER, f );
  gl.tex_parameteri( gl::GL::TEXTURE_2D, gl::GL::TEXTURE_MAG_FILTER, f );
}

// ============================================================================
// Backend trait impl
// ============================================================================

impl Backend for WebGlBackend
{
  fn load_assets( &mut self, assets : &Assets ) -> Result< (), RenderError >
  {
    let gl = &self.gl;
    let map_err = | e : gl::WebglError | RenderError::BackendError( format!( "{e:?}" ) );

    // ---- Images → GPU textures ----
    self.resources.textures.clear();

    for img in &assets.images
    {
      let texture = gl.create_texture()
        .ok_or_else( || RenderError::BackendError( "failed to create texture".into() ) )?;

      gl.bind_texture( gl::GL::TEXTURE_2D, Some( &texture ) );
      apply_texture_filter( gl, &img.filter );
      gl.tex_parameteri( gl::GL::TEXTURE_2D, gl::GL::TEXTURE_WRAP_S, gl::GL::CLAMP_TO_EDGE as i32 );
      gl.tex_parameteri( gl::GL::TEXTURE_2D, gl::GL::TEXTURE_WRAP_T, gl::GL::CLAMP_TO_EDGE as i32 );

      let ( w, h ) = match &img.source
      {
        crate::assets::ImageSource::Bitmap { bytes, width, height, format } =>
        {
          let gl_fmt = match format
          {
            crate::assets::PixelFormat::Rgba8 => gl::GL::RGBA,
            crate::assets::PixelFormat::Rgb8 => gl::GL::RGB,
            crate::assets::PixelFormat::Gray8 => gl::GL::LUMINANCE,
            crate::assets::PixelFormat::GrayAlpha8 => gl::GL::LUMINANCE_ALPHA,
          };
          let _ = gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
          (
            gl::GL::TEXTURE_2D, 0, gl_fmt as i32,
            *width as i32, *height as i32, 0,
            gl_fmt, gl::GL::UNSIGNED_BYTE, Some( bytes ),
          );
          ( *width, *height )
        }
        crate::assets::ImageSource::Encoded( _bytes ) => ( 0, 0 ), // TODO: decode
        crate::assets::ImageSource::Path( _path ) => ( 0, 0 ),     // N/A in wasm
      };

      self.resources.store_texture( img.id, GpuTexture { texture, width : w, height : h, filter : img.filter } );
    }

    // ---- Geometry → VAOs ----
    self.resources.geometries.clear();

    for geom in &assets.geometries
    {
      let vao = gl::vao::create( gl ).map_err( map_err )?;
      gl.bind_vertex_array( Some( &vao ) );

      if let crate::assets::Source::Bytes( ref bytes ) = geom.positions
      {
        let buffer = gl::buffer::create( gl ).map_err( map_err )?;
        gl::buffer::upload( gl, &buffer, bytes, gl::GL::STATIC_DRAW );
        gl.enable_vertex_attrib_array( 0 );
        gl.vertex_attrib_pointer_with_i32( 0, 2, gl::GL::FLOAT, false, 0, 0 );
      }

      if let Some( crate::assets::Source::Bytes( ref bytes ) ) = geom.uvs
      {
        let buffer = gl::buffer::create( gl ).map_err( map_err )?;
        gl::buffer::upload( gl, &buffer, bytes, gl::GL::STATIC_DRAW );
        gl.enable_vertex_attrib_array( 1 );
        gl.vertex_attrib_pointer_with_i32( 1, 2, gl::GL::FLOAT, false, 0, 0 );
      }

      let mut index_count = None;
      if let Some( crate::assets::Source::Bytes( ref bytes ) ) = geom.indices
      {
        let buffer = gl::buffer::create( gl ).map_err( map_err )?;
        gl.bind_buffer( gl::GL::ELEMENT_ARRAY_BUFFER, Some( &buffer ) );
        let u8_array = js_sys::Uint8Array::from( bytes.as_slice() );
        gl.buffer_data_with_array_buffer_view( gl::GL::ELEMENT_ARRAY_BUFFER, &u8_array, gl::GL::STATIC_DRAW );
        index_count = Some( ( bytes.len() / 2 ) as u32 );
      }

      gl.bind_vertex_array( None );

      let vertex_count = if let crate::assets::Source::Bytes( ref bytes ) = geom.positions
      { ( bytes.len() / 8 ) as u32 } else { 0 };

      self.resources.store_geometry( geom.id, GpuGeometry { vao, vertex_count, index_count } );
    }

    // TODO: gradients, patterns, clip masks, fonts

    Ok( () )
  }

  fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >
  {
    let viewport = self.viewport_size();

    for cmd in commands
    {
      match cmd
      {
        RenderCommand::Clear( c ) =>
        {
          let [ r, g, b, a ] = c.color;
          self.gl.clear_color( r, g, b, a );
          self.gl.clear( gl::GL::COLOR_BUFFER_BIT );
        }

        // ---- Path streaming ----
        RenderCommand::BeginPath( bp ) =>
        {
          self.path_active = true;
          self.path_vertices.clear();
          self.path_style = Some( *bp );
        }
        RenderCommand::MoveTo( m ) => { self.path_vertices.extend_from_slice( &[ m.0, m.1 ] ); }
        RenderCommand::LineTo( l ) => { self.path_vertices.extend_from_slice( &[ l.0, l.1 ] ); }
        RenderCommand::QuadTo( q ) => { self.path_vertices.extend_from_slice( &[ q.x, q.y ] ); } // TODO: flatten
        RenderCommand::CubicTo( c ) => { self.path_vertices.extend_from_slice( &[ c.x, c.y ] ); } // TODO: flatten
        RenderCommand::ArcTo( a ) => { self.path_vertices.extend_from_slice( &[ a.x, a.y ] ); } // TODO: decompose
        RenderCommand::ClosePath( _ ) => {} // TODO: close subpath
        RenderCommand::EndPath( _ ) =>
        {
          self.path_active = false;
          self.flush_path();
        }

        // ---- Text streaming ----
        RenderCommand::BeginText( bt ) =>
        {
          self.text_active = true;
          self.text_cursor = bt.position;
          self.text_style = Some( *bt );
        }
        RenderCommand::Char( _ch ) => {} // TODO: glyph lookup + cursor advance
        RenderCommand::EndText( _ ) =>
        {
          self.text_active = false;
          self.flush_text();
        }

        // ---- Mesh ----
        RenderCommand::Mesh( m ) =>
        {
          if let Some( geom ) = self.resources.geometry( m.geometry )
          {
            let mat = transform_to_mat3( &m.transform );
            let color = match m.fill { FillRef::Solid( c ) => c, _ => [ 1.0, 1.0, 1.0, 1.0 ] };
            self.apply_blend( &m.blend );

            if let Some( tex_id ) = m.texture
            {
              if let Some( gpu_tex ) = self.resources.texture( tex_id )
              {
                self.gl.active_texture( gl::GL::TEXTURE0 );
                self.gl.bind_texture( gl::GL::TEXTURE_2D, Some( &gpu_tex.texture ) );
              }
            }

            self.mesh.draw( &self.gl, geom, &mat, &color, topology_to_gl( &m.topology ), &viewport );
          }
        }

        // ---- Sprite ----
        RenderCommand::Sprite( s ) =>
        {
          let mat = transform_to_mat3( &s.transform );
          self.apply_blend( &s.blend );
          // TODO: look up SpriteAsset → sheet texture + region → uv_rect
          let uv_rect = [ 0.0, 0.0, 1.0, 1.0 ]; // placeholder: full texture
          self.sprite.draw( &self.gl, &mat, &uv_rect, &s.tint, &viewport );
        }

        // ---- Sprite batch recording ----
        RenderCommand::BeginRecordSpriteBatch( brb ) =>
        {
          self.recording_batch = Some( brb.batch );
          self.instance_buffer_data.clear();
        }
        RenderCommand::SpriteInstance( si ) =>
        {
          let mat = transform_to_mat3( &si.transform );
          self.instance_buffer_data.extend_from_slice( &mat );
          self.instance_buffer_data.extend_from_slice( &si.tint );
          self.instance_buffer_data.push( si.sprite.inner() as f32 );
        }
        RenderCommand::EndRecordSpriteBatch( _ ) =>
        {
          if let Some( _batch_id ) = self.recording_batch.take()
          {
            // TODO: create GPU buffer, store as GpuBatch::Sprite
            self.instance_buffer_data.clear();
          }
        }

        // ---- Mesh batch recording ----
        RenderCommand::BeginRecordMeshBatch( brb ) =>
        {
          self.recording_batch = Some( brb.batch );
          self.instance_buffer_data.clear();
        }
        RenderCommand::MeshInstance( mi ) =>
        {
          let mat = transform_to_mat3( &mi.transform );
          self.instance_buffer_data.extend_from_slice( &mat );
        }
        RenderCommand::EndRecordMeshBatch( _ ) =>
        {
          if let Some( _batch_id ) = self.recording_batch.take()
          {
            // TODO: create GPU buffer, store as GpuBatch::Mesh
            self.instance_buffer_data.clear();
          }
        }

        // ---- Grouping ----
        RenderCommand::BeginGroup( _ ) => {} // TODO: transform stack, stencil, FBO
        RenderCommand::EndGroup( _ ) => {}   // TODO: pop state
      }
    }

    Ok( () )
  }

  fn output( &self ) -> Result< Output, RenderError >
  {
    Ok( Output::Presented )
  }

  fn resize( &mut self, width : u32, height : u32 )
  {
    self.config.width = width;
    self.config.height = height;
    self.gl.viewport( 0, 0, width as i32, height as i32 );
  }

  fn capabilities( &self ) -> Capabilities
  {
    Capabilities
    {
      paths : true,
      text : true,
      meshes : true,
      sprites : true,
      batches : true,
      gradients : true,
      patterns : true,
      clip_masks : true,
      effects : true,
      blend_modes : true,
      text_on_path : false,
      max_texture_size : 4096,
    }
  }
}

// ============================================================================
// BatchBackend trait impl
// ============================================================================

impl BatchBackend for WebGlBackend
{
  fn draw_batch( &mut self, batch : ResourceId< Batch > ) -> Result< (), RenderError >
  {
    let gpu_batch = self.resources.batch( batch ).ok_or( RenderError::MissingAsset( batch.inner() ) )?;

    match gpu_batch
    {
      GpuBatch::Sprite { .. } => self.sprite.draw_batch( &self.gl, gpu_batch ),
      GpuBatch::Mesh { .. } => self.mesh.draw_batch( &self.gl, gpu_batch ),
    }

    Ok( () )
  }

  fn update_sprite_instance( &mut self, batch : ResourceId< Batch >, index : u32, instance : &SpriteInstance ) -> Result< (), RenderError >
  {
    let _gpu_batch = self.resources.batch( batch )
      .ok_or( RenderError::MissingAsset( batch.inner() ) )?;

    let _ = ( index, instance );
    // TODO: gl.buffer_sub_data — offset = index * SPRITE_INSTANCE_STRIDE

    Ok( () )
  }

  fn update_mesh_instance( &mut self, batch : ResourceId< Batch >, index : u32, instance : &MeshInstance ) -> Result< (), RenderError >
  {
    let _gpu_batch = self.resources.batch( batch )
      .ok_or( RenderError::MissingAsset( batch.inner() ) )?;

    let _ = ( index, instance );
    // TODO: gl.buffer_sub_data — offset = index * MESH_INSTANCE_STRIDE

    Ok( () )
  }

  fn delete_batch( &mut self, batch : ResourceId< Batch > ) -> Result< (), RenderError >
  {
    if let Some( b ) = self.resources.batches.remove( &batch )
    {
      let buf = match &b
      {
        GpuBatch::Sprite { instance_buffer, .. } => instance_buffer,
        GpuBatch::Mesh { instance_buffer, .. } => instance_buffer,
      };
      self.gl.delete_buffer( Some( buf ) );
    }
    Ok( () )
  }
}
