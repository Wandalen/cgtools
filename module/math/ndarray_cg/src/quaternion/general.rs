mod private
{
  use crate::*;
  
  impl< E > Quat< E >
  where 
    E : MatEl
  {
    pub fn x( &self ) -> E
    {
      self.0[ 0 ]
    }

    pub fn y( &self ) -> E
    {
      self.0[ 1 ]
    }

    pub fn z( &self ) -> E
    {
      self.0[ 2 ]
    }

    pub fn w( &self ) -> E
    {
      self.0[ 3 ]
    }
  }

  impl< E > AbsDiffEq for Quat< E >
  where
    E : AbsDiffEq + MatEl,
    E::Epsilon : Copy,
  {
    type Epsilon = < Vector< E, 4 > as AbsDiffEq< Vector< E, 4 > > >::Epsilon;

    fn default_epsilon() -> Self::Epsilon 
    {
      E::default_epsilon()
    }

    fn abs_diff_eq( &self, other: &Self, epsilon: Self::Epsilon ) -> bool 
    {
      < Vector< E, 4 > as AbsDiffEq< Vector< E, 4 > > >::abs_diff_eq( &self.0, &other.0, epsilon )   
    }
  }

  impl< E > RelativeEq for Quat< E >
  where
    E : RelativeEq + MatEl,
    E::Epsilon : Copy,
  {
    fn default_max_relative() -> Self::Epsilon 
    {
      E::default_max_relative()
    }

    fn relative_eq( &self, other: &Self, epsilon: Self::Epsilon, max_relative: Self::Epsilon ) -> bool 
    {
      < Vector< E, 4 > as RelativeEq< Vector< E, 4 > > >::relative_eq( &self.0, &other.0, epsilon, max_relative )
    }
  }

  impl< E > UlpsEq for Quat< E >
  where
    E : UlpsEq + MatEl,
    E::Epsilon : Copy,
  {
    fn default_max_ulps() -> u32 
    {
      E::default_max_ulps()
    }

    fn ulps_eq( &self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32 ) -> bool 
    {
      < Vector< E, 4 > as UlpsEq< Vector< E, 4 > > >::ulps_eq( &self.0, &other.0, epsilon, max_ulps )
    }
  }
}

crate::mod_interface!
{
  
}
