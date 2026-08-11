mod private
{
  use crate::{ From, mem, GL, WebGlBuffer, Into, WebGlProgram, js_sys, JsValue };

  /// Upload data to a uniform block object.
  #[ inline ]
  pub fn upload< Data >
  (
    gl : &GL,
    buffer : &WebGlBuffer,
    block_point : u32,
    buffer_data : &Data,
    data_usage : u32,
  )
  where
    Data : mem::AsBytes + ?Sized,
  {
    gl.bind_buffer_base( GL::UNIFORM_BUFFER, block_point, Some( buffer ) );
    gl.buffer_data_with_u8_array( GL::UNIFORM_BUFFER, mem::cast_slice( buffer_data.as_bytes() ), data_usage );
  }

  /// Contains comprehensive diagnostics information about a Uniform Block Object (UBO).
  #[ cfg( feature = "diagnostics" ) ]
  #[ derive( Debug ) ]
  #[ non_exhaustive ]
  pub struct UbosInfo
  {
    /// The name of the uniform block.
    pub block_name : String,
    /// Block index.
    pub block_index : u32,
    /// Block binding point.
    pub block_binding_point : i32,
    /// The number of active uniforms within the block.
    pub active_uniforms : i32,
    /// The total size of the uniform block in bytes.
    pub block_size : i32,
    /// Is uniform block used in vertex shader.
    pub block_is_used_in_vertex_shader : bool,
    /// Is uniform block used in fragment shader.
    pub block_is_used_in_fragment_shader : bool,
    /// A list of `UboInfo` structures, each describing an individual uniform within the block.
    pub uniforms : Vec< UboInfo >,
  }

  /// Represents diagnostics information about a single uniform within a Uniform Block Object (UBO).
  #[ cfg( feature = "diagnostics" ) ]
  #[ derive( Debug ) ]
  #[ non_exhaustive ]
  pub struct UboInfo
  {
    /// The index of the uniform within the UBO.
    pub index : u32,
    /// The byte offset of the uniform within the UBO.
    pub offset : u32,
    /// The stride between elements in an array of this uniform, if applicable.
    pub array_stride : i32,
    /// The stride between columns in a matrix uniform, if applicable.
    pub matrix_stride : i32,
    /// Indicates whether the matrix is stored in row-major order. `None` if not applicable.
    pub is_row_major : Option< i32 >,
  }

  /// Uniform buffer object ID. Either block index of block name.
  #[ cfg( feature = "diagnostics" ) ]
  #[ derive( Debug, From ) ]
  pub enum BlockId
  {
    BlockName( String ),
    BlockIndex( u32 ),
  }

  /// Converts a WebGL block/uniform parameter's `f64` value ( as returned by
  /// `getActiveUniformBlockParameter`/`getActiveUniforms` ) to `i32`.
  // WebGL block/uniform parameters read here ( binding point, byte size, uniform count,
  // array/matrix stride ) are always small, driver-bounded counts per the WebGL2 spec's
  // UNIFORM_BLOCK_*/UNIFORM_* parameter definitions -- realistic values never approach
  // i32::MAX, so this single narrow conversion point is safe by construction.
  #[ cfg( feature = "diagnostics" ) ]
  fn param_as_i32( value : f64 ) -> i32
  {
    value as i32
  }

  /// Converts a WebGL parameter's `f64` value to `u32` ( for ids/offsets that are never negative ).
  // Uniform indices and byte offsets read here are never negative and always small,
  // driver-bounded counts per the WebGL2 spec -- realistic values never approach `u32::MAX`,
  // so this single narrow conversion point is safe by construction.
  #[ cfg( feature = "diagnostics" ) ]
  fn param_as_u32( value : f64 ) -> u32
  {
    value as u32
  }

  /// Reads a numeric ( `f64` ) uniform-block parameter via `getActiveUniformBlockParameter`.
  ///
  /// # Panics
  /// Panics if the query returns no value or a non-numeric one -- every `pname` this is
  /// called with is a `UNIFORM_BLOCK_*` query guaranteed numeric by the WebGL2 spec.
  #[ cfg( feature = "diagnostics" ) ]
  #[ inline ]
  fn block_param_f64( gl : &GL, program : &WebGlProgram, block_index : u32, pname : u32 ) -> f64
  {
    gl.get_active_uniform_block_parameter( program, block_index, pname ).unwrap().as_f64().unwrap()
  }

  /// Reads a boolean uniform-block parameter via `getActiveUniformBlockParameter`.
  ///
  /// # Panics
  /// Panics if the query returns no value or a non-boolean one -- every `pname` this is
  /// called with is a `UNIFORM_BLOCK_REFERENCED_BY_*` query guaranteed boolean by the WebGL2 spec.
  #[ cfg( feature = "diagnostics" ) ]
  #[ inline ]
  fn block_param_bool( gl : &GL, program : &WebGlProgram, block_index : u32, pname : u32 ) -> bool
  {
    gl.get_active_uniform_block_parameter( program, block_index, pname ).unwrap().as_bool().unwrap()
  }

  /// Reads a numeric ( `f64` ) per-uniform parameter via `getActiveUniforms` for a single index.
  ///
  /// # Panics
  /// Panics if the query returns no value or a non-numeric one -- every `pname` this is
  /// called with is a `UNIFORM_*` query guaranteed numeric by the WebGL2 spec.
  #[ cfg( feature = "diagnostics" ) ]
  #[ inline ]
  fn uniform_param_f64( gl : &GL, program : &WebGlProgram, index_js_value : &JsValue, pname : u32 ) -> f64
  {
    js_sys::Array::from( &gl.get_active_uniforms( program, index_js_value, pname ) )
    .get( 0 )
    .as_f64()
    .unwrap()
  }

  /// Gathers diagnostics ( offset, array/matrix stride, row-major flag ) for a single active
  /// uniform within a UBO.
  #[ cfg( feature = "diagnostics" ) ]
  #[ inline ]
  fn collect_uniform_info( gl : &GL, program : &WebGlProgram, index : u32 ) -> UboInfo
  {
    let index_js_value = js_sys::Array::of1( &JsValue::from( index ) );

    let offset = param_as_u32( uniform_param_f64( gl, program, &index_js_value, GL::UNIFORM_OFFSET ) );
    let array_stride = param_as_i32( uniform_param_f64( gl, program, &index_js_value, GL::UNIFORM_ARRAY_STRIDE ) );
    let matrix_stride = param_as_i32( uniform_param_f64( gl, program, &index_js_value, GL::UNIFORM_MATRIX_STRIDE ) );
    let is_row_major = js_sys::Array::from( &gl.get_active_uniforms( program, &index_js_value, GL::UNIFORM_IS_ROW_MAJOR ) )
    .get( 0 )
    .as_f64()
    .map( param_as_i32 );

    UboInfo
    {
      index,
      offset,
      array_stride,
      matrix_stride,
      is_row_major,
    }
  }

  /// Resolves a `BlockId` to its `( block_index, block_name )` pair.
  #[ cfg( feature = "diagnostics" ) ]
  #[ inline ]
  fn resolve_block_id( gl : &GL, program : &WebGlProgram, block_id : BlockId ) -> ( u32, String )
  {
    match block_id
    {
      BlockId::BlockName( block_name ) =>
      {
        let block_index = gl.get_uniform_block_index( program, &block_name );
        ( block_index, block_name.clone() )
      },
      BlockId::BlockIndex( block_index ) =>
      {
        let block_name = gl.get_active_uniform_block_name( program, block_index ).unwrap_or_default();
        ( block_index, block_name )
      },
    }
  }

  /// Retrieves diagnostic information about a Uniform Block Object (UBO).
  ///
  /// This function gathers detailed information about a UBO, including its size,
  /// binding point, usage in shaders, and details about each uniform within the block.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL context.
  /// * `program` - The WebGL program containing the UBO.
  /// * `block_index` - The index of the uniform block within the program.
  /// * `block_name` - The name of the uniform block.
  ///
  /// # Returns
  ///
  /// * `UbosInfo` - A struct containing comprehensive diagnostic information about the UBO.
  ///
  /// # Panics
  /// Panics if any expected UBO/uniform parameter query returns no value or an
  /// unexpectedly-typed one -- every query here targets a `pname` the WebGL2 spec
  /// guarantees is present and typed as queried for an active uniform block.
  #[ cfg( feature = "diagnostics" ) ]
  #[ inline ]
  pub fn diagnostic_info< IntoBlockId >
  (
    gl : &GL,
    program : &WebGlProgram,
    block_id : IntoBlockId,
  )
  -> UbosInfo
  where
    IntoBlockId : Into< BlockId >,
  {
    let ( block_index, block_name ) = resolve_block_id( gl, program, block_id.into() );

    let block_binding_point = param_as_i32( block_param_f64( gl, program, block_index, GL::UNIFORM_BLOCK_BINDING ) );
    let block_size = param_as_i32( block_param_f64( gl, program, block_index, GL::UNIFORM_BLOCK_DATA_SIZE ) );
    let block_is_used_in_vertex_shader = block_param_bool( gl, program, block_index, GL::UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER );
    let block_is_used_in_fragment_shader = block_param_bool( gl, program, block_index, GL::UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER );
    let active_uniforms = param_as_i32( block_param_f64( gl, program, block_index, GL::UNIFORM_BLOCK_ACTIVE_UNIFORMS ) );

    let indices_js_value = gl.get_active_uniform_block_parameter( program, block_index, GL::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES ).unwrap();
    let indices : Vec< u32 > = js_sys::Array::from( &indices_js_value )
    .iter()
    .map( | x | param_as_u32( x.as_f64().unwrap() ) )
    .collect();

    let uniforms = indices.iter().map( | &index | collect_uniform_info( gl, program, index ) ).collect();

    UbosInfo
    {
      block_name,
      block_index,
      block_binding_point,
      active_uniforms,
      block_size,
      block_is_used_in_vertex_shader,
      block_is_used_in_fragment_shader,
      uniforms,
    }

  }

}

crate::mod_interface!
{

  own use
  {
    upload,
  };

  #[ cfg( feature = "diagnostics" ) ]
  own use
  {
    UboInfo,
    UbosInfo,
    diagnostic_info,
  };

}
