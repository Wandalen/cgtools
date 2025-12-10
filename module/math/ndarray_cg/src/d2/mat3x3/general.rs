use crate::*;

impl< E, Descriptor > Mat3< E, Descriptor >
where
E : MatEl + nd::NdFloat,
Descriptor : mat::Descriptor,
Self : RawSliceMut< Scalar = E > +
       ScalarMut< Scalar = E, Index = Ix2 > +
       ConstLayout< Index = Ix2 > +
       IndexingMut< Scalar = E, Index = Ix2 >
{
  /// Computes the determinant of the matrix
  pub fn determinant( &self ) -> E
  {
    let a = *self.scalar_ref( Ix2( 0, 0 ) );
    let b = *self.scalar_ref( Ix2( 0, 1 ) );
    let c = *self.scalar_ref( Ix2( 0, 2 ) );

    let d = *self.scalar_ref( Ix2( 1, 0 ) );
    let e = *self.scalar_ref( Ix2( 1, 1 ) );
    let f = *self.scalar_ref( Ix2( 1, 2 ) );

    let g = *self.scalar_ref( Ix2( 2, 0 ) );
    let h = *self.scalar_ref( Ix2( 2, 1 ) );
    let i = *self.scalar_ref( Ix2( 2, 2 ) );

    ( a * e * i ) +
    ( b * f * g ) +
    ( c * d * h ) -
    ( c * e * g ) -
    ( b * d * i ) -
    ( a * f * h )
  }

  /// Computes the inverse of the matrix.
  /// If the determinant is zero - return `None`
  pub fn inverse( &self ) -> Option< Self >
  {
    let det = self.determinant();

    if det == E::zero() { return None; }

    let mut iter = self.iter_msfirst();

    let x = Vector::< E, 3 >::from( [ *iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap() ] );
    let y = Vector::< E, 3 >::from( [ *iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap() ] );
    let z = Vector::< E, 3 >::from( [ *iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap() ] );

    let a = y.cross( z );
    let b = z.cross( x );
    let c = x.cross( y );

    let res = Self::from_cols( a, b, c ) / det;
    Some( res.transpose() )
  }

   /// Creates a transformation matrix from scale, rotation( angle ) and translation
  pub fn from_scale_rotation_translation< Vec >
  (
    scale : Vec,
    rotation : E,
    translation : Vec
  ) -> Self
  where
    Vec : VectorIter< E, 2 >,
  {
    let mut siter = scale.vector_iter();
    let sx = *siter.next().unwrap();
    let sy = *siter.next().unwrap();

    let mut titer = translation.vector_iter();
    let tx = *titer.next().unwrap();
    let ty = *titer.next().unwrap();

    let ( s, c ) = rotation.sin_cos();

    let mut res = Self::default();

    *res.scalar_mut(  Ix2( 0, 0 ) ) = c * sx;
    *res.scalar_mut(  Ix2( 1, 0 ) ) = s * sx;
    *res.scalar_mut(  Ix2( 2, 0 ) ) = E::zero();

    *res.scalar_mut(  Ix2( 0, 1 ) ) = -s * sy;
    *res.scalar_mut(  Ix2( 1, 1 ) ) = c * sy;
    *res.scalar_mut(  Ix2( 2, 1 ) ) = E::zero();

    *res.scalar_mut(  Ix2( 0, 2 ) ) = tx;
    *res.scalar_mut(  Ix2( 1, 2 ) ) = ty;
    *res.scalar_mut(  Ix2( 2, 2 ) ) = E::one();

    res
  }
}

impl< E, Descriptor > Mat< 3, 3, E, Descriptor >
where
E : MatEl + nd::NdFloat,
Descriptor : mat::Descriptor,
Self : RawSlice< Scalar = E >
{
  /// Converts the matrix to an array
  pub fn to_array( &self ) -> [ E; 9 ]
  {
    self.raw_slice().try_into().unwrap()
  }

  /// Converts the matrix to a 4x4 homogenous matrix
  pub fn to_homogenous( &self ) -> Mat4< E, Descriptor >
  where
    Mat4< E, Descriptor > : RawSliceMut< Scalar = E >
  {
    let s = self.raw_slice();
    let mut mat = Mat4::< E, Descriptor >::default();
    let rs = mat.raw_slice_mut();

    rs[ 0 ] = s[ 0 ];
    rs[ 1 ] = s[ 1 ];
    rs[ 2 ] = s[ 2 ];
    rs[ 3 ] = E::zero();

    rs[ 4 ] = s[ 3 ];
    rs[ 5 ] = s[ 4 ];
    rs[ 6 ] = s[ 5 ];
    rs[ 7 ] = E::zero();

    rs[ 8 ] = s[ 6 ];
    rs[ 9 ] = s[ 7 ];
    rs[ 10 ] = s[ 8 ];
    rs[ 11 ] = E::zero();

    rs[ 12 ] = E::zero();
    rs[ 13 ] = E::zero();
    rs[ 14 ] = E::zero();
    rs[ 15 ] = E::one();

    mat
  }

  /// Convertes this matrix into the 3x3 matrix
  pub fn truncate( &self ) -> Mat< 2, 2, E, Descriptor >
  where
    Mat< 2, 2, E, Descriptor > : RawSliceMut< Scalar = E >
  {
    let slice = self.raw_slice();

    let trunc_slice =
    [
      slice[ 0 ],
      slice[ 1 ],

      slice[ 3 ],
      slice[ 4 ],
    ];

    let mut mat3 = Mat::< 2, 2, E, Descriptor >::default();
    mat3.raw_set_slice( &trunc_slice );
    mat3
  }
}

impl< E, Descriptor > Mat< 3, 3, E, Descriptor >
where
E : MatEl + nd::NdFloat,
Descriptor : mat::Descriptor,
Self : RawSliceMut< Scalar = E >
{
  /// Construct a matrix from columns
  pub fn from_cols
  (
    x : Vector< E, 3 >,
    y : Vector< E, 3 >,
    z : Vector< E, 3 >
  ) -> Self
  {
    Self::from_column_major
    (
      [
        x[ 0 ], x[ 1 ], x[ 2 ],
        y[ 0 ], y[ 1 ], y[ 2 ],
        z[ 0 ], z[ 1 ], z[ 2 ]
      ]
    )
  }

  /// Creates a rotation matrix from a unit quaternion
  pub fn from_quat( quat : Quat< E > ) -> Self
  {
    let x2 = quat.x() + quat.x();
    let y2 = quat.y() + quat.y();
    let z2 = quat.z() + quat.z();
    let xx = quat.x() * x2;
    let xy = quat.x() * y2;
    let xz = quat.x() * z2;
    let yy = quat.y() * y2;
    let yz = quat.y() * z2;
    let zz = quat.z() * z2;
    let wx = quat.w() * x2;
    let wy = quat.w() * y2;
    let wz = quat.w() * z2;

    Self::from_column_major
    (
      [
        E::one() - ( yy + zz ), xy + wz, xz - wy,
        xy - wz, E::one() - ( xx + zz ), yz + wx,
        xz + wy, yz - wx, E::one() - ( xx + yy )
      ]
    )
  }
}

/// Creates a 3x3 identity matrix
pub fn identity< E >() -> Mat3< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Mat3< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >
{
  Mat3::from_column_major
  (
    [
      E::one(),  E::zero(), E::zero(),
      E::zero(), E::one(),  E::zero(),
      E::zero(), E::zero(), E::one(),
    ]
  )
}
