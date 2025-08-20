struct Color
{
  r : f32,
  g : f32,
  b : f32
};

struct PushConstant
{
  aspect_scale : vec2< f32 >,
  translation : vec2< f32 >,
  rotation_sin_cos : vec2< f32 >,
  scale : vec2< f32 >,
  color : Color,
};

var< push_constant > pc : PushConstant;

@vertex
fn vs_main( @location( 0 ) position : vec2< f32 > ) -> @builtin( position ) vec4< f32 >
{
  let rot = mat2x2< f32 >
  (
    vec2< f32 >( pc.rotation_sin_cos.y, pc.rotation_sin_cos.x ),
    vec2< f32 >( -pc.rotation_sin_cos.x, pc.rotation_sin_cos.y )
  );

  let pos = ( rot * ( pc.scale * position ) + pc.translation ) * pc.aspect_scale;

  return vec4< f32 >( pos, 0.0, 1.0 );
}

@fragment
fn fs_main() -> @location( 0 ) vec4< f32 >
{
  return vec4< f32 >( pc.color.r, pc.color.g, pc.color.b, 1.0 );
}
