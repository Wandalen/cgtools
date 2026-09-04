use crate::{Mat2, MatNum, mat, RawSliceMut, ScalarMut, Ix2, ConstLayout, IndexingMut, MatEl, nd, Mat, RawSlice, TryInto, Mat3};

impl< E, Descriptor > Mat2< E, Descriptor >
where
E : MatNum + ::num_traits::Signed,
Descriptor : mat::Descriptor,
Self : RawSliceMut< Scalar = E > +
       ScalarMut< Scalar = E, Index = Ix2 > +
       ConstLayout< Index = Ix2 > +
       IndexingMut< Scalar = E, Index = Ix2 >
{
  /// Computes the determinant of the matrix. Requires a signed scalar because
  /// the cofactor expansion subtracts (`a*d - b*c`), which can be negative; the
  /// result is undefined (panic or wrap) for unsigned element types.
  #[ inline ]
  pub fn determinant( &self ) -> E
  {
    let a = *self.scalar_ref( Ix2( 0, 0 ) );
    let b = *self.scalar_ref( Ix2( 0, 1 ) );
    let c = *self.scalar_ref( Ix2( 1, 0 ) );
    let d = *self.scalar_ref( Ix2( 1, 1 ) );

    a * d - b * c
  }
}

impl< E, Descriptor > Mat2< E, Descriptor >
where
E : MatEl + nd::NdFloat + ::num_traits::Signed,
Descriptor : mat::Descriptor,
Self : RawSliceMut< Scalar = E > +
       ScalarMut< Scalar = E, Index = Ix2 > +
       ConstLayout< Index = Ix2 > +
       IndexingMut< Scalar = E, Index = Ix2 >
{
  /// Computes the inverse of the matrix.
  /// If the determinant is zero - return `None`
  #[ inline ]
  pub fn inverse( &self ) -> Option< Self >
  {
    let det = self.determinant();

    if det == E::zero() { return None; }

    let a = *self.scalar_ref( Ix2( 0, 0 ) );
    let b = *self.scalar_ref( Ix2( 0, 1 ) );
    let c = *self.scalar_ref( Ix2( 1, 0 ) );
    let d = *self.scalar_ref( Ix2( 1, 1 ) );

    let inverse = Self::from_column_major
    (
      [ d, -c, -b, a ]
    );

    Some( inverse / det )
  }
}

impl< E, Descriptor > Mat< 2, 2, E, Descriptor >
where
E : MatNum,
Descriptor : mat::Descriptor,
Self : RawSlice< Scalar = E >
{
  /// Converts the matrix to an array
  ///
  /// # Panics
  /// Panics if the underlying raw slice does not have exactly 4 elements
  /// (guaranteed not to happen for a well-formed `Mat< 2, 2, E, Descriptor >`).
  #[ inline ]
  pub fn to_array( &self ) -> [ E; 4 ]
  {
    self.raw_slice().try_into().unwrap()
  }

  /// Converts the matrix to a 3x3 homogenous matrix
  #[ inline ]
  pub fn to_homogenous( &self ) -> Mat3< E, Descriptor >
  where
    Mat3< E, Descriptor > : RawSliceMut< Scalar = E >
  {
    let s = self.raw_slice();
    let mut mat = Mat3::< E, Descriptor >::default();
    let rs = mat.raw_slice_mut();

    rs[ 0 ] = s[ 0 ];
    rs[ 1 ] = s[ 1 ];
    rs[ 2 ] = E::zero();

    rs[ 3 ] = s[ 2 ];
    rs[ 4 ] = s[ 3 ];
    rs[ 5 ] = E::zero();

    rs[ 6 ] = E::zero();
    rs[ 7 ] = E::zero();
    rs[ 8 ] = E::one();

    mat
  }
}

impl< E, Descriptor > Mat< 2, 2, E, Descriptor >
where
E : MatNum,
Descriptor : mat::Descriptor,
Self : RawSliceMut< Scalar = E >
{
  /// Creates a 2x2 identity matrix.
  #[ inline ]
  #[ must_use ]
  pub fn identity() -> Self
  {
    let mat = Self::default();
    mat.raw_set( identity().to_array() )
  }
}

/// Creates a 2x2 identity matrix
#[ inline ]
#[ must_use ]
pub fn identity< E >() -> Mat2< E, mat::DescriptorOrderColumnMajor >
where
  E : MatNum,
  Mat2< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >
{
  Mat2::from_column_major
  (
    [
      E::one(),  E::zero(),
      E::zero(), E::one(),
    ]
  )
}
