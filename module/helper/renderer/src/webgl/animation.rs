mod private
{
  use std::{ cell::RefCell, collections::HashMap, rc::Rc };
  use animation::
  {
    easing::
    {
      Cubic, EasingBuilder, EasingFunction, Linear, Step
    },
    Sequencer,
    Transform,
    Tween
  };
  use gltf::
  {
    Gltf,
    animation::{ Interpolation, Property }
  };
  use mingl::{ F32x3, QuatF32, VectorIter };
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

  fn decode_accessor( accessor : &Accessor ) -> Option< Vec< f32 > >
  {
    let view = accessor.view()?;
    let buffer = view.buffer();
    let slice = buffer.source.slice( &blob );

    let start = view.offset() + accessor.offset();
    let end = start + accessor.count() * accessor.size();
    let data = &slice[ start..end ];

    let mut decoded_data = Vec::with_capacity( data.len() / 4 );
    let mut chunks = data.chunks_exact( 4 );
    while let Some( chunk ) = chunks.next()
    {
      let mut bytes = [ 0u8; 4 ];
      bytes.copy_from_slice( chunk );
      decoded_data.push( f32::from_le_bytes( bytes ) );
    }

    Some( decoded_data )
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

        let Some( times ) = decode_accessor( in_acc )
        else
        {
          continue;
        };

        let Some( values ) = decode_accessor( out_acc )
        else
        {
          continue;
        };

        let iter = times.into_iter()
        .zip( values.chunks( 2 ) );

        let easing : Box< dyn EasingFunction > = match sampler.interpolation()
        {
          Interpolation::Linear => Linear::new(),
          Interpolation::Step => Box::new( Step::new( 1 ) ),
          Interpolation::CubicSpline => Box::new( Cubic::new( [ , , , ] ) )
        };

        let easing = match sampler.interpolation()
        {
          Interpolation::Linear => 1,
          Interpolation::Step => Box::new( Step::new( 1 ) ),
          Interpolation::CubicSpline => Box::new( Cubic::new( [ , , , ] ) )
        };

        for ( t, v ) in iter
        {
          match target.property()
          {
            Property::Translation =>
            {
              let name = ;
              F32x3::from_slice( [ , , ] );
              let tween = Tween::new( start, end, duration, easing )
              .with_delay( delay );
              sequencer.add_tween( name, tween );
            },
            Property::Rotation =>
            {
              let name = ;
              QuatF32::from( rotation );
              let tween = Tween::new( start, end, duration, easing )
              .with_delay( delay );
              sequencer.add_tween( name, tween );
            },
            Property::Scale =>
            {
              let name = ;
              F32x3::from_slice( [ , , ] );
              let tween = Tween::new( start, end, duration, easing )
              .with_delay( delay );
              sequencer.add_tween( name, tween );
            },
            _ => ()
            // Property::MorphTargetWeights => todo!(),
          }
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
