struct Uniform
{
  color : vec4< f32 >,
  translation : vec2< f32 >,
  scale : f32,
};

@group( 0 ) @binding( 0 ) var< uniform > uniform_data : Uniform;

@vertex
fn vs_main( @location( 0 ) position: vec2< f32 > ) -> @builtin( position ) vec4<f32>
{
  let pos = ( position + uniform_data.translation ) * uniform_data.scale;
  return vec4< f32 >( pos, 0.0, 1.0 );
}

@fragment
fn fs_main() -> @location( 0 ) vec4< f32 >
{
  return uniform_data.color;
}
