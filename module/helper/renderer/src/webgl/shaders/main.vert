layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec3 normal;
layout( location = 2 ) in vec2 uv_0;
layout( location = 3 ) in vec2 uv_1;
layout( location = 4 ) in vec2 uv_2;
layout( location = 5 ) in vec2 uv_3;
layout( location = 6 ) in vec2 uv_4;
layout( location = 7 ) in vec4 color_0;
layout( location = 8 ) in vec4 color_1;
#ifdef USE_TANGENTS 
  layout( location = 9 ) in vec4 tangent;
#endif
layout( location = 10 ) in float object_id;

uniform mat4x4 worldMatrix;
uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;

out vec3 vWorldPos;
out vec3 vNormal;
out float vObjectId;
out vec2 vUv_0;
out vec2 vUv_1;
out vec2 vUv_2;
out vec2 vUv_3;
out vec2 vUv_4;
out vec4 vColor_0;
out vec4 vColor_1;
#ifdef USE_TANGENTS
  out vec4 vTangent;
#endif

void main()
{
  vUv_0 = uv_0;
  vUv_1 = uv_1;
  vUv_2 = uv_2;
  vUv_3 = uv_3;
  vUv_4 = uv_4;
  vColor_0 = color_0;
  vColor_1 = color_1;
  #ifdef USE_TANGENTS
    vTangent = tangent;
  #endif
  vNormal = normalize( mat3x3( worldMatrix ) * normal );
  vObjectId = object_id;

  vec4 worldPos = worldMatrix * vec4( position, 1.0 );
  vec4 viewPos = viewMatrix * worldPos;

  vWorldPos = worldPos.xyz;

  gl_Position = projectionMatrix * viewPos;
}
