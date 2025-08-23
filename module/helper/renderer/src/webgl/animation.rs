mod private
{
  use std::{ rc::Rc, cell::RefCell };
  use animation::Sequencer;

  /// Contains data for animating [`Mesh`]
  pub struct Animation
  {
    /// Animation name
    pub name : Option< Box< str > >,
    /// Animation behavior
    pub sequencer : Rc< RefCell< Sequencer > >,
    /// Related animated [`Node`]'s
    pub nodes : HashMap< Box< str >, Rc< RefCell< Node > > >
  }

  fn load( gltf_file : &Gltf, nodes : &[ Node ] ) -> Vec< Animation >
  {
    let mut animations = Vec::new();
    for animation in gltf_file.animations()
    {
      let sequencer =

      for channel in animation.channels()
      {

      }

      for sampler in animation.samplers()
      {

      }

      let animation = Animation
      {
        name : animation.name().map( | s | s.to_string().into_boxed_str() ),
        sequencer : Rc::new( RefCell::new(  ) ),
        nodes :
      };

      animations.push( animation );
    }

    animations
  }
}

crate::mod_interface!
{
  orphan use
  {
    Animation
  };
}
