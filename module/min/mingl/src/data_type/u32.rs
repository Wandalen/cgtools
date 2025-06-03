use super::*;

impl IntoVectorDataType for u32
{
  fn into_vector_data_type() -> VectorDataType
  {
    VectorDataType
    {
      scalar : DataType::U32,
      natoms : 1,
      nelements : 1,
    }
  }
}

impl< const N : usize > IntoVectorDataType for [ u32 ; N ]
{
  fn into_vector_data_type() -> VectorDataType
  {
    VectorDataType
    {
      scalar : DataType::U32,
      natoms : N as _,
      nelements : 1,
    }
  }
}

impl< const N : usize, const N2 : usize > IntoVectorDataType for [ [ u32 ; N2 ] ; N ]
{
  fn into_vector_data_type() -> VectorDataType
  {
    VectorDataType
    {
      scalar : DataType::U32,
      natoms : ( N * N2 ) as i32,
      nelements : N2 as _,
    }
  }
}
