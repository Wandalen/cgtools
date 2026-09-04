
mod private
{
  use interpoli::
  {
    Composition,
    Content,
    Draw,
    Geometry,
    Shape,
    Stroke,
    Brush,
    animated::Spline
  };
  use kurbo::Affine;
  use renderer::webgl::loaders::gltf::GLTF;
  use rustc_hash::FxHashMap;
  use minwebgl as gl;
  use gl::
  {
    F32x4,
    F32x4x4,
    GL,
    math::nd
  };
  use core::cell::RefCell;
  use core::ops::Range;
  use std::rc::Rc;
  use crate::primitive_data::primitives_data_to_gltf;
  use primitive_generation::primitive_data::Transform;

  use renderer::webgl::
  {
    Scene,
    Node
  };
  use crate::primitive_data::{ Behavior, PrimitiveData };

  /// Converts a 2D `Affine` transformation matrix to a 4x4 `F32x4x4` matrix, suitable for 3D rendering.
  pub fn affine_to_matrix( affine : Affine ) -> F32x4x4
  {
    let [ a, b, c, d , e, f ] = affine.as_coeffs();

    let mut matrix = F32x4x4::identity();

    {
      *matrix.scalar_mut( nd::Ix2( 0, 0 ) ) = a as f32;
      *matrix.scalar_mut( nd::Ix2( 1, 0 ) ) = b as f32;
      *matrix.scalar_mut( nd::Ix2( 0, 1 ) ) = c as f32;
      *matrix.scalar_mut( nd::Ix2( 1, 1 ) ) = d as f32;
      *matrix.scalar_mut( nd::Ix2( 0, 3 ) ) = e as f32;
      *matrix.scalar_mut( nd::Ix2( 1, 3 ) ) = f as f32;
    }

    matrix
  }

  /// Converts an `interpoli::Brush` to an `F32x4` color vector for a given frame.
  fn brush_to_color( brush : &interpoli::Brush, frame : f64 ) -> F32x4
  {
    let color = match brush.evaluate( 1.0, frame ).into_owned()
    {
      peniko::Brush::Solid( color ) => Some( color ),
      _ => None
    };

    

    if let Some( color ) = color
    {
      let [ r, g, b, a ] = color.to_rgba8().to_u8_array();
      
      F32x4::from_array
      (
        [ f32::from(r) / 255.0, f32::from(g) / 255.0, f32::from(b) / 255.0, f32::from(a) / 255.0 ]
      )
    }
    else
    {
      F32x4::default()
    }
  }

  /// A repeater assignment : ( layer index, target primitive id range, repeater ).
  type RepeaterAssignment = ( usize, Range< usize >, interpoli::Repeater );

  /// Builds a primitive from a shape geometry : a spline becomes stroked curve geometry,
  /// anything else is evaluated and triangulated as a filled contour.
  fn geometry_to_primitive( geometry : &Geometry, stroke_width : f32 ) -> Option< PrimitiveData >
  {
    if let Geometry::Spline( Spline { values, .. } ) = geometry
    {
      let path = values.first()?;
      let contour = path.iter()
      .map( | p | [ p.x as f32, p.y as f32 ] )
      .collect::< Vec< _ > >();
      crate::primitive::curve_to_geometry( contour.as_slice(), stroke_width )
    }
    else
    {
      let mut path = vec![];
      geometry.evaluate( 0.0, &mut path );
      let contours = crate::primitive::path_to_points( path );
      crate::primitive::contours_to_fill_geometry( &[ contours ] )
    }
  }

  /// Builds the base primitive of a layer — the node every other primitive of the layer parents to.
  fn layer_base_primitive( i : usize, layer : &interpoli::Layer, brush : &Brush ) -> PrimitiveData
  {
    PrimitiveData
    {
      name : Some( format!( "{i}" ).into_boxed_str() ),
      attributes : None,
      parent : layer.parent,
      behavior : Behavior
      {
        animated_transform : Some( layer.transform.clone() ),
        repeater : None,
        brush : brush.clone(),
        frames : layer.frames.clone()
      },
      transform : Transform::default(),
    }
  }

