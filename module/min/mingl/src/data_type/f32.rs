use super::*;

impl IntoVectorDataType for f32
{
  fn into_vector_data_type() -> VectorDataType
  {
    VectorDataType
    {
      scalar : DataType::F32,
      len : 1,
      element_len : 1,
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
      len : N as _,
      element_len : 1,
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
      len : ( N * N2 ) as i32,
      element_len : N2 as _,
    }
  }
}
