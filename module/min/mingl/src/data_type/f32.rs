use super::{ DataType, IntoVectorDataType, VectorDataType, dim_as_i32 };

impl IntoVectorDataType for f32
{
  #[ inline ]
  fn into_vector_data_type() -> VectorDataType
  {
    VectorDataType
    {
      scalar : DataType::F32,
      natoms : 1,
      nelements : 1,
    }
  }
}

impl< const N : usize > IntoVectorDataType for [ f32 ; N ]
{
  #[ inline ]
  fn into_vector_data_type() -> VectorDataType
  {
    VectorDataType
    {
      scalar : DataType::F32,
      natoms : dim_as_i32( N ),
      nelements : 1,
    }
  }
}

impl< const N : usize, const N2 : usize > IntoVectorDataType for [ [ f32 ; N2 ] ; N ]
{
  #[ inline ]
  fn into_vector_data_type() -> VectorDataType
  {
    VectorDataType
    {
      scalar : DataType::F32,
      natoms : dim_as_i32( N * N2 ),
      nelements : dim_as_i32( N2 ),
    }
  }
}
