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
#ifdef USE_SKINNING
  #ifdef USE_JOINTS_0
    layout( location = 10 ) in vec4 joints_0;
  #endif
  #ifdef USE_JOINTS_1
    layout( location = 11 ) in vec4 joints_1;
  #endif
  #ifdef USE_JOINTS_2
    layout( location = 12 ) in vec4 joints_2;
  #endif
  #ifdef USE_WEIGHTS_0
    layout( location = 13 ) in vec4 weights_0;
  #endif
  #ifdef USE_WEIGHTS_1
    layout( location = 14 ) in vec4 weights_1;
  #endif
  #ifdef USE_WEIGHTS_2
    layout( location = 15 ) in vec4 weights_2;
  #endif
#endif

uniform mat4x4 worldMatrix;
uniform mat3x3 normalMatrix;
uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;

out vec3 vWorldPos;
out vec3 vViewPos;
out vec3 vNormal;
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

#ifdef USE_SKINNING
  uniform sampler2D inverseBindMatrices;
  uniform sampler2D globalJointTransformMatrices;
  uniform uvec2 matricesTextureSize;

  // Retrieves 4x4 matrices from inverseBindMatrices and
  // globalJointTransformMatrices textures and multipy them.
  //
  // The textures are assumed to store matrices as a sequence of pixels,
  // where each matrix is represented by four consecutive pixels (columns).
  //
  // @param i The index of the matrix to retrieve.
  // @return The 4x4 matrices multiplication result
  //         at the specified index.
  //
  // Joint matrix calculation source:
  //  - https://www.khronos.org/files/gltf20-reference-guide.pdf(Page 6, Computing the joint matrices)
  //  - https://chatgpt.com/share/68c40aca-ac08-8013-bdba-be12e3498bba
  mat4 joint_matrix( int i )
  {
    int x_base = ( i * 4 ) % int( matricesTextureSize.x );
    int y_base = ( i * 4 ) / int( matricesTextureSize.x );

    vec4 gcol0 = texelFetch( globalJointTransformMatrices, ivec2( x_base,     y_base ), 0 );
    vec4 gcol1 = texelFetch( globalJointTransformMatrices, ivec2( x_base + 1, y_base ), 0 );
    vec4 gcol2 = texelFetch( globalJointTransformMatrices, ivec2( x_base + 2, y_base ), 0 );
    vec4 gcol3 = texelFetch( globalJointTransformMatrices, ivec2( x_base + 3, y_base ), 0 );

    vec4 icol0 = texelFetch( inverseBindMatrices, ivec2( x_base,     y_base ), 0 );
    vec4 icol1 = texelFetch( inverseBindMatrices, ivec2( x_base + 1, y_base ), 0 );
    vec4 icol2 = texelFetch( inverseBindMatrices, ivec2( x_base + 2, y_base ), 0 );
    vec4 icol3 = texelFetch( inverseBindMatrices, ivec2( x_base + 3, y_base ), 0 );

    return mat4( gcol0, gcol1, gcol2, gcol3 ) * mat4( icol0, icol1, icol2, icol3 );
  }

  // Calculates skin matrix from one vertex attribute pair ( joints_{i}, weights_{i} )
  mat4 one_attribute_skin_matrix( vec4 joints, vec4 weights )
  {
    mat4 skinMatrix = weights.x * joint_matrix( int( joints.x ) ) +
    weights.y * joint_matrix( int( joints.y ) ) +
    weights.z * joint_matrix( int( joints.z ) ) +
    weights.w * joint_matrix( int( joints.w ) );

    return skinMatrix;
  }

  // Full skin matrix calculation
  mat4 skin_matrix()
  {
    mat4 skinMatrix = mat4( 0.0 );

    #if defined( USE_JOINTS_0 ) && defined( USE_WEIGHTS_0 )
      skinMatrix += one_attribute_skin_matrix( joints_0, weights_0 );
    #endif

    #if defined( USE_JOINTS_1 ) && defined( USE_WEIGHTS_1 )
      skinMatrix += one_attribute_skin_matrix( joints_1, weights_1 );
    #endif

    #if defined( USE_JOINTS_2 ) && defined( USE_WEIGHTS_2 )
      skinMatrix += one_attribute_skin_matrix( joints_2, weights_2 );
    #endif

    if ( skinMatrix[ 0 ][ 0 ] == 0.0 )
    {
      skinMatrix = mat4( 1.0 );
    }

    return skinMatrix;
  }
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
  vNormal = normalize( normalMatrix * normal );
  //vNormal = vec3( -1.0, -1.0)
  //vNormal *= -1.0;
  //vNormal = normalize( mat3x3( worldMatrix ) * normal );
  //vNormal = normal;

  vec4 position = vec4( position, 1.0 );

  #ifdef USE_SKINNING
    position = skin_matrix() * position;
  #endif

  vec4 worldPos = worldMatrix * position;
  vec4 viewPos = viewMatrix * worldPos;

  vViewPos = viewPos.xyz;
  vWorldPos = worldPos.xyz;

  gl_Position = projectionMatrix * viewPos;
}
