//! WebGL backend adapter.
//!
//! Hardware-accelerated 2D rendering via WebGL2 (wasm32 target).
//! Uses `minwebgl` for GL calls. Quad vertices are generated in
//! the vertex shader via `gl_VertexID` — no quad VAO needed.

use std::rc::Rc;
use std::cell::{ Cell, RefCell };
use std::marker::PhantomData;
use web_sys::HtmlImageElement;
use wasm_bindgen::prelude::*;
use minwebgl as gl;
use nohash_hasher::IntMap;
use crate::assets::Assets;
use crate::backend::*;
use crate::commands::*;
use crate::types::*;

// ============================================================================
// ArrayBuffer — GPU-side Vec<T>
// ============================================================================

/// GPU array buffer with `Vec`-like semantics.
///
/// Stores elements of type `T` in a WebGL `ARRAY_BUFFER`.
/// Tracks length and capacity; grows by 2× when full using
/// `copy_buffer_sub_data` (GPU-to-GPU, no CPU readback).
pub struct ArrayBuffer< T >
{
  gl : gl::GL,
  buffer : web_sys::WebGlBuffer,
  len : u32,
  capacity : u32,
  _marker : PhantomData< T >,
}

impl< T : gl::AsBytes > ArrayBuffer< T >
{
  /// Creates a new GPU array buffer with the given initial capacity (in elements).
  pub fn new( gl : &gl::GL, capacity : u32 ) -> Result< Self, gl::WebglError >
  {
    let buffer = gl::buffer::create( gl )?;
    let byte_size = capacity * Self::stride();
    gl.bind_buffer( gl::ARRAY_BUFFER, Some( &buffer ) );
    gl.buffer_data_with_i32( gl::ARRAY_BUFFER, byte_size as i32, gl::DYNAMIC_DRAW );
    gl.bind_buffer( gl::ARRAY_BUFFER, None );
    Ok( Self { gl : gl.clone(), buffer, len : 0, capacity, _marker : PhantomData } )
  }

  /// Byte size of one element.
  fn stride() -> u32
  {
    std::mem::size_of::< T >() as u32
  }

  /// Number of elements currently stored.
  pub fn len( &self ) -> u32 { self.len }

  /// Whether the buffer is empty.
  pub fn is_empty( &self ) -> bool { self.len == 0 }

  /// Current capacity in elements.
  pub fn capacity( &self ) -> u32 { self.capacity }

  /// Returns a reference to the underlying `WebGlBuffer`.
  pub fn buffer( &self ) -> &web_sys::WebGlBuffer { &self.buffer }

  /// Appends an element at the end, growing if necessary.
  pub fn push( &mut self, value : &T ) -> Result< (), gl::WebglError >
  {
    if self.len >= self.capacity
    {
      self.grow()?;
    }
    let offset = self.len * Self::stride();
    self.gl.bind_buffer( gl::ARRAY_BUFFER, Some( &self.buffer ) );
    self.gl.buffer_sub_data_with_i32_and_u8_array( gl::ARRAY_BUFFER, offset as i32, value.as_bytes() );
    self.gl.bind_buffer( gl::ARRAY_BUFFER, None );
    self.len += 1;
    Ok( () )
  }

  /// Updates the element at `index` in-place.
  ///
  /// # Panics
  /// Panics if `index >= len`.
  pub fn set( &self, index : u32, value : &T )
  {
    assert!( index < self.len, "ArrayBuffer::set index out of bounds" );
    let offset = index * Self::stride();
    self.gl.bind_buffer( gl::ARRAY_BUFFER, Some( &self.buffer ) );
    self.gl.buffer_sub_data_with_i32_and_u8_array( gl::ARRAY_BUFFER, offset as i32, value.as_bytes() );
    self.gl.bind_buffer( gl::ARRAY_BUFFER, None );
  }

