//! The glTF loader's skins and morph targets : joint matrices and morph displacements read on
//! the CPU from the document's buffers, and the [`Skeleton`]( crate::webgl::Skeleton ) a mesh
//! gets from them.

mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use gltf::mesh::iter::MorphTargets;
  use minwebgl as gl;
  use gl::F32x4x4;
  use crate::webgl::{ skeleton, Node, Skeleton };
  use crate::webgl::loaders::gltf_quantization::{ accessor_readable, accessor_vec3_read };

  /// Reads a skin's inverse bind matrices and resolves each joint to its node by
  /// [`gltf::Node::index()`] against the flat, index-ordered `nodes` slice -- the same
  /// resolution convention every other node lookup in this loader uses.
  ///
  /// `None` when the skin has no inverse bind matrices, or when their accessor is not `MAT4`
  /// `f32`, does not hold one matrix per joint, or has ranges that do not fit ( see
  /// `accessor_readable` ); the last three are logged.
  #[ must_use ]
  pub fn skeleton_transforms_data_load
  (
    skin : &gltf::Skin< '_ >,
    nodes : &[ Rc< RefCell< Node > > ],
    buffers : &[ Vec< u8 > ]
  )
  -> Option< skeleton::TransformsData >
  {
    let reader = skin.reader
    (
      | buffer | Some( buffers[ buffer.index() ].as_slice() )
    );

    let accessor = skin.inverse_bind_matrices()?;
    // The spec requires one matrix per joint; `joints` below pairs them positionally.
    if accessor.count() != skin.joints().len()
    || !accessor_readable( &accessor, gltf::accessor::Dimensions::Mat4, &[ gltf::accessor::DataType::F32 ] )
    {
      gl::warn!( "glTF skin {} has unreadable inverse bind matrices ( accessor {} ), loading it without a skin", skin.index(), accessor.index() );
      return None;
    }
    let inverse_bind_matrices_iter = reader.read_inverse_bind_matrices()?;

    let matrices = inverse_bind_matrices_iter
    .map
    (
      | m |
      {
        let mut matrix = [ 0.0_f32; 16 ];
        for ( dst, src ) in matrix.iter_mut().zip( m.iter().flatten() )
        {
          *dst = *src;
        }
        F32x4x4::from_column_major( matrix )
      }
    )
    .collect::< Vec< _ > >();

    // Fix(BUG-173): joints were resolved by matching `joint.name()` against a name-keyed
    // map, silently dropping any joint whose node had no `name` (optional per glTF spec)
    // or whose name collided with another node's -- corrupting every subsequent joint's
    // binding once one was dropped, since `JOINTS_0`/`JOINTS_1` vertex attributes index
    // this list positionally.
    // Root cause: `skin.joints()`'s iteration position IS the joint index vertex data
    // references, but resolution went through a name lookup instead of `joint.index()`
    // against the flat, index-ordered node list every other lookup in this file already
    // uses.
    // Pitfall: an optional, non-unique glTF field (node name) used as a resolution key
    // for data that is actually positionally/numerically indexed is a silent-drop trap --
    // it only surfaces once an asset has an unnamed or duplicate-named joint node.
    let joints = skin.joints()
    .zip( matrices )
    .map( | ( joint, matrix ) | ( nodes[ joint.index() ].clone(), matrix ) )
    .collect::< Vec< _ > >();

    Some( skeleton::TransformsData::new( joints ) )
  }

  /// One morph target channel's displacements : the accessor's values, or
  /// `vertices_count` zeros when the target does not declare that channel or
  /// `accessor_vec3_read` refuses it.
  fn morph_channel_read
  (
    channel : Option< gltf::Accessor< '_ > >,
    vertices_count : usize,
    buffers : &[ Vec< u8 > ]
  )
  -> Vec< [ f32; 3 ] >
  {
    channel.and_then( | acc | accessor_vec3_read( acc, buffers ) )
    .unwrap_or_else( || vec![ [ 0.0; 3 ]; vertices_count ] )
  }

  /// Interleaves one primitive's per-target displacement lists vertex by vertex :
  /// every target's value for vertex 0, then for vertex 1, and so on.
  fn targets_pack
  (
    targets_array : &[ Vec< [ f32; 3 ] > ]
  )
  -> Vec< [ f32; 3 ] >
  {
    if targets_array.is_empty()
    {
      return vec![];
    }
    let mut packed_array = Vec::with_capacity( targets_array.first().unwrap().len() * targets_array.len() );
    for i in 0..targets_array.first().unwrap().len()
    {
      let targets_item = targets_array.iter()
      .map( | arr | arr[ i ] )
      .collect::< Vec< _ > >();
      packed_array.extend( targets_item );
    }

    packed_array
  }

  /// Packs a glTF primitive's morph-target position / normal / tangent
  /// displacements ( plus mesh-level morph weights ) into a
  /// [`skeleton::DisplacementsData`], or `None` if `primitives_morph_targets`
  /// is `None`. Pure data transform over the parsed document and raw buffer
  /// bytes -- no GL calls.
  ///
  /// # Panics
  ///
  /// Does not panic under normal control flow : `targets_pack`'s two
  /// `.unwrap()` calls only run once `targets_array.first()` is known to be
  /// `Some`, guarded immediately above by `targets_array.is_empty()`.
  #[ must_use ]
  pub fn skeleton_displacements_data_load
  (
    primitives_morph_targets : Option< &Vec< MorphTargets< '_ > > >,
    primitives_vertices_count : &[ usize ],
    weights : Option< Vec< f32 > >,
    buffers : &[ Vec< u8 > ]
  )
  -> Option< skeleton::DisplacementsData >
  {
    let skin_vertices_count = primitives_vertices_count.iter().sum::< usize >();
    let ( positions, normals, tangents ) =
    {
      let primitives_morph_targets = primitives_morph_targets?;

      let mut skin_positions = Vec::with_capacity( skin_vertices_count );
      let mut skin_normals = Vec::with_capacity( skin_vertices_count );
      let mut skin_tangents = Vec::with_capacity( skin_vertices_count );

      for ( i, morph_targets ) in primitives_morph_targets.iter().enumerate()
      {
        let vertices_count = primitives_vertices_count[ i ];
        let mut targets_positions = Vec::with_capacity( vertices_count );
        let mut targets_normals = Vec::with_capacity( vertices_count );
        let mut targets_tangents = Vec::with_capacity( vertices_count );

        for morph_target in morph_targets.clone()
        {
          targets_positions.push( morph_channel_read( morph_target.positions(), vertices_count, buffers ) );
          targets_normals.push( morph_channel_read( morph_target.normals(), vertices_count, buffers ) );
          targets_tangents.push( morph_channel_read( morph_target.tangents(), vertices_count, buffers ) );
        }

        let primitive_positions = targets_pack( &targets_positions );
        let primitive_normals = targets_pack( &targets_normals );
        let primitive_tangents = targets_pack( &targets_tangents );

        skin_positions.extend( primitive_positions );
        skin_normals.extend( primitive_normals );
        skin_tangents.extend( primitive_tangents );
      }

      (
        ( !skin_positions.is_empty() ).then_some( skin_positions ),
        ( !skin_normals.is_empty() ).then_some( skin_normals ),
        ( !skin_tangents.is_empty() ).then_some( skin_tangents )
      )
    };

    let mut displacements = skeleton::DisplacementsData::new();

    let _ = displacements.displacement_set( positions, &gltf::Semantic::Positions, skin_vertices_count );
    let _ = displacements.displacement_set( normals, &gltf::Semantic::Normals, skin_vertices_count );
    let _ = displacements.displacement_set( tangents, &gltf::Semantic::Tangents, skin_vertices_count );
    if let Some( weights ) = weights
    {
      let weights_rc = displacements.morph_weights_get();
      *weights_rc.borrow_mut() = weights;
    }

    Some( displacements )
  }

  /// Loads [`Skeleton`] for one [`Mesh`]( crate::webgl::Mesh )
  pub fn skeleton_load
  (
    skin : Option< gltf::Skin< '_ > >,
    nodes : &[ Rc< RefCell< Node > > ],
    primitives_morph_targets : Option< &Vec< MorphTargets< '_ > > >,
    primitives_vertices_count : &[ usize ],
    weights : Option< Vec< f32 > >,
    buffers : &[ Vec< u8 > ]
  )
  -> Option< Rc< RefCell< Skeleton > > >
  {
    let mut skeleton = Skeleton::new();

    *skeleton.transforms_as_mut() = skin
    .and_then(| s | skeleton_transforms_data_load( &s, nodes, buffers ));
    *skeleton.displacements_as_mut() = skeleton_displacements_data_load
    (
      primitives_morph_targets,
      primitives_vertices_count,
      weights,
      buffers
    );

    if skeleton.has_skin() || skeleton.has_morph_targets()
    {
      Some( Rc::new( RefCell::new( skeleton ) ) )
    }
    else
    {
      None
    }
  }
}

crate::mod_interface!
{
  own use
  {
    skeleton_transforms_data_load,
    skeleton_displacements_data_load,
  };
}

// Only `skeletons_attach` ( `gltf_scene` ) builds a skeleton for a mesh.
pub( crate ) use private::skeleton_load;
