use crate::*;
//use mdmath_core::vector::arithmetics::*;

pub fn from_angle_x< E >( angle : E ) -> Mat3< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Mat3< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >
{
  let ( s, c ) = angle.sin_cos();
  Mat3::from_row_major
  (
    [
      E::one(),  E::zero(), E::zero(),
      E::zero(), c,         -s,
      E::zero(), s,         c, 
    ]
  )
}

pub fn from_angle_y< E >( angle : E ) -> Mat3< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Mat3< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >
{
  let ( s, c ) = angle.sin_cos();
  Mat3::from_row_major
  (
    [
      c,         E::zero(), s,
      E::zero(), E::one(),  E::zero(),
      -s,        E::zero(), c
    ]
  )
}

pub fn from_angle_z< E >( angle : E ) -> Mat3< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Mat3< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >
{
  let ( s, c ) = angle.sin_cos();
  Mat3::from_row_major
  (
    [
      c,         -s,        E::zero(),
      s,         c,         E::zero(),
      E::zero(), E::zero(), E::one()
    ]
  )
}

pub fn from_axis_angle< E, Vec3 >( axis : Vec3, angle : f32 ) -> Mat3< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Vec3 : VectorIter< E, 3 >,
  Mat3< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >
{
  let s = E::from( angle.sin() ).unwrap();
  let c = E::from( angle.cos() ).unwrap();
  let _1subc = E::one() - c ;

  let mut iter = axis.vector_iter();
  let ux = *iter.next().unwrap();
  let uy = *iter.next().unwrap();
  let uz = *iter.next().unwrap();

  let r1c1 = ux * ux * _1subc + c;
  let r1c2 = ux * uy * _1subc - uz * s;
  let r1c3 = ux * uz * _1subc + uy * s;

  let r2c1 = ux * uy * _1subc + uz * s;
  let r2c2 = uy * uy * _1subc + c;
  let r2c3 = uy * uz * _1subc - ux * s;

  let r3c1 = ux * uz * _1subc - uy * s;
  let r3c2 = uy * uz * _1subc + ux * s;
  let r3c3 = uz * uz * _1subc + c;
  Mat3::from_row_major
  (
    [
      r1c1, r1c2, r1c3,
      r2c1, r2c2, r2c3,
      r3c1, r3c2, r3c3
    ]
  )
}


