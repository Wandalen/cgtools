//! WebGL adapter helpers.
//!
//! Extracted from `webgl.rs` to keep that file under the per-source-file size
//! budget. Contains self-contained types, POD instance data, and pure
//! mapping/setup helpers with no dependency on `WebGlBackend` internals.

mod private
{
  use core::cell::Cell;
  use core::marker::PhantomData;
  use minwebgl as gl;
  use nohash_hasher::IntMap;
  use crate::backend::RenderError;
  use crate::commands::{ SpriteBatchParams, MeshBatchParams };
  use crate::types::{ asset, Batch, BlendMode, ResourceId, SamplerFilter, MipmapMode, Topology };

  // ============================================================================
  // ArrayBuffer — GPU-side Vec<T>
  // ============================================================================

  /// GPU array buffer with `Vec`-like semantics.
  ///
  /// Stores elements of type `T` in a WebGL `ARRAY_BUFFER`.
  /// Tracks length and capacity; grows by 2× when full using
  /// `copy_buffer_sub_data` (GPU-to-GPU copy into a freshly allocated buffer).
  /// `swap_remove` uses a persistent one-element scratch buffer as an intermediary
  /// to avoid the WebGL2 spec violation of binding the same buffer to both
  /// `COPY_READ_BUFFER` and `COPY_WRITE_BUFFER` simultaneously.
  pub struct ArrayBuffer< T >
  {
    gl : gl::GL,
    buffer : web_sys::WebGlBuffer,
    /// One-element scratch buffer used by `swap_remove` as a GPU-side intermediary.
    scratch : web_sys::WebGlBuffer,
    len : u32,
    capacity : u32,
    _marker : PhantomData< T >,
  }

  impl< T : gl::AsBytes > ArrayBuffer< T >
  {
    /// Creates a new GPU array buffer with the given initial capacity (in elements).
    ///
    /// Allocates two GPU buffers: the main data buffer (`capacity * stride` bytes)
    /// and a one-element scratch buffer (`stride` bytes) used by `swap_remove`.
    ///
    /// # Errors
    /// Returns `WebglError` if any GPU buffer cannot be created, or if
    /// `capacity * stride` overflows `i32` (WebGL buffer size limit).
    pub fn new( gl : &gl::GL, capacity : u32 ) -> Result< Self, gl::WebglError >
    {
      let buffer = gl::buffer::create( gl )?;
      let byte_size = capacity
        .checked_mul( Self::stride() )
        .and_then( | n | i32::try_from( n ).ok() )
        .ok_or( gl::WebglError::FailedToAllocateResource( "Buffer" ) )?;
      gl.bind_buffer( gl::ARRAY_BUFFER, Some( &buffer ) );
      gl.buffer_data_with_i32( gl::ARRAY_BUFFER, byte_size, gl::DYNAMIC_DRAW );
      gl.bind_buffer( gl::ARRAY_BUFFER, None );

      let scratch = gl::buffer::create( gl )?;
      gl.bind_buffer( gl::ARRAY_BUFFER, Some( &scratch ) );
      gl.buffer_data_with_i32( gl::ARRAY_BUFFER, Self::stride() as i32, gl::DYNAMIC_DRAW );
      gl.bind_buffer( gl::ARRAY_BUFFER, None );

      Ok( Self { gl : gl.clone(), buffer, scratch, len : 0, capacity, _marker : PhantomData } )
    }

    /// Byte size of one element.
    fn stride() -> u32
    {
      core::mem::size_of::< T >() as u32
    }

    /// Number of elements currently stored.
    #[ must_use ]
    pub fn len( &self ) -> u32 { self.len }

    /// Whether the buffer is empty.
    #[ must_use ]
    pub fn is_empty( &self ) -> bool { self.len == 0 }

    /// Returns a reference to the underlying `WebGlBuffer`.
    #[ must_use ]
    pub fn buffer( &self ) -> &web_sys::WebGlBuffer { &self.buffer }

