/// Internal namespace.
mod private
{
  //use crate::*;
  use std::collections::HashSet ;
  use tobj::Material;

  #[ derive( Debug ) ]
  pub struct BoundingBox
  {
    pub min : ndarray_cg::F32x3,
    pub max : ndarray_cg::F32x3
  }

  impl Default for BoundingBox 
  {
    fn default() -> Self 
    {
      BoundingBox
      {
        min : ndarray_cg::F32x3::MAX,
        max : ndarray_cg::F32x3::MIN
      }
    }
  }

  impl BoundingBox
  {
    /// Computes the bounding box of the model from the provided positions array
    /// Positions should be in the form [ x, y, z, x, y, z, ...]
    pub fn compute( positions : &[ f32 ] ) -> Self
    {
      let mut bounding_box = BoundingBox::default();

      for i in 0..positions.len() / 3
      {
        let x = positions[ i * 3 + 0 ];
        let y = positions[ i * 3 + 1 ];
        let z = positions[ i * 3 + 2 ];

        let p = ndarray_cg::F32x3::new( x, y, z );

        bounding_box.min = p.min( bounding_box.min );
        bounding_box.max = p.max( bounding_box.max );
      }

      bounding_box
    }
  }

  #[ derive( Debug ) ]
  pub struct BoundingSphere
  {
    pub center : ndarray_cg::F32x3,
    pub radius : f32 
  }

  impl Default for BoundingSphere 
  {
    fn default() -> Self 
    {
      BoundingSphere
      {
        center : ndarray_cg::F32x3::ZERO,
        radius : 0.0
      }
    }
  }

  impl BoundingSphere
  {
    /// Computes the bounding sphere of the model form the provided positions array.
    /// Positions should be in the form [ x, y, z, x, y, z, ...].
    /// Requires BoundingBox to be computed first.
    pub fn compute( positions : &[ f32 ], bounding_box : &BoundingBox ) -> Self
    {
      let mut bs = BoundingSphere::default();
      bs.center =  0.5 * ( bounding_box.min + bounding_box.max );

      for i in 0..positions.len() / 3
      {
        let x = positions[ i * 3 + 0 ];
        let y = positions[ i * 3 + 1 ];
        let z = positions[ i * 3 + 2 ];
        let p = ndarray_cg::F32x3::new( x, y, z );

        bs.radius = bs.center.distance_squared( &p ).max( bs.radius );
      }

      bs.radius = bs.radius.sqrt();

      bs
    }
  }

  /// Returns size in bytes the model occupies when loaded in memory
  pub fn compute_size_in_memory( model : &tobj::Model ) -> usize
  {
    let mesh = &model.mesh;
    let mut size_in_bytes = 0;

    size_in_bytes += std::mem::size_of_val( &( *mesh.positions ) );
    size_in_bytes += std::mem::size_of_val( &( *mesh.vertex_color ) );
    size_in_bytes += std::mem::size_of_val( &( *mesh.normals ) );
    size_in_bytes += std::mem::size_of_val( &( *mesh.texcoords ) );
    size_in_bytes += std::mem::size_of_val( &( *mesh.indices ) );
    size_in_bytes += std::mem::size_of_val( &( *mesh.face_arities ) );
    size_in_bytes += std::mem::size_of_val( &( *mesh.texcoord_indices ) );
    size_in_bytes += std::mem::size_of_val( &( *mesh.normal_indices ) );

    size_in_bytes
  }

  /// Containes useful information about the model
  #[ derive( Debug, Default ) ]
  pub struct ReportObjModel< 'model, 'mtl >
  {
    pub name : &'model str,
    pub size_in_bytes : usize,
    pub num_vertices : usize,
    pub num_indices : usize,
    pub num_normals : usize,
    pub num_texcoords : usize,
    pub num_vertex_colors : usize,
    pub num_faces : usize,
    pub num_of_arities : HashSet< u32 >,
    pub num_texcoords_indicies : usize,
    pub num_normal_indicies : usize,
    pub bounding_box : BoundingBox,
    pub bounding_sphere : BoundingSphere,
    pub material : Option< &'mtl Material >,
  }

  impl< 'model, 'mtl > ReportObjModel< 'model, 'mtl > {
    pub fn new( model : &'model tobj::Model, materials : &'mtl [ tobj::Material ] ) -> Self
    {
      let mesh = &model.mesh;
      let bounding_box = BoundingBox::compute( &mesh.positions );
      let bounding_sphere = BoundingSphere::compute( &mesh.positions, &bounding_box );
      let num_faces = mesh.face_arities.len();
      let mut num_of_arities = HashSet::new();
      
      // The defualt amount of arities is three, so when the object either containes only triangles,
      // Or "triangulate" option is chosen when loading with tobj crate, then the face_arities array is going
      // to be empty, implying the amount of arities per face equal to 3
      if num_faces == 0 
      { 
        num_of_arities.insert( 3 ); 
      } 
      else 
      { 
        mesh.face_arities.iter().for_each( | &a | { num_of_arities.insert( a ); } );
      };

      let name = &model.name;
      let size_in_bytes = compute_size_in_memory( model );
      let num_vertices = mesh.positions.len() / 3;
      let num_indices = mesh.indices.len();
      let num_normals = mesh.normals.len() / 3;
      let num_texcoords = mesh.texcoords.len() / 2;
      let num_vertex_colors = mesh.vertex_color.len() / 3;
      let num_texcoords_indicies = mesh.texcoord_indices.len();
      let num_normal_indicies = mesh.normal_indices.len();

      let material = 
      match mesh.material_id 
      {
        Some( id ) if id < materials.len() =>
        {
          Some( &materials[ id ] )
        },
        _ => None
      };

      ReportObjModel
      {
        name,
        size_in_bytes,
        num_vertices,
        num_indices,
        material,
        bounding_box,
        num_normals,
        num_texcoords,
        bounding_sphere,
        num_vertex_colors,
        num_faces,
        num_of_arities,
        num_texcoords_indicies,
        num_normal_indicies
      }
    }
  }

  pub fn make_reports< 'model, 'mtl >
  ( 
    models : &'model [ tobj::Model ], 
    materials : &'mtl [ tobj::Material ] 
  ) 
  -> Vec< ReportObjModel< 'model, 'mtl > >
  {   
    let mut reports = Vec::with_capacity( models.len() );
    for i in 0..models.len()
    {
      reports.push
      (
        ReportObjModel::new( &models[ i ], &materials )
      );
    }

    reports
  }

}

crate::mod_interface!
{

  orphan use 
  {
    make_reports,
    ReportObjModel,
    BoundingBox,
    BoundingSphere
  };

}