  /// Removes the element at `index` by swapping with the last element.
  /// Returns the new length.
  pub fn swap_remove( &mut self, index : u32 ) -> u32
  {
    assert!( index < self.len, "ArrayBuffer::swap_remove index out of bounds" );
    self.len -= 1;
    if index < self.len
    {
      let stride = Self::stride() as i32;
      let src_offset = self.len as i32 * stride;
      let dst_offset = index as i32 * stride;

      self.gl.bind_buffer( gl::COPY_READ_BUFFER, Some( &self.buffer ) );
      self.gl.bind_buffer( gl::COPY_WRITE_BUFFER, Some( &self.buffer ) );
      self.gl.copy_buffer_sub_data_with_i32_and_i32_and_i32
      (
        gl::COPY_READ_BUFFER,
        gl::COPY_WRITE_BUFFER,
        src_offset,
        dst_offset,
        stride,
      );
      self.gl.bind_buffer( gl::COPY_READ_BUFFER, None );
      self.gl.bind_buffer( gl::COPY_WRITE_BUFFER, None );
    }
    self.len
  }

  /// Doubles the capacity, copying old data GPU-to-GPU.
  fn grow( &mut self ) -> Result< (), gl::WebglError >
  {
    let new_capacity = if self.capacity == 0 { 4 } else { self.capacity * 2 };
    let new_byte_size = new_capacity * Self::stride();

    let new_buffer = gl::buffer::create( &self.gl )?;
    self.gl.bind_buffer( gl::ARRAY_BUFFER, Some( &new_buffer ) );
    self.gl.buffer_data_with_i32( gl::ARRAY_BUFFER, new_byte_size as i32, gl::DYNAMIC_DRAW );
    self.gl.bind_buffer( gl::ARRAY_BUFFER, None );

    if self.len > 0
    {
      let copy_bytes = self.len * Self::stride();
      self.gl.bind_buffer( gl::COPY_READ_BUFFER, Some( &self.buffer ) );
      self.gl.bind_buffer( gl::COPY_WRITE_BUFFER, Some( &new_buffer ) );
      self.gl.copy_buffer_sub_data_with_i32_and_i32_and_i32
      (
        gl::COPY_READ_BUFFER,
        gl::COPY_WRITE_BUFFER,
        0,
        0,
        copy_bytes as i32,
      );
      self.gl.bind_buffer( gl::COPY_READ_BUFFER, None );
      self.gl.bind_buffer( gl::COPY_WRITE_BUFFER, None );
    }

    self.gl.delete_buffer( Some( &self.buffer ) );
    self.buffer = new_buffer;
    self.capacity = new_capacity;
    Ok( () )
  }
}

impl< T > Drop for ArrayBuffer< T >
{
  fn drop( &mut self )
  {
    self.gl.delete_buffer( Some( &self.buffer ) );
  }
}

// ============================================================================
// GPU resource handles
// ============================================================================

/// Manages GPU-side resources: textures and geometry buffers.
struct GpuResources
{
  textures : IntMap< ResourceId< asset::Image >, GpuTexture >,
  sprites : IntMap< ResourceId< asset::Sprite >, GpuSprite >,
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
      sprites : IntMap::default(),
      geometries : IntMap::default(),
      batches : IntMap::default(),
    }
  }

  fn texture( &self, id : ResourceId< asset::Image > ) -> Option< &GpuTexture >
  {
    self.textures.get( &id )
  }

  fn sprite( &self, id : ResourceId< asset::Sprite > ) -> Option< &GpuSprite >
  {
    self.sprites.get( &id )
  }

  fn geometry( &self, id : ResourceId< asset::Geometry > ) -> Option< &GpuGeometry >
  {
    self.geometries.get( &id )
  }

  fn batch( &self, id : ResourceId< Batch > ) -> Option< &GpuBatch >
  {
    self.batches.get( &id )
  }

  fn batch_mut( &mut self, id : ResourceId< Batch > ) -> Option< &mut GpuBatch >
  {
    self.batches.get_mut( &id )
  }

  fn store_texture( &mut self, id : ResourceId< asset::Image >, tex : GpuTexture )
  {
    self.textures.insert( id, tex );
  }

  fn store_sprite( &mut self, id : ResourceId< asset::Sprite >, sprite : GpuSprite )
  {
    self.sprites.insert( id, sprite );
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
  width : Cell< u32 >,
  height : Cell< u32 >,
  _filter : SamplerFilter,
}

/// Sprite lookup data: sheet reference + pixel region.
/// UV rect and size are computed at draw time from the sheet's dimensions.
struct GpuSprite
{
  /// Sheet texture to bind.
  sheet : ResourceId< asset::Image >,
  /// Region within the sheet: `[x, y, w, h]` in pixels.
  region : [ f32; 4 ],
}