    /// Appends an element at the end, growing if necessary.
    ///
    /// # Errors
    /// Returns `WebglError` if the GPU buffer needs to grow and allocation fails.
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
    ///
    /// # Panics
    /// Panics if `index >= len`.
    pub fn swap_remove( &mut self, index : u32 ) -> u32
    {
      assert!( index < self.len, "ArrayBuffer::swap_remove index out of bounds" );
      self.len -= 1;
      if index < self.len
      {
        let stride = Self::stride() as i32;
        let src_offset = self.len as i32 * stride;
        let dst_offset = index as i32 * stride;

        // Binding the same buffer to both COPY_READ_BUFFER and COPY_WRITE_BUFFER is a
        // WebGL2 spec violation (INVALID_OPERATION). Use a persistent one-element scratch
        // buffer as an intermediary: last → scratch → removed slot. Both copies use
        // distinct buffer objects, so the spec is satisfied and the copies are GPU-only.
        self.gl.bind_buffer( gl::COPY_READ_BUFFER, Some( &self.buffer ) );
        self.gl.bind_buffer( gl::COPY_WRITE_BUFFER, Some( &self.scratch ) );
        self.gl.copy_buffer_sub_data_with_i32_and_i32_and_i32
        (
          gl::COPY_READ_BUFFER,
          gl::COPY_WRITE_BUFFER,
          src_offset,
          0,
          stride,
        );
        self.gl.bind_buffer( gl::COPY_READ_BUFFER, Some( &self.scratch ) );
        self.gl.bind_buffer( gl::COPY_WRITE_BUFFER, Some( &self.buffer ) );
        self.gl.copy_buffer_sub_data_with_i32_and_i32_and_i32
        (
          gl::COPY_READ_BUFFER,
          gl::COPY_WRITE_BUFFER,
          0,
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
      // saturating_mul avoids wrapping; the byte_size check below catches any overflow.
      let new_capacity = self.capacity.saturating_mul( 2 ).max( 4 );
      let new_byte_size = new_capacity
        .checked_mul( Self::stride() )
        .and_then( | n | i32::try_from( n ).ok() )
        .ok_or( gl::WebglError::FailedToAllocateResource( "Buffer" ) )?;

      let new_buffer = gl::buffer::create( &self.gl )?;
      self.gl.bind_buffer( gl::ARRAY_BUFFER, Some( &new_buffer ) );
      self.gl.buffer_data_with_i32( gl::ARRAY_BUFFER, new_byte_size, gl::DYNAMIC_DRAW );
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
      self.gl.delete_buffer( Some( &self.scratch ) );
    }
  }

  // ============================================================================
  // Instance data for batches
  // ============================================================================

  /// Per-instance data for sprite batches (18 floats = 72 bytes).
  #[ repr( C ) ]
  #[ derive( Clone, Copy, bytemuck::Zeroable, bytemuck::Pod ) ]
  pub struct SpriteInstanceData
  {
    /// Row-major 3×3 affine transform as 9 f32s (column-major layout on GPU).
    pub transform : [ f32; 9 ],
    /// Sub-rect in the source atlas: `[ u_min, v_min, u_max, v_max ]`.
    pub region : [ f32; 4 ],
    /// Per-instance tint multiplied into the sampled texel (premultiplied RGBA).
    pub tint : [ f32; 4 ],
    /// Depth value in `[0, 1]` used by the depth test.
    pub depth : f32,
  }

  impl gl::AsBytes for SpriteInstanceData
  {
    fn as_bytes( &self ) -> &[ u8 ] { bytemuck::bytes_of( self ) }
    fn len( &self ) -> usize { 1 }
  }

  /// Per-instance data for mesh batches (10 floats = 40 bytes).
  #[ repr( C ) ]
  #[ derive( Clone, Copy, bytemuck::Zeroable, bytemuck::Pod ) ]
  pub struct MeshInstanceData
  {
    /// Row-major 3×3 affine transform as 9 f32s (column-major layout on GPU).
    pub transform : [ f32; 9 ],
    /// Depth value in `[0, 1]` used by the depth test.
    pub depth : f32,
  }

  impl gl::AsBytes for MeshInstanceData
  {
    fn as_bytes( &self ) -> &[ u8 ] { bytemuck::bytes_of( self ) }
    fn len( &self ) -> usize { 1 }
  }

  // Compile-time layout assertions — GPU attrib setup depends on these exact sizes.
  const _ : () = assert!( core::mem::size_of::< SpriteInstanceData >() == 72 ); // 18 floats × 4
  const _ : () = assert!( core::mem::size_of::< MeshInstanceData >() == 40 ); // 10 floats × 4
  const _ : () = assert!( core::mem::align_of::< SpriteInstanceData >() == 4 ); // f32 alignment
  const _ : () = assert!( core::mem::align_of::< MeshInstanceData >() == 4 );

  // ============================================================================
  // GPU resource handles
  // ============================================================================

  /// Manages GPU-side resources: textures and geometry buffers.
  #[ derive( Default ) ]
  pub struct GpuResources
  {
    /// Texture cache keyed by image asset id.
    pub textures : IntMap< ResourceId< asset::Image >, GpuTexture >,
    /// Sprite cache keyed by sprite asset id.
    pub sprites : IntMap< ResourceId< asset::Sprite >, GpuSprite >,
    /// Geometry cache keyed by geometry asset id.
    pub geometries : IntMap< ResourceId< asset::Geometry >, GpuGeometry >,
    /// Active batches keyed by batch id.
    pub batches : IntMap< ResourceId< Batch >, GpuBatch >,
    /// Incremented on every `load_assets` call. In-flight `spawn_local` futures
    /// from a previous cycle capture the old value and bail out on resolve,
    /// so stale async data cannot overwrite freshly-loaded entries.
    pub generation : u32,
  }

  impl GpuResources
  {
    /// Creates an empty resource cache.
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Looks up a texture by image asset id.
    #[ must_use ]
    pub fn texture( &self, id : ResourceId< asset::Image > ) -> Option< &GpuTexture >
    {
      self.textures.get( &id )
    }

    /// Looks up a sprite by sprite asset id.
    #[ must_use ]
    pub fn sprite( &self, id : ResourceId< asset::Sprite > ) -> Option< &GpuSprite >
    {
      self.sprites.get( &id )
    }

    /// Looks up geometry by geometry asset id.
    #[ must_use ]
    pub fn geometry( &self, id : ResourceId< asset::Geometry > ) -> Option< &GpuGeometry >
    {
      self.geometries.get( &id )
    }

    /// Looks up a batch by batch id.
    #[ must_use ]
    pub fn batch( &self, id : ResourceId< Batch > ) -> Option< &GpuBatch >
    {
      self.batches.get( &id )
    }

    /// Mutable batch lookup (for instance add / set / remove operations).
    pub fn batch_mut( &mut self, id : ResourceId< Batch > ) -> Option< &mut GpuBatch >
    {
      self.batches.get_mut( &id )
    }

    /// Inserts a texture into the cache.
    pub fn store_texture( &mut self, id : ResourceId< asset::Image >, tex : GpuTexture )
    {
      self.textures.insert( id, tex );
    }

    /// Inserts a sprite into the cache.
    pub fn store_sprite( &mut self, id : ResourceId< asset::Sprite >, sprite : GpuSprite )
    {
      self.sprites.insert( id, sprite );
    }

    /// Inserts geometry into the cache.
    pub fn store_geometry( &mut self, id : ResourceId< asset::Geometry >, geom : GpuGeometry )
    {
      self.geometries.insert( id, geom );
    }

    /// Inserts a batch into the cache.
    pub fn store_batch( &mut self, id : ResourceId< Batch >, batch : GpuBatch )
    {
      self.batches.insert( id, batch );
    }
  }

  /// GPU-side texture handle plus cached natural size (filled once the image loads).
  pub struct GpuTexture
  {
    /// GL context held for cleanup in `Drop`.
    pub gl : gl::GL,
    /// The underlying GL texture object.
    pub texture : web_sys::WebGlTexture,
    /// Natural width in pixels — populated when the async image load completes.
    pub width : Cell< u32 >,
    /// Natural height in pixels — populated when the async image load completes.
    pub height : Cell< u32 >,
    /// Sampler filter recorded at creation time; kept for parity with future re-applies.
    pub filter : SamplerFilter,
    /// Mipmap mode recorded at creation time; kept for parity with future re-applies.
    pub mipmap : MipmapMode,
  }

  impl Drop for GpuTexture
  {
    fn drop( &mut self )
    {
      self.gl.delete_texture( Some( &self.texture ) );
    }
  }

  /// Sprite lookup data: sheet reference + pixel region.
  /// UV rect and size are computed at draw time from the sheet's dimensions.
  pub struct GpuSprite
  {
    /// Sheet texture to bind.
    pub sheet : ResourceId< asset::Image >,
    /// Region within the sheet: `[x, y, w, h]` in pixels.
    pub region : [ f32; 4 ],
  }

  /// GPU-side geometry: VAO plus the backing buffers.
  pub struct GpuGeometry
  {
    /// GL context held for cleanup in `Drop`.
    pub gl : gl::GL,
    /// VAO pre-configured with position/uv/index attribs.
    pub vao : web_sys::WebGlVertexArrayObject,
    /// Position buffer (attrib 0). `None` if the geometry has no positions.
    pub position_buffer : Option< web_sys::WebGlBuffer >,
    /// UV buffer (attrib 1). `None` if the geometry has no UVs.
    pub uv_buffer : Option< web_sys::WebGlBuffer >,
    /// Index buffer. `None` if the geometry draws via `drawArrays`.
    pub index_buffer : Option< web_sys::WebGlBuffer >,
    /// Vertex count used when `index_count` is `None`.
    pub vertex_count : u32,
    /// Number of indices and the GL type constant (`UNSIGNED_BYTE` / `UNSIGNED_SHORT` / `UNSIGNED_INT`).
    /// `None` if the geometry has no index buffer (draw with `drawArrays`).
    pub index_count : Option< ( u32, u32 ) >,
  }

  impl Drop for GpuGeometry
  {
    fn drop( &mut self )
    {
      self.gl.delete_vertex_array( Some( &self.vao ) );
      if let Some( ref buf ) = self.position_buffer { self.gl.delete_buffer( Some( buf ) ); }
      if let Some( ref buf ) = self.uv_buffer { self.gl.delete_buffer( Some( buf ) ); }
      if let Some( ref buf ) = self.index_buffer { self.gl.delete_buffer( Some( buf ) ); }
    }
  }

  // ============================================================================
  // GpuBatch — persistent instance buffer + VAO
  // ============================================================================

  /// Persistent batch — sprite or mesh.
  pub enum GpuBatch
  {
    /// Sprite batch: sprite-instance buffer and the parameters that define the batch.
    Sprite
    {
      /// GL context handle held for cleanup in `Drop`.
      gl : gl::GL,
      /// Per-instance sprite data (transform / region / tint / depth).
      instances : ArrayBuffer< SpriteInstanceData >,
      /// VAO holding the instance attrib bindings (locations 0–5).
      vao : web_sys::WebGlVertexArrayObject,
      /// Batch-wide parameters (atlas / blend).
      params : SpriteBatchParams,
    },
    /// Mesh batch: mesh-instance buffer bound alongside a geometry's VBOs.
    Mesh
    {
      /// GL context handle held for cleanup in `Drop`.
      gl : gl::GL,
      /// Per-instance mesh data (transform / depth).
      instances : ArrayBuffer< MeshInstanceData >,
      /// VAO holding geometry attribs (0–1) and instance attribs (2–5).
      vao : web_sys::WebGlVertexArrayObject,
      /// Batch-wide parameters (geometry / texture / blend).
      params : MeshBatchParams,
    },
  }

  impl Drop for GpuBatch
  {
    fn drop( &mut self )
    {
      match self
      {
        Self::Sprite { gl, vao, .. } | Self::Mesh { gl, vao, .. } =>
          gl.delete_vertex_array( Some( vao ) ),
      }
    }
  }

  // ============================================================================
  // VAO setup helpers
  // ============================================================================

  /// Binds instance attrib pointers for a sprite batch VAO.
  pub fn setup_sprite_batch_vao( gl : &gl::GL, vao : &web_sys::WebGlVertexArrayObject, buffer : &web_sys::WebGlBuffer )
  {
    gl.bind_vertex_array( Some( vao ) );
    gl.bind_buffer( gl::ARRAY_BUFFER, Some( buffer ) );

    let stride = core::mem::size_of::< SpriteInstanceData >() as i32;

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
    // depth: float at location 5, offset 68
    gl.enable_vertex_attrib_array( 5 );
    gl.vertex_attrib_pointer_with_i32( 5, 1, gl::FLOAT, false, stride, 68 );
    gl.vertex_attrib_divisor( 5, 1 );

    gl.bind_vertex_array( None );
  }

  /// Sets up a mesh batch VAO with geometry attribs (0–1) and instance attribs (2–5).
  ///
  /// Takes the geometry buffers directly (position, uv, index) as `Option`s so this
  /// helper stays decoupled from the adapter's internal `GpuGeometry` struct.
  pub fn setup_mesh_batch_vao
  (
    gl : &gl::GL,
    vao : &web_sys::WebGlVertexArrayObject,
    position_buffer : Option< &web_sys::WebGlBuffer >,
    uv_buffer : Option< &web_sys::WebGlBuffer >,
    index_buffer : Option< &web_sys::WebGlBuffer >,
    instance_buffer : &web_sys::WebGlBuffer,
  )
  {
    gl.bind_vertex_array( Some( vao ) );

    // Geometry: positions (attrib 0)
    if let Some( buf ) = position_buffer
    {
      gl.bind_buffer( gl::ARRAY_BUFFER, Some( buf ) );
      gl.enable_vertex_attrib_array( 0 );
      gl.vertex_attrib_pointer_with_i32( 0, 2, gl::FLOAT, false, 0, 0 );
    }

    // Geometry: UVs (attrib 1)
    if let Some( buf ) = uv_buffer
    {
      gl.bind_buffer( gl::ARRAY_BUFFER, Some( buf ) );
      gl.enable_vertex_attrib_array( 1 );
      gl.vertex_attrib_pointer_with_i32( 1, 2, gl::FLOAT, false, 0, 0 );
    }

    // Geometry: indices
    if let Some( buf ) = index_buffer
    {
      gl.bind_buffer( gl::ELEMENT_ARRAY_BUFFER, Some( buf ) );
    }

    // Instance: transform 3 × vec3 at locations 2, 3, 4
    gl.bind_buffer( gl::ARRAY_BUFFER, Some( instance_buffer ) );
    let stride = core::mem::size_of::< MeshInstanceData >() as i32;
    for i in 0..3_u32
    {
      let loc = i + 2;
      gl.enable_vertex_attrib_array( loc );
      gl.vertex_attrib_pointer_with_i32( loc, 3, gl::FLOAT, false, stride, ( i * 12 ) as i32 );
      gl.vertex_attrib_divisor( loc, 1 );
    }
    // depth: float at location 5, offset 36
    gl.enable_vertex_attrib_array( 5 );
    gl.vertex_attrib_pointer_with_i32( 5, 1, gl::FLOAT, false, stride, 36 );
    gl.vertex_attrib_divisor( 5, 1 );

    gl.bind_vertex_array( None );
  }

  // ============================================================================
  // Async load helpers
  // ============================================================================

  /// Data source that can be sent into a `spawn_local` future.
  /// Clones the bytes (for `Bytes`) or the path string (for `Path`).
  pub enum Loadable
  {
    /// Bytes are already in memory — no I/O needed.
    Ready( Vec< u8 > ),
    /// Path to fetch asynchronously via `gl::file::load`.
    Fetch( String ),
  }

  /// Converts an `assets::Source` into an owned `Loadable` that can cross
  /// an `async` boundary (clones bytes or paths).
  #[ must_use ]
  pub fn source_to_loadable( source : &crate::assets::Source ) -> Loadable
  {
    match source
    {
      crate::assets::Source::Bytes( bytes ) => Loadable::Ready( bytes.clone() ),
      crate::assets::Source::Path( path ) => Loadable::Fetch( path.to_string_lossy().into_owned() ),
    }
  }

  /// Resolves a `Loadable` to bytes — trivial pass-through for `Ready`,
  /// or an async fetch for `Fetch`. Returns `None` on fetch failure.
  pub async fn resolve_loadable( loadable : Loadable ) -> Option< Vec< u8 > >
  {
    match loadable
    {
      Loadable::Ready( bytes ) => Some( bytes ),
      Loadable::Fetch( path ) => gl::file::load( &path ).await.ok(),
    }
  }

  // ============================================================================
  // Pure GL state / format mappers
  // ============================================================================

  /// Maps a `DataType` to `(bytes_per_index, gl_type_constant)` for `drawElements`.
  ///
  /// # Errors
  /// Returns `Err` for `F32`, which is not a valid WebGL index type.
  pub fn index_format( dt : &crate::assets::DataType ) -> Result< ( u32, u32 ), RenderError >
  {
    use crate::assets::DataType;
    match dt
    {
      DataType::U8  => Ok( ( 1, gl::UNSIGNED_BYTE ) ),
      DataType::U16 => Ok( ( 2, gl::UNSIGNED_SHORT ) ),
      DataType::U32 => Ok( ( 4, gl::UNSIGNED_INT ) ),
      DataType::F32 => Err( RenderError::BackendError
      (
        "GeometryAsset.data_type: F32 is not a valid index format; use U8, U16, or U32".into()
      )),
    }
  }

  /// Sets mag/min filter on the currently bound `TEXTURE_2D` based on `filter` and `mipmap`.
  /// Caller is responsible for calling `generate_mipmap` separately when `mipmap != Off`
  /// (after the level-0 upload completes — on the async path that's inside the `on_load` callback).
  pub fn apply_texture_filter( gl : &gl::GL, filter : &SamplerFilter, mipmap : &MipmapMode )
  {
    // mag_filter ignores mipmaps — magnification samples only level 0.
    let mag = match filter
    {
      SamplerFilter::Nearest => gl::NEAREST,
      SamplerFilter::Linear => gl::LINEAR,
    };
    // min_filter combines within-level interpolation with between-level interpolation.
    let min = match ( filter, mipmap )
    {
      ( SamplerFilter::Nearest, MipmapMode::Off )     => gl::NEAREST,
      ( SamplerFilter::Linear,  MipmapMode::Off )     => gl::LINEAR,
      ( SamplerFilter::Nearest, MipmapMode::Nearest ) => gl::NEAREST_MIPMAP_NEAREST,
      ( SamplerFilter::Linear,  MipmapMode::Nearest ) => gl::LINEAR_MIPMAP_NEAREST,
      ( SamplerFilter::Nearest, MipmapMode::Linear )  => gl::NEAREST_MIPMAP_LINEAR,
      ( SamplerFilter::Linear,  MipmapMode::Linear )  => gl::LINEAR_MIPMAP_LINEAR,
    };
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min as i32 );
  }

