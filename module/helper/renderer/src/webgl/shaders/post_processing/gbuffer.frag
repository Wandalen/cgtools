#define MAX_OBJECT_COUNT 1024

#ifdef ALBEDO
  in vec4 vAlbedo;
#endif
#ifdef NORMAL
  in vec3 vNormal;
#endif
#ifdef PBR_INFO
  flat in uint vObjectId;
  flat in uint vMaterialId;
  in vec2 vTexCoord;
#endif
#ifdef OBJECT_COLOR_ID
  flat in int vObjectColorId;
#endif

#ifdef POSITION 
  layout( location = 0 ) out vec3 FragPosition;
#endif
#ifdef ALBEDO 
  layout( location = 1 ) out vec4 FragAlbedo;
#endif
#ifdef NORMAL 
  layout( location = 2 ) out vec3 FragNormal;
#endif
#ifdef PBR_INFO 
  layout( location = 3 ) out vec4 FragPbrInfo;
#endif
#ifdef OBJECT_COLOR
  layout( location = 4 ) out vec4 FragObjectColor;
#endif

layout( std140 ) uniform ObjectColorBlock
{
  vec4 objectColors[ MAX_OBJECT_COUNT ];
};

#ifdef POSITION 
  uniform float near;
  uniform float far;

  float linearizeDepth( float depth )
  {
    return ( 2.0 * near * far ) / ( far + near - ( depth * 2.0 - 1.0 ) * ( far - near ) );
  }
#endif

void main()
{
  #ifdef POSITION 
    FragPosition = vec3( gl_FragCoord.xy, linearizeDepth( gl_FragCoord.z ) ); 
  #endif
  #ifdef ALBEDO 
    FragAlbedo = vAlbedo;
  #endif
  #ifdef NORMAL 
    FragNormal = normalize( vNormal ) * 0.5 + 0.5;
  #endif
  #ifdef PBR_INFO
    FragPbrInfo = vec4( vec2( float( vObjectId ), float( vMaterialId ) ), vTexCoord );
  #endif
  #ifdef OBJECT_COLOR
    FragObjectColor = vObjectColor;
  #endif
}