@group( 0 ) @binding( 0 ) var color_accum_texture : texture_2d< f32 >;


@vertex
fn vs_main( @builtin( vertex_index ) id : u32 ) -> @builtin( position ) vec4f
{
  var positions = array< vec2f, 4 >
  (
    vec2f( -1.0, -1.0 ),
    vec2f( 1.0, -1.0 ),
    vec2f( -1.0, 1.0 ),
    vec2f( 1.0, 1.0 )
  );
  return vec4f( positions[ id ], 0.0, 1.0 );
}

@fragment
fn fs_main( @builtin( position ) coords : vec4f ) -> @location( 0 ) vec4f
{
  let uv = vec2< u32 >( floor( coords.xy ) );
  let color = textureLoad( color_accum_texture, uv, 0 );

  return vec4f( color.xyz, 1.0 );
}