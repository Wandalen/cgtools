mod private
{
  use crate::*;

  /// Upload data to a uniform block object.
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
  #[ cfg( feature = "diagnostics" ) ]
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
    let block_id = block_id.into();
    let ( block_index, block_name ) : ( u32, String ) = match block_id
    {
      BlockId::BlockName( block_name ) =>
      {
        let block_index = gl.get_uniform_block_index( program, &block_name );
        ( block_index, block_name.to_string() )
      },
      BlockId::BlockIndex( ref block_index ) =>
      {
        let name = gl.get_active_uniform_block_name( program, *block_index );
        let block_name = if let Some( name ) = name
        {
          name
        }
        else
        {
          "".to_string()
        };
        ( *block_index, block_name )
      },
    };

    // Get the block binding point
    let block_binding_point = gl.get_active_uniform_block_parameter
    (
      program,
      block_index,
      GL::UNIFORM_BLOCK_BINDING
    )
    .unwrap()
    .as_f64()
    .unwrap() as i32;

    // Get the size of the block
    let block_size = gl.get_active_uniform_block_parameter
    (
      program,
      block_index,
      GL::UNIFORM_BLOCK_DATA_SIZE
    )
    .unwrap()
    .as_f64()
    .unwrap() as i32;

    // Block is used in vertex shader
    let block_is_used_in_vertex_shader = gl.get_active_uniform_block_parameter
    (
      program,
      block_index,
      GL::UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER
    )
    .unwrap()
    .as_bool()
    .unwrap() as bool;

    // Block is used in fragment shader
    let block_is_used_in_fragment_shader = gl.get_active_uniform_block_parameter
    (
      program,
      block_index,
      GL::UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER
    )
    .unwrap()
    .as_bool()
    .unwrap() as bool;

    // Get the number of active uniforms in the block
    let active_uniforms = gl.get_active_uniform_block_parameter
    (
      program,
      block_index,
      GL::UNIFORM_BLOCK_ACTIVE_UNIFORMS
    )
    .unwrap()
    .as_f64()
    .unwrap() as i32;

    // Get the indices of the active uniforms
    let indices_js_value = gl.get_active_uniform_block_parameter
    (
      program,
      block_index,
      GL::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES
    )
    .unwrap();
    let indices : Vec< u32 > = js_sys::Array::from( &indices_js_value )
    .iter()
    .map( | x | x.as_f64().unwrap() as u32 )
    .collect();

    // Retrieve and store information about each uniform
    let mut uniforms = Vec::new();
    for &index in &indices
    {
      let index_js_value = js_sys::Array::of1( &JsValue::from( index ) );

      let offset_js_value = gl.get_active_uniforms
      (
        program,
        &index_js_value,
        GL::UNIFORM_OFFSET
      );
      let offset = js_sys::Array::from( &offset_js_value )
      .get( 0 )
      .as_f64()
      .unwrap() as u32;

      let array_stride_js_value = gl.get_active_uniforms
      (
        program,
        &index_js_value,
        GL::UNIFORM_ARRAY_STRIDE
      );
      let array_stride = js_sys::Array::from( &array_stride_js_value )
      .get( 0 )
      .as_f64()
      .unwrap() as i32;

      let matrix_stride_js_value = gl.get_active_uniforms
      (
        program,
        &index_js_value,
        GL::UNIFORM_MATRIX_STRIDE
      );
      let matrix_stride = js_sys::Array::from( &matrix_stride_js_value )
      .get( 0 )
      .as_f64()
      .unwrap() as i32;

      let is_row_major_js_value = gl.get_active_uniforms
      (
        program,
        &index_js_value,
        GL::UNIFORM_IS_ROW_MAJOR
      );
      let is_row_major = js_sys::Array::from( &is_row_major_js_value )
      .get( 0 )
      .as_f64()
      .map( | v | v as i32 );

      uniforms.push
      (
        UboInfo
        {
          index,
          offset,
          array_stride,
          matrix_stride,
          is_row_major,
        }
      );
    }

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
