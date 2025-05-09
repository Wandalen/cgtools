#version 300 es

out vec2 v_tex_coord;

const vec2 TEXCOORDS[] = vec2[]
(
  vec2( -1.0, -1.0 ),
  vec2( 1.0, -1.0 ),
  vec2( -1.0,  1.0 ),

  vec2( -1.0,  1.0 ),
  vec2( 1.0, -1.0 ),
  vec2( 1.0,  1.0 )
);

void main() 
{
  // Convert quad pos ( -1..1 ) to tex coord ( 0..1 )
  v_tex_coord = TEXCOORDS[ gl_VertexID ] * 0.5 + 0.5; 
  gl_Position = vec4( v_tex_coord, 0.0, 1.0 );
}