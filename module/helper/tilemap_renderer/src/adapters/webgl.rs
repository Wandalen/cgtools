//! WebGL backend adapter.
//!
//! Hardware-accelerated 2D rendering via WebGL2 (wasm32 target).
//! Uses `minwebgl` for GL calls. Quad vertices are generated in
//! the vertex shader via `gl_VertexID` — no quad VAO needed.

use std::rc::Rc;
use std::cell::RefCell;
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

struct GpuTexture
{
  texture : web_sys::WebGlTexture,
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
    gl.draw_arrays( gl::TRIANGLE_STRIP, 0, 4 );
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
      gl.draw_elements_with_i32( topology, count as i32, gl::UNSIGNED_SHORT, 0 );
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
  resources : Rc< RefCell< GpuResources > >,
  sprite : SpriteRenderer,
  mesh : MeshRenderer,

  // -- batch recording state --
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
    gl.enable( gl::BLEND );
    gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );

    if !matches!( config.antialias, Antialias::None )
    {
      gl.enable( gl::SAMPLE_COVERAGE );
    }

    Ok( Self
    {
      config,
      gl,
      resources : Rc::new( RefCell::new( GpuResources::new() ) ),
      sprite,
      mesh,
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
      BlendMode::Normal => gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA ),
      BlendMode::Add => gl.blend_func( gl::SRC_ALPHA, gl::ONE ),
      BlendMode::Multiply => gl.blend_func( gl::DST_COLOR, gl::ONE_MINUS_SRC_ALPHA ),
      BlendMode::Screen => gl.blend_func( gl::ONE, gl::ONE_MINUS_SRC_COLOR ),
      BlendMode::Overlay => gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA ),
    }
  }

  // ---- Command handlers ----

  fn cmd_clear( &self, c : &Clear )
  {
    let [ r, g, b, a ] = c.color;
    self.gl.clear_color( r, g, b, a );
    self.gl.clear( gl::COLOR_BUFFER_BIT );
  }

  fn cmd_mesh( &self, m : &Mesh, viewport : &[ f32; 2 ] )
  {
    let res = self.resources.borrow();
    if let Some( geom ) = res.geometry( m.geometry )
    {
      let mat = m.transform.to_mat3();
      let color = match m.fill { FillRef::Solid( c ) => c, _ => [ 1.0, 1.0, 1.0, 1.0 ] };
      self.apply_blend( &m.blend );

      if let Some( tex_id ) = m.texture
      {
        if let Some( gpu_tex ) = res.texture( tex_id )
        {
          self.gl.active_texture( gl::TEXTURE0 );
          self.gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );
        }
      }

      self.mesh.draw( &self.gl, geom, &mat, &color, topology_to_gl( &m.topology ), viewport );
    }
  }

  fn cmd_sprite( &self, s : &Sprite, viewport : &[ f32; 2 ] )
  {
    let mat = s.transform.to_mat3();
    self.apply_blend( &s.blend );
    // TODO: look up SpriteAsset → sheet texture + region → uv_rect
    let uv_rect = [ 0.0, 0.0, 1.0, 1.0 ]; // placeholder: full texture
    self.sprite.draw( &self.gl, &mat, &uv_rect, &s.tint, viewport );
  }

  fn cmd_begin_record_sprite_batch( &mut self, brb : &BeginRecordSpriteBatch )
  {
    self.recording_batch = Some( brb.batch );
    self.instance_buffer_data.clear();
  }

  fn cmd_sprite_instance( &mut self, si : &SpriteInstance )
  {
    let mat = si.transform.to_mat3();
    self.instance_buffer_data.extend_from_slice( &mat );
    self.instance_buffer_data.extend_from_slice( &si.tint );
    self.instance_buffer_data.push( si.sprite.inner() as f32 );
  }

  fn cmd_end_record_sprite_batch( &mut self )
  {
    if let Some( _batch_id ) = self.recording_batch.take()
    {
      // TODO: create GPU buffer, store as GpuBatch::Sprite
      self.instance_buffer_data.clear();
    }
  }

  fn cmd_begin_record_mesh_batch( &mut self, brb : &BeginRecordMeshBatch )
  {
    self.recording_batch = Some( brb.batch );
    self.instance_buffer_data.clear();
  }

  fn cmd_mesh_instance( &mut self, mi : &MeshInstance )
  {
    let mat = mi.transform.to_mat3();
    self.instance_buffer_data.extend_from_slice( &mat );
  }

  fn cmd_end_record_mesh_batch( &mut self )
  {
    if let Some( _batch_id ) = self.recording_batch.take()
    {
      // TODO: create GPU buffer, store as GpuBatch::Mesh
      self.instance_buffer_data.clear();
    }
  }

  fn cmd_draw_batch( &self, db : &DrawBatch )
  {
    let res = self.resources.borrow();
    if let Some( gpu_batch ) = res.batch( db.batch )
    {
      match gpu_batch
      {
        GpuBatch::Sprite { .. } => self.sprite.draw_batch( &self.gl, gpu_batch ),
        GpuBatch::Mesh { .. } => self.mesh.draw_batch( &self.gl, gpu_batch ),
      }
    }
  }

  fn cmd_begin_update_batch( &mut self, bub : &BeginUpdateBatch )
  {
    self.recording_batch = Some( bub.batch );
  }

  fn cmd_end_update_batch( &mut self )
  {
    self.recording_batch = None;
  }

  fn cmd_delete_batch( &mut self, db : &DeleteBatch )
  {
    if let Some( b ) = self.resources.borrow_mut().batches.remove( &db.batch )
    {
      let buf = match &b
      {
        GpuBatch::Sprite { instance_buffer, .. } => instance_buffer,
        GpuBatch::Mesh { instance_buffer, .. } => instance_buffer,
      };
      self.gl.delete_buffer( Some( buf ) );
    }
  }

  // ---- Asset loading ----

  fn load_images( &mut self, images : &[ crate::assets::ImageAsset ] ) -> Result< (), RenderError >
  {
    let gl = &self.gl;
    self.resources.borrow_mut().textures.clear();

    for img in images
    {
      let texture = match &img.source
      {
        crate::assets::ImageSource::Bitmap { bytes, width, height, format } =>
        {
          let texture = gl.create_texture()
          .ok_or_else( || RenderError::BackendError( "failed to create texture".into() ) )?;

          gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );

          let gl_fmt = match format
          {
            crate::assets::PixelFormat::Rgba8 => gl::RGBA,
            crate::assets::PixelFormat::Rgb8 => gl::RGB,
            crate::assets::PixelFormat::Gray8 => gl::LUMINANCE,
            crate::assets::PixelFormat::GrayAlpha8 => gl::LUMINANCE_ALPHA,
          };

          let _ = gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
          (
            gl::TEXTURE_2D, 0, gl_fmt as i32,
            *width as i32, *height as i32, 0,
            gl_fmt, gl::UNSIGNED_BYTE, Some( bytes ),
          );

          texture
        }
        crate::assets::ImageSource::Encoded( _ ) => { continue; } // TODO: decode
        crate::assets::ImageSource::Path( path ) =>
        {
          let path = path.as_path().to_str()
            .ok_or_else( || RenderError::BackendError( "non-UTF-8 image path".into() ) )?;
          let texture = gl::texture::d2::upload_image_from_path( gl, path, true );
          gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
          texture
        }
      };

      apply_texture_filter( gl, &img.filter );
      gl::texture::d2::wrap_clamp( gl );

      self.resources.borrow_mut().store_texture( img.id, GpuTexture { texture, filter : img.filter } );
    }

    Ok( () )
  }

  fn load_geometries( &mut self, geometries : &[ crate::assets::GeometryAsset ] ) -> Result< (), RenderError >
  {
    let gl = &self.gl;
    let map_err = | e : gl::WebglError | RenderError::BackendError( format!( "{e:?}" ) );
    self.resources.borrow_mut().geometries.clear();

    for geom in geometries
    {
      let has_path =
        matches!( geom.positions, crate::assets::Source::Path( _ ) )
        || matches!( geom.uvs, Some( crate::assets::Source::Path( _ ) ) )
        || matches!( geom.indices, Some( crate::assets::Source::Path( _ ) ) );

      if has_path
      {
        // Create empty VAO and register geometry immediately so the id is available.
        // The spawn_local future will fetch data and populate the VAO later.
        let vao = gl::vao::create( gl ).map_err( map_err )?;
        self.resources.borrow_mut().store_geometry( geom.id, GpuGeometry { vao : vao.clone(), vertex_count : 0, index_count : None } );

        let gl_clone = gl.clone();
        let resources = Rc::clone( &self.resources );
        let id = geom.id;

        let positions_source = source_to_loadable( &geom.positions );
        let uvs_source = geom.uvs.as_ref().map( source_to_loadable );
        let indices_source = geom.indices.as_ref().map( source_to_loadable );

        gl::spawn_local( async move
        {
          let gl = &gl_clone;

          let positions = resolve_loadable( positions_source ).await;
          let uvs = match uvs_source { Some( s ) => Some( resolve_loadable( s ).await ), None => None };
          let indices = match indices_source { Some( s ) => Some( resolve_loadable( s ).await ), None => None };

          gl.bind_vertex_array( Some( &vao ) );

          // Positions (attrib 0)
          if let Some( ref bytes ) = positions
          {
            if let Ok( buffer ) = gl::buffer::create( gl )
            {
              gl::buffer::upload( gl, &buffer, bytes, gl::STATIC_DRAW );
              gl.enable_vertex_attrib_array( 0 );
              gl.vertex_attrib_pointer_with_i32( 0, 2, gl::FLOAT, false, 0, 0 );
            }
          }

          // UVs (attrib 1)
          if let Some( Some( ref bytes ) ) = uvs
          {
            if let Ok( buffer ) = gl::buffer::create( gl )
            {
              gl::buffer::upload( gl, &buffer, bytes, gl::STATIC_DRAW );
              gl.enable_vertex_attrib_array( 1 );
              gl.vertex_attrib_pointer_with_i32( 1, 2, gl::FLOAT, false, 0, 0 );
            }
          }

          // Indices
          let mut index_count = None;
          if let Some( Some( ref bytes ) ) = indices
          {
            if let Ok( buffer ) = gl::buffer::create( gl )
            {
              gl.bind_buffer( gl::ELEMENT_ARRAY_BUFFER, Some( &buffer ) );
              let u8_array = gl::js_sys::Uint8Array::from( bytes.as_slice() );
              gl.buffer_data_with_array_buffer_view( gl::ELEMENT_ARRAY_BUFFER, &u8_array, gl::STATIC_DRAW );
              index_count = Some( ( bytes.len() / 2 ) as u32 );
            }
          }

          gl.bind_vertex_array( None );

          let vertex_count = positions.as_ref().map_or( 0, | b | ( b.len() / 8 ) as u32 );

          // Update the existing entry with actual vertex/index counts.
          resources.borrow_mut().store_geometry( id, GpuGeometry { vao, vertex_count, index_count } );
        });
      }
      else
      {
        // Synchronous path — all data is already in memory.
        let vao = gl::vao::create( gl ).map_err( map_err )?;
        gl.bind_vertex_array( Some( &vao ) );

        if let crate::assets::Source::Bytes( ref bytes ) = geom.positions
        {
          let buffer = gl::buffer::create( gl ).map_err( map_err )?;
          gl::buffer::upload( gl, &buffer, bytes, gl::STATIC_DRAW );
          gl.enable_vertex_attrib_array( 0 );
          gl.vertex_attrib_pointer_with_i32( 0, 2, gl::FLOAT, false, 0, 0 );
        }

        if let Some( crate::assets::Source::Bytes( ref bytes ) ) = geom.uvs
        {
          let buffer = gl::buffer::create( gl ).map_err( map_err )?;
          gl::buffer::upload( gl, &buffer, bytes, gl::STATIC_DRAW );
          gl.enable_vertex_attrib_array( 1 );
          gl.vertex_attrib_pointer_with_i32( 1, 2, gl::FLOAT, false, 0, 0 );
        }

        let mut index_count = None;
        if let Some( crate::assets::Source::Bytes( ref bytes ) ) = geom.indices
        {
          let buffer = gl::buffer::create( gl ).map_err( map_err )?;
          gl.bind_buffer( gl::ELEMENT_ARRAY_BUFFER, Some( &buffer ) );
          let u8_array = js_sys::Uint8Array::from( bytes.as_slice() );
          gl.buffer_data_with_array_buffer_view( gl::ELEMENT_ARRAY_BUFFER, &u8_array, gl::STATIC_DRAW );
          index_count = Some( ( bytes.len() / 2 ) as u32 );
        }

        gl.bind_vertex_array( None );

        let vertex_count = if let crate::assets::Source::Bytes( ref bytes ) = geom.positions
        { ( bytes.len() / 8 ) as u32 } else { 0 };

        self.resources.borrow_mut().store_geometry( geom.id, GpuGeometry { vao, vertex_count, index_count } );
      }
    }

    Ok( () )
  }
}