  /// Processes one layer : builds its base primitive plus one primitive per geometry shape,
  /// queues group contents as new sublayers, and records repeater assignments.
  fn layer_to_primitives
  (
    i : usize,
    layers : &mut Vec< interpoli::Layer >,
    repeaters : &mut Vec< RepeaterAssignment >
  ) -> Option< Vec< PrimitiveData > >
  {
    let layer = layers[ i ].clone();
    let Content::Shape( shapes ) = layer.content.clone()
    else
    {
      return None;
    };

    let mut layer_primitives = vec![];

    let mut brush = Brush::Fixed( interpoli::fixed::Brush::Solid( color::AlphaColor::from_rgba8( 0, 0, 0, 0 ) ) );

    let mut stroke_width = 1.0;

    layer_primitives.push( layer_base_primitive( i, &layer, &brush ) );

    let mut last_repeater_id = 0;
    let mut last_repeater : Option< interpoli::Repeater > = None;

    for shape in shapes
    {
      match shape
      {
        Shape::Group( shapes, group_transform ) =>
        {
          let mut sublayer = layer.clone();
          sublayer.content = Content::Shape( shapes );
          sublayer.parent = Some( i );
          if let Some( group_transform ) = group_transform
          {
            sublayer.transform = group_transform.transform.clone();
            sublayer.opacity = group_transform.opacity.clone();
          }
          layers.push( sublayer );
          if let Some( ref repeater ) = last_repeater
          {
            repeaters.push( ( layers.len() - 1, 0..0, repeater.clone() ) );
          }
        },
        Shape::Geometry( geometry ) =>
        {
          if let Some( mut primitive ) = geometry_to_primitive( &geometry, stroke_width )
          {
            primitive.behavior = Behavior
            {
              animated_transform : None,
              repeater : None,
              brush : brush.clone(),
              frames : layer.frames.clone()
            };
            layer_primitives.push( primitive );
          }
        },
        Shape::Draw
        (
          Draw
          {
            stroke,
            brush : draw_brush,
            ..
          }
        ) =>
        {
          if let Some( Stroke::Fixed( stroke ) ) = stroke
          {
            stroke_width = stroke.width as f32;
          }

          brush = draw_brush.clone();
        },
        Shape::Repeater( repeater ) =>
        {
          repeaters.push( ( i, last_repeater_id..layer_primitives.len(), repeater.clone() ) );
          last_repeater = Some( repeater.clone() );
          last_repeater_id = layer_primitives.len();
        },
      }
    }

    Some( layer_primitives )
  }

  /// Walks the composition layers — group contents are appended as new sublayers while
  /// iterating — and produces per-layer primitive lists plus all repeater assignments.
  fn layer_primitives_collect
  (
    layers : &mut Vec< interpoli::Layer >
  ) -> ( Vec< Vec< PrimitiveData > >, Vec< RepeaterAssignment > )
  {
    let mut primitives = vec![];
    let mut repeaters = vec![];

    let mut i = 0;
    while i < layers.len()
    {
      let Some( layer_primitives ) = layer_to_primitives( i, layers, &mut repeaters )
      else
      {
        continue;
      };
      primitives.push( layer_primitives );
      i += 1;
    }

    ( primitives, repeaters )
  }

  /// Applies collected repeater assignments : an empty id range targets the layer's base
  /// primitive, otherwise every primitive in the range receives the repeater.
  fn repeaters_apply( primitives : &mut [ Vec< PrimitiveData > ], repeaters : Vec< RepeaterAssignment > )
  {
    for ( layer, primitive_ids, repeater ) in repeaters
    {
      if primitive_ids.end == 0
      {
        primitives[ layer ][ 0 ].behavior.repeater = Some( repeater );
      }
      else
      {
        for primitive_id in primitive_ids
        {
          primitives[ layer ][ primitive_id ].behavior.repeater = Some( repeater.clone() );
        }
      }
    }
  }

