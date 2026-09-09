//! OpenPBR test scenes built procedurally — no external assets.
//!
//! Step 1: a correct, well-lit sphere ( geometry verified ).
//! Step 2: apply a real `.mtlx` OpenPBR surface to that sphere via the
//! `OpenPbrSurface → PbrMaterial` runtime bridge.

use std::{ cell::RefCell, rc::Rc };

use minwebgl as gl;
use primitive_generation::{ AttributesData, PrimitiveData, Transform, primitives_data_to_gltf };
use renderer::webgl::material::{ OpenPbrSurface, PbrMaterial };
use renderer::webgl::loaders::gltf::GLTF;

/// A unit icosphere subdivided `4` times ( 2562 vertices / 5120 faces ),
/// scaled to `radius`, with radial normals ( a sphere's normal is its own
/// position ).
fn icosphere_attributes( radius : f32 ) -> AttributesData
{
  let ( raw_positions, indices ) = primitive_generation::solid::icosphere_subdivided( 4 );

  let positions : Vec< [ f32; 3 ] > = raw_positions.iter()
  .map( | p | [ p[ 0 ] * radius, p[ 1 ] * radius, p[ 2 ] * radius ] )
  .collect();

  let normals : Vec< [ f32; 3 ] > = raw_positions.iter()
  .map( | p |
  {
    let len = ( p[ 0 ] * p[ 0 ] + p[ 1 ] * p[ 1 ] + p[ 2 ] * p[ 2 ] ).sqrt();
    if len > 1e-6 { [ p[ 0 ] / len, p[ 1 ] / len, p[ 2 ] / len ] } else { [ 0.0, 1.0, 0.0 ] }
  } )
  .collect();

  AttributesData { positions, indices, normals }
}

/// Builds a renderer `GLTF` scene containing a single icosphere whose material
/// is driven by a parsed OpenPBR surface ( from the `.mtlx` lane ) through
/// `PbrMaterial::openpbr_surface_apply` — the `USE_OPENPBR` shading path.
#[ must_use ]
pub fn sphere_with_surface( gl : &gl::WebGl2RenderingContext, surface : &OpenPbrSurface ) -> GLTF
{
  let attributes = Rc::new( RefCell::new( icosphere_attributes( 0.5 ) ) );

  let primitive = PrimitiveData
  {
    name : Some( Box::from( "openpbr_sphere" ) ),
    parent : None,
    attributes : Some( attributes ),
    color : gl::F32x4::from( [ 1.0, 1.0, 1.0, 1.0 ] ),
    transform : Transform::default(),
  };

  let gltf = primitives_data_to_gltf( gl, &[ primitive ] );

  // Swap the generated fallback material for one configured from the surface.
  let material = gltf.materials.first().cloned().expect( "sphere material exists" );
  let mut configured = PbrMaterial::new( gl );
  configured.openpbr_surface_apply( surface );
  // The icosphere is a closed, outward-wound surface: cull its back faces so
  // back-facing fragments don't z-fight / sparkle at the silhouette contour.
  configured.cull_mode = Some( renderer::webgl::material::CullMode::Back );
  configured.double_sided = false;
  *material.borrow_mut() = Box::new( configured );

  gltf
}

/// Axis-aligned unit cube, 24 vertices ( 4 per face, face-aligned normals so the
/// corners do NOT need smoothing ), authored as 6 genuine quad faces ( 24
/// corner indices ) - the USD loader fan-triangulates them, exercising the same
/// quad path real USD assets use.
#[ must_use ]
fn cube_attributes( half : f32 ) -> ( Vec< [ f32 ; 3 ] >, Vec< [ f32 ; 3 ] >, Vec< u32 > )
{
  let faces : [ ( [ f32 ; 3 ], [ [ f32 ; 3 ] ; 4 ] ) ; 6 ] =
  [
    ( [ 0.0, 0.0, 1.0 ], [ [ -half, -half, half ], [ half, -half, half ], [ half, half, half ], [ -half, half, half ] ] ),
    ( [ 0.0, 0.0, -1.0 ], [ [ half, -half, -half ], [ -half, -half, -half ], [ -half, half, -half ], [ half, half, -half ] ] ),
    ( [ 1.0, 0.0, 0.0 ], [ [ half, -half, half ], [ half, -half, -half ], [ half, half, -half ], [ half, half, half ] ] ),
    ( [ -1.0, 0.0, 0.0 ], [ [ -half, -half, -half ], [ -half, -half, half ], [ -half, half, half ], [ -half, half, -half ] ] ),
    ( [ 0.0, 1.0, 0.0 ], [ [ -half, half, half ], [ half, half, half ], [ half, half, -half ], [ -half, half, -half ] ] ),
    ( [ 0.0, -1.0, 0.0 ], [ [ -half, -half, -half ], [ half, -half, -half ], [ half, -half, half ], [ -half, -half, half ] ] ),
  ];

  let mut positions = Vec::with_capacity( 24 );
  let mut normals = Vec::with_capacity( 24 );
  let mut indices = Vec::with_capacity( 24 );
  for ( f, ( n, corners ) ) in faces.iter().enumerate()
  {
    for ( c, corner ) in corners.iter().enumerate()
    {
      positions.push( *corner );
      normals.push( *n );
      indices.push( ( f * 4 + c ) as u32 );
    }
  }
  ( positions, normals, indices )
}