struct GpuGeometry
{
  vao : web_sys::WebGlVertexArrayObject,
  vertex_count : u32,
  index_count : Option< u32 >,
}

// ---- Instance data for batches ----

/// Per-instance data for sprite batches (17 floats = 68 bytes).
#[ repr( C ) ]
#[ derive( Clone, Copy ) ]
struct SpriteInstanceData
{
  transform : [ f32; 9 ],
  region : [ f32; 4 ],
  tint : [ f32; 4 ],
}

unsafe impl bytemuck::Zeroable for SpriteInstanceData {}
unsafe impl bytemuck::Pod for SpriteInstanceData {}

impl gl::AsBytes for SpriteInstanceData
{
  fn as_bytes( &self ) -> &[ u8 ] { bytemuck::bytes_of( self ) }
  fn len( &self ) -> usize { 1 }
}

/// Per-instance data for mesh batches (9 floats = 36 bytes).
#[ repr( C ) ]
#[ derive( Clone, Copy ) ]
struct MeshInstanceData
{
  transform : [ f32; 9 ],
}

unsafe impl bytemuck::Zeroable for MeshInstanceData {}
unsafe impl bytemuck::Pod for MeshInstanceData {}

impl gl::AsBytes for MeshInstanceData
{
  fn as_bytes( &self ) -> &[ u8 ] { bytemuck::bytes_of( self ) }
  fn len( &self ) -> usize { 1 }
}

/// Persistent batch — sprite or mesh.
enum GpuBatch
{
  Sprite
  {
    instances : ArrayBuffer< SpriteInstanceData >,
    vao : web_sys::WebGlVertexArrayObject,
    params : SpriteBatchParams,
  },
  Mesh
  {
    instances : ArrayBuffer< MeshInstanceData >,
    params : MeshBatchParams,
  },
}

/// Binds instance attrib pointers for a sprite batch VAO.
fn setup_sprite_batch_vao( gl : &gl::GL, vao : &web_sys::WebGlVertexArrayObject, buffer : &web_sys::WebGlBuffer )
{
  gl.bind_vertex_array( Some( vao ) );
  gl.bind_buffer( gl::ARRAY_BUFFER, Some( buffer ) );

  let stride = std::mem::size_of::< SpriteInstanceData >() as i32;

  // transform: 3 × vec3 at locations 0, 1, 2
  for i in 0..3_u32
  {
    gl.enable_vertex_attrib_array( i );
    gl.vertex_attrib_pointer_with_i32( i, 3, gl::FLOAT, false, stride, ( i * 12 ) as i32 );
    gl.vertex_attrib_divisor( i, 1 );
  }
  // region: vec4 at location 3, offset 36
  gl.enable_vertex_attrib_array( 3 );
  gl.vertex_attrib_pointer_with_i32( 3, 4, gl::FLOAT, false, stride, 36 );
  gl.vertex_attrib_divisor( 3, 1 );
  // tint: vec4 at location 4, offset 52
  gl.enable_vertex_attrib_array( 4 );
  gl.vertex_attrib_pointer_with_i32( 4, 4, gl::FLOAT, false, stride, 52 );
  gl.vertex_attrib_divisor( 4, 1 );
}

// ============================================================================
// Sprite renderer
// ============================================================================

