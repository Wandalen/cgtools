#ifdef NORMAL
  in vec3 vNormal;
#endif
#ifdef OBJECT_ID
  flat in int vObjectId;
#endif
#ifdef OBJECT_COLOR
  in vec4 vObjectColor;
#endif

#ifdef NORMAL 
  layout( location = 0 ) out vec3 FragNormal;
#endif
#ifdef DEPTH 
  layout( location = 1 ) out float FragDepth;
#endif
#ifdef OBJECT_ID 
  layout( location = 2 ) out int FragObjectId;
#endif
#ifdef OBJECT_COLOR
  layout( location = 3 ) out vec4 FragObjectColor;
#endif

#ifdef DEPTH 
  uniform float near;
  uniform float far;

  float linearizeDepth( float depth )
  {
    return ( 2.0 * near * far ) / ( far + near - ( depth * 2.0 - 1.0 ) * ( far - near ) );
  }
#endif

void main()
{
  #ifdef NORMAL 
    FragNormal = normalize( vNormal ) * 0.5 + 0.5;
  #endif
  #ifdef DEPTH 
    FragDepth = linearizeDepth( gl_FragCoord.z ); 
  #endif
  #ifdef OBJECT_ID 
    FragObjectId = vObjectId;
  #endif
  #ifdef OBJECT_COLOR
    FragObjectColor = vObjectColor;
  #endif
}