#define MAX_OBJECT_COUNT 1024

precision highp float;

in vec3 vPosition;
#ifdef COLOR
  in vec4 vColor;
#endif
#ifdef NORMAL
  in vec3 vNormal;
#endif
#ifdef PBR_INFO
  flat in float vObjectId;
  flat in float vMaterialId;
  in vec2 vTexCoord;
#endif
#ifdef OBJECT_COLOR_ID
  flat in float vObjectColorId;
#endif

#ifdef POSITION 
  layout( location = 0 ) out vec4 FragPosition;
#endif
#ifdef ALBEDO 
  layout( location = 1 ) out vec4 FragAlbedo;
#endif
#ifdef NORMAL 
  layout( location = 2 ) out vec4 FragNormal;
#endif
#ifdef PBR_INFO 
  layout( location = 3 ) out vec4 FragPbrInfo;
#endif
#ifdef OBJECT_COLOR_ID
  layout( location = 4 ) out vec4 FragObjectColorId;
#endif

#ifdef POSITION 
  uniform float near;
  uniform float far;

  float linearizeDepth( float depth )
  {
    return ( 2.0 * near * far ) / ( far + near - ( depth * 2.0 - 1.0 ) * ( far - near ) );
  }
#endif

#ifdef ALBEDO
  #ifdef PBR_INFO
    uniform sampler2D albedoTexture;
  #endif
#endif

void main()
{
  #ifdef POSITION 
    FragPosition = vec4( normalize( vPosition ), linearizeDepth( gl_FragCoord.z ) / 10.0 ); 
  #endif
  #ifdef ALBEDO 
    #if defined( PBR_INFO )
      FragAlbedo = texture( albedoTexture, vTexCoord );
    #elif defined( COLOR )
      FragAlbedo = vColor;
    #else
      FragAlbedo = vec4( 1.0 );
    #endif
  #endif
  #ifdef NORMAL 
    FragNormal = vec4( normalize( vNormal ) * 0.5 + 0.5, 1.0 );
  #endif
  #ifdef PBR_INFO
    FragPbrInfo = vec4( vec2( vObjectId, vMaterialId ), vTexCoord );
  #endif
  #ifdef OBJECT_COLOR_ID
    FragObjectColorId = vec4( vec3( vObjectColorId ), 1.0 );
  #endif
}