/// Handles single sprite draws and sprite batch instancing.
/// Quad is generated in vertex shader from `gl_VertexID` (triangle strip, 4 vertices).
struct SpriteRenderer
{
  program : gl::Program,
  batch_program : gl::Program,
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
    let batch_program = gl::Program::new
    (
      gl.clone(),
      include_str!( "shaders/sprite_batch.vert" ),
      include_str!( "shaders/sprite_batch.frag" ),
    )?;
    Ok( Self { program, batch_program } )
  }

  /// Draw a single sprite as a textured quad (triangle strip, 4 vertices from gl_VertexID).
  fn draw( &self, gl : &gl::GL, transform : &[ f32; 9 ], uv_rect : &[ f32; 4 ], sprite_size : &[ f32; 2 ], tint : &[ f32; 4 ], viewport : &[ f32; 2 ] )
  {
    self.program.activate();
    self.program.uniform_matrix_upload( "u_transform", transform.as_slice(), true );
    self.program.uniform_upload( "u_uv_rect", uv_rect );
    self.program.uniform_upload( "u_sprite_size", sprite_size );
    self.program.uniform_upload( "u_tint", tint );
    self.program.uniform_upload( "u_viewport", viewport );
    gl.draw_arrays( gl::TRIANGLE_STRIP, 0, 4 );
  }

  /// Draw an instanced sprite batch.
  fn draw_batch( &self, gl : &gl::GL, batch : &GpuBatch, resources : &GpuResources, viewport : &[ f32; 2 ] )
  {
    let GpuBatch::Sprite { instances, vao, params } = batch else { return; };
    if instances.is_empty() { return; }

    let Some( gpu_tex ) = resources.texture( params.sheet ) else { return; };
    let tw = gpu_tex.width.get();
    let th = gpu_tex.height.get();
    if tw == 0 || th == 0 { return; }

    gl.active_texture( gl::TEXTURE0 );
    gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );

    self.batch_program.activate();
    self.batch_program.uniform_upload( "u_viewport", viewport );
    self.batch_program.uniform_upload( "u_tex_size", &[ tw as f32, th as f32 ] );
    let parent_mat = params.transform.to_mat3();
    self.batch_program.uniform_matrix_upload( "u_parent", &parent_mat, true );

    gl.bind_vertex_array( Some( vao ) );
    gl.draw_arrays_instanced( gl::TRIANGLE_STRIP, 0, 4, instances.len() as i32 );
  }
}

// ============================================================================
// Mesh renderer
// ============================================================================

