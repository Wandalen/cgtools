precision highp float;

in vec3 vPosition;
#ifdef COLOR
  in vec4 vColor;
#endif
#ifdef NORMAL
  in vec3 vNormal;
#endif
#ifdef UV_1
  in vec2 vTexCoord;
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
#ifdef OBJECT_COLOR
  layout( location = 4 ) out vec4 FragObjectColor;
#endif

#ifdef POSITION 
  uniform vec2 near_far;

  float linearizeDepth( float depth )
  {
    float near = near_far.x;
    float far = near_far.y;
    return ( 2.0 * near * far ) / ( far + near - ( depth * 2.0 - 1.0 ) * ( far - near ) );
  }
#endif

#ifdef ALBEDO
  #ifdef PBR_INFO
    uniform sampler2D albedoTexture;
  #endif
#endif

#ifdef PBR_INFO
  uniform uint objectId;
  uniform uint materialId;
#endif

#ifdef OBJECT_COLOR
  uniform vec4 objectColor;
#endif

void main()
{
  #ifdef POSITION 
    FragPosition = vec4( normalize( vPosition ), linearizeDepth( gl_FragCoord.z ) / 2000.0 ); 
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
    FragPbrInfo = vec4( vec2( float( objectId ), float( materialId ) ), vTexCoord );
  #endif
  #ifdef OBJECT_COLOR
    FragObjectColor = objectColor;
  #endif
}