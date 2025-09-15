use ndarray::Dimension;

use crate::*;

fn minor
<
  E : MatEl + nd::NdFloat,
  Descriptor : mat::Descriptor
>
(
  from : &Mat4< E, Descriptor >,
  to : &mut Mat3< E, Descriptor >,
  i : usize,
  j : usize
)
where
Mat4< E, Descriptor > : RawSliceMut< Scalar = E > + IndexingRef< Scalar = E, Index = Ix2 >,
Mat3< E, Descriptor > : RawSliceMut< Scalar = E >
{
  for( id, ( _, v ) ) in from
  .iter_indexed_unstable()
  .filter(
    | ( id, _ ) |
    {
      let ( r, c ) = id.into_pattern();
      r != i && c != j
    }
  ).enumerate()
  {
    to.raw_slice_mut()[ id ] = *v;
  }
}

fn cofactor
<
  E : MatEl + nd::NdFloat,
  Descriptor : mat::Descriptor
>
(
  from : &Mat4< E, Descriptor >,
  to : &mut Mat3< E, Descriptor >,
  i : usize,
  j : usize
) -> E
where
Mat4< E, Descriptor > :
  RawSliceMut< Scalar = E > +
  IndexingRef< Scalar = E, Index = Ix2 >,
Mat3< E, Descriptor > :
  RawSliceMut< Scalar = E > +
  ScalarRef< Scalar = E, Index = Ix2 > +
  ConstLayout< Index = Ix2 > +
  IndexingMut< Scalar = E, Index = Ix2 >
{
  let k = E::from( ( -1i32 ).pow( ( i + j ) as u32 ) ).unwrap();
  minor( from, to, i, j );
  k * to.determinant()
}

impl< E, Descriptor > Mat< 4, 4, E, Descriptor >
where
E : MatEl + nd::NdFloat,
Descriptor : mat::Descriptor,
Self : ScalarMut< Scalar = E, Index = Ix2 > +
       RawSliceMut< Scalar = E > +
       ConstLayout< Index = Ix2 > +
       IndexingMut< Scalar = E, Index = Ix2 >
{
  /// Computes the determinant of the matrix
  pub fn determinant( &self ) -> E
  where
    Mat< 3, 3, E, Descriptor > :
      RawSliceMut< Scalar = E > +
      ScalarMut< Scalar = E, Index = Ix2 > +
      ConstLayout< Index = Ix2 > +
      IndexingMut< Scalar = E, Index = Ix2 >
  {
    let _a11 = *self.scalar_ref( Ix2( 0, 0 ) );
    let _a12 = *self.scalar_ref( Ix2( 0, 1 ) );
    let _a13 = *self.scalar_ref( Ix2( 0, 2 ) );
    let _a14 = *self.scalar_ref( Ix2( 0, 3 ) );

    let mut m = Mat3::< E, Descriptor >::default();

    minor( self, &mut m, 0, 0 );
    let _det11 = m.determinant();
    minor( self, &mut m, 0, 1 );
    let _det12 = m.determinant();
    minor( self, &mut m, 0, 2 );
    let _det13 = m.determinant();
    minor( self, &mut m, 0, 3 );
    let _det14 = m.determinant();

    _a11 * _det11 - _a12 * _det12 + _a13 * _det13 - _a14 * _det14
  }

  /// Computes the inverse of the matrix.
  /// If the determinant is zero - return `None`
  pub fn inverse( &self ) -> Option< Self >
  where
    Mat< 3, 3, E, Descriptor > :
      RawSliceMut< Scalar = E > +
      ScalarMut< Scalar = E, Index = Ix2 > +
      ConstLayout< Index = Ix2 > +
      IndexingMut< Scalar = E, Index = Ix2 >
  {
    let det = self.determinant();

    if det == E::zero() { return None; }

    let mut cfm = Mat3::default();
    let mut cf = | i, j |
    {
      cofactor( self, &mut cfm, i, j )
    };

    let adj = Self::from_column_major
    ([
      cf( 0, 0 ), cf( 0, 1 ), cf( 0, 2 ), cf( 0, 3 ),
      cf( 1, 0 ), cf( 1, 1 ), cf( 1, 2 ), cf( 1, 3 ),
      cf( 2, 0 ), cf( 2, 1 ), cf( 2, 2 ), cf( 2, 3 ),
      cf( 3, 0 ), cf( 3, 1 ), cf( 3, 2 ), cf( 3, 3 ),
    ]);

    Some( adj / det )
  }
}