/// Handles single mesh draws and mesh batch instancing.
struct MeshRenderer
{
  program : gl::Program,
  batch_program : gl::Program,
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
    let batch_program = gl::Program::new
    (
      gl.clone(),
      include_str!( "shaders/mesh_batch.vert" ),
      include_str!( "shaders/mesh.frag" ),
    )?;
    Ok( Self { program, batch_program } )
  }

  /// Draw a single mesh.
  fn draw
  (
    &self,
    gl : &gl::GL,
    geom : &GpuGeometry,
    transform : &[ f32; 9 ],
    color : &[ f32; 4 ],
    topology : u32,
    viewport : &[ f32; 2 ],
    use_texture : bool
  )
  {
    self.program.activate();
    self.program.uniform_matrix_upload( "u_transform", transform.as_slice(), true );
    self.program.uniform_upload( "u_color", color );
    self.program.uniform_upload( "u_viewport", viewport );
    self.program.uniform_upload( "u_use_texture", &( use_texture as i32 ) );

    gl.bind_vertex_array( Some( &geom.vao ) );

    if let Some( count ) = geom.index_count
    {
      gl.draw_elements_with_i32( topology, count as i32, gl::UNSIGNED_SHORT, 0 );
    }
    else
    {
      gl.draw_arrays( topology, 0, geom.vertex_count as i32 );
    }
  }

  /// Draw an instanced mesh batch.
  fn draw_batch( &self, gl : &gl::GL, batch : &GpuBatch, resources : &GpuResources, viewport : &[ f32; 2 ] )
  {
    let GpuBatch::Mesh { instances, params } = batch else { return };
    if instances.is_empty() { return; }

    let Some( geom ) = resources.geometry( params.geometry ) else { return };
    let color = match params.fill { FillRef::Solid( c ) => c, _ => [ 1.0, 1.0, 1.0, 1.0 ] };
    let topology = topology_to_gl( &params.topology );

    let mut use_texture = false;
    if let Some( tex_id ) = params.texture
    {
      if let Some( gpu_tex ) = resources.texture( tex_id )
      {
        gl.active_texture( gl::TEXTURE0 );
        gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );
        use_texture = true;
      }
    }

    self.batch_program.activate();
    self.batch_program.uniform_upload( "u_viewport", viewport );
    self.batch_program.uniform_upload( "u_color", &color );
    self.batch_program.uniform_upload( "u_use_texture", &( use_texture as i32 ) );
    let parent_mat = params.transform.to_mat3();
    self.batch_program.uniform_matrix_upload( "u_parent", &parent_mat, true );

    // Bind geometry VAO and add instance attribs temporarily
    gl.bind_vertex_array( Some( &geom.vao ) );
    gl.bind_buffer( gl::ARRAY_BUFFER, Some( instances.buffer() ) );

    let stride = std::mem::size_of::< MeshInstanceData >() as i32;

    for i in 0..3_u32
    {
      let loc = i + 2;
      gl.enable_vertex_attrib_array( loc );
      gl.vertex_attrib_pointer_with_i32( loc, 3, gl::FLOAT, false, stride, ( i * 12 ) as i32 );
      gl.vertex_attrib_divisor( loc, 1 );
    }

    if let Some( count ) = geom.index_count
    {
      gl.draw_elements_instanced_with_i32( topology, count as i32, gl::UNSIGNED_SHORT, 0, instances.len() as i32 );
    }
    else
    {
      gl.draw_arrays_instanced( topology, 0, geom.vertex_count as i32, instances.len() as i32 );
    }
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

  // -- batch editing state --
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

      let mut use_texture = false;
      if let Some( tex_id ) = m.texture && let Some( gpu_tex ) = res.texture( tex_id )
      {
        self.gl.active_texture( gl::TEXTURE0 );
        self.gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );
        use_texture = true;
      }

      self.mesh.draw( &self.gl, geom, &mat, &color, topology_to_gl( &m.topology ), viewport, use_texture );
    }
  }

  fn cmd_sprite( &self, s : &Sprite, viewport : &[ f32; 2 ] )
  {
    let res = self.resources.borrow();
    let Some( gpu_sprite ) = res.sprite( s.sprite ) else { return };
    let Some( gpu_tex ) = res.texture( gpu_sprite.sheet ) else { return };

    let tw = gpu_tex.width.get();
    let th = gpu_tex.height.get();
    if tw == 0 || th == 0 { return; } // image not loaded yet

    self.gl.active_texture( gl::TEXTURE0 );
    self.gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );

    let [ rx, ry, rw, rh ] = gpu_sprite.region;
    let tw = tw as f32;
    let th = th as f32;
    let uv_rect = [ rx / tw, ry / th, rw / tw, rh / th ];
    let sprite_size = [ rw, rh ];

    let mat = s.transform.to_mat3();
    self.apply_blend( &s.blend );
    self.sprite.draw( &self.gl, &mat, &uv_rect, &sprite_size, &s.tint, viewport );
  }

  fn cmd_create_sprite_batch( &mut self, cmd : &CreateSpriteBatch )
  {
    let gl = &self.gl;
    let Ok( instances ) = ArrayBuffer::< SpriteInstanceData >::new( gl, 16 ) else { return };
    let Ok( vao ) = gl::vao::create( gl ) else { return };
    setup_sprite_batch_vao( gl, &vao, instances.buffer() );
    self.resources.borrow_mut().store_batch( cmd.batch, GpuBatch::Sprite
    {
      instances,
      vao,
      params : cmd.params,
    });
  }

  fn cmd_create_mesh_batch( &mut self, cmd : &CreateMeshBatch )
  {
    let Ok( instances ) = ArrayBuffer::< MeshInstanceData >::new( &self.gl, 16 ) else { return };
    self.resources.borrow_mut().store_batch( cmd.batch, GpuBatch::Mesh
    {
      instances,
      params : cmd.params,
    });
  }

  fn cmd_bind_batch( &mut self, cmd : &BindBatch )
  {
    self.recording_batch = Some( cmd.batch );
  }

  fn cmd_add_sprite_instance( &mut self, si : &AddSpriteInstance )
  {
    let Some( batch_id ) = self.recording_batch else { return };
    let mut res = self.resources.borrow_mut();
    let Some( region ) = res.sprite( si.sprite ).map( | s | s.region ) else { return };
    let data = SpriteInstanceData
    {
      transform : si.transform.to_mat3(),
      region,
      tint : si.tint,
    };
    if let Some( GpuBatch::Sprite { instances, vao, .. } ) = res.batch_mut( batch_id )
    {
      let old_cap = instances.capacity();
      let _ = instances.push( &data );
      if instances.capacity() != old_cap
      {
        setup_sprite_batch_vao( &self.gl, vao, instances.buffer() );
      }
    }
  }

  fn cmd_add_mesh_instance( &mut self, mi : &AddMeshInstance )
  {
    let Some( batch_id ) = self.recording_batch else { return };
    let data = MeshInstanceData { transform : mi.transform.to_mat3() };
    let mut res = self.resources.borrow_mut();
    if let Some( GpuBatch::Mesh { instances, .. } ) = res.batch_mut( batch_id )
    {
      let _ = instances.push( &data );
    }
  }

  fn cmd_set_sprite_instance( &mut self, si : &SetSpriteInstance )
  {
    let Some( batch_id ) = self.recording_batch else { return };
    let mut res = self.resources.borrow_mut();
    let Some( region ) = res.sprite( si.sprite ).map( | s | s.region ) else { return };
    let data = SpriteInstanceData
    {
      transform : si.transform.to_mat3(),
      region,
      tint : si.tint,
    };
    if let Some( GpuBatch::Sprite { instances, .. } ) = res.batch_mut( batch_id )
    {
      instances.set( si.index, &data );
    }
  }

  fn cmd_set_mesh_instance( &mut self, mi : &SetMeshInstance )
  {
    let Some( batch_id ) = self.recording_batch else { return };
    let data = MeshInstanceData { transform : mi.transform.to_mat3() };
    let mut res = self.resources.borrow_mut();
    if let Some( GpuBatch::Mesh { instances, .. } ) = res.batch_mut( batch_id )
    {
      instances.set( mi.index, &data );
    }
  }

  fn cmd_remove_instance( &mut self, ri : &RemoveInstance )
  {
    let Some( batch_id ) = self.recording_batch else { return };
    let mut res = self.resources.borrow_mut();
    if let Some( batch ) = res.batch_mut( batch_id )
    {
      match batch
      {
        GpuBatch::Sprite { instances, .. } => { instances.swap_remove( ri.index ); },
        GpuBatch::Mesh { instances, .. } => { instances.swap_remove( ri.index ); },
      }
    }
  }

  fn cmd_set_sprite_batch_params( &mut self, cmd : &SetSpriteBatchParams )
  {
    let Some( batch_id ) = self.recording_batch else { return };
    let mut res = self.resources.borrow_mut();
    if let Some( GpuBatch::Sprite { params, .. } ) = res.batch_mut( batch_id )
    {
      *params = cmd.params;
    }
  }

  fn cmd_set_mesh_batch_params( &mut self, cmd : &SetMeshBatchParams )
  {
    let Some( batch_id ) = self.recording_batch else { return };
    let mut res = self.resources.borrow_mut();
    if let Some( GpuBatch::Mesh { params, .. } ) = res.batch_mut( batch_id )
    {
      *params = cmd.params;
    }
  }

  fn cmd_unbind_batch( &mut self )
  {
    self.recording_batch = None;
  }

  fn cmd_draw_batch( &self, db : &DrawBatch, viewport : &[ f32; 2 ] )
  {
    let res = self.resources.borrow();
    if let Some( gpu_batch ) = res.batch( db.batch )
    {
      self.apply_blend( match gpu_batch
      {
        GpuBatch::Sprite { params, .. } => &params.blend,
        GpuBatch::Mesh { params, .. } => &params.blend,
      });
      match gpu_batch
      {
        GpuBatch::Sprite { .. } => self.sprite.draw_batch( &self.gl, gpu_batch, &res, viewport ),
        GpuBatch::Mesh { .. } => self.mesh.draw_batch( &self.gl, gpu_batch, &res, viewport ),
      }
    }
  }

  fn cmd_delete_batch( &mut self, db : &DeleteBatch )
  {
    // ArrayBuffer::drop handles GPU buffer cleanup
    self.resources.borrow_mut().batches.remove( &db.batch );
  }

  // ---- Asset loading ----

  fn load_images( &mut self, images : &[ crate::assets::ImageAsset ] ) -> Result< (), RenderError >
  {
    let gl = &self.gl;
    self.resources.borrow_mut().textures.clear();

    for img in images
    {
      let ( texture, w, h ) = match &img.source
      {
        crate::assets::ImageSource::Bitmap { bytes, width, height, format } =>
        {
          let tex = gl.create_texture()
          .ok_or_else( || RenderError::BackendError( "failed to create texture".into() ) )?;

          gl.bind_texture( gl::TEXTURE_2D, Some( &tex ) );

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

          ( tex, *width, *height )
        }
        crate::assets::ImageSource::Encoded( _ ) => { continue; } // TODO: decode
        crate::assets::ImageSource::Path( path ) =>
        {
          let path = path.as_path().to_str()
            .ok_or_else( || RenderError::BackendError( "non-UTF-8 image path".into() ) )?;
          let tex = upload_image_from_path( gl, path, img.id, &self.resources );
          gl.bind_texture( gl::TEXTURE_2D, Some( &tex ) );
          ( tex, 0, 0 )
        }
      };

      apply_texture_filter( gl, &img.filter );
      gl::texture::d2::wrap_clamp( gl );

      self.resources.borrow_mut().store_texture( img.id, GpuTexture
      {
        texture,
        width : Cell::new( w ),
        height : Cell::new( h ),
        _filter : img.filter,
      });
    }

    Ok( () )
  }

  fn load_sprites( &mut self, sprites : &[ crate::assets::SpriteAsset ] )
  {
    self.resources.borrow_mut().sprites.clear();

    for spr in sprites
    {
      self.resources.borrow_mut().store_sprite( spr.id, GpuSprite
      {
        sheet : spr.sheet,
        region : spr.region,
      });
    }
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
    self.load_sprites( &assets.sprites );
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

        // Batch lifecycle
        RenderCommand::CreateSpriteBatch( c ) => self.cmd_create_sprite_batch( c ),
        RenderCommand::CreateMeshBatch( c ) => self.cmd_create_mesh_batch( c ),
        RenderCommand::BindBatch( b ) => self.cmd_bind_batch( b ),
        RenderCommand::AddSpriteInstance( si ) => self.cmd_add_sprite_instance( si ),
        RenderCommand::AddMeshInstance( mi ) => self.cmd_add_mesh_instance( mi ),
        RenderCommand::SetSpriteInstance( si ) => self.cmd_set_sprite_instance( si ),
        RenderCommand::SetMeshInstance( mi ) => self.cmd_set_mesh_instance( mi ),
        RenderCommand::RemoveInstance( ri ) => self.cmd_remove_instance( ri ),
        RenderCommand::SetSpriteBatchParams( sp ) => self.cmd_set_sprite_batch_params( sp ),
        RenderCommand::SetMeshBatchParams( mp ) => self.cmd_set_mesh_batch_params( mp ),
        RenderCommand::UnbindBatch( _ ) => self.cmd_unbind_batch(),
        RenderCommand::DrawBatch( db ) => self.cmd_draw_batch( db, &viewport ),
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
      max_texture_size : 8192,
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

/// Like `gl::texture::d2::upload_image_from_path`, but updates
/// `GpuTexture.width` / `height` cells once the image loads.
fn upload_image_from_path
(
  gl : &gl::GL,
  src : &str,
  id : ResourceId< asset::Image >,
  resources : &Rc< RefCell< GpuResources > >,
) -> web_sys::WebGlTexture
{
  let document = web_sys::window().expect( "no window" ).document().expect( "no document" );

  let texture = gl.create_texture().expect( "failed to create texture" );

  let img : HtmlImageElement = document.create_element( "img" )
    .expect( "can't create img" )
    .dyn_into()
    .expect( "not an HtmlImageElement" );
  img.style().set_property( "display", "none" ).expect( "can't hide img" );

  let on_load : Closure< dyn Fn() > = Closure::new(
  {
    let gl = gl.clone();
    let img = img.clone();
    let texture = texture.clone();
    let resources = Rc::clone( resources );
    move ||
    {
      gl::texture::d2::upload( &gl, Some( &texture ), &img );

      if let Some( gpu_tex ) = resources.borrow().texture( id )
      {
        gpu_tex.width.set( img.natural_width() );
        gpu_tex.height.set( img.natural_height() );
      }

      img.remove();
    }
  });

  img.set_onload( Some( on_load.as_ref().unchecked_ref() ) );
  img.set_src( src );
  on_load.forget();

  texture
}

fn apply_texture_filter( gl : &gl::GL, filter : &SamplerFilter )
{
  match filter
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
