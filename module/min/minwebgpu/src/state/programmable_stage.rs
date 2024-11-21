/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Clone ) ]
  pub struct ProgrammableStage< 'a >
  {
    module : &'a web_sys::GpuShaderModule,
    entry_point : Option< &'a str >,
  }

  impl< 'a > ProgrammableStage< 'a > 
  {
    pub fn new( module : &'a web_sys::GpuShaderModule ) -> Self
    {
      let entry_point = None;

      ProgrammableStage
      {
        module,
        entry_point
      }
    }

    pub fn entry_point( mut self, entry : &'a str ) -> Self
    {
      self.entry_point = Some( entry );
      self
    }
  }

  impl From< ProgrammableStage< '_ > > for web_sys::GpuProgrammableStage
  {
    fn from( value: ProgrammableStage< '_ > ) -> Self 
    {
      let state = web_sys::GpuProgrammableStage::new( &value.module );

      if let Some( v ) = value.entry_point { state.set_entry_point( v ); }

      state
    }   
  }
}

crate::mod_interface!
{
  exposed use
  {
    ProgrammableStage
  };
}