/// Writes `points` / `normals` ( vertex-interpolated ) / `faceVertexCounts`
/// ( from `counts`, so quads stay quads ) + `faceVertexIndices` lines for one
/// `def Mesh` body, assuming 8-space indentation ( inside an object `Xform` ).
fn push_mesh_body( s : &mut String, name : &str, positions : &[ [ f32 ; 3 ] ], normals : &[ [ f32 ; 3 ] ], indices : &[ u32 ], counts : &[ u32 ], binding : &str )
{
  use std::fmt::Write as _;

  // faceVertexCounts must describe `indices` exactly, otherwise the loader's
  // fan triangulation spans face boundaries ( the "half a quad lands on the
  // opposite face" bug ).
  assert_eq!( counts.iter().sum::< u32 >() as usize, indices.len(), "mesh counts/indices mismatch" );

  let mut row = String::new();
  let _ = writeln!( s, "        def Mesh \"{name}\"" );
  s.push_str( "        (\n            apiSchemas = [ \"MaterialBindingAPI\" ]\n        )\n        {\n" );
  let _ = writeln!( s, "            rel material:binding = <{binding}>" );

  let _ = write!( s, "            uniform int[] faceVertexCounts = [ " );
  for ( i, c ) in counts.iter().enumerate()
  {
    let _ = write!( row, "{c}{0}", if i + 1 == counts.len() { "" } else { ", " } );
    if row.len() > 72 { s.push_str( &row ); row.clear(); }
  }
  s.push_str( &row ); row.clear();
  s.push_str( " ]\n" );

  let _ = write!( s, "            uniform int[] faceVertexIndices = [ " );
  for ( i, idx ) in indices.iter().enumerate()
  {
    let _ = write!( row, "{idx}{0}", if i + 1 == indices.len() { "" } else { ", " } );
    if row.len() > 72 { s.push_str( &row ); row.clear(); }
  }
  s.push_str( &row ); row.clear();
  s.push_str( " ]\n" );

  let _ = write!( s, "            uniform point3f[] points = [ " );
  for ( i, p ) in positions.iter().enumerate()
  {
    let _ = write!( row, "( {0}, {1}, {2} ){3}", p[ 0 ], p[ 1 ], p[ 2 ], if i + 1 == positions.len() { "" } else { ", " } );
    if row.len() > 72 { s.push_str( &row ); row.clear(); }
  }
  s.push_str( &row ); row.clear();
  s.push_str( " ]\n" );

  let _ = write!( s, "            uniform normal3f[] normals = [ " );
  for ( i, n ) in normals.iter().enumerate()
  {
    let _ = write!( row, "( {0}, {1}, {2} ){3}", n[ 0 ], n[ 1 ], n[ 2 ], if i + 1 == normals.len() { "" } else { ", " } );
    if row.len() > 72 { s.push_str( &row ); row.clear(); }
  }
  s.push_str( &row );
  s.push_str( " ]\n            (\n                interpolation = \"vertex\"\n            )\n" );
  s.push_str( "            uniform token subdivisionScheme = \"none\"\n        }\n" );
}