// ============================================================================
// Backend trait impl
// ============================================================================

impl Backend for WebGlBackend
{
  fn load_assets( &mut self, assets : &Assets ) -> Result< (), RenderError >
  {
    self.load_images( &assets.images )?;
    self.load_geometries( &assets.geometries )?;
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
        RenderCommand::Clear( c ) => self.cmd_clear( c ),

        // Mesh & sprite
        RenderCommand::Mesh( m ) => self.cmd_mesh( m, &viewport ),
        RenderCommand::Sprite( s ) => self.cmd_sprite( s, &viewport ),

        // Sprite batch recording
        RenderCommand::BeginRecordSpriteBatch( brb ) => self.cmd_begin_record_sprite_batch( brb ),
        RenderCommand::SpriteInstance( si ) => self.cmd_sprite_instance( si ),
        RenderCommand::EndRecordSpriteBatch( _ ) => self.cmd_end_record_sprite_batch(),

        // Mesh batch recording
        RenderCommand::BeginRecordMeshBatch( brb ) => self.cmd_begin_record_mesh_batch( brb ),
        RenderCommand::MeshInstance( mi ) => self.cmd_mesh_instance( mi ),
        RenderCommand::EndRecordMeshBatch( _ ) => self.cmd_end_record_mesh_batch(),

        // Batch draw / update / delete
        RenderCommand::DrawBatch( db ) => self.cmd_draw_batch( db ),
        RenderCommand::BeginUpdateBatch( bub ) => self.cmd_begin_update_batch( bub ),
        RenderCommand::SetSpriteInstance( _ ) => {} // TODO
        RenderCommand::SetMeshInstance( _ ) => {} // TODO
        RenderCommand::EndUpdateBatch( _ ) => self.cmd_end_update_batch(),
        RenderCommand::DeleteBatch( db ) => self.cmd_delete_batch( db ),

        // Path — skip (TODO)
        RenderCommand::BeginPath( _ )
        | RenderCommand::MoveTo( _ )
        | RenderCommand::LineTo( _ )
        | RenderCommand::QuadTo( _ )
        | RenderCommand::CubicTo( _ )
        | RenderCommand::ArcTo( _ )
        | RenderCommand::ClosePath( _ )
        | RenderCommand::EndPath( _ ) => {}

        // Text — skip (TODO)
        RenderCommand::BeginText( _ )
        | RenderCommand::Char( _ )
        | RenderCommand::EndText( _ ) => {}

        // Grouping — skip (TODO)
        RenderCommand::BeginGroup( _ )
        | RenderCommand::EndGroup( _ ) => {}
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
// Shared utilities
// ============================================================================

/// Data source that can be sent into a `spawn_local` future.
/// Clones the bytes (for `Bytes`) or the path string (for `Path`).
enum Loadable
{
  Ready( Vec< u8 > ),
  Fetch( String ),
}

fn source_to_loadable( source : &crate::assets::Source ) -> Loadable
{
  match source
  {
    crate::assets::Source::Bytes( bytes ) => Loadable::Ready( bytes.clone() ),
    crate::assets::Source::Path( path ) => Loadable::Fetch( path.to_string_lossy().into_owned() ),
  }
}

async fn resolve_loadable( loadable : Loadable ) -> Option< Vec< u8 > >
{
  match loadable
  {
    Loadable::Ready( bytes ) => Some( bytes ),
    Loadable::Fetch( path ) => gl::file::load( &path ).await.ok(),
  }
}

fn apply_texture_filter( gl : &gl::GL, filter : &SamplerFilter )
{
  let f = match filter
  {
    SamplerFilter::Nearest => gl::texture::d2::filter_nearest( gl ),
    SamplerFilter::Linear => gl::texture::d2::filter_linear( gl ),
  };
}

fn topology_to_gl( t : &Topology ) -> u32
{
  match t
  {
    Topology::TriangleList => gl::TRIANGLES,
    Topology::TriangleStrip => gl::TRIANGLE_STRIP,
    Topology::LineList => gl::LINES,
    Topology::LineStrip => gl::LINE_STRIP,
  }
}
