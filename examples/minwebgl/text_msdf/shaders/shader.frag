#version 300 es 
precision mediump float;
#pragma vscode_glsllint_stage : frag

const float PI = 3.1415926;

uniform vec2 u_resolution;
uniform float u_metallic;
uniform float u_roughness;
uniform vec3 u_base_color;
uniform float u_reflectance;
uniform float u_time;

in vec2 vUv;
out vec4 frag_color;

float circleSDF( vec2 pos, float r ) 
{
  return length( pos ) - r;
}

// Schlick ver.
vec3 freshel( vec3 viewDir, vec3 halfway ) 
{
  vec3 f0 = vec3( 0.16 * u_reflectance * u_reflectance );
  f0 = mix( f0, u_base_color, u_metallic );

  return f0 + ( 1.0 - f0 ) * pow( ( 1.0 - dot( viewDir, halfway ) ), 5.0 );
}

// Normal distribution function
float NDF( vec3 normal, vec3 halfway ) 
{
  float alpha = u_roughness * u_roughness;
  float alpha2 = alpha * alpha;
  float denom = PI * pow( pow( dot( normal, halfway ), 2.0 ) * ( alpha2 - 1.0 ) + 1.0, 2.0 );

  return alpha2 / denom;
}

float germ_schlick_ggx( vec3 normal, vec3 v ) 
{
  float alpha = u_roughness * u_roughness;
  float k = alpha / 2.0;
  float NoV = dot( normal, v );
  float denom = NoV * ( 1.0 - k ) + k;

  return max( NoV, 0.001 ) / denom;
}

// Geometry term, Smith ver.
float Germ( vec3 lightDir, vec3 viewDir, vec3 normal ) 
{
  return germ_schlick_ggx( normal, lightDir ) * germ_schlick_ggx( normal, viewDir );
}

vec3 BRDF( vec3 lightDir, vec3 viewDir, vec3 normal ) 
{
  vec3 halfway = normalize( lightDir + viewDir );

  vec3 F = freshel( viewDir, halfway );
  float D = NDF( normal, halfway );
  float G = Germ( lightDir, viewDir, normal );

  float denom = 4.0 * max( dot( normal, lightDir ) * dot( normal, viewDir ), 0.001 );

  vec3 specular = F * D * G / denom;

  vec3 diffuse = u_base_color;
  diffuse *= vec3( 1.0 ) - F; // Amount of transmitted light
  diffuse *= 1.0 - u_metallic;// Metals do not have diffuse light
  diffuse /= PI;
  
  return diffuse + specular;
}

mat2x2 rot( float angle ) 
{
  float s = sin( angle );
  float c = cos( angle );
  return mat2x2( c, s, -s, c );
}

void main()
{
  //Translate Uv coordinates to pixel coordinates centered at the center of the scareen
  vec2 pixelCoords = ( vUv - 0.5 ) * u_resolution;

  float circleRadius = 350.0;
  float circle = circleSDF( pixelCoords, circleRadius );

  vec2 xy = pixelCoords;

  // This is needed to smoothout the edges of the sphere
  if( circle > 0.0 ) 
  {
    xy = circleRadius * normalize( xy );
  }

  float r = length( xy );
  float z = sqrt( max( circleRadius * circleRadius - r * r, 0.0 ) ); 

  vec3 position = vec3( xy, z );
  vec3 viewDir = normalize( vec3( 0.0, 0.0, 1.0 ) );
  vec3 lightDir[ 3 ] = vec3[]
  ( 
    normalize( vec3( 10.0, 1.0, 1.0 ) ),
    normalize( vec3( 1.0, 0.0, 0.0 ) ),
    normalize( vec3( 1.0, 0.0, -1.0 ) )
  );

  vec3 lightColor[ 3 ] = vec3[]
  ( 
    vec3( 1.0, 0.0, 0.0 ),
    vec3( 0.0, 1.0, 0.0 ),
    vec3( 0.0, 0.0, 1.0 )
  );

  vec3 normal = normalize( position );

  vec3 color = vec3( 0.0 );
  for( int i = 0; i < 3; i++ ) 
  {
    float time = u_time / 1000.0;

    // Animate lights
    if( i == 0 ) { lightDir[ i ].xy *= rot( time ) * lightDir[ i ].xy; }
    else if( i == 1 ) { lightDir[ i ].xz = rot( time ) * lightDir[ i ].xz; }
    else if( i == 2 ) { lightDir[ i ].yz = rot( time ) * lightDir[ i ].yz; }

    // Amount of light recieved
    float irradiance = max( dot( normal, lightDir[ i ] ), 0.0 ) * 4.0;

    // Resulting color value for the current light
    color += BRDF( lightDir[ i ], viewDir, normal ) * vec3( irradiance ) * lightColor[ i ];
  }

  //Smooth out the edges of the sphere
  color = mix( color, vec3( 0.0 ), smoothstep( 0.0, 2.0, circle ) );

  //Gamma correction
  color = pow( color, vec3( 1.0 / 2.2 ) );

  frag_color = vec4(  color, 1.0  );
}
