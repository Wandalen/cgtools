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
      cubic::CubicHermite,
      EasingBuilder,
      EasingFunction,
      Linear,
      Squad,
      Step
    },
    Sequence,
    Sequencer,
    Tween
  };
  use gltf::
  {
    animation::
    {
      util::ReadOutputs, Channel, Interpolation, Property
    },
    Gltf
  };
  use minwebgl as gl;
  use gl::{ F64x3, F32x3, QuatF32, QuatF64 };
  use crate::webgl::Node;

  /// Prefix used for getting [`Node`] translation
  pub const TRANSLATION_PREFIX : &'static str = ".translation";
  /// Prefix used for getting [`Node`] rotation
  pub const ROTATION_PREFIX : &'static str = ".rotation";
  /// Prefix used for getting [`Node`] scale
  pub const SCALE_PREFIX : &'static str = ".scale";

  /// 3D transformation data including translation, rotation, and scale components.
  pub struct Transform
  {
    /// Translation
    pub translation : F64x3,
    /// Rotation
    pub rotation : QuatF64,
    /// Scale
    pub scale : F64x3,
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
    /// [`Pose`] constructor
    ///
    /// Parameters:
    /// * _nodes - list of [`Node`]'s which current 3D
    ///   transformation parameters are used for defining [`Pose`]
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
            translation : F64x3::from_array( n.borrow().get_translation().map( | v | v as f64 ) ),
            rotation : QuatF64::from( n.borrow().get_rotation().0.map( | v | v as f64 ) ),
            scale : F64x3::from_array( n.borrow().get_scale().map( | v | v as f64 ) )
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
      for ( name, t ) in &self.transforms
      {
        if let Some( node ) = self.nodes.get( name )
        {
          let mut node_mut = node.borrow_mut();

          node_mut.set_translation( F32x3::from_array( t.translation.0.map( | v | v as f32 ) ) );
          node_mut.set_rotation( QuatF32::from( t.rotation.0.map( | v | v as f32 ) ) );
          node_mut.set_scale( F32x3::from_array( t.scale.0.map( | v | v as f32 ) ) );
        }
      }
    }
  }

  /// Contains data for animating [`Mesh`]
  #[ derive( Clone ) ]
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
    /// Updates all [`AnimatableValue`]'s for current [`Animation`]
    pub fn update( &self, delta_time : f64 )
    {
      self.sequencer.borrow_mut().update( delta_time.into() );
    }

    /// Sets all simple 3D transformations for every
    /// [`Node`] related to this [`Animation`]
    pub fn set( &self )
    {
      for ( name, node ) in &self.nodes
      {
        if let Some( translation ) = self.sequencer.borrow()
        .get_value::< Sequence< Tween< F64x3 > > >
        (
          &format!( "{}{}", name, TRANSLATION_PREFIX )
        )
        {
          if let Some( translation ) = translation.get_current()
          {
            let translation = translation.get_current_value().0.map( | v | v as f32 );
            node.borrow_mut().set_translation( F32x3::from_array( translation ) );
          }
        }

        if let Some( rotation ) = self.sequencer.borrow()
        .get_value::< Sequence< Tween< QuatF64 > > >
        (
          &format!( "{}{}", name, ROTATION_PREFIX )
        )
        {
          if let Some( rotation ) = rotation.get_current()
          {
            let rotation = rotation.get_current_value().0.map( | v | v as f32 );
            node.borrow_mut().set_rotation( QuatF32::from( rotation ) );
          }
        }

        if let Some( scale ) = self.sequencer.borrow()
        .get_value::< Sequence< Tween< F64x3 > > >
        (
          &format!( "{}{}", name, SCALE_PREFIX )
        )
        {
          if let Some( scale ) = scale.get_current()
          {
            let scale = scale.get_current_value().0.map( | v | v as f32 );
            node.borrow_mut().set_scale( F32x3::from_array( scale ) );
          }
        }
      }
    }
  }

  async fn decode_channel< 'a >
  (
    channel : Channel< '_ >,
    buffers : &'a [ Vec< u8 > ],
  ) -> Option< ( usize, Vec< f64 >, ReadOutputs< 'a > ) >
  {
    let sampler = channel.sampler();
    let reader = channel.reader
    (
      | buffer | Some( buffers[ buffer.index() ].as_slice() )
    );

    let Some( times ) = reader.read_inputs()
    else
    {
      return None;
    };

    let Some( values ) = reader.read_outputs()
    else
    {
      return None;
    };

    let components = if let Interpolation::CubicSpline = sampler.interpolation()
    {
      3
    }
    else
    {
      1
    };

    Some
    (
      (
        components,
        times.map( | t | t as f64 ).collect::< Vec< _ > >(),
        values
      )
    )
  }

  async fn quat_sequence
  (
    channel : Channel< '_ >,
    buffers : &[ Vec< u8 > ],
  ) -> Option< Sequence< Tween< QuatF64 > > >
  {
    let Some
    (
      ( components, times, values )
    )
    = decode_channel( channel.clone(), buffers ).await
    else
    {
      return None;
    };

    let ReadOutputs::Rotations( rotations ) = values
    else
    {
      return None;
    };
    let rotations = rotations.into_f32().collect::< Vec< _ > >();

    let iter = times.into_iter()
    .zip( rotations.chunks( components ) );

    let mut tweens = vec![];
    let mut last_time = None;
    let mut last_value = None;

    for ( t2, v ) in iter
    {
      let mut items_iter = v.iter();

      let mut in_tangent = None;
      if channel.sampler().interpolation() == Interpolation::CubicSpline
      {
        let Some( _in_tangent ) = items_iter.next().cloned()
        else
        {
          continue;
        };
        in_tangent = Some( _in_tangent.map( | v | v as f64 ) );
      }

      let Some( value ) = items_iter.next().cloned()
      else
      {
        continue;
      };

      let mut out_tangent = None;
      if channel.sampler().interpolation() == Interpolation::CubicSpline
      {
        let Some( _out_tangent ) = items_iter.next().cloned()
        else
        {
          continue;
        };
        out_tangent = Some( _out_tangent.map( | v | v as f64 ) );
      }

      let r2 = QuatF64::from( value.map( | v | v as f64 ) );

      let r1 = last_value.unwrap_or( r2 );
      let t1 = last_time.unwrap_or( t2 );

      let easing : Box< dyn EasingFunction< AnimatableType = QuatF64 > > = match channel.sampler().interpolation()
      {
        Interpolation::Linear => Linear::new(),
        Interpolation::Step => Box::new( Step::new( 1.0 ) ),
        Interpolation::CubicSpline =>
        {
          let in_tangent = QuatF64::from( in_tangent.unwrap() );
          let out_tangent = QuatF64::from( out_tangent.unwrap() );
          Box::new( Squad::new( in_tangent, out_tangent ) )
        },
      };

      last_time = Some( t2 );
      last_value = Some( r2 );
      let duration = t2 - t1;
      let delay = t1;

      let tween = Tween::new( r1, r2, duration.into(), easing )
      .with_delay( delay.into() );
      tweens.push( tween );
    }

    Sequence::new( tweens ).ok()
  }

  async fn vec3_sequence
  (
    channel : Channel< '_ >,
    buffers : &[ Vec< u8 > ],
  ) -> Option< Sequence< Tween< F64x3 > > >
  {
    let Some
    (
      ( components, times, values )
    )
    = decode_channel( channel.clone(), buffers ).await
    else
    {
      return None;
    };

    let values = match values
    {
      ReadOutputs::Translations( v ) |
      ReadOutputs::Scales( v ) =>
      {
        v.collect::< Vec< _ > >()
      }
      _ => { return None; }
    };

    let iter = times.into_iter()
    .zip( values.chunks( components ) );

    let mut tweens = vec![];
    let mut last_time = None;
    let mut last_value = None;

    for ( t2, v ) in iter
    {
      let mut items_iter = v.iter();

      let mut m1 = None;
      if channel.sampler().interpolation() == Interpolation::CubicSpline
      {
        let Some( _m1 ) = items_iter.next().cloned()
        else
        {
          continue;
        };
        m1 = Some( _m1.map( | v | v as f64 ) );
      }

      let Some( v2 ) = items_iter.next().cloned()
      else
      {
        continue;
      };

      let mut m2 = None;
      if channel.sampler().interpolation() == Interpolation::CubicSpline
      {
        let Some( _m2 ) = items_iter.next().cloned()
        else
        {
          continue;
        };
        m2 = Some( _m2.map( | v | v as f64 ) );
      }

      let v2 = F64x3::from_array( v2.map( | v | v as f64 ) );
      let t1 = last_time.unwrap_or( t2 );
      let v1 = last_value
      .unwrap_or( v2 );

      let easing : Box< dyn EasingFunction< AnimatableType = F64x3 > > = match channel.sampler().interpolation()
      {
        Interpolation::Linear => Linear::new(),
        Interpolation::Step => Box::new( Step::new( 1.0 ) ),
        Interpolation::CubicSpline =>
        {
          let m1 = F64x3::from_array( m1.unwrap() );
          let m2 = F64x3::from_array( m2.unwrap() );
          Box::new( CubicHermite::new( m1, m2 ) )
        },
      };

      last_time = Some( t2 );
      last_value = Some( v2 );
      let duration = t2 - t1;
      let delay = t1;
      let tween = Tween::new( v1, v2, duration.into(), easing )
      .with_delay( delay.into() );
      tweens.push( tween );
    }

    Sequence::new( tweens ).ok()
  }

  /// Load all animations from [`Gltf`] file
  pub async fn load
  (
    gltf_file : &Gltf,
    buffers : &[ Vec< u8 > ],
    nodes : &[ Rc< RefCell< Node > > ]
  )
  -> Vec< Animation >
  {
    let mut animations = Vec::new();
    for animation in gltf_file.animations()
    {
      let mut animated_nodes = HashMap::new();
      let mut sequencer = Sequencer::new();

      for channel in animation.channels()
      {
        let node = nodes[ channel.target().node().index() ].clone();
        let Some( name ) = node.borrow().get_name()
        else
        {
          continue;
        };

        animated_nodes.insert( name.clone(), node );

        match channel.target().property()
        {
          Property::Translation =>
          {
            let Some( sequence ) = vec3_sequence( channel, buffers ).await
            else
            {
              continue;
            };
            sequencer.add( &format!( "{}{}", name, TRANSLATION_PREFIX ), sequence );
          },
          Property::Scale =>
          {
            let Some( sequence ) = vec3_sequence( channel, buffers ).await
            else
            {
              continue;
            };
            sequencer.add( &format!( "{}{}", name, SCALE_PREFIX ), sequence );
          }
          Property::Rotation =>
          {
            let Some( sequence ) = quat_sequence( channel, buffers ).await
            else
            {
              continue;
            };
            sequencer.add( &format!( "{}{}", name, ROTATION_PREFIX ), sequence );
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