/// One object of the "playground set" test scene : which mesh, which material
/// ( provider key, e.g. `./gold.mtlx`, or `preview` for the inline
/// `UsdPreviewSurface` ), and its `xformOp` triple.
pub struct UsdSetObject< 'a >
{
  /// `"sphere"` or `"cube"`.
  pub mesh : &'a str,
  /// Material target : `./name.mtlx` or `"preview"`.
  pub material : &'a str,
  /// `xformOp:translate`.
  pub translate : [ f32 ; 3 ],
  /// `xformOp:rotateXYZ` in degrees.
  pub rotate_deg : [ f32 ; 3 ],
  /// Uniform `xformOp:scale`.
  pub scale : f32,
}

/// Builds a multi-object scene exercising the parts the single-sphere scene
/// cannot : several meshes ( two primitive types, quad faces + triangles ), a
/// parent `Xform` per object with translate/rotate/scale ops ( non-identity
/// transforms + hierarchy composition ), five `.mtlx`-bound materials covering
/// distinct carriers ( metal / coat / high IOR / painted / fuzz ) AND one
/// inline `UsdPreviewSurface` - both material lanes in one scene.
#[ must_use ]
pub fn usd_set_scene_text( objects : &[ UsdSetObject< '_ > ] ) -> String
{
  use std::fmt::Write as _;

  let mut s = String::from( "#usda 1.0\n(\n    defaultPrim = \"World\"\n    upAxis = \"Y\"\n    metersPerUnit = 1.0\n)\n\ndef Xform \"World\"\n{\n" );

  // Materials.
  s.push_str( "    def Scope \"Looks\"\n    {\n" );
  for ( i, o ) in objects.iter().enumerate()
  {
    if o.material == "preview"
    {
      let _ = writeln!( s, "        def Material \"M{i}\"" );
      s.push_str( "        {\n            token outputs:surface.connect = </World/Looks/" );
      let _ = write!( s, "M{i}/Surf.outputs:surface>\n\n" );
      s.push_str( "            def Shader \"Surf\"\n            {\n                uniform token info:id = \"UsdPreviewSurface\"\n" );
      s.push_str( "                color3f inputs:diffuseColor = ( 0.1, 0.6, 0.2 )\n                float inputs:roughness = 0.15\n                token outputs:surface\n            }\n        }\n" );
    }
    else
    {
      let _ = writeln!( s, "        def Material \"M{i}\" ( prepend references = @{}@ )\n        {{\n        }}", o.material );
    }
  }
  s.push_str( "    }\n\n" );

  // Objects : one Xform each ( nesting proves hierarchy composition ).
  let sphere = icosphere_attributes( 0.5 );
  let sphere_counts = vec![ 3u32 ; sphere.indices.len() / 3 ];
  let ( cube_p, cube_n, cube_i ) = cube_attributes( 0.5 );
  for ( i, o ) in objects.iter().enumerate()
  {
    // `xformOp:*` are ordinary attributes : they belong in the prim BODY `{ }`,
    // not the parenthesized metadata block ( the USDA parser rejects assignments
    // there - "want: Punctuation('='), got NamespacedIdentifier" ).
    let _ = writeln!( s, "    def Xform \"Obj{i}\"" );
    s.push_str( "    {\n        double3 xformOp:translate = (" );
    let _ = write!( s, " {0}, {1}, {2} ", o.translate[ 0 ], o.translate[ 1 ], o.translate[ 2 ] );
    s.push_str( ")\n        float3 xformOp:rotateXYZ = (" );
    let _ = write!( s, " {0}, {1}, {2} ", o.rotate_deg[ 0 ], o.rotate_deg[ 1 ], o.rotate_deg[ 2 ] );
    s.push_str( ")\n        uniform float3 xformOp:scale = (" );
    let _ = write!( s, " {0}, {0}, {0} ", o.scale );
    s.push_str( ")\n        uniform token[] xformOpOrder = [ \"xformOp:translate\", \"xformOp:rotateXYZ\", \"xformOp:scale\" ]\n" );
    let binding = format!( "/World/Looks/M{i}" );
    match o.mesh
    {
      "cube" => push_mesh_body( &mut s, "Cube", &cube_p, &cube_n, &cube_i, &[ 4 ; 6 ], &binding ),
      _ => push_mesh_body( &mut s, "Sphere", &sphere.positions, &sphere.normals, &sphere.indices, &sphere_counts, &binding ),
    }
    s.push_str( "    }\n\n" );
  }
  s.push_str( "}\n" );
  s
}
