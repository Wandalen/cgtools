#version 300 es
precision mediump float;

in vec3 v_normal;
in vec3 v_worldpos;
in vec2 v_texcoord;

uniform sampler2D u_diffuse_tex;

layout( location = 0 ) out vec4 frag_color;

void main()
{
  // shader for rendering an object with a hardcoded lighting
  // and diffuse color from texture
  const float AMBIENT = 0.4;
  const vec3 DIRECTIONAL_LIGHT = vec3( -1.0, -1.0, -1.0 );
  const vec3 POINT_LIGHT_POSITION = vec3( 0.0, 0.0, 0.0 );
  const vec3 LIGHT_COLOR = vec3( 1.0, 1.0, 1.0 );

  vec3 normal = normalize( v_normal );

  float directional = max( dot( normal, normalize( -DIRECTIONAL_LIGHT ) ), 0.0 );

  vec3 offset = POINT_LIGHT_POSITION - v_worldpos;
  vec3 direction = normalize( offset );
  float point = max( dot( normal, direction ), 0.0 );

  vec4 light = vec4( LIGHT_COLOR * ( directional + point + AMBIENT ), 1.0 );

  frag_color = texture( u_diffuse_tex, v_texcoord ) * light;
}
