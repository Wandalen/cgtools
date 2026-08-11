use velato::Composition;
use velato::model::
{
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
use std::collections::HashMap;
use minwebgl as gl;
use gl::
{
  F32x4,
  F32x4x4,
  GL,
  math::nd
};
use std::cell::RefCell;
use std::rc::Rc;
use primitive_generation::{ PrimitiveData, Transform, primitives_data_to_gltf };
use std::ops::Range;

use renderer::webgl::
{
  Scene,
  Node
};

/// Defines the dynamic behavior of a primitive, including animation, repetition, and color.
#[ derive( Debug, Clone ) ]
pub struct Behavior
{
  pub animated_transform : Option< velato::model::Transform >,
  pub repeater : Option< velato::model::Repeater >,
  pub brush : velato::model::Brush,
  pub frames : Range< f64 >,
}

impl Default for Behavior
{
  fn default() -> Self
  {
    Self
    {
      animated_transform : None,
      repeater : None,
      brush : velato::model::Brush::Fixed( velato::model::fixed::Brush::default() ),
      frames : 0.0..0.0
    }
  }
}

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

/// Evaluates a `Brush` at a given `frame` and converts the resulting color to a `F32x4` array.
fn brush_to_color( brush : &velato::model::Brush, frame : f64 ) -> F32x4
{
  let color = match brush.evaluate( 1.0, frame ).into_owned()
  {
    velato::model::fixed::Brush::Solid( color ) => Some( color ),
    _ => None
  };

  if let Some( color ) = color
  {
    let [ r, g, b, a ] = color.to_rgba8().to_u8_array();
    F32x4::from_array
    (
      [
        f32::from( r ),
        f32::from( g ),
        f32::from( b ),
        f32::from( a )
      ]
    )
    / 255.0
  }
  else
  {
    F32x4::default()
  }
}

/// Repeater assignment collected while processing layers : ( layer index, primitive id range, repeater ).
type RepeaterAssignment = ( usize, Range< usize >, velato::model::Repeater );

/// Expands `Instance` layers into their asset sublayers and returns the extended layer list.
fn expand_instance_layers( composition : &Composition ) -> Vec< velato::model::Layer >
{
  let assets = composition.assets.clone();

  let mut layers = composition.layers.clone();

  let mut additional_layers : Vec< velato::model::Layer > = vec![];

  let layers_count = layers.len();

  for ( i, layer ) in layers.iter_mut().enumerate()
  {
    let velato::model::Content::Instance{ name : ref asset_name, .. } = layer.content
    else
    {
      continue;
    };

    if let Some( asset_layers ) = assets.get( asset_name ).cloned()
    {
      for mut sublayer in asset_layers
      {
        sublayer.parent = if let Some( k ) = sublayer.parent
        {
          let j = layers_count + additional_layers.len();
          Some( j + k )
        }
        else
        {
          Some( i )
        };

        additional_layers.push( sublayer );
      }
    }
  }

  layers.extend( additional_layers );

  layers
}

/// Scans a layer's shapes for the last draw brush, defaulting to a fully transparent solid brush.
fn scan_layer_brush( shapes : &[ Shape ] ) -> Brush
{
  let mut brush = Brush::Fixed( velato::model::fixed::Brush::Solid( color::AlphaColor::from_rgba8( 0, 0, 0, 0 ) ) );

  for shape in shapes
  {
    if let Shape::Draw( Draw { brush : b, .. } ) = shape
    {
      brush = b.clone();
    }
  }

  brush
}

/// Converts a geometry shape into primitive data, returning `None` when tessellation yields nothing.
fn geometry_to_primitive( geometry : &Geometry, stroke_width : f32 ) -> Option< PrimitiveData >
{
  if let Geometry::Spline( Spline { values, .. } ) = geometry
  {
    let path = values.first()?;
    let contour = path.start.clone().into_iter()
    .map( | p | [ p.x as f32, p.y as f32 ] )
    .collect::< Vec< _ > >();
    primitive_generation::primitive::curve_to_geometry( contour.as_slice(), stroke_width )
  }
  else
  {
    let mut path = vec![];
    geometry.evaluate( 0.0, &mut path );
    let contours = primitive_generation::path_to_points( path );
    primitive_generation::primitive::contours_to_fill_geometry( &[ contours ] )
  }
}

/// Builds the base primitive and behavior for a layer from its scanned brush.
fn layer_base_primitive( i : usize, layer : &velato::model::Layer, brush : &Brush ) -> ( PrimitiveData, Behavior )
{
  (
    PrimitiveData
    {
      name : Some( format!( "{i}" ).into_boxed_str() ),
      attributes : None,
      parent : layer.parent,
      transform : Transform::default(),
      color : F32x4::default(),
    },
    Behavior
    {
      animated_transform : Some( layer.transform.clone() ),
      repeater : None,
      brush : brush.clone(),
      frames : layer.frames.clone()
    }
  )
}

/// Processes one layer's shape content into primitives, appending group sublayers to `layers` and repeater assignments to `repeaters`; returns `None` for non-shape content.
fn layer_to_primitives
(
  i : usize,
  layers : &mut Vec< velato::model::Layer >,
  repeaters : &mut Vec< RepeaterAssignment >
) -> Option< Vec< ( PrimitiveData, Behavior ) > >
{
  let layer = layers[ i ].clone();
  let Content::Shape( shapes ) = layer.content.clone()
  else
  {
    return None;
  };

  let mut layer_primitives = vec![];

  let mut brush = scan_layer_brush( &shapes );

  let mut stroke_width = 1.0;

  layer_primitives.push( layer_base_primitive( i, &layer, &brush ) );

  let mut last_repeater_id = 0;
  let mut last_repeater : Option< velato::model::Repeater > = None;

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
        if let Some( primitive ) = geometry_to_primitive( &geometry, stroke_width )
        {
          let behavior = Behavior
          {
            animated_transform : None,
            repeater : None,
            brush : brush.clone(),
            frames : layer.frames.clone()
          };
          layer_primitives.push( ( primitive, behavior ) );
        }
      },
      Shape::Draw
      (
        Draw
        {
          stroke,
          brush : b,
          ..
        }
      ) =>
      {
        if let Some( Stroke::Fixed( stroke ) ) = stroke
        {
          stroke_width = stroke.width as f32;
        }

        brush = b.clone();
      },
      Shape::Repeater( repeater ) =>
      {
        repeaters.push( ( i, last_repeater_id..layer_primitives.len(), repeater.clone() ) );
        last_repeater = Some( repeater.clone() );
        last_repeater_id = layer_primitives.len();
      },
      Shape::Trim( _ ) => {}
    }
  }

  Some( layer_primitives )
}

