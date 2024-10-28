use super::*;

impl IntoVectorDataType for i16
{
  fn into_vector_data_type() -> VectorDataType
  {
    VectorDataType
    {
      scalar : DataType::I16,
      len : 1,
      element_len : 1,
    }
  }
}

impl< const N : usize > IntoVectorDataType for [ i16 ; N ]
{
  fn into_vector_data_type() -> VectorDataType
  {
    VectorDataType
    {
      scalar : DataType::I16,
      len : N as _,
      element_len : 1,
    }
  }
}

impl< const N : usize, const N2 : usize > IntoVectorDataType for [ [ i16 ; N2 ] ; N ]
{
  fn into_vector_data_type() -> VectorDataType
  {
    VectorDataType
    {
      scalar : DataType::I16,
      len : ( N * N2 ) as i32,
      element_len : N2 as _,
    }
  }
}
