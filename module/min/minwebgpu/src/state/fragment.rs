/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Clone ) ]
  pub struct FragmentState< 'a >
  {
    module : &'a web_sys::GpuShaderModule,
    entry_point : Option< &'a str >,
    targets : Vec< ColorTargetState >
  }

  impl< 'a > FragmentState< 'a > 
  {
    pub fn new( module :  &'a web_sys::GpuShaderModule ) -> Self
    {
      let entry_point = None;
      let targets = Vec::with_capacity( 1 );

      FragmentState
      {
        module,
        entry_point,
        targets
      }
    }

    pub fn entry_point( mut self, entry : &'a str ) -> Self
    {
      self.entry_point = Some( entry );
      self
    }

    pub fn target( mut self, target : ColorTargetState ) -> Self
    {
      self.targets.push( target );
      self
    } 
  }

  impl From< FragmentState< '_ > > for web_sys::GpuFragmentState 
  {
    fn from( value: FragmentState< '_ > ) -> Self 
    {
      let targets : Vec< web_sys::GpuColorTargetState > = value.targets.into_iter().map( | t | t.into() ).collect();
      let state = web_sys::GpuFragmentState::new( &value.module, &targets.into() );

      if let Some( v ) = value.entry_point { state.set_entry_point( v ); }

      state
    }   
  }

}

crate::mod_interface!
{
  exposed use
  {
    FragmentState
  };
}