impl< E, Descriptor > Mat< 4, 4, E, Descriptor >
where
E : MatEl + nd::NdFloat,
Descriptor : mat::Descriptor,
Self : RawSlice< Scalar = E >
{
  /// Converts the matrix to an array
  pub fn to_array( &self ) -> [ E; 16 ]
  {
    self.raw_slice().try_into().unwrap()
  }

  /// Convertes this matrix into the 3x3 matrix
  pub fn truncate( &self ) -> Mat< 3, 3, E, Descriptor >
  where
    Mat< 3, 3, E, Descriptor > : RawSliceMut< Scalar = E >
  {
    let slice = self.raw_slice();

    let trunc_slice =
    [
      slice[ 0 ],
      slice[ 1 ],
      slice[ 2 ],

      slice[ 4 ],
      slice[ 5 ],
      slice[ 6 ],

      slice[ 8 ],
      slice[ 9 ],
      slice[ 10 ],
    ];

    let mut mat3 = Mat::< 3, 3, E, Descriptor >::default();
    mat3.raw_set_slice( &trunc_slice );
    mat3
  }
}

impl< E, Descriptor > Mat< 4, 4, E, Descriptor >
where
E : MatEl + nd::NdFloat,
Descriptor : mat::Descriptor,
Self : ScalarMut< Scalar = E > +
       IndexingMut< Scalar = E, Index = Ix2 >
{
  /// Creates a transformation matrix from scale, rotation and translation
  pub fn from_scale_rotation_translation< Vec, Q >
  (
    scale : Vec,
    rotation : Q,
    translation : Vec
  ) -> Self
  where
    Vec : VectorIter< E, 3 >,
    Q : Into< Quat< E > >
  {
    let rot = rotation.into().to_matrix();

    let mut siter = scale.vector_iter();
    let sx = *siter.next().unwrap();
    let sy = *siter.next().unwrap();
    let sz = *siter.next().unwrap();

    let mut titer = translation.vector_iter();
    let tx = *titer.next().unwrap();
    let ty = *titer.next().unwrap();
    let tz = *titer.next().unwrap();

    let rot = rot.raw_slice();

    let mut res = Self::default();

    *res.scalar_mut(  Ix2( 0, 0 ) ) = rot[ 0 ] * sx;
    *res.scalar_mut(  Ix2( 1, 0 ) ) = rot[ 1 ] * sx;
    *res.scalar_mut(  Ix2( 2, 0 ) ) = rot[ 2 ] * sx;
    *res.scalar_mut(  Ix2( 3, 0 ) ) = E::zero();

    *res.scalar_mut(  Ix2( 0, 1 ) ) = rot[ 3 ] * sy;
    *res.scalar_mut(  Ix2( 1, 1 ) ) = rot[ 4 ] * sy;
    *res.scalar_mut(  Ix2( 2, 1 ) ) = rot[ 5 ] * sy;
    *res.scalar_mut(  Ix2( 3, 1 ) ) = E::zero();

    *res.scalar_mut(  Ix2( 0, 2 ) ) = rot[ 6 ] * sz;
    *res.scalar_mut(  Ix2( 1, 2 ) ) = rot[ 7 ] * sz;
    *res.scalar_mut(  Ix2( 2, 2 ) ) = rot[ 8 ] * sz;
    *res.scalar_mut(  Ix2( 3, 2 ) ) = E::zero();

    *res.scalar_mut(  Ix2( 0, 3 ) ) = tx;
    *res.scalar_mut(  Ix2( 1, 3 ) ) = ty;
    *res.scalar_mut(  Ix2( 2, 3 ) ) = tz;
    *res.scalar_mut(  Ix2( 3, 3 ) ) = E::one();

    res
  }
}

impl< E, Descriptor > Mat< 4, 4, E, Descriptor >
where
E : MatEl + nd::NdFloat,
Descriptor : mat::Descriptor,
Self : RawSlice< Scalar = E > +
ScalarRef< Scalar = E, Index = Ix2 > +
ConstLayout< Index = Ix2 > +
IndexingRef< Scalar = E, Index = Ix2 >
{
  /// Decompose a transformation matrix to scale, rotation and translation
  ///
  /// Source: https://github.com/mrdoob/three.js/blob/27151c8325d1dba520d4abfb5a2e1077dd59de22/src/math/Matrix4.js#L1050
  pub fn decompose( &self ) -> Option< ( Vector< E, 3 >, Quat< E >, Vector< E, 3 > ) >
  {
    let a = *self.scalar_ref( Ix2( 0, 0 ) );
    let b = *self.scalar_ref( Ix2( 1, 0 ) );
    let c = *self.scalar_ref( Ix2( 2, 0 ) );

    let d = *self.scalar_ref( Ix2( 0, 1 ) );
    let e = *self.scalar_ref( Ix2( 1, 1 ) );
    let f = *self.scalar_ref( Ix2( 2, 1 ) );

    let g = *self.scalar_ref( Ix2( 0, 2 ) );
    let h = *self.scalar_ref( Ix2( 1, 2 ) );
    let i = *self.scalar_ref( Ix2( 2, 2 ) );

    let tx = *self.scalar_ref( Ix2( 0, 3 ) );
    let ty = *self.scalar_ref( Ix2( 1, 3 ) );
    let tz = *self.scalar_ref( Ix2( 2, 3 ) );

    let translation = Vector::< E, 3 >::from_array( [ tx, ty, tz ] );

    let mut sx = Vector::< E, 3 >::from_array( [ a, b, c ] ).mag();
    let sy = Vector::< E, 3 >::from_array( [ d, e, f ] ).mag();
    let sz = Vector::< E, 3 >::from_array( [ g, h, i ] ).mag();

    let rot_mat = Mat3::< E, mat::DescriptorOrderColumnMajor >::from_column_major( [ a, b, c, d, e, f, g, h, i ] );

    let det = rot_mat.determinant();
    if det < E::zero()
    {
      sx = - sx;
    }

    if sx == E::zero() || sy == E::zero() || sz == E::zero()
    {
      return None;
    }

    let scale = Vector::< E, 3 >::from_array( [ sx, sy, sz ] );

    let i_sx = E::one() / sx;
    let i_sy = E::one() / sy;
    let i_sz = E::one() / sz;

    let rot_mat = Mat3::< E, mat::DescriptorOrderColumnMajor >::from_column_major
    (
      [
        a / i_sx, b / i_sx, c / i_sx,
        d / i_sy, e / i_sy, f / i_sy,
        g / i_sz, h / i_sz, i / i_sz
      ]
    );

    let rotation = Quat::< E >::from( rot_mat );

    Some( ( translation, rotation, scale ) )
  }
}

/// Creates a 4x4 identity matrix.
pub fn identity< E >() -> Mat4< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Mat4< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >
{
  Mat4::from_column_major
  (
    [
      E::one(),  E::zero(), E::zero(), E::zero(),
      E::zero(), E::one(),  E::zero(), E::zero(),
      E::zero(), E::zero(), E::one(),  E::zero(),
      E::zero(), E::zero(), E::zero(), E::one(),
    ]
  )
}