/// Collects primitives for every layer, including group sublayers appended during processing.
fn collect_layer_primitives
(
  layers : &mut Vec< velato::model::Layer >
) -> ( Vec< Vec< ( PrimitiveData, Behavior ) > >, Vec< RepeaterAssignment > )
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

/// Applies collected repeater assignments onto their target primitives.
fn apply_repeaters
(
  primitives : &mut [ Vec< ( PrimitiveData, Behavior ) > ],
  repeaters : Vec< RepeaterAssignment >
)
{
  for ( layer, primitive_ids, repeater ) in repeaters
  {
    if primitive_ids.end == 0
    {
      primitives[ layer ][ 0 ].1.repeater = Some( repeater );
    }
    else
    {
      for primitive_id in primitive_ids
      {
        primitives[ layer ][ primitive_id ].1.repeater = Some( repeater.clone() );
      }
    }
  }
}

/// Assigns parent links and hierarchical names across all collected primitives.
fn assign_parents
(
  layers : &[ velato::model::Layer ],
  primitives : &mut [ Vec< ( PrimitiveData, Behavior ) > ]
)
{
  let layer_iter = layers.iter().enumerate()
  .zip( primitives.iter_mut() );

  let mut last_element_id = 0;
  let mut parent_layer_to_primitive_id = HashMap::new();
  for ( ( i, layer ), primitives ) in layer_iter
  {
    parent_layer_to_primitive_id.insert( i, last_element_id );
    if layer.parent.is_some()
    {
      primitives[ 0 ].0.parent = layer.parent;
    }
    let layer_name = primitives[ 0 ].0.name.clone();
    for ( j, primitive ) in primitives.iter_mut().skip( 1 ).enumerate()
    {
      primitive.0.parent = Some( last_element_id );
      primitive.0.name = Some( format!( "{}_{j}", layer_name.clone().unwrap() ).into_boxed_str() );
    }
    last_element_id += primitives.len();
  }

  let layer_iter = layers.iter()
  .zip( primitives.iter_mut() );
  for ( layer, primitives ) in layer_iter
  {
    if let Some( parent_id ) = layer.parent
    {
      primitives[ 0 ].0.parent = parent_layer_to_primitive_id.get( &parent_id ).copied();
    }
  }
}

/// Represents a loaded and parsed animation, ready to be rendered.
pub struct Animation
{
  /// The GLTF scene data for the animation.
  gltf : GLTF,
  /// A map of node names to their associated behaviors, such as transforms and repeaters.
  behaviors : HashMap< Box< str >, Behavior >,
  /// The original Lottie composition data.
  #[ expect( dead_code, reason = "original composition retained alongside its derived GLTF scene and behaviors; not read back after construction" ) ]
  composition : Composition
}

