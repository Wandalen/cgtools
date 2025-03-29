use super::*;

impl IntoVectorDataType for i16
{
  fn into_vector_data_type() -> VectorDataType
  {
    VectorDataType
    {
      scalar : DataType::I16,
      natoms : 1,
      nelements : 1,
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
      natoms : N as _,
      nelements : 1,
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
      natoms : ( N * N2 ) as i32,
      nelements : N2 as _,
    }
  }
}
