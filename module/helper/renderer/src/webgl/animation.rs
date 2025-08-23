mod private
{
  use std::{ cell::RefCell, collections::HashMap, rc::Rc };
  use animation::
  {
    easing::
    {
      EasingBuilder,
      EasingFunction,
      Linear,
      Step,
      Cubic
    },
    Sequencer,
    Transform
  };
  use gltf::
  {
    Gltf,
    animation::{ Interpolation, Property }
  };
  use crate::webgl::Node;

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

  /// Load all animations from [`Gltf`] file
  fn load( gltf_file : &Gltf, nodes : &[ Node ] ) -> Vec< Animation >
  {
    let mut animations = Vec::new();
    for animation in gltf_file.animations()
    {
      let mut animated_nodes = HashMap::new();
      let mut sequencer = Sequencer::new();

      for channel in animation.channels()
      {
        let node = nodes[ target.node().index() ].clone();
        if let Some( name ) = node.borrow().get_name()
        {
          animated_nodes.insert( name, node );
        }

        let target = channel.target();
        let sampler = channel.sampler();

        let in_acc = sampler.input();
        let out_acc = sampler.output();



        let easing : Box< dyn EasingFunction > = match sampler.interpolation()
        {
          Interpolation::Linear => Linear::new(),
          Interpolation::Step => Box::new( Step::new( 0 ) ),
          Interpolation::CubicSpline => Box::new( Cubic::new( [ , , , ] ) )
        };

        let mut transform = Transform
        {
          translation : None,
          rotation : None,
          scale : None
        };

        match target.property()
        {
          Property::Translation =>
          {
            transform.translation = ;
          },
          Property::Rotation =>
          {
            transform.rotation = ;
          },
          Property::Scale =>
          {
            transform.scale = ;
          },
          _ => ()
          // Property::MorphTargetWeights => todo!(),
        }
      }

      let animation = Animation
      {
        name : animation.name().map( | s | s.to_string().into_boxed_str() ),
        sequencer : Rc::new( RefCell::new( sequencer ) ),
        nodes : animated_nodes
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
