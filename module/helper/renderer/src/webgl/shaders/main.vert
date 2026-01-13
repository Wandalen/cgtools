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
  uniform sampler2D inverseBindMatricesTexture;
  uniform sampler2D globalJointTransformMatricesTexture;
  uniform uvec2 skinMatricesTextureSize;
#endif

#ifdef USE_MORPH_TARGET
  // Covers 99% of use cases, most models have <20 targets,
  // but some can have 60 and more
  #define MAX_MORPH_TARGETS 100

  uniform float morphWeights[ MAX_MORPH_TARGETS ];
  uniform uint primitiveOffset;
  uniform sampler2D morphTargetsDisplacementsTexture;
  uniform uvec2 displacementsTextureSize;
  uniform uint morphTargetsCount;
  uniform ivec3 morphTargetsDisplacementsOffsets;
#endif

#ifdef USE_SKINNING
  // Retrieves 4x4 matrices from inverseBindMatricesTexture and
  // globalJointTransformMatricesTexture textures and multipy them.
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
    int x_base = ( i * 4 ) % int( skinMatricesTextureSize.x );
    int y_base = ( i * 4 ) / int( skinMatricesTextureSize.x );

    vec4 gcol0 = texelFetch( globalJointTransformMatricesTexture, ivec2( x_base,     y_base ), 0 );
    vec4 gcol1 = texelFetch( globalJointTransformMatricesTexture, ivec2( x_base + 1, y_base ), 0 );
    vec4 gcol2 = texelFetch( globalJointTransformMatricesTexture, ivec2( x_base + 2, y_base ), 0 );
    vec4 gcol3 = texelFetch( globalJointTransformMatricesTexture, ivec2( x_base + 3, y_base ), 0 );

    vec4 icol0 = texelFetch( inverseBindMatricesTexture, ivec2( x_base,     y_base ), 0 );
    vec4 icol1 = texelFetch( inverseBindMatricesTexture, ivec2( x_base + 1, y_base ), 0 );
    vec4 icol2 = texelFetch( inverseBindMatricesTexture, ivec2( x_base + 2, y_base ), 0 );
    vec4 icol3 = texelFetch( inverseBindMatricesTexture, ivec2( x_base + 3, y_base ), 0 );

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

#ifdef USE_MORPH_TARGET
  uint get_morph_targets_attributes_count()
  {
    uint c = 0u;
    c += uint( morphTargetsDisplacementsOffsets.x != -1 );
    c += uint( morphTargetsDisplacementsOffsets.y != -1 );
    c += uint( morphTargetsDisplacementsOffsets.z != -1 );
    return c;
  }

  uint get_morph_targets_vertex_data_offset()
  {
    uint components = get_morph_targets_attributes_count();
    if ( components == 0u ) return 0u;
    return ( primitiveOffset + uint( gl_VertexID ) ) * morphTargetsCount * components;
  }

  /// Retrieves displacement vector for a specific morph target and vertex attribute.
  ///
  /// @param target Target index (0 to morphTargetsCount-1)
  /// @param offset Attribute offset
  /// @return Displacement vector (xyz) for the target/attribute pair
  ///
  /// Index calculation matches CPU texture packing order:
  /// For each vertex: [T0_POS, T1_POS, ..., T0_NORM, T1_NORM, ..., T0_TAN, T1_TAN, ...]
  vec3 get_target_attribute( uint target, uint offset )
  {
    int i = int( get_morph_targets_vertex_data_offset() + ( offset * morphTargetsCount ) + target );

    int x_base = i % int( displacementsTextureSize.x );
    int y_base = i / int( displacementsTextureSize.x );

    return texelFetch( morphTargetsDisplacementsTexture, ivec2( x_base, y_base ), 0 ).xyz;
  }

  vec3 get_position_displacement( uint target )
  {
    int off = morphTargetsDisplacementsOffsets.x;
    if ( off < 0 ) return vec3( 0.0 );
    return get_target_attribute( target, uint( off ) );
  }

  vec3 get_normal_displacement( uint target )
  {
    int off = morphTargetsDisplacementsOffsets.y;
    if ( off < 0 ) return vec3( 0.0 );
    return get_target_attribute( target, uint( off ) );
  }

  vec3 get_tangent_displacement( uint target )
  {
    int off = morphTargetsDisplacementsOffsets.z;
    if ( off < 0 ) return vec3( 0.0 );
    return get_target_attribute( target, uint( off ) );
  }

  vec3 displace_position( vec3 basePosition )
  {
    if ( morphTargetsDisplacementsOffsets.x == -1 ) return basePosition;

    vec3 pos = basePosition;
    uint cnt = min( morphTargetsCount, uint( MAX_MORPH_TARGETS ) );

    for ( uint i = 0u; i < cnt; ++i )
    {
      float w = morphWeights[ i ];
      pos += w * get_position_displacement( i );
    }

    return pos;
  }

  vec3 displace_normal( vec3 baseNormal )
  {
    if ( morphTargetsDisplacementsOffsets.y == -1 ) return baseNormal;

    vec3 n = baseNormal;
    uint cnt = min( morphTargetsCount, uint( MAX_MORPH_TARGETS ) );

    for ( uint i = 0u; i < cnt; ++i )
    {
      float w = morphWeights[ i ];
      n += w * get_normal_displacement( i );
    }

    return normalize( n );
  }

  vec3 displace_tangent( vec3 baseTangent )
  {
    if ( morphTargetsDisplacementsOffsets.z == -1 ) return baseTangent;

    vec3 t = baseTangent;
    uint cnt = min( morphTargetsCount, uint( MAX_MORPH_TARGETS ) );

    for ( uint i = 0u; i < cnt; ++i )
    {
      float w = morphWeights[i];
      t += w * get_tangent_displacement( i );
    }

    return normalize( t );
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

  vec4 position = vec4( position, 1.0 );

  #ifdef USE_MORPH_TARGET
    position.xyz = displace_position( position.xyz );
    vNormal = normalize( normalMatrix * displace_normal( normal ) );

    #ifdef USE_TANGENTS
      vTangent.xyz = displace_tangent( vTangent.xyz );
    #endif
  #else
    vNormal = normalize( normalMatrix * normal );
  #endif

  #ifdef USE_SKINNING
    position = skin_matrix() * position;
  #endif

  vec4 worldPos = worldMatrix * position;
  vec4 viewPos = viewMatrix * worldPos;

  vViewPos = viewPos.xyz;
  vWorldPos = worldPos.xyz;

  gl_Position = projectionMatrix * viewPos;
}
