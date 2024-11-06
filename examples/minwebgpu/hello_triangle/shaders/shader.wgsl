
@vertex
fn vs_main( @builtin( vertex_index ) id : u32 ) -> @builtin( position ) vec4f
{
  var positions = array< vec3f, 3 >
  (
    vec3f( -0.5, -0.5, 0.0 ),
    vec3f( 0.0, 0.5, 0.0 ),
    vec3f( 0.5, -0.5, 0.0 ),
  );

  return vec4f( positions[ id ], 1.0 );
}

@fragment
fn fs_main() -> @location( 0 ) vec4f
{
  let color = vec3f( 1.0, 0.0, 0.0);
  return vec4f( color, 1.0 );
}