use super::*;

impl IntoVectorDataType for f32
{
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
  fn into_vector_data_type() -> VectorDataType
  {
    VectorDataType
    {
      scalar : DataType::F32,
      natoms : N as _,
      nelements : 1,
    }
  }
}

// qqq : xxx : implement similar for other primitive types
impl< const N : usize, const N2 : usize > IntoVectorDataType for [ [ f32 ; N2 ] ; N ]
{
  fn into_vector_data_type() -> VectorDataType
  {
    VectorDataType
    {
      scalar : DataType::F32,
      natoms : ( N * N2 ) as i32,
      nelements : N2 as _,
    }
  }
}
