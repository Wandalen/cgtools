layout( location = 0 ) in vec3 position;
#ifdef ALBEDO
  layout( location = 1 ) in vec4 albedo;
#endif
#ifdef NORMAL
  layout( location = 2 ) in vec3 normal;
#endif
#ifdef PBR_INFO
  layout( location = 3 ) in vec2 texCoord;
  layout( location = 4 ) in uint objectId;
  layout( location = 5 ) in uint materialId;  
#endif
#ifdef OBJECT_COLOR_ID
  layout( location = 6 ) in int objectColorId;
#endif

uniform mat4x4 worldMatrix;
uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;

#ifdef ALBEDO
  out vec4 vAlbedo;
#endif
#ifdef NORMAL
  out vec3 vNormal;
#endif
#ifdef PBR_INFO
  out vec2 vTexCoord;
  flat out uint vObjectId;
  flat out uint vMaterialId;  
#endif
#ifdef OBJECT_COLOR_ID
  flat out int vObjectColorId;
#endif

void main()
{
  gl_Position = projectionMatrix * viewMatrix * worldMatrix * vec4( position, 1.0 );
  #ifdef ALBEDO
    vAlbedo = albedo;
  #endif
  #ifdef NORMAL
    vNormal = normalize( mat3x3( worldMatrix ) * normal );
  #endif
  #ifdef PBR_INFO
    vTexCoord = texCoord;
    vObjectId = objectId;  
    vMaterialId = materialId;  
  #endif
  #ifdef OBJECT_COLOR_ID
    vObjectColorId = objectColorId;
  #endif
}