mod private
{
  //! USD scene ingestion on the pure-Rust `openusd` stack ( OpenPBR adoption
  //! plan §2.3 N3 ). Two layers, deliberately split like the rest of the
  //! loaders :
  //!
  //! 1. A **pure, off-GPU core** : [`UsdInMemoryResolver`] serves a stage and
  //!    all of its referenced layers from a `path -> bytes` map ( no
  //!    filesystem - the browser path fetches the text over HTTP and hands the
  //!    bytes here ; native tests do the same with inline fixtures ),
  //!    [`usd_stage_open`] composes such a stage, and [`usd_mesh_extract`]
  //!    converts a composed `UsdGeomMesh` into CPU-side vertex/index buffers
  //!    ( fan triangulation, primvar corner resolution for vertex /
  //!    faceVarying / indexed interpolation ).
  //! 2. A thin **GL layer** on top that turns those buffers plus the `Xform`
  //!    stack into `Geometry` / `Node` and binds materials through the
  //!    existing OpenPBR lanes.
  //!
  //! Coordinate space : USD is right-handed Y-up, same as glTF, so no axis
  //! flip is applied ( `metersPerUnit` / `upAxis` deviations are deferred -
  //! see the adoption plan register ).

  use std::collections::HashMap;
  use std::fmt;

  use openusd::ar;
  use openusd::gf;
  use openusd::sdf;
  use openusd::usd::{ self, SchemaBase as _ };
  use openusd_schemas::geom::{ self, Gprim, PointBased, Xformable };

  /// Anything that can go wrong while reading a USD scene.
  #[ derive( Debug ) ]
  pub enum UsdError
  {
    /// Failure from the `openusd` core ( parse, composition, typed reads ).
    Core( openusd::Error ),
    /// Failure from a typed schema view ( `openusd-schemas` ).
    Schema( openusd_schemas::SchemaError ),
    /// Structurally composed but not renderable data ( e.g. a mesh without
    /// `points`, an attribute whose element count contradicts its own indices ).
    Malformed( String ),
  }

