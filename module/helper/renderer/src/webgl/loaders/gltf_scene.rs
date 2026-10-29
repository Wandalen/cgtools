//! The glTF loader's scene graph : the document's nodes with their hierarchy, transforms,
//! meshes and lights, the skeletons attached to rigged meshes, and the scenes over them.

mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use gltf::mesh::iter::MorphTargets;
  use minwebgl as gl;
  use crate::webgl::{ helpers, material::PbrMaterial, Mesh, Node, Object3D, Scene };
  use crate::webgl::loaders::gltf_lights::{ light_get, light_list_get };
  use crate::webgl::loaders::gltf_skeleton::skeleton_load;

  /// A node prepared for skeleton attachment : the node, its glTF skin, its
  /// primitives' morph targets, and its mesh's morph weights.
  pub type RiggedNode< 'a > =
  (
    Rc< RefCell< Node > >,
    Option< gltf::Skin< 'a > >,
    Option< Vec< MorphTargets< 'a > > >,
    Option< Vec< f32 > >
  );

  /// Product of [`nodes_create`] : the flat node list, per-node
  /// skeleton-attachment data, and the nodes carrying lights.
  pub struct NodesCreated< 'a >
  {
    /// Every node in the glTF document, in document order, with hierarchy
    /// wired via [`Node::child_add`] and transform/object resolved.
    pub nodes : Vec< Rc< RefCell< Node > > >,
    /// Each node's skeleton-attachment data, which `gltf::load` passes to [`skeletons_attach`].
    /// Not public, as on `master`, where only `gltf.rs` could read it: `skeletons_attach` is,
    /// so a caller outside the crate builds its own `RiggedNode` list for it, as
    /// `tests/gltf_node_scene_test.rs` does. `pub( crate )` only because `load` now lives in
    /// another module.
    pub( crate ) rigged_nodes : Vec< RiggedNode< 'a > >,
    /// The subset of `nodes` whose `object` resolved to [`Object3D::Light`].
    pub lights : Vec< Rc< RefCell< Node > > >
  }

  /// Instantiates every glTF node with its transform and object ( mesh,
  /// light, or plain ), wires the child hierarchy, and returns the flat node
  /// list, skeleton-attachment data, and the light nodes. Pure data
  /// transform over the parsed document and pre-built meshes -- no GL calls.
  #[ must_use ]
  pub fn nodes_create< 'a >
  (
    gltf_file : &'a gltf::Gltf,
    meshes : &[ Rc< RefCell< Mesh > > ]
  )
  -> NodesCreated< 'a >
  {
    let gltf_lights = light_list_get( gltf_file ).unwrap_or_default();

    let mut nodes = Vec::new();
    let mut rigged_nodes = Vec::new();
    let mut lights = Vec::new();

    for gltf_node in gltf_file.nodes()
    {
      let mut node = Node::default();
      node.visibility_set( true, true );
      let mut is_light = false;

      let ( translation, rotation, scale ) = gltf_node.transform().decomposed();
      node.scale_set( scale );
      node.translation_set( translation );
      node.rotation_set( gl::QuatF32::from( rotation ) );

      node.object = if let Some( mesh ) = gltf_node.mesh()
      {
        Object3D::Mesh( meshes[ mesh.index() ].clone() )
      }
      else if let Some( light ) = light_get( &gltf_node, &node, &gltf_lights )
      {
        is_light = true;
        Object3D::Light( light )
      }
      else
      {
        Object3D::Other
      };

      if let Some( name ) = gltf_node.name() { node.name_set( name ); }

      let node = Rc::new( RefCell::new( node ) );

      let ( primitives_morph_targets, weights ) = if let Some( mesh ) = gltf_node.mesh()
      {
        (
          Some( mesh.primitives().map( | p | p.morph_targets() ).collect::< Vec< _ > >() ),
          mesh.weights().map( <[f32]>::to_vec )
        )
      }
      else
      {
        ( None, None )
      };
      rigged_nodes.push( ( node.clone(), gltf_node.skin(), primitives_morph_targets, weights ) );

      if is_light
      {
        lights.push( node.clone() );
      }

      nodes.push( node );
    }

    for gltf_node in gltf_file.nodes()
    {
      let mut node = nodes[ gltf_node.index() ].borrow_mut();
      for child in gltf_node.children()
      {
        node.child_add( nodes[ child.index() ].clone() );
      }
    }

    NodesCreated { nodes, rigged_nodes, lights }
  }

  /// Attaches a [`Skeleton`]( crate::webgl::Skeleton ) to every rigged mesh, switching its materials
  /// onto the skinning / morph-target shader paths. A rigged node with no
  /// skin and no morph targets is a no-op : `gltf_skeleton::skeleton_load` returns `None`
  /// and the mesh's `skeleton` field ( and its material's defines ) are left
  /// untouched : `PbrMaterial::define_add` is only reached once a real skeleton
  /// comes back. Takes no GL context and makes no GL calls.
  pub fn skeletons_attach
  (
    nodes : &[ Rc< RefCell< Node > > ],
    rigged_nodes : Vec< RiggedNode< '_ > >,
    bin_buffers : &[ Vec< u8 > ]
  )
  {
    for ( node, skin, primitives_morph_targets, weights ) in rigged_nodes
    {
      if let Object3D::Mesh( mesh ) = &node.borrow().object
      {
        let primitives_vertices_count = mesh.borrow().primitives.iter()
        .map( | p | p.borrow().geometry.borrow().vertex_count as usize )
        .collect::< Vec< _ > >();
        if let Some( skeleton ) = skeleton_load
        (
          skin,
          nodes,
          primitives_morph_targets.as_ref(),
          primitives_vertices_count.as_slice(),
          weights,
          bin_buffers
        )
        {
          mesh.borrow_mut().skeleton = Some( skeleton.clone() );
          for primitive in &mesh.borrow().primitives
          {
            let p = primitive.borrow();
            let mut mat_mut = helpers::cast_unchecked_material_to_ref_mut::< PbrMaterial >(  p.material.borrow_mut() );

            if skeleton.borrow().has_skin()
            {
              mat_mut.define_add( "USE_SKINNING", String::new() );
            }

            if skeleton.borrow().has_morph_targets()
            {
              mat_mut.define_add( "USE_MORPH_TARGET", String::new() );
            }
          }
        }
      }
    }
  }

  /// Builds every glTF scene from the instantiated nodes and computes the
  /// initial world matrices. Pure data transform over already-instantiated
  /// nodes -- no GL calls.
  #[ must_use ]
  pub fn scenes_create
  (
    gltf_file : &gltf::Gltf,
    nodes : &[ Rc< RefCell< Node > > ]
  )
  -> Vec< Rc< RefCell< Scene > > >
  {
    let mut scenes = Vec::new();

    for gltf_scene in gltf_file.scenes()
    {
      let mut scene = Scene::default();
      for gltf_node in gltf_scene.nodes()
      {
        scene.add( nodes[ gltf_node.index() ].clone() );
      }
      scene.world_matrix_update();
      scenes.push( Rc::new( RefCell::new( scene ) ) );
    }

    scenes
  }
}

crate::mod_interface!
{
  own use
  {
    RiggedNode,
    NodesCreated,
    nodes_create,
    skeletons_attach,
    scenes_create,
  };
}
