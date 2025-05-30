layout( location = 0 ) in vec3 position;
#ifdef NORMAL
  layout( location = 1 ) in vec3 normal;
#endif
#ifdef OBJECT_ID
  layout( location = 2 ) in int objectId;
#endif
#ifdef OBJECT_COLOR
  layout( location = 3 ) in vec3 objectColor;
#endif

uniform mat4x4 worldMatrix;
uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;

#ifdef NORMAL
  out vec3 vNormal;
#endif
#ifdef OBJECT_ID
  flat out int vObjectId;
#endif
#ifdef OBJECT_COLOR
  out vec4 vObjectColor;
#endif

void main()
{
  gl_Position = projectionMatrix * viewMatrix * worldMatrix * vec4( position, 1.0 );
  #ifdef NORMAL
    vNormal = normalize( mat3x3( worldMatrix ) * normal );
  #endif
  #ifdef OBJECT_ID
    vObjectId = objectId;
  #endif
  #ifdef OBJECT_COLOR
    vObjectColor = vec4( objectColor, 1.0 );
  #endif
}