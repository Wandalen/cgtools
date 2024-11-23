mod private
{
  use crate::*;
  //use vector::arithmetics::inner_product::*;

  impl< E : MatEl + NdFloat > Vector< E, 4 >
  {
    #[ inline ]
    pub fn x( &self ) -> E
    {
      self.0[ 0 ]
    }

    #[ inline ]
    pub fn y( &self ) -> E
    {
      self.0[ 1 ]
    }

    #[ inline ]
    pub fn z( &self ) -> E
    {
      self.0[ 2 ]
    }

    #[ inline ]
    pub fn w( &self ) -> E
    {
      self.0[ 2 ]
    }
  }

  impl< E, Vec2 > From< ( Vec2, Vec2 ) > for Vector< E, 4 > 
  where
  Vec2 : VectorIter< E, 2 >,
  E : MatEl
  {
    fn from( value: ( Vec2, Vec2 ) ) -> Self 
    {
      let mut iter1 = value.0.vector_iter();
      let mut iter2 = value.1.vector_iter();
      let x = *iter1.next().unwrap();
      let y = *iter1.next().unwrap();
      let z = *iter2.next().unwrap();
      let w = *iter2.next().unwrap();

      Self( [ x, y, z, w ] )
    }
  }
}

crate::mod_interface!
{
  
}