impl Animation
{
  /// Creates a new `Animation` from a `Composition` object. This function processes the composition's layers and shapes to build a GLTF scene and a map of animation behaviors.
  pub fn new( gl : &GL, composition : impl Into< Composition > ) -> Self
  {
    let composition : Composition = composition.into();

    let mut layers = expand_instance_layers( &composition );
    let ( mut primitives, repeaters ) = collect_layer_primitives( &mut layers );
    apply_repeaters( &mut primitives, repeaters );
    assign_parents( &layers, &mut primitives );

    let primitives_data = primitives.into_iter()
    .flatten()
    .collect::< Vec< _ > >();

    let behaviors = primitives_data.iter()
    .filter_map
    (
      | p |
      {
        p.0.name.as_ref().map( | name | ( name.clone(), p.1.clone() ) )
      }
    )
    .collect::< HashMap< _, _ > >();

    let primitives = primitives_data.into_iter()
    .map( | ( p, _ ) | p )
    .collect::< Vec< _ > >();
    let gltf = primitives_data_to_gltf( gl, &primitives );

    Self
    {
      gltf,
      behaviors,
      composition
    }
  }

  /// Traverses and updates the scene's nodes based on the animation behaviors for a given frame.
  fn update_scene( &self, scene : &mut Scene, frame : f64 )
  {
    let mut nodes_to_insert = vec![];

    let mut update =
    |
      node : Rc< RefCell< Node > >
    | -> Result< (), gl::WebglError >
    {
      let Some( node_name ) = node.borrow().get_name()
      else
      {
        return Ok( () );
      };

      if let Some( behaviour ) = self.behaviors.get( &node_name )
      {
        if let Some( animated_transform ) = &behaviour.animated_transform
        {
          let matrix = affine_to_matrix( animated_transform.evaluate( frame ).into_owned() );
          node.borrow_mut().set_local_matrix( matrix );
        }

        let Some( ref repeater ) = behaviour.repeater
        else
        {
          return Ok( () );
        };

        let Some( parent ) = node.borrow().get_parent().clone()
        else
        {
          return Ok( () );
        };

        let Some( id ) = parent.borrow().get_children()
        .iter()
        .enumerate()
        .find( | ( _, child ) | child.borrow().get_name().as_ref() == Some( &node_name ) )
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

        let matrix = node.borrow_mut().get_local_matrix();

        let mut ids_and_children = Vec::with_capacity( repeater.copies );

        for i in ( 0..repeater.copies ).rev()
        {
          let node_clone = node.borrow().clone_tree();
          let transform = affine_to_matrix( repeater.transform( i ) );

          node_clone.borrow_mut().set_local_matrix( matrix * transform );
          node_clone.borrow_mut().set_parent( Some( parent.clone() ) );
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
        parent.borrow_mut().insert_child( i, child );
      }
    }
  }

  /// Filters and removes nodes from the scene that are outside of their active frame range for a given frame.
  fn filter_nodes( &self, scene : &mut Scene, frame : f64 )
  {
    let mut nodes_to_remove = HashMap::new();

    let mut get_nodes_to_remove =
    |
      node : Rc< RefCell< Node > >
    | -> Result< (), gl::WebglError >
    {
      let Some( name ) = node.borrow_mut().get_name()
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
        let Some( name ) = n.borrow().get_name()
        else
        {
          return true;
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

      for ( i, child ) in node.borrow().get_children().iter().enumerate()
      {
        let Some( name ) = child.borrow().get_name()
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
        if node.borrow().get_children().get( *i ).is_none()
        {
          continue;
        }
        let child = node.borrow_mut().remove_child( *i );
        child.borrow_mut().set_parent( None );
      }

      nodes.extend( node.borrow().get_children().iter().cloned() );

      i += 1;
    }
  }

  /// Gathers and returns the colors for all visible nodes in the scene at a given frame.
  fn colors_from_scene( &self, scene : &mut Scene, frame : f64 ) -> Vec< F32x4 >
  {
    let mut colors = vec![];

    let mut add_color =
    |
      node : Rc< RefCell< Node > >
    | -> Result< (), gl::WebglError >
    {
      let Some( name ) = node.borrow_mut().get_name()
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

  /// Calculates and returns the updated scene and colors for a specific animation frame.
  pub fn frame( &self, frame : f64 ) -> Option< ( Scene, Vec< F32x4 > ) >
  {
    let scene = self.gltf.scenes.first()?;

    let mut scene = scene.borrow().clone();

    self.filter_nodes( &mut scene, frame );
    self.update_scene( &mut scene, frame );
    let colors = self.colors_from_scene( &mut scene, frame );

    scene.update_world_matrix();

    Some( ( scene, colors ) )
  }

  /// Sets the world matrix for all scenes within the GLTF data.
  pub fn set_world_matrix( &self, world_matrix : F32x4x4 )
  {
    for scene in &self.gltf.scenes
    {
      let old_local_matrix = scene.borrow().get_local_matrix();
      scene.borrow_mut().set_local_matrix( world_matrix * old_local_matrix );
      scene.borrow_mut().update_world_matrix();
    }
  }
}

/// Asynchronously loads a Lottie animation file from a given path and constructs a new `Animation` object.
pub async fn load_animation( gl : &GL, path : &str ) -> Animation
{
  let lottie_json_bin = gl::file::load( path ).await.unwrap();
  let composition = Composition::from_slice( lottie_json_bin.as_slice() ).unwrap();
  Animation::new( gl, composition )
}
