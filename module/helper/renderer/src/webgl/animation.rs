mod private
{
  use std::
  {
    cell::RefCell,
    collections::HashMap,
    rc::Rc
  };
  use animation::
  {
    easing::
    {
      Cubic,
      EasingBuilder,
      EasingFunction,
      Linear,
      Squad,
      Step
    },
    Sequencer,
    Transform,
    Tween
  };
  use gltf::
  {
    animation::
    {
      Channel,
      Interpolation,
      Property
    },
    Gltf
  };
  use mingl::{ math::mat2x2h::translate, F32x3, QuatF32, VectorIter };
  use crate::webgl::Node;

  /// Prefix used for getting [`Node`] translation
  const TRANSLATION_PREFIX : &'static str = ".translation";
  /// Prefix used for getting [`Node`] rotation
  const ROTATION_PREFIX : &'static str = ".rotation";
  /// Prefix used for getting [`Node`] scale
  const SCALE_PREFIX : &'static str = ".scale";

  /// 3D transformation data including translation, rotation, and scale components.
  pub struct Transform
  {
    /// Translation
    pub translation : F32x3,
    /// Rotation
    pub rotation : QuatF32,
    /// Scale
    pub scale : F32x3,
  }

  /// Use this struct for saving simple 3D transformations
  /// for every [`Node`] of one object
  pub struct Pose
  {
    /// Stores [`Transform`] for every [`Node`]
    transforms : HashMap< Box< str >, Transform >,
    /// Stores links to [`Node`]'s
    nodes : HashMap< Box< str >, Rc< RefCell< Node > > >
  }

  impl Pose
  {
    pub fn new( _nodes : &[ Rc< RefCell< Node > > ] ) -> Self
    {
      let transforms = _nodes.iter()
      .filter_map
      (
        | n |
        {
          let Some( name ) = n.borrow().get_name()
          else
          {
            return None;
          };

          let transform = Transform
          {
            translation : n.borrow().get_translation(),
            rotation : n.borrow().get_rotation(),
            scale : n.borrow().get_scale()
          };

          Some( ( name, transform ) )
        }
      )
      .collect::< HashMap< _, _ > >();

      let nodes = _nodes.iter()
      .filter_map
      (
        | n |
        {
          let Some( name ) = n.borrow().get_name()
          else
          {
            return None;
          };

          let transform = Transform
          {
            translation : n.borrow().get_translation(),
            rotation : n.borrow().get_rotation(),
            scale : n.borrow().get_scale()
          };

          Some( ( name, n.clone() ) )
        }
      )
      .collect::< HashMap< _, _ > >();

      Self
      {
        transforms,
        nodes
      }
    }

    /// Set [`Transform`]'s for each related [`Node`]
    pub fn set( &self )
    {
      for ( name, t ) in self.transforms
      {
        if let Some( node ) = self.nodes.get( name )
        {
          let mut node_mut = node.borrow_mut();

          node_mut.set_translation( t.translation );
          node_mut.set_rotation( t.rotation );
          node_mut.set_scale( t.scale );
        }
      }
    }
  }

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

  impl Animation
  {
    pub fn update( &self, delta_time : f32 )
    {
      self.sequencer.borrow_mut().update( delta_time );
    }

    pub fn set( &self )
    {
      for ( name, node ) in self.nodes
      {
        if let Some( translation ) = self.sequencer.borrow().get_value( name + TRANSLATION_PREFIX )
        {
          node.borrow_mut().set_translation( translation );
        }

        if let Some( rotation ) = self.sequencer.borrow().get_value( name + ROTATION_PREFIX )
        {
          node.borrow_mut().set_rotation( rotation );
        }

        if let Some( scale ) = self.sequencer.borrow().get_value( name + SCALE_PREFIX )
        {
          node.borrow_mut().set_scale( scale );
        }
      }
    }
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

  fn decode_channel( channel : Channel< '_ > ) -> Option< ( usize, usize, Vec< f32 >, Vec< f32 > ) >
  {
    let target = channel.target();
    let sampler = channel.sampler();

    let in_acc = sampler.input();
    let out_acc = sampler.output();

    let Some( times ) = decode_accessor( in_acc )
    else
    {
      return None;
    };

    let Some( values ) = decode_accessor( out_acc )
    else
    {
      return None;
    };

    let elements = match target.property()
    {
      Property::Translation | Property::Scale => 3,
      Property::Rotation => 4,
      _ => return None
      // Property::MorphTargetWeights => todo!(),
    };

    let components = if let Interpolation::CubicSpline = sampler.interpolation()
    {
      3
    }
    else
    {
      1
    };

    Some( ( elements, components, times, values ) )
  }

  fn quat_sequence( channel : Channel< '_ > ) -> Option< Sequence >
  {
    let Some( ( elements, components, times, values ) ) = decode_channel( channel )
    else
    {
      return None;
    };

    let mut iter = times.into_iter()
    .zip( values.chunks( elements * components ) );

    let mut tweens = vec![];
    let mut last_time = None;
    let mut last_value = None;

    for ( t2, v ) in iter
    {
      let items = v.as_chunks::< 4 >().0;
      let mut items_iter = items.iter();
      let Some( in_tangent ) = items_iter.next().cloned()
      else
      {
        continue;
      };
      let Some( value ) = items_iter.next().cloned()
      else
      {
        continue;
      };
      let Some( out_tangent ) = items_iter.next().cloned()
      else
      {
        continue;
      };

      let in_tangent = QuatF32::from( in_tangent );
      let r2 = QuatF32::from( value );
      let out_tangent = QuatF32::from( out_tangent );
      let r1 = last_value.unwrap_or( r2 );

      let easing = Squad::new( r1, r2, in_tangent, out_tangent );

      last_time = Some( t2 );
      last_value = Some( r2 );
      let duration = t2 - t1;
      let delay = t1;

      let tween = Tween::new( r1, r2, duration, easing )
      .with_delay( delay );
      tweens.push( tween );
    }

    Some( Sequence::new( tweens ) )
  }

  fn vec3_sequence( channel : Channel< '_ > ) -> Sequence
  {
    let Some( ( elements, components, times, values ) ) = decode_channel( channel )
    else
    {
      return None;
    };

    let mut tweens = vec![];
    let mut last_time = None;
    let mut last_value = None;

    for ( t2, v ) in iter
    {
      let mut easing : Vec< Box< dyn EasingFunction > > = vec![];

      let items = v.as_chunks::< 3 >().0;
      let mut items_iter = items.iter();
      let Some( [ mx1, my1, mz1 ] ) = items_iter.next().cloned()
      else
      {
        continue;
      };
      let Some( [ x2, y2, z2 ] ) = items_iter.next().cloned()
      else
      {
        continue;
      };
      let Some( [ mx2, my2, mz2 ] ) = items_iter.next().cloned()
      else
      {
        continue;
      };

      let t1 = last_time.unwrap_or( t2 );
      let [ x1, y1, z1 ] = last_value.unwrap_or( [ x2, y2, z2 ] );

      let easing : Vec< Box< dyn EasingFunction > > = match sampler.interpolation()
      {
        Interpolation::Linear =>
        {
          vec![ Linear::new(), Linear::new(), Linear::new() ]
        },
        Interpolation::Step =>
        {
          vec!
          [
            Box::new( Step::new( 1 ) ),
            Box::new( Step::new( 1 ) ),
            Box::new( Step::new( 1 ) )
          ]
        }
        Interpolation::CubicSpline =>
        {
          vec!
          [
            Box::new( CubicHermite::new( t1, x1, mx1, t2, x2, mx2 ).into::< CubicBezier >() ),
            Box::new( CubicHermite::new( t1, y1, my1, t2, y2, my2 ).into::< CubicBezier >() ),
            Box::new( CubicHermite::new( t1, z1, mz1, t2, z2, mz2 ).into::< CubicBezier >() )
          ]
        }
      };

      last_time = Some( t2 );
      last_value = Some( [ x2, y2, z2 ] );
      let duration = t2 - t1;
      let delay = t1;
      let tween =
      [
        Tween::new( x1, x2, duration, easing[ 0 ] ).with_delay( delay ),
        Tween::new( y1, y2, duration, easing[ 1 ] ).with_delay( delay ),
        Tween::new( z1, z2, duration, easing[ 2 ] ).with_delay( delay ),
      ];
      tweens.push( tween );
    }

    Some( Sequence::new( tweens ) )
  }

  /// Load all animations from [`Gltf`] file
  pub fn load( gltf_file : &Gltf, nodes : &[ Node ] ) -> Vec< Animation >
  {
    let mut animations = Vec::new();
    for animation in gltf_file.animations()
    {
      let mut animated_nodes = HashMap::new();
      let mut sequencer = Sequencer::new();

      for channel in animation.channels()
      {
        let node = nodes[ target.node().index() ].clone();
        let Some( name ) = node.borrow().get_name()
        else
        {
          continue;
        };

        animated_nodes.insert( name, node );

        match target.property()
        {
          Property::Translation =>
          {
            let Some( sequence ) = vec3_sequence( channel )
            else
            {
              continue;
            };
            sequencer.add( name + TRANSLATION_PREFIX, sequence );
          },
          Property::Scale =>
          {
            let Some( sequence ) = vec3_sequence( channel )
            else
            {
              continue;
            };
            sequencer.add( name + SCALE_PREFIX, sequence );
          }
          Property::Rotation =>
          {
            let Some( sequence ) = quat_sequence( channel )
            else
            {
              continue;
            };
            sequencer.add( name + ROTATION_PREFIX, sequence );
          },
          _ => continue
          // Property::MorphTargetWeights => todo!(),
        };
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
    Animation,
    Pose,
    Transform
  };

  own use
  {
    load,
    TRANSLATION_PREFIX,
    ROTATION_PREFIX,
    SCALE_PREFIX
  };
}