  /// Programs the GL blend function for a given `BlendMode`.
  ///
  /// Alpha factors are always `(ONE, ONE_MINUS_SRC_ALPHA)` so the framebuffer
  /// alpha follows Porter-Duff "over": `a = src_a + dst_a * (1 - src_a)`. Using
  /// the RGB factors on the alpha channel would produce wrong framebuffer alpha
  /// (e.g. `src_a^2` under `Normal`) and break readPixels / compositing onto a
  /// transparent canvas background.
  pub fn apply_blend( gl : &gl::GL, blend : &BlendMode )
  {
    match blend
    {
      // Color: src + dst. Alpha: standard over.
      BlendMode::Add => gl.blend_func_separate( gl::SRC_ALPHA, gl::ONE, gl::ONE, gl::ONE_MINUS_SRC_ALPHA ),
      // Approximation: diverges from Photoshop Multiply when src_alpha < 1 — the
      // DST_COLOR factor multiplies dst by raw src.rgb (not src.rgb*src_a), so
      // partially transparent sources darken the destination more than the
      // reference formula prescribes. Exact only when src_alpha = 1.
      // qqq(FBO): replace with Photoshop-accurate formula — see BlendMode::Multiply doc.
      // Color: src*dst + dst*(1-src_a). Alpha: standard over.
      BlendMode::Multiply => gl.blend_func_separate( gl::DST_COLOR, gl::ONE_MINUS_SRC_ALPHA, gl::ONE, gl::ONE_MINUS_SRC_ALPHA ),
      // Same class of approximation as Multiply: the ONE / ONE_MINUS_SRC_COLOR
      // factors use raw src.rgb, so a partially transparent source still
      // contributes its full (unmultiplied) color. Exact only when src_alpha = 1
      // or when the source is premultiplied.
      // Color: src + dst*(1-src). Alpha: standard over.
      BlendMode::Screen => gl.blend_func_separate( gl::ONE, gl::ONE_MINUS_SRC_COLOR, gl::ONE, gl::ONE_MINUS_SRC_ALPHA ),
      // qqq: true Overlay (Multiply where dst<0.5, Screen where dst>0.5) cannot be
      // expressed as a single blend_func call — it requires a custom shader or a
      // separate FBO read-back pass, neither of which is implemented yet.
      // Overlay falls back to Normal so rendering is at least visible; the
      // console.warn below makes the silent substitution observable.
      BlendMode::Overlay =>
      {
        web_sys::console::warn_1
        (
          &"BlendMode::Overlay is not supported in WebGL2 without an FBO pass; falling back to Normal".into()
        );
        gl.blend_func_separate( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ONE, gl::ONE_MINUS_SRC_ALPHA );
      }
      // Color: src*src_a + dst*(1-src_a). Alpha: standard over.
      BlendMode::Normal => gl.blend_func_separate( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ONE, gl::ONE_MINUS_SRC_ALPHA ),
    }
  }

  /// Maps a `Topology` to the corresponding WebGL primitive constant.
  #[ must_use ]
  pub fn topology_to_gl( t : &Topology ) -> u32
  {
    match t
    {
      Topology::TriangleList => gl::TRIANGLES,
      Topology::TriangleStrip => gl::TRIANGLE_STRIP,
      Topology::LineList => gl::LINES,
      Topology::LineStrip => gl::LINE_STRIP,
    }
  }
}

mod_interface::mod_interface!
{
  own use ArrayBuffer;
  own use SpriteInstanceData;
  own use MeshInstanceData;
  own use GpuResources;
  own use GpuTexture;
  own use GpuSprite;
  own use GpuGeometry;
  own use GpuBatch;
  own use setup_sprite_batch_vao;
  own use setup_mesh_batch_vao;
  own use apply_blend;
  own use Loadable;
  own use source_to_loadable;
  own use resolve_loadable;
  own use index_format;
  own use apply_texture_filter;
  own use topology_to_gl;
}
