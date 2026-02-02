@group( 0 ) @binding( 0 ) var< uniform > scale : f32;
var< push_constant > color : vec3< f32 >;

@vertex
fn vs_main
(
  @location( 0 ) position : vec2< f32 >,
  @location( 1 ) translation : vec2< f32 >
) -> @builtin( position ) vec4< f32 >
{
  let pos = ( position + translation ) * scale;
  return vec4< f32 >( pos, 0.0, 1.0 );
}

@fragment
fn fs_main() -> @location( 0 ) vec4< f32 >
{
  return vec4< f32 >( color.rgb, 1.0 );
}
