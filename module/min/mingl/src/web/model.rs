mod private
{
  use std::fmt::Debug;

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



crate::mod_interface!
{

  // Web utilities for the model in obj format
  #[ cfg( feature = "web_model_obj" ) ]
  layer obj;

  own use
  {
    ForBrowser
  };
}
