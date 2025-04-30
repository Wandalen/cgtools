layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec3 normal;
layout( location = 2 ) in vec2 uv_0;
layout( location = 3 ) in vec2 uv_1;
layout( location = 4 ) in vec2 uv_2;
layout( location = 5 ) in vec2 uv_3;
layout( location = 6 ) in vec2 uv_4;
layout( location = 7 ) in vec4 color_0;
layout( location = 8 ) in vec4 color_1;

uniform mat4x4 worldMatrix;
uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;

out vec3 vWorldPos;
out vec3 vNormal;
out vec2 vUv_0;
out vec2 vUv_1;
out vec2 vUv_2;
out vec2 vUv_3;
out vec2 vUv_4;
out vec4 vColor_0;
out vec4 vColor_1;

const vec3 poss[] = vec3[]
(
  vec3( 0.5, 0.0, 0.0 ),
  vec3( 0.5, 0.5, 0.0 ),
  vec3( 0.0, 0.5, 0.0 )
);


const vec2 uvss[] = vec2[]
(
  vec2( 0.0, 0.0 ),
  vec2( 1.0, 1.0 ),
  vec2( 0.0, 1.0 )
);

void main()
{
  vNormal = normal;
  vUv_0 = uv_0;
  vUv_1 = uv_1;
  vUv_2 = uv_2;
  vUv_3 = uv_3;
  vUv_4 = uv_4;
  vColor_0 = color_0;
  vColor_1 = color_1;

  vec4 worldPos = worldMatrix * vec4( position, 1.0 );
  vec4 viewPos = viewMatrix * worldPos;

  vWorldPos = worldPos.xyz;

  gl_Position = projectionMatrix * viewPos;

  //gl_Position = vec4( poss[ gl_VertexID % 3 ], 1.0 );
}
