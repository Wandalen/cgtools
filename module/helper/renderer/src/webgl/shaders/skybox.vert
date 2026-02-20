#version 300 es
out vec3 vDir;

uniform mat4 invProjection;
uniform mat4 invView;

void main()
{
  float x = float( gl_VertexID / 2 );
  float y = float( gl_VertexID % 2 );

  vec2 uv = vec2( x, y ) * 2.0;

  gl_Position = vec4( x * 4.0 - 1.0, y * 4.0 - 1.0, 1.0, 1.0 );

  vec4 clip = vec4( uv * 2.0 - 1.0, 1.0, 1.0 );
  vec4 view = invProjection * clip;

  vDir = ( invView * view ).xyz;
}
