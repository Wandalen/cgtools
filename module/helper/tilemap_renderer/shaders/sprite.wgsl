struct PushConstant
{
  aspect_scale : vec2< f32 >,
  translation : vec2< f32 >,
  rotation_sin_cos : vec2< f32 >,
  scale : vec2< f32 >,
};

struct VertexOutput
{
  @builtin( position ) position : vec4< f32 >,
  @location( 0 ) texcoord : vec2< f32 >,
};

var< push_constant > pc : PushConstant;

@vertex
fn vs_main( @builtin( vertex_index ) vertex_index : u32 ) -> VertexOutput
{
  let vertices = array< vec2< f32 >, 4 >
  (
    vec2< f32 >(  1.0, -1.0 ),
    vec2< f32 >( -1.0, -1.0 ),
    vec2< f32 >(  1.0,  1.0 ),
    vec2< f32 >( -1.0,  1.0 )
  );

  var out : VertexOutput;
  let position = vertices[ vertex_index ];
  out.texcoord = -( position * 0.5 - 0.5 );

  let rot = mat2x2< f32 >
  (
    vec2< f32 >( pc.rotation_sin_cos.y, pc.rotation_sin_cos.x ),
    vec2< f32 >( -pc.rotation_sin_cos.x, pc.rotation_sin_cos.y )
  );

  let pos = ( rot * ( pc.scale * position ) + pc.translation ) * pc.aspect_scale;
  out.position = vec4< f32 >( pos, 0.0, 1.0 );

  return out;
}

@group( 0 ) @binding( 0 )
var tex : texture_2d< f32 >;
@group( 0 ) @binding( 1 )
var tex_sampler : sampler;

@fragment
fn fs_main( @location( 0 ) v_tex_coord : vec2< f32 > ) -> @location( 0 ) vec4< f32 >
{
  var color = textureSample( tex, tex_sampler, v_tex_coord );
  return color;
}
