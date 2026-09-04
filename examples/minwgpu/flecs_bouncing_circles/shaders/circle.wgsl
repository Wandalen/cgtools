struct VertexOutput
{
  @builtin( position ) clip_position : vec4< f32 >,
  @location( 0 ) local_pos : vec2< f32 >,
  @location( 1 ) color : vec3< f32 >,
}

@vertex
fn vs_main
(
  @location( 0 ) corner : vec2< f32 >,
  @location( 1 ) center : vec2< f32 >,
  @location( 2 ) radius : f32,
  @location( 3 ) color : vec3< f32 >,
) -> VertexOutput
{
  var out : VertexOutput;
  out.clip_position = vec4< f32 >( center + corner * radius, 0.0, 1.0 );
  out.local_pos = corner;
  out.color = color;
  return out;
}

@fragment
fn fs_main( in : VertexOutput ) -> @location( 0 ) vec4< f32 >
{
  let dist = length( in.local_pos );
  let alpha = 1.0 - smoothstep( 0.9, 1.0, dist );
  if ( alpha <= 0.0 )
  {
    discard;
  }
  return vec4< f32 >( in.color, alpha );
}