  /// Resolves parent links : layer bases keep their layer parent, in-layer primitives parent
  /// to their layer base, and layer-parent indices are remapped to primitive ids.
  fn parents_assign( composition : &Composition, primitives : &mut [ Vec< PrimitiveData > ] )
  {
    let layer_iter = composition.layers.iter().enumerate()
    .zip( primitives.iter_mut() );

    let mut last_element_id = 0;
    let mut parent_layer_to_primitive_id = FxHashMap::default();
    for ( ( i, layer ), primitives ) in layer_iter
    {
      parent_layer_to_primitive_id.insert( i, last_element_id );
      if layer.parent.is_some()
      {
        primitives[ 0 ].parent = layer.parent;
      }
      let layer_name = primitives[ 0 ].name.clone();
      for ( j, primitive ) in primitives.iter_mut().skip( 1 ).enumerate()
      {
        primitive.parent = Some( last_element_id );
        primitive.name = Some( format!( "{}_{j}", layer_name.clone().unwrap() ).into_boxed_str() );
      }
      last_element_id += primitives.len();
    }

    let layer_iter = composition.layers.iter()
    .zip( primitives.iter_mut() );
    for ( layer, primitives ) in layer_iter
    {
      if let Some( parent_id ) = layer.parent
      {
        primitives[ 0 ].parent = parent_layer_to_primitive_id.get( &parent_id ).copied();
      }
    }
  }

  /// Represents a complete animation, holding the GLTF scene data and animation behaviors.
  pub struct Animation
  {
    gltf : GLTF,
    behaviors : FxHashMap< Box< str >, Behavior >,
    _composition : Composition
  }

  impl Animation
  {
    /// Creates a new `Animation` instance from a composition and a WebGL context.
    pub fn new( gl : &GL, composition : impl Into< Composition > ) -> Self
    {
      let composition : Composition = composition.into();

      let mut layers = composition.layers.clone();
      let ( mut primitives, repeaters ) = layer_primitives_collect( &mut layers );
      repeaters_apply( &mut primitives, repeaters );
      parents_assign( &composition, &mut primitives );

      let primitives_data = primitives.into_iter()
      .flatten()
      .collect::< Vec< _ > >();

      let behaviors = primitives_data.iter()
      .filter_map
      (
        | p |
        {
          p.name.as_ref().map( | name | ( name.clone(), p.behavior.clone() ) )
        }
      )
      .collect::< FxHashMap< _, _ > >();

      let gltf = primitives_data_to_gltf( gl, primitives_data.as_slice() );

      Self
      {
        gltf,
        behaviors,
        _composition : composition
      }
    }

    /// Updates the scene nodes with their animated transformations and repeater logic for a given frame.
    fn scene_update( &self, scene : &mut Scene, frame : f64 )
    {
      let mut nodes_to_insert = vec![];

      let mut update =
      |
        node : Rc< RefCell< Node > >
      | -> Result< (), gl::WebglError >
      {
        let Some( node_name ) = node.borrow().name_get()
        else
        {
          return Ok( () );
        };

        if let Some( behaviour ) = self.behaviors.get( &node_name )
        {
          if let Some( animated_transform ) = &behaviour.animated_transform
          {
            let matrix = affine_to_matrix( animated_transform.evaluate( frame ).into_owned() );
            node.borrow_mut().local_matrix_set( matrix );
          }

          let Some( ref repeater ) = behaviour.repeater
          else
          {
            return Ok( () );
          };

          let Some( parent ) = node.borrow().parent_get().clone()
          else
          {
            return Ok( () );
          };

          let Some( id ) = parent.borrow().children_get()
          .iter()
          .enumerate()
          .find( | ( _, child ) | child.borrow().name_get().as_ref() == Some( &node_name ) )
          .map( | ( i, _ ) | i )
          else
          {
            return Ok( () );
          };

          let repeater = repeater.evaluate( frame ).into_owned();

          if repeater.copies < 2
          {
            return Ok( () );
          }

          let matrix = node.borrow_mut().local_matrix_get();

          let mut ids_and_children = Vec::with_capacity( repeater.copies );

          for i in ( 0..repeater.copies ).rev()
          {
            let node_clone = node.borrow().tree_clone();
            let transform = affine_to_matrix( repeater.transform( i ) );

            node_clone.borrow_mut().local_matrix_set( matrix * transform );
            node_clone.borrow_mut().parent_set( Some( parent.clone() ) );
            ids_and_children.push( ( id + 1, node_clone.clone() ) );
          }

          nodes_to_insert.push( ( parent.clone(), ids_and_children ) );
        }

        Ok( () )
      };

      let _ = scene.traverse( &mut update );

      for ( parent, ids_and_children ) in nodes_to_insert.into_iter().rev()
      {
        for ( i, child ) in ids_and_children.into_iter().rev()
        {
          parent.borrow_mut().child_insert( i, child );
        }
      }
    }

    /// Filters and removes nodes from the scene that are outside of their defined frame range.
    fn nodes_filter( &self, scene : &mut Scene, frame : f64 )
    {
      let mut nodes_to_remove = FxHashMap::default();

      let mut get_nodes_to_remove =
      |
        node : Rc< RefCell< Node > >
      | -> Result< (), gl::WebglError >
      {
        let Some( name ) = node.borrow_mut().name_get()
        else
        {
          return Ok( () );
        };
        if let Some( behaviour ) = self.behaviors.get( &name )
        {
          if !( behaviour.frames.start <= frame && frame <= behaviour.frames.end )
          {
            nodes_to_remove.insert( name, node.clone() );
            return Ok( () );
          }
        }

        Ok( () )
      };

      let _ = scene.traverse( &mut get_nodes_to_remove );

      scene.children
      .retain
      (
        | n |
        {
          let Some( name ) = n.borrow().name_get()
          else
          {
            return false;
          };
          !nodes_to_remove.contains_key( &name )
        }
      );

      let mut nodes = scene.children.clone();

      let mut i = 0;
      while i < nodes.len()
      {
        let Some( node ) = nodes.get( i ).cloned()
        else
        {
          break;
        };

        let mut id_to_remove = vec![];

        for ( i, child )  in node.borrow().children_get().iter().enumerate()
        {
          let Some( name ) = child.borrow().name_get()
          else
          {
            continue;
          };
          if nodes_to_remove.contains_key( &name )
          {
            id_to_remove.push( i );
          }
        }

        for i in id_to_remove.iter().rev()
        {
          if node.borrow().children_get().get( *i ).is_none()
          {
            continue;
          }
          let child = node.borrow_mut().child_remove( *i );
          child.borrow_mut().parent_set( None );
        }

        nodes.extend( node.borrow().children_get().iter().cloned() );

        i += 1;
      }
    }

    /// Retrieves the color for each node in the scene based on its associated brush behavior.
    fn colors_from_scene( &self, scene : &mut Scene, frame : f64 ) -> Vec< F32x4 >
    {
      let mut colors = vec![];

      let mut add_color =
      |
        node : Rc< RefCell< Node > >
      | -> Result< (), gl::WebglError >
      {
        let Some( name ) = node.borrow_mut().name_get()
        else
        {
          return Ok( () );
        };

        let color = if let Some( behaviour ) = self.behaviors.get( &name )
        {
          brush_to_color( &behaviour.brush, frame )
        }
        else
        {
          F32x4::from_array( [ 0.0; 4 ] )
        };

        colors.push( color );

        Ok( () )
      };

      let _ = scene.traverse( &mut add_color );

      colors
    }

    /// Returns a new scene and a list of colors for a specific animation frame.
    /// Receives as input time moment from animation start in milliseconds
    pub fn frame( &self, frame : f64 ) -> Option< ( Scene, Vec< F32x4 > ) >
    {
      let scene = self.gltf.scenes.first()?;

      let mut scene = scene.borrow().clone();

      self.nodes_filter( &mut scene, frame );
      self.scene_update( &mut scene, frame );
      let colors = self.colors_from_scene( &mut scene, frame );

      scene.world_matrix_update();

      Some( ( scene, colors ) )
    }

    /// Sets the world matrix for all scenes within the GLTF data.
    pub fn world_matrix_set( &self, world_matrix : F32x4x4 )
    {
      for scene in &self.gltf.scenes
      {
        let old_local_matrix = scene.borrow().local_matrix_get();
        scene.borrow_mut().local_matrix_set( world_matrix * old_local_matrix );
        scene.borrow_mut().world_matrix_update();
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    affine_to_matrix,
    Animation
  };
}
