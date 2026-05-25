#version 300 es
precision highp float;

in vec2 vUv;
out vec4 fragColor;

uniform samplerCube envMap;
uniform int face;

const float PI = 3.141592653589793;

vec3 faceDirFromUV( vec2 uv, int f )
{
  vec2 ndc = uv * 2.0 - 1.0;
  vec3 dir;
  if( f == 0 ) dir = vec3(  1.0, -ndc.y, -ndc.x );
  else if( f == 1 ) dir = vec3( -1.0, -ndc.y,  ndc.x );
  else if( f == 2 ) dir = vec3(  ndc.x,  1.0,  ndc.y );
  else if( f == 3 ) dir = vec3(  ndc.x, -1.0, -ndc.y );
  else if( f == 4 ) dir = vec3(  ndc.x, -ndc.y,  1.0 );
  else              dir = vec3( -ndc.x, -ndc.y, -1.0 );
  return normalize( dir );
}

void main()
{
  vec3 normal = faceDirFromUV( vUv, face );

  vec3 up = abs( normal.y ) < 0.999 ? vec3( 0.0, 1.0, 0.0 ) : vec3( 0.0, 0.0, 1.0 );
  vec3 right = normalize( cross( up, normal ) );
  up = cross( normal, right );

  vec3 irradiance = vec3( 0.0 );
  float nrSamples = 0.0;
  float sampleDelta = 0.025;

  for( float phi = 0.0; phi < 2.0 * PI; phi += sampleDelta )
  {
    for( float theta = 0.0; theta < 0.5 * PI; theta += sampleDelta )
    {
      vec3 tangentSample = vec3(
        sin( theta ) * cos( phi ),
        sin( theta ) * sin( phi ),
        cos( theta )
      );
      vec3 sampleVec = tangentSample.x * right + tangentSample.y * up + tangentSample.z * normal;

      irradiance += texture( envMap, sampleVec ).rgb * cos( theta ) * sin( theta );
      nrSamples += 1.0;
    }
  }

  irradiance = PI * irradiance / nrSamples;
  fragColor = vec4( irradiance, 1.0 );
}
