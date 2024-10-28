mod private
{
  use std::fmt::Debug;
  pub use super::obj_model as obj;

  #[ derive( Debug ) ]
  pub struct ForBrowser< T : Debug >
  {
    pub report : T
  }

  impl< T : Debug > ForBrowser< T > {
    pub fn new( r : T ) -> Self
    {
      ForBrowser
      {
        report : r
      }
    }

    pub fn from_report( r : T ) -> Self
    {
      ForBrowser::new( r )
    }

    pub fn from_reports( r : Vec< T > ) -> Vec< Self >
    {
      r.into_iter().map( | r | ForBrowser::new( r ) ).collect()
    }
  }
}

pub mod obj_model;

crate::mod_interface!
{

  exposed use 
  {
    ForBrowser,
    obj,
  };
}