  impl fmt::Display for UsdError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        UsdError::Core( e ) => write!( f, "usd core error: {e}" ),
        UsdError::Schema( e ) => write!( f, "usd schema error: {e}" ),
        UsdError::Malformed( m ) => write!( f, "usd malformed data: {m}" ),
      }
    }
  }

  impl std::error::Error for UsdError {}

  impl From< openusd::Error > for UsdError
  {
    fn from( value : openusd::Error ) -> Self { UsdError::Core( value ) }
  }

  impl From< openusd_schemas::SchemaError > for UsdError
  {
    fn from( value : openusd_schemas::SchemaError ) -> Self { UsdError::Schema( value ) }
  }

  /// `path -> bytes` map implementing [`ar::Resolver`] : the stage root and
  /// every asset reference (`@./material.mtlx@`, sublayers, `.usdz` entries)
  /// resolve to entries of the map, served as seekable in-memory assets.
  ///
  /// Relative paths are normalized against the referencing layer's directory
  /// (`@./x@` / `@x@` under the root's folder), which covers the asset layout
  /// of real OpenPBR corpora ( OpenPBRShaderPlayground's `./<name>.mtlx` ).
  #[ derive( Default, Clone ) ]
  pub struct UsdInMemoryResolver
  {
    files : HashMap< String, Vec< u8 > >,
  }

  impl UsdInMemoryResolver
  {
    /// Creates an empty resolver.
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Inserts one asset. Keys are compared after [`normalize_asset_path`].
    pub fn insert( &mut self, path : impl Into< String >, bytes : impl Into< Vec< u8 > > )
    {
      self.files.insert( normalize_asset_path( &path.into() ), bytes.into() );
    }

    /// Looks an asset up by ( normalized ) path.
    #[ must_use ]
    pub fn get( &self, path : &str ) -> Option< &Vec< u8 > >
    {
      self.files.get( &normalize_asset_path( path ) )
    }
  }

  /// Strips `./` segments so `@./gold.mtlx@`, `./gold.mtlx` and `gold.mtlx`
  /// all key to the same entry. Not a general path canonizer - USD asset paths
  /// in practice stay relative and shallow.
  fn normalize_asset_path( path : &str ) -> String
  {
    let mut out = String::with_capacity( path.len() );
    for seg in path.split( '/' )
    {
      if seg == "." || seg.is_empty()
      {
        continue;
      }
      if !out.is_empty()
      {
        out.push( '/' );
      }
      out.push_str( seg );
    }
    out
  }

  /// `ar::Asset` over a byte slice - the in-memory counterpart of a file.
  struct MemoryAsset
  {
    data : std::io::Cursor< Vec< u8 > >,
  }

  impl std::io::Read for MemoryAsset
  {
    fn read( &mut self, buf : &mut [ u8 ] ) -> std::io::Result< usize >
    {
      self.data.read( buf )
    }
  }

  impl std::io::Seek for MemoryAsset
  {
    fn seek( &mut self, pos : std::io::SeekFrom ) -> std::io::Result< u64 >
    {
      std::io::Seek::seek( &mut self.data, pos )
    }
  }

  impl ar::Asset for MemoryAsset
  {
    fn size( &self ) -> std::io::Result< u64 >
    {
      Ok( self.data.get_ref().len() as u64 )
    }
  }

  impl ar::Resolver for UsdInMemoryResolver
  {
    fn resolve( &self, asset_path : &str ) -> Option< ar::ResolvedPath >
    {
      self.get( asset_path ).map( |_| ar::ResolvedPath::new( asset_path ) )
    }

    fn resolve_for_new_asset( &self, asset_path : &str ) -> Option< ar::ResolvedPath >
    {
      Some( ar::ResolvedPath::new( asset_path ) )
    }

    fn create_identifier( &self, asset_path : &str, context : Option< &ar::ResolvedPath > ) -> String
    {
      // Relative references resolve against the context layer's directory;
      // without one the path is its own identifier.
      let Some( ctx ) = context else { return asset_path.to_string() };
      if asset_path.starts_with( '/' )
      {
        return asset_path.to_string();
      }
      let ctx = ctx.to_string_lossy().replace( '\\', "/" );
      let dir = ctx.rsplit_once( '/' ).map_or( "", | ( d, _ ) | d );
      if dir.is_empty() { normalize_asset_path( asset_path ) }
      else { normalize_asset_path( &format!( "{dir}/{asset_path}" ) ) }
    }

    fn open_asset( &self, resolved_path : &ar::ResolvedPath ) -> std::io::Result< Box< dyn ar::Asset > >
    {
      let key = resolved_path.to_string_lossy().to_string();
      match self.get( &key )
      {
        Some( bytes ) => Ok( Box::new( MemoryAsset { data : std::io::Cursor::new( bytes.clone() ) } ) ),
        None => Err( std::io::Error::new( std::io::ErrorKind::NotFound, format!( "in-memory usd asset not found: {key}" ) ) ),
      }
    }
  }

  /// Composes a [`usd::Stage`] whose root layer ( and every reference served
  /// by `resolver` ) comes from memory. `root_path` is the identifier of the
  /// root asset inside the resolver, e.g. `"scene.usda"`.
  ///
  /// # Errors
  ///
  /// [`UsdError::Core`] when the root layer or any composed arc fails to
  /// parse / resolve ( including a missing asset in the resolver ).
  pub fn usd_stage_open( root_path : &str, resolver : UsdInMemoryResolver ) -> Result< usd::Stage, UsdError >
  {
    let stage = usd::Stage::builder().resolver( resolver ).open( root_path )?;
    Ok( stage )
  }

  /// CPU-side triangle soup + index buffer for one mesh prim, the unit the
  /// GL layer uploads. Arrays are per-vertex after corner de-duplication, so
  /// `normals` / `uvs` ( when present ) are index-aligned with `positions`.
  #[ derive( Clone, Debug, PartialEq ) ]
  pub struct UsdMeshData
  {
    /// Vertex positions, 3 floats each.
    pub positions : Vec< [ f32 ; 3 ] >,
    /// Per-vertex normals when the mesh authored any.
    pub normals : Option< Vec< [ f32 ; 3 ] > >,
    /// Per-vertex UVs from `primvars:st` when present.
    pub uvs : Option< Vec< [ f32 ; 2 ] > >,
    /// Per-vertex display color from `primvars:displayColor` when present.
    pub colors : Option< Vec< [ f32 ; 3 ] > >,
    /// Triangle indices into the arrays above.
    pub indices : Vec< u32 >,
  }

  /// How a primvar's values map to face corners ( UsdGeomPrimvar semantics ).
  #[ derive( Clone, Copy, Debug, PartialEq, Eq ) ]
  enum Interpolation
  {
    /// One value per vertex; indexed by `faceVertexIndices`.
    Vertex,
    /// One value per corner ( optionally with an `:indices` indirection ).
    FaceVarying,
    /// One value for the whole prim.
    Uniform,
  }

  impl Interpolation
  {
    fn from_token( token : Option< &str > ) -> Self
    {
      match token
      {
        Some( "vertex" ) => Interpolation::Vertex,
        Some( "uniform" | "constant" ) => Interpolation::Uniform,
        _ => Interpolation::FaceVarying,
      }
    }
  }

  /// Reads `Vec<i32>` from an attribute handle ( `int[]` arrays come back as
  /// `sdf::Value::IntVec` ).
  fn int_vec( attr : &usd::Attribute ) -> Result< Option< Vec< i32 > >, UsdError >
  {
    match attr.get::<sdf::Value>()?
    {
      Some( sdf::Value::IntVec( v ) ) => Ok( Some( v ) ),
      Some( other ) => Err( UsdError::Malformed( format!( "expected int[] attribute, got {other:?}" ) ) ),
      None => Ok( None ),
    }
  }

  fn interpolation_of( attr : &usd::Attribute ) -> Interpolation
  {
    let token = attr.get_metadata::<String>( "interpolation" ).ok().flatten();
    Interpolation::from_token( token.as_deref() )
  }

  /// Resolves one primvar's value index for the `corner`-th corner of the
  /// triangulated output, given its interpolation and optional `:indices`.
  fn corner_value_index( interp : Interpolation, face_vertex_index : i32, corner : usize, indices : Option< &Vec< i32 > > ) -> usize
  {
    match interp
    {
      Interpolation::Vertex => face_vertex_index.max( 0 ) as usize,
      Interpolation::Uniform => 0,
      Interpolation::FaceVarying => match indices
      {
        Some( idx ) => idx.get( corner ).copied().unwrap_or( 0 ).max( 0 ) as usize,
        None => corner,
      },
    }
  }

  /// Key identifying one mesh corner : the ( point, normal, uv, color )
  /// source-value indices it resolves to. Equal keys dedup to one vertex.
  type CornerKey = ( u32, Option< u32 >, Option< u32 >, Option< u32 > );

  /// Converts a composed `UsdGeomMesh` prim into [`UsdMeshData`] : fan
  /// triangulation of `faceVertexCounts`/`faceVertexIndices` over `points`,
  /// with `normals`, `primvars:st` and `primvars:displayColor` resolved per
  /// corner and de-duplicated into a per-vertex layout.
  ///
  /// Subdivision surfaces are NOT tessellated - `subdivisionScheme`
  /// `catmullClark`/`loop` meshes render as their coarse cage until a
  /// tessellator lands ( adoption plan register ).
  ///
  /// # Errors
  ///
  /// [`UsdError::Core`] for failing typed reads, [`UsdError::Malformed`] when
  /// required topology attributes are absent, an `int[]` attribute carries a
  /// non-int value, or `faceVertexIndices` references a missing point.
  pub fn usd_mesh_extract( mesh : &geom::Mesh ) -> Result< UsdMeshData, UsdError >
  {
    let points : Vec< gf::Vec3f > = mesh.points_attr().get::<Vec< gf::Vec3f >>()?
    .ok_or_else( || UsdError::Malformed( format!( "mesh {} has no points", mesh.path().as_str() ) ) )?;
    let counts = int_vec( &mesh.face_vertex_counts_attr() )?
    .ok_or_else( || UsdError::Malformed( format!( "mesh {} has no faceVertexCounts", mesh.path().as_str() ) ) )?;
    let face_indices = int_vec( &mesh.face_vertex_indices_attr() )?
    .ok_or_else( || UsdError::Malformed( format!( "mesh {} has no faceVertexIndices", mesh.path().as_str() ) ) )?;

    let normals_attr = mesh.normals_attr();
    let normals : Option< Vec< gf::Vec3f > > = normals_attr.get::<Vec< gf::Vec3f >>()?;
    let normals_interp = interpolation_of( &normals_attr );
    let normals_indices = int_vec( &mesh.prim().attribute( "normals:indices" ) )?;

    let st_attr = mesh.prim().attribute( "primvars:st" );
    let uvs : Option< Vec< gf::Vec2f > > = st_attr.get::<Vec< gf::Vec2f >>()?;
    let st_interp = interpolation_of( &st_attr );
    let st_indices = int_vec( &mesh.prim().attribute( "primvars:st:indices" ) )?;

    let display_color_attr = mesh.display_color_attr();
    let colors : Option< Vec< gf::Vec3f > > = display_color_attr.get::<Vec< gf::Vec3f >>()?;
    let color_interp = interpolation_of( &display_color_attr );
    let color_indices = int_vec( &mesh.prim().attribute( "primvars:displayColor:indices" ) )?;

    // Topology sanity : every face's corners must have a faceVertexIndices entry.
    let total_corners : usize = counts.iter().map( | c | ( *c ).max( 0 ) as usize ).sum();
    if total_corners > face_indices.len()
    {
      return Err( UsdError::Malformed( format!( "mesh {}: faceVertexCounts sum to {total_corners} corners but faceVertexIndices holds only {}", mesh.path().as_str(), face_indices.len() ) ) );
    }

    // Corner -> unique vertex map keyed by ( point, normal, uv, color ) value
    // indices; identical corners ( the common `vertex`-everything case ) share
    // one output vertex, so the result stays watertight without extra dedup.
    let mut positions : Vec< [ f32 ; 3 ] > = Vec::new();
    let mut out_normals = normals.is_some().then( Vec::new );
    let mut out_uvs = uvs.is_some().then( Vec::new );
    let mut out_colors = colors.is_some().then( Vec::new );
    let mut indices : Vec< u32 > = Vec::new();
    let mut corner_map : HashMap< CornerKey, u32 > = HashMap::new();

    let mut corner : usize = 0; // index into the faceVarying space of the source mesh
    for count in counts
    {
      if count < 3
      {
        // Degenerate face ( point/line ) - contributes no triangles, but its
        // corners still consume faceVertexIndices entries.
        corner += count.max( 0 ) as usize;
        continue;
      }
      let first = corner;
      // Fan triangulation : (0, k, k+1) for k in 1..count-1.
      for k in 1..( count - 1 ) as usize
      {
        for &which in &[ 0usize, 1, 2 ]
        {
          let ( c, fvi ) = match which
          {
            0 => ( first, face_indices[ first ] ),
            1 => ( first + k, face_indices[ first + k ] ),
            _ => ( first + k + 1, face_indices[ first + k + 1 ] ),
          };
          let pi = fvi.max( 0 ) as usize;
          let ni = normals.is_some().then( || corner_value_index( normals_interp, fvi, c, normals_indices.as_ref() ) );
          let ui = uvs.is_some().then( || corner_value_index( st_interp, fvi, c, st_indices.as_ref() ) );
          let ci = colors.is_some().then( || corner_value_index( color_interp, fvi, c, color_indices.as_ref() ) );

          let key : CornerKey =
          (
            pi as u32,
            ni.map( | v | v as u32 ),
            ui.map( | v | v as u32 ),
            ci.map( | v | v as u32 ),
          );
          let vid = if let Some( vid ) = corner_map.get( &key )
          {
            *vid
          }
          else
          {
            let vid = positions.len() as u32;
            let p = points.get( pi ).ok_or_else( || UsdError::Malformed( format!( "faceVertexIndices references point {pi} out of {}", points.len() ) ) )?;
            positions.push( [ p.x, p.y, p.z ] );
            if let Some( out ) = out_normals.as_mut()
            {
              let n = ni.and_then( | i | normals.as_ref().and_then( | s | s.get( i ) ) );
              out.push( n.map_or( [ 0.0, 0.0, 0.0 ], | n | [ n.x, n.y, n.z ] ) );
            }
            if let Some( out ) = out_uvs.as_mut()
            {
              let u = ui.and_then( | i | uvs.as_ref().and_then( | s | s.get( i ) ) );
              out.push( u.map_or( [ 0.0, 0.0 ], | u | [ u.x, u.y ] ) );
            }
            if let Some( out ) = out_colors.as_mut()
            {
              let c = ci.and_then( | i | colors.as_ref().and_then( | s | s.get( i ) ) );
              out.push( c.map_or( [ 1.0, 1.0, 1.0 ], | c | [ c.x, c.y, c.z ] ) );
            }
            corner_map.insert( key, vid );
            vid
          };
          indices.push( vid );
        }
      }
      corner += count.max( 0 ) as usize;
    }

    if positions.is_empty()
    {
      return Err( UsdError::Malformed( format!( "mesh {} triangulated to zero vertices", mesh.path().as_str() ) ) );
    }
    Ok( UsdMeshData { positions, normals : out_normals, uvs : out_uvs, colors : out_colors, indices } )
  }

  /// The local-to-world transform of `path` at time 0 : product of
  /// `local_to_parent_transform` walking up to the stage root. Ancestors that
  /// are not a transform-typed view ( e.g. untyped `def` groups carrying
  /// `xformOp:*` ) contribute identity - author scenes with `def Xform` ( the
  /// OpenPBR corpora all do ).
  ///
  /// # Errors
  ///
  /// [`UsdError::Schema`] / [`UsdError::Core`] when a composed `xformOp` value
  /// is malformed or unreadable.
  pub fn usd_local_to_world( stage : &usd::Stage, path : &sdf::Path ) -> Result< gf::Matrix4d, UsdError >
  {
    let mut chain : Vec< gf::Matrix4d > = Vec::new();
    let mut current = Some( path.clone() );
    while let Some( p ) = current
    {
      if let Some( m ) = maybe_local_to_parent( stage, &p )?
      {
        chain.push( m );
      }
      current = p.parent();
    }
    // chain collected leaf -> root ; world = root * ... * leaf ( row-vector
    // convention : a point row-vector `p * world` maps local -> world ).
    let mut world = gf::Matrix4d::IDENTITY;
    for m in chain.iter().rev()
    {
      world = world * *m;
    }
    Ok( world )
  }

  /// `local_to_parent_transform` for whichever transform-typed view `path`
  /// resolves to ( Xform grouping first, then the leaf geometry types the
  /// renderer ingests ). Untyped / non-xformable prims yield `None`.
  fn maybe_local_to_parent( stage : &usd::Stage, path : &sdf::Path ) -> Result< Option< gf::Matrix4d >, UsdError >
  {
    if let Some( x ) = geom::Xform::get( stage, path.clone() )?
    {
      return Ok( Some( x.local_to_parent_transform( 0.0 )? ) );
    }
    if let Some( m ) = geom::Mesh::get( stage, path.clone() )?
    {
      return Ok( Some( m.local_to_parent_transform( 0.0 )? ) );
    }
    Ok( None )
  }
}

// Layer exports ( via `mod_interface` ).
crate::mod_interface!
{
  own use
  {
    UsdInMemoryResolver,
    UsdError,
    usd_stage_open,
    UsdMeshData,
    usd_mesh_extract,
    usd_local_to_world
  };
}
