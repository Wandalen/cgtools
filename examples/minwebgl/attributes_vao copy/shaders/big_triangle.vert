#version 300 es

out vec2 vUv;
out vec3 worldDir;

uniform mat4x4 invProjectionMatrix;
uniform mat4x4 viewMatrix;

void main()
{
  int x = gl_VertexID / 2;
  int y = gl_VertexID % 2;

  vUv = vec2( x, y ) * 2.0;

  mat3x3 invViewMatrix = transpose( mat3x3( viewMatrix ) );
  vec4 clipPos = vec4( vec2( float( x ) * 4.0 - 1.0, 1.0 - float( y ) * 4.0 ), 0.0, 1.0 );

  worldDir = invViewMatrix * ( invProjectionMatrix * clipPos ).xyz;

  gl_Position = clipPos;
}