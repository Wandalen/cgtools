#version 300 es
// Set the precision for float calculations. mediump is usually sufficient for color.
precision mediump float;

// Input normal from the vertex shader.
in vec3 v_norm;
in float v_object_id;

// Output fragment color. This will be written to the framebuffer's color attachment.
layout( location = 0 ) out vec4 FragColor;
// Output fragment normal. This will be written to the framebuffer's normal attachment.
layout( location = 1 ) out vec3 FragNorm;
// Output fragment depth. This will be written to the framebuffer's depth attachment.
layout( location = 2 ) out float FragDepth;

layout( std140 ) uniform ObjectColorBlock
{
  vec4 u_object_colors[ 256 ];
}; 

uniform float near;
uniform float far;

float linearizeDepth( float depth )
{
  return ( 2.0 * near * far ) / ( far + near - ( depth * 2.0 - 1.0 ) * ( far - near ) );
}

void main()
{
	FragColor = u_object_colors[ uint( v_object_id ) ];
	FragDepth = linearizeDepth( gl_FragCoord.z );  
	// Output the normalized view-space normal.
  // We store it in a vec4 and normalize it here to ensure it's a unit vector.
  // Store it as a color by mapping the [-1, 1] range to [0, 1].
  FragNorm = normalize( v_norm ) * 0.5 + 0.5;
}
