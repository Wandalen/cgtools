layout ( location = 0 ) in vec3 position;
#ifdef COLOR
  layout ( location = 1 ) in vec4 color;
#endif
#ifdef NORMAL
  layout ( location = 2 ) in vec3 normal;
#endif
#ifdef PBR_INFO
  layout ( location = 3 ) in vec2 texCoord;  
#endif

uniform mat4x4 worldMatrix;
uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;
uniform mat3x3 normalMatrix;

out vec3 vPosition;
#ifdef COLOR
  out vec4 vColor;
#endif
#ifdef NORMAL
  out vec3 vNormal;
#endif
#ifdef PBR_INFO
  out vec2 vTexCoord; 
#endif

void main()
{
  vPosition = position;
  gl_Position = projectionMatrix * viewMatrix * worldMatrix * vec4( position, 1.0 ); 
  #ifdef COLOR
    vColor = color;
  #endif
  #ifdef NORMAL
    vNormal = normalize( normalMatrix * normal );
  #endif
  #ifdef PBR_INFO
    vTexCoord = texCoord; 
  #endif
}