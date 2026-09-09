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
  use openusd_schemas::geom::{ self, Gprim, Imageable, PointBased, Xformable };
  use openusd_schemas::shade::{ self, MaterialBindingAPI };
  use std::cell::RefCell;
  use std::rc::Rc;
  use minwebgl as gl;
  use crate::webgl::{ AttributeInfo, Geometry, IndexInfo, Material, Mesh, Node, Object3D, Primitive, Scene, material::{ OpenPbrSurface, PbrMaterial } };
  use crate::webgl::loaders::openpbr_mtlx::openpbr_surfaces_from_mtlx;

  /// Shared, ref-counted material handle ( same shape as the glTF loader uses ).
  type SharedMaterial = Rc< RefCell< Box< dyn Material > > >;

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

  impl From< sdf::PathParseError > for UsdError
  {
    fn from( value : sdf::PathParseError ) -> Self { UsdError::Malformed( value.to_string() ) }
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
    /// Maps the `interpolation` metadatum; unauthored falls back to `vertex`,
    /// matching `UsdGeomPrimvar::GetInterpolation`.
    fn from_token( token : Option< &str > ) -> Self
    {
      match token
      {
        Some( "faceVarying" ) => Interpolation::FaceVarying,
        Some( "uniform" | "constant" ) => Interpolation::Uniform,
        _ => Interpolation::Vertex,
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
    // The `interpolation` metadatum is a `token` value; decoding it as `String`
    // fails with a type-cast error ( silently, once `.ok()` flattens it ), which
    // would silently downgrade every `vertex` primvar to the faceVarying default
    // - zero-filling most normals of a shared-vertex mesh ( the dark-sphere bug ).
    let token = attr.get_metadata::<openusd::tf::Token>( "interpolation" ).ok().flatten();
    Interpolation::from_token( token.as_ref().map( openusd::tf::Token::as_str ) )
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

  // ---------------------------------------------------------------------------
  // Scene analysis ( pure, off-GPU )
  // ---------------------------------------------------------------------------

  /// Runtime material factors resolved from a bound `UsdPreviewSurface` ( or a
  /// full OpenPBR `Surface`, when the bound material references an external
  /// `.mtlx` / embeds a MaterialX nodegraph ), in the shape the GL layer
  /// stamps onto a `PbrMaterial`. Preview-surface texture channels are ignored
  /// for now ( kept at their scalar / default value ) - wiring USD textures to
  /// the `TextureInfo` slots is a follow-up ( adoption plan register ).
  #[ derive( Clone, Debug, PartialEq ) ]
  pub struct UsdMaterialData
  {
    /// Linear base color + opacity ( diffuseColor RGB, opacity A ).
    pub base_color_rgba : [ f32 ; 4 ],
    /// Metallic factor ( UsdPreviewSurface `metallic` ), default 0.
    pub metallic : Option< f32 >,
    /// Roughness factor ( `roughness` ), default 0.5 ( USD's own default ).
    pub roughness : Option< f32 >,
    /// Emissive color ( `emissiveColor` ).
    pub emissive : Option< [ f32 ; 3 ] >,
    /// Index of refraction ( `ior` ), default 1.5.
    pub ior : Option< f32 >,
    /// Full OpenPBR parameter surface ( the native `.mtlx` / nodegraph lane ).
    /// Takes precedence over the preview-surface factors when present.
    pub surface : Option< OpenPbrSurface >,
  }

  impl Default for UsdMaterialData
  {
    fn default() -> Self
    {
      Self
      {
        base_color_rgba : [ 0.8, 0.8, 0.8, 1.0 ],
        metallic : None,
        roughness : None,
        emissive : None,
        ior : None,
        surface : None,
      }
    }
  }

  /// A USD geometry/group node, resolved to everything the GL assembly layer
  /// needs without touching `openusd` types or the GPU.
  #[ derive( Clone, Debug, PartialEq ) ]
  pub struct UsdPrimData
  {
    /// Composed prim path, e.g. `/World/Model/Mesh`.
    pub path : String,
    /// Parent path ( `None` for scene roots / the pseudo-root's direct children ).
    pub parent : Option< String >,
    /// Prim name ( last path segment ).
    pub name : String,
    /// Local-to-parent transform as a column-major `f32` matrix, ready for
    /// `gl::F32x4x4::from_column_major` ( see [`gf_matrix_to_column_major`] ).
    pub local_to_parent : [ f32 ; 16 ],
    /// `Some` for a `Mesh` prim ( its triangulated data ); `None` for
    /// grouping prims ( `Xform` / `Scope` / untyped ).
    pub mesh : Option< UsdMeshData >,
    /// The bound material's identity path, when a `Mesh` has one.
    pub material_path : Option< String >,
    /// Resolved material factors, when a `Mesh` had a readable bound material.
    pub material : Option< UsdMaterialData >,
    /// USD `doubleSided` on the mesh gprim.
    pub double_sided : bool,
  }

  /// Converts a `gf` row-vector, row-major matrix into the renderer's
  /// column-major `f32` layout. gf stores `p' = p · M` ( translation in the
  /// last row, elements 12..14 ); the renderer stores `p' = M · p` ( column
  /// vectors ). The two are transposes, and a row-vector matrix's raw
  /// elements are already the correct column-major layout for the equivalent
  /// column-vector transform, so the conversion is a plain f64→f32 cast with
  /// no reordering.
  #[ must_use ]
  pub fn gf_matrix_to_column_major( m : &gf::Matrix4d ) -> [ f32 ; 16 ]
  {
    let mut out = [ 0.0f32 ; 16 ];
    for ( dst, src ) in out.iter_mut().zip( m.0.iter() )
    {
      *dst = *src as f32;
    }
    out
  }

  /// Maps a `UsdPreviewSurface` ( as read by `openusd_schemas::shade` ) onto
  /// [`UsdMaterialData`]. Pure; texture-connected channels fall back to the
  /// channel's authored value if any, else the `UsdPreviewSurface` default.
  #[ must_use ]
  pub fn usd_preview_surface_to_material( ps : &shade::ReadPreviewSurface ) -> UsdMaterialData
  {
    let mut data = UsdMaterialData::default();
    if let Some( c ) = ps.diffuse_color.value()
    {
      data.base_color_rgba[ 0 ] = c.x;
      data.base_color_rgba[ 1 ] = c.y;
      data.base_color_rgba[ 2 ] = c.z;
    }
    if let Some( o ) = ps.opacity.value()
    {
      data.base_color_rgba[ 3 ] = *o;
    }
    data.metallic = ps.metallic.value().copied();
    data.roughness = ps.roughness.value().copied();
    data.emissive = ps.emissive_color.value().map( | c | [ c.x, c.y, c.z ] );
    data.ior = ps.ior.value().copied();
    data
  }

  /// A source of USD-referenced text assets ( the `.mtlx` files a `Material`
  /// prim points at via `references = @./x.mtlx@` ), addressed by the authored
  /// asset-path token. [`UsdInMemoryResolver`] implements it; the browser lane
  /// supplies HTTP-fetched bytes through the same resolver.
  pub trait UsdAssetProvider
  {
    /// Returns the UTF-8 text of `path` ( as authored inside a `@…@` token ),
    /// if this source knows it.
    fn asset_text( &self, path : &str ) -> Option< String >;
  }

  impl UsdAssetProvider for UsdInMemoryResolver
  {
    fn asset_text( &self, path : &str ) -> Option< String >
    {
      self.get( path ).and_then( | bytes | std::str::from_utf8( bytes ).ok() ).map( ToString::to_string )
    }
  }

  /// `true` if `type_name` marks a shading prim ( a `Material` or a node-graph
  /// building block ) that must not become a renderable scene node.
  fn is_shading_type( type_name : Option< &str > ) -> bool
  {
    matches!( type_name, Some( "Material" | "Shader" | "NodeGraph" | "Look" ) )
  }

  /// The first `.mtlx`-suffixed `references` asset path authored on `path`
  /// ( any list-op flavour ), as authored. This is the OpenPBR content pattern :
  /// `def Material "x" ( prepend references = @./x.mtlx@</…> )`.
  fn mtlx_reference( stage : &usd::Stage, path : &sdf::Path ) -> Result< Option< String >, UsdError >
  {
    let prim = stage.prim( path.clone() )?;
    let Some( sdf::Value::ReferenceListOp( op ) ) = prim.get_metadata::<sdf::Value>( "references" )? else
    {
      return Ok( None );
    };
    let found = op.explicit_items.iter()
    .chain( op.prepended_items.iter() )
    .chain( op.appended_items.iter() )
    .chain( op.added_items.iter() )
    .map( | r | &r.asset_path )
    .find( | a | a.rsplit_once( '.' ).is_some_and( | ( _, ext ) | ext.eq_ignore_ascii_case( "mtlx" ) ) )
    .cloned();
    Ok( found )
  }

  /// Parses a referenced `.mtlx` into the richest [`UsdMaterialData`] the
  /// renderer can consume : the full [`OpenPbrSurface`] ( preferred by the GL
  /// layer via `openpbr_surface_apply` ) plus its glTF-shaped reduction.
  /// Selection is the first `open_pbr_surface` in document order ( named
  /// `mtlx_target` selection is a known register gap ).
  #[ must_use ]
  pub fn usd_material_from_mtlx( xml : &str ) -> Option< UsdMaterialData >
  {
    let surfaces = openpbr_surfaces_from_mtlx( xml ).ok()?;
    let surface = surfaces.into_iter().next()?;
    let runtime = crate::webgl::material::openpbr_to_runtime( &surface );
    Some( UsdMaterialData
    {
      base_color_rgba : runtime.base_color_rgba,
      metallic : Some( runtime.base_metalness ),
      roughness : Some( runtime.specular_roughness ),
      emissive : None,
      ior : runtime.specular_ior,
      surface : Some( surface ),
    } )
  }

  /// Resolves the material bound to `path` ( or inherited from an ancestor ),
  /// returning its path and factors. A `UsdPreviewSurface` reads directly;
  /// otherwise a referenced `.mtlx` is fetched from `assets` ( when supplied )
  /// and parsed through the N2 reader.
  fn resolve_bound_material( stage : &usd::Stage, path : &sdf::Path, assets : Option< &dyn UsdAssetProvider > ) -> Result< Option< ( String, UsdMaterialData ) >, UsdError >
  {
    let mut current = Some( path.clone() );
    while let Some( p ) = current
    {
      if let Some( api ) = MaterialBindingAPI::get( stage, p.clone() )?
      {
        if let Some( mat_path ) = api.compute_bound_material( "" )?
        {
          let data = if let Some( ps ) = shade::read_preview_surface( stage, &mat_path )?
          {
            usd_preview_surface_to_material( &ps )
          }
          else
          {
            // Not a UsdPreviewSurface : the native OpenPBR lane ( external
            // `.mtlx` reference or embedded MaterialX nodegraph ).
            let mut data = UsdMaterialData::default();
            if let ( Some( assets ), Some( asset ) ) = ( assets, mtlx_reference( stage, &mat_path )? )
            {
              if let Some( xml ) = assets.asset_text( &asset )
              {
                data = usd_material_from_mtlx( &xml ).unwrap_or_default();
              }
            }
            data
          };
          return Ok( Some( ( mat_path.to_string(), data ) ) );
        }
      }
      current = p.parent();
    }
    Ok( None )
  }

  /// Reads a mesh's bound material and `doubleSided` flag.
  fn mesh_material_and_sidedness( stage : &usd::Stage, mesh : &geom::Mesh, assets : Option< &dyn UsdAssetProvider > ) -> Result< ( Option< String >, Option< UsdMaterialData >, bool ), UsdError >
  {
    let double_sided = mesh.double_sided_attr().get::<bool>()?.unwrap_or( false );
    match resolve_bound_material( stage, mesh.path(), assets )?
    {
      Some( ( path, data ) ) => Ok( ( Some( path ), Some( data ), double_sided ) ),
      None => Ok( ( None, None, double_sided ) ),
    }
  }

  /// Analyzes a composed USD stage into a flat, renderable prim list : every
  /// geometry/group prim except the shading subtree, with its local transform,
  /// triangulated mesh data ( when a `Mesh` ), resolved material, and sidedness.
  ///
  /// Invisible prims ( `compute_visibility` == `Invisible` ) are dropped along
  /// with nothing referencing them; prims whose purpose is not `Default` /
  /// `Render` ( e.g. `Proxy` / `Guide` ) are dropped too. Cameras / lights are
  /// out of this slice's scope ( the renderer builds those from glTF / its own
  /// light rig ).
  ///
  /// # Errors
  ///
  /// Propagates [`UsdError`] from the underlying composed reads.
  pub fn usd_scene_analyze( stage : &usd::Stage, assets : Option< &dyn UsdAssetProvider > ) -> Result< Vec< UsdPrimData >, UsdError >
  {
    let mut paths : Vec< sdf::Path > = Vec::new();
    stage.traverse( usd::PrimPredicate::ALL, | p | paths.push( p.clone() ) )?;

    // Shading prim set : a prim is skipped if it or any ancestor is one.
    let mut shading : std::collections::HashSet< String > = std::collections::HashSet::new();
    for path in &paths
    {
      let prim = stage.prim( path.clone() )?;
      let type_name = prim.type_name()?;
      if is_shading_type( type_name.as_ref().map( openusd::tf::Token::as_str ) )
      {
        shading.insert( path.to_string() );
      }
    }

    let under_shading = | path : &sdf::Path | -> bool
    {
      let s = path.to_string();
      if shading.contains( &s )
      {
        return true;
      }
      // any ancestor marked as shading
      let mut cur = path.parent();
      while let Some( a ) = cur
      {
        if shading.contains( &a.to_string() )
        {
          return true;
        }
        cur = a.parent();
      }
      false
    };

    let mut out : Vec< UsdPrimData > = Vec::new();
    for path in &paths
    {
      if path == &sdf::Path::abs_root()
      {
        continue;
      }
      if under_shading( path )
      {
        continue;
      }

      let prim = stage.prim( path.clone() )?;
      let name = path.name().map( ToString::to_string ).unwrap_or_default();
      let parent = path.parent().filter( | p | p != &sdf::Path::abs_root() ).map( | p | p.to_string() );

      let local_to_parent = match maybe_local_to_parent( stage, path )?
      {
        Some( m ) => gf_matrix_to_column_major( &m ),
        None => gf_matrix_to_column_major( &gf::Matrix4d::IDENTITY ),
      };

      // Mesh leaf, or a grouping prim.
      if let Some( mesh ) = geom::Mesh::get( stage, path.clone() )?
      {
        // Visibility / purpose gate only meaningful on Imageable geometry.
        let vis = mesh.compute_visibility()?;
        let purpose = mesh.compute_purpose()?;
        let render = vis != geom::Visibility::Invisible
        && matches!( purpose, geom::Purpose::Default | geom::Purpose::Render );
        if !render || !prim.is_active()?
        {
          continue;
        }
        let data = usd_mesh_extract( &mesh )?;
        let ( material_path, material, double_sided ) = mesh_material_and_sidedness( stage, &mesh, assets )?;
        out.push( UsdPrimData { path : path.to_string(), parent, name, local_to_parent, mesh : Some( data ), material_path, material, double_sided } );
      }
      else
      {
        // Grouping prim ( Xform / Scope / untyped ) : carried so child meshes
        // keep their transform chain. Non-imageable / inactive groups skipped.
        if !prim.is_active()?
        {
          continue;
        }
        out.push( UsdPrimData { path : path.to_string(), parent, name, local_to_parent, mesh : None, material_path : None, material : None, double_sided : false } );
      }
    }

    Ok( out )
  }

  // ---------------------------------------------------------------------------
  // GL assembly ( thin )
  // ---------------------------------------------------------------------------

  /// Flattens per-vertex `[ f32 ; N ]` rows into a single `f32` buffer.
  #[ must_use ]
  fn flatten< const N : usize >( data : &[ [ f32 ; N ] ] ) -> Vec< f32 >
  {
    data.iter().flatten().copied().collect()
  }

  /// Uploads one tightly-packed `f32` attribute buffer for `location = slot`.
  /// `bb` carries the attribute's bounding box ( meaningful for `positions`;
  /// pass `BoundedBox::default()` elsewhere ).
  fn cpu_attribute( gl : &gl::WebGl2RenderingContext, flat : &[ f32 ], dims : i32, slot : u32, bb : gl::geometry::BoundingBox ) -> Result< AttributeInfo, gl::WebglError >
  {
    let buffer = gl::buffer::create( gl )?;
    gl::buffer::upload( gl, &buffer, flat, gl::STATIC_DRAW );
    let descriptor = gl::BufferDescriptor::new::< f32 >()
    .vector( gl::VectorDataType::new( gl::DataType::F32, dims, 1 ) );
    Ok
    (
      AttributeInfo
      {
        slot,
        buffer,
        descriptor,
        bounding_box : bb,
      }
    )
  }

  /// Axis-aligned bounds of a positions array ( mesh-local space ).
  #[ must_use ]
  fn positions_bounding_box( positions : &[ [ f32 ; 3 ] ] ) -> gl::geometry::BoundingBox
  {
    let mut bb = gl::geometry::BoundingBox::default();
    for p in positions
    {
      let v = gl::F32x3::from( *p );
      bb.min = bb.min.min( v );
      bb.max = bb.max.max( v );
    }
    bb
  }

  /// Builds a [`Geometry`] from CPU [`UsdMeshData`] ( positions always; normals
  /// and uv when authored ), ready for a `Primitive`.
  ///
  /// # Errors
  ///
  /// Propagates GL errors from buffer / VAO creation.
  pub fn usd_geometry_create( gl : &gl::WebGl2RenderingContext, data : &UsdMeshData ) -> Result< Geometry, gl::WebglError >
  {
    let mut geometry = Geometry::new( gl )?;
    geometry.draw_mode = gl::TRIANGLES;
    geometry.vertex_count = data.positions.len() as u32;

    let bb = positions_bounding_box( &data.positions );
    geometry.attribute_add( gl, "positions", cpu_attribute( gl, &flatten( &data.positions ), 3, 0, bb )? )?;
    if let Some( normals ) = &data.normals
    {
      geometry.attribute_add( gl, "normals", cpu_attribute( gl, &flatten( normals ), 3, 1, gl::geometry::BoundingBox::default() )? )?;
    }
    if let Some( uvs ) = &data.uvs
    {
      geometry.attribute_add( gl, "texture_coordinates_2", cpu_attribute( gl, &flatten( uvs ), 2, 2, gl::geometry::BoundingBox::default() )? )?;
    }

    let index_buffer = gl::buffer::create( gl )?;
    gl::index::upload( gl, &index_buffer, &data.indices, gl::STATIC_DRAW );
    geometry.index_add( gl, IndexInfo
    {
      buffer : index_buffer,
      count : data.indices.len() as u32,
      offset : 0,
      data_type : gl::UNSIGNED_INT,
    })?;

    Ok( geometry )
  }

  /// Stamps [`UsdMaterialData`] onto a fresh `PbrMaterial`. When the data
  /// carries a full [`OpenPbrSurface`] ( the native `.mtlx` / MaterialX lane )
  /// it is applied through `openpbr_surface_apply`, which also sets the
  /// specular / clearcoat / fuzz / thin-film carriers. Otherwise the
  /// `UsdPreviewSurface` factors are stamped directly ( base color / opacity ->
  /// alpha mode, metallic, roughness, emissive, IOR ). Pure aside from the
  /// `PbrMaterial::new` `&GL` handle it borrows.
  #[ must_use ]
  pub fn usd_material_apply( gl : &gl::WebGl2RenderingContext, data : &UsdMaterialData, double_sided : bool ) -> PbrMaterial
  {
    let mut m = PbrMaterial::new( gl );
    m.double_sided = double_sided;
    if let Some( surface ) = &data.surface
    {
      // Richest lane : full OpenPBR parameter surface. Opacity still rides the
      // alpha-mode toggle below ( surface carries no geometry_opacity here ).
      m.openpbr_surface_apply( surface );
    }
    else
    {
      m.base_color_factor = gl::F32x4::from( data.base_color_rgba );
      m.metallic_factor = data.metallic.unwrap_or( 0.0 );
      m.roughness_factor = data.roughness.unwrap_or( 0.5 );
      if let Some( e ) = data.emissive
      {
        m.emissive_factor = gl::F32x3::from( e );
      }
      if let Some( ior ) = data.ior.filter( | v | ( *v - 1.5 ).abs() > f32::EPSILON )
      {
        m.openpbr_params_set( crate::webgl::material::OpenPbrParams { ior : Some( ior ), ..Default::default() } );
      }
    }
    // Sub-1 opacity means a blended transparent pass ( not opaque ).
    if data.base_color_rgba[ 3 ] < 1.0
    {
      m.alpha_mode_set( crate::webgl::AlphaMode::Blend );
    }
    m
  }

  /// The directory prefix of an asset path ( everything up to and including the
  /// last `/` ), used to resolve `@./sibling@` references relative to the
  /// referencing file. Empty for a bare filename.
  #[ must_use ]
  fn asset_dir( path : &str ) -> &str
  {
    match path.rfind( '/' )
    {
      Some( i ) => &path[ ..=i ],
      None => "",
    }
  }

  /// Browser feed : fetches a `.usda` scene ( and the external `.mtlx` assets it
  /// references, discovered via the N3-lite `usda_mtlx_references` scanner ) over
  /// HTTP with `gl::file::load`, then assembles a renderer [`Scene`] through
  /// [`usd_scene_from_texts`] - the same pipeline the native tests exercise.
  ///
  /// `.usdc` / `.usdz` roots are fetched too ( `openusd` sniffs the format ), but
  /// their references are only discovered for `.usda` text ( the scanner is a
  /// text tool ); a binary root therefore loads its own layer but not sibling
  /// `.mtlx` files until their references are readable - a known N3 gap.
  ///
  /// # Errors
  ///
  /// [`UsdError::Malformed`] on a fetch failure, UTF-8 error, or GL/analysis
  /// error ( the underlying cause is `Debug`-formatted into the message ).
  pub async fn usd_scene_load_http( gl : &gl::WebGl2RenderingContext, root_path : &str ) -> Result< Rc< RefCell< Scene > >, UsdError >
  {
    let fetch = | path : String | async move
    {
      gl::file::load( &path ).await
      .map_err( | e | UsdError::Malformed( format!( "usd fetch '{path}': {e:?}" ) ) )
      .map( | bytes | ( path, bytes ) )
    };

    // 1. root scene.
    let ( root_key, root_bytes ) = fetch( root_path.to_string() ).await?;
    let root_text = std::str::from_utf8( &root_bytes ).map_err( | e | UsdError::Malformed( format!( "usd root not utf-8: {e}" ) ) )?.to_string();

    // 2. referenced `.mtlx` assets, resolved relative to the scene's folder.
    //    Kept in a separate store from the stage's resolver : `.mtlx` is not a
    //    USD layer, so handing it to the stage resolver would make composition
    //    try ( and fail ) to parse it as USDA - analyze reads it as text instead.
    let dir = asset_dir( &root_key );
    let mut assets : Vec< ( String, String ) > = Vec::new();
    let is_text_usda = root_key.rsplit_once( '.' ).is_some_and( | ( _, ext ) | ext.eq_ignore_ascii_case( "usda" ) );
    if is_text_usda
    {
      for reference in crate::webgl::loaders::openpbr_usda::usda_mtlx_references( &root_text )
      {
        // `mtlx_asset` is the authored `@…@` token, e.g. `./iceCube.mtlx`.
        let joined = if reference.mtlx_asset.starts_with( '/' )
        {
          reference.mtlx_asset.clone()
        }
        else
        {
          format!( "{dir}{}", normalize_asset_path( &reference.mtlx_asset ) )
        };
        let ( _, bytes ) = fetch( joined ).await?;
        let text = String::from_utf8( bytes ).map_err( | e | UsdError::Malformed( format!( "usd asset not utf-8: {e}" ) ) )?;
        assets.push( ( reference.mtlx_asset.clone(), text ) );
      }
    }

    let asset_refs : Vec< ( &str, &str ) > = assets.iter().map( | ( p, t ) | ( p.as_str(), t.as_str() ) ).collect();
    usd_scene_from_texts( gl, &root_key, &root_text, &asset_refs )
  }

  /// Assembles a [`Scene`] from in-memory texts : the `.usda` root plus any
  /// assets ( `.mtlx` etc. ) that bound materials reference. The root goes to
  /// the stage's own resolver; the `assets` form a separate [`UsdAssetProvider`]
  /// store for the `.mtlx` lane ( see [`usd_scene_load_http`] - same reason :
  /// MaterialX is not a USD layer and must never reach composition ). No
  /// filesystem, no network : works in the browser with embedded strings and
  /// natively in tests.
  ///
  /// # Errors
  ///
  /// [`UsdError`] from stage composition / analysis, or a GL error ( as
  /// `Malformed` ) from geometry upload.
  pub fn usd_scene_from_texts( gl : &gl::WebGl2RenderingContext, root_path : &str, root_text : &str, assets : &[ ( &str, &str ) ] ) -> Result< Rc< RefCell< Scene > >, UsdError >
  {
    let mut stage_store = UsdInMemoryResolver::new();
    stage_store.insert( root_path, root_text.as_bytes() );

    let mut provider = UsdInMemoryResolver::new();
    for ( path, text ) in assets
    {
      provider.insert( *path, text.as_bytes() );
    }

    let stage = usd_stage_open( root_path, stage_store )?;
    usd_scene_load( gl, &stage, Some( &provider ) )
  }

  /// Analyzes `stage` and assembles a renderer [`Scene`] : a `Node` per renderable
  /// prim, `Mesh` leaves with uploaded geometry + materials, transforms wired via
  /// each prim's authored local matrix ( the renderer composes world matrices ).
  ///
  /// This is the browser-visible half of the USD lane : it issues GL calls, so it
  /// is verified in `gltf_viewer` / a wasm run, not by the native suite.
  ///
  /// # Errors
  ///
  /// [`UsdError`] from analysis, or a GL error from geometry upload.
  pub fn usd_scene_load( gl : &gl::WebGl2RenderingContext, stage : &usd::Stage, assets : Option< &dyn UsdAssetProvider > ) -> Result< Rc< RefCell< Scene > >, UsdError >
  {
    let prims = usd_scene_analyze( stage, assets )?;

    let mut nodes_by_path : HashMap< String, usize > = HashMap::new();
    let mut nodes : Vec< Rc< RefCell< Node > > > = Vec::with_capacity( prims.len() );
    let mut materials : HashMap< String, SharedMaterial > = HashMap::new();

    for ( i, prim ) in prims.iter().enumerate()
    {
      nodes_by_path.insert( prim.path.clone(), i );

      let mut node = Node::default();
      node.name_set( prim.name.clone() );
      node.local_matrix_set( gl::F32x4x4::from_column_major( prim.local_to_parent ) );

      if let Some( mesh_data ) = &prim.mesh
      {
        let geometry = usd_geometry_create( gl, mesh_data ).map_err( | e | UsdError::Malformed( format!( "usd geometry upload: {e:?}" ) ) )?;
        // Materials sharing one USD binding path share one PbrMaterial instance
        // ( same shader program, one upload ); unbound meshes get a private default.
        let material = if let Some( ref key ) = prim.material_path
        {
          if let Some( shared ) = materials.get( key ).cloned()
          {
            shared
          }
          else
          {
            let default = UsdMaterialData::default();
            let data = prim.material.as_ref().unwrap_or( &default );
            let pbr = usd_material_apply( gl, data, prim.double_sided );
            let shared : SharedMaterial = Rc::new( RefCell::new( Box::new( pbr ) as Box< dyn Material > ) );
            materials.insert( key.clone(), shared.clone() );
            shared
          }
        }
        else
        {
          let pbr = usd_material_apply( gl, &UsdMaterialData::default(), prim.double_sided );
          Rc::new( RefCell::new( Box::new( pbr ) as Box< dyn Material > ) )
        };
        let primitive = Primitive
        {
          geometry : Rc::new( RefCell::new( geometry ) ),
          material,
        };
        let mut mesh = Mesh::default();
        mesh.primitive_add( Rc::new( RefCell::new( primitive ) ) );
        node.object = Object3D::Mesh( Rc::new( RefCell::new( mesh ) ) );
      }

      nodes.push( Rc::new( RefCell::new( node ) ) );
    }

    // Wire hierarchy : child -> parent via the authored path strings.
    let mut scene = Scene::default();
    for ( i, prim ) in prims.iter().enumerate()
    {
      let node = &nodes[ i ];
      let parent_node = prim.parent.as_ref().and_then( | p | nodes_by_path.get( p ) ).map( | j | &nodes[ *j ] );
      if let Some( parent ) = parent_node
      {
        parent.borrow_mut().child_add( node.clone() );
        node.borrow_mut().parent_set( Some( parent.clone() ) );
      }
      else
      {
        // Root ( no parent, or parent filtered out of the scene ).
        scene.add( node.clone() );
      }
    }
    scene.world_matrix_update();

    Ok( Rc::new( RefCell::new( scene ) ) )
  }
}

// Layer exports ( via `mod_interface` ).
crate::mod_interface!
{
  own use
  {
    UsdInMemoryResolver,
    UsdAssetProvider,
    UsdError,
    usd_stage_open,
    UsdMeshData,
    usd_mesh_extract,
    usd_local_to_world,
    UsdPrimData,
    UsdMaterialData,
    usd_scene_analyze,
    gf_matrix_to_column_major,
    usd_preview_surface_to_material,
    usd_material_from_mtlx,
    usd_geometry_create,
    usd_material_apply,
    usd_scene_load,
    usd_scene_load_http,
    usd_scene_from_texts
  };
}
