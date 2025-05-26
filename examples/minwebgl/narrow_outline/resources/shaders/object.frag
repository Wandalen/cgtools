#version 300 es
// Set the precision for float calculations. mediump is usually sufficient for color.
precision mediump float;

// Input normal from the vertex shader.
in vec3 v_norm;

// Output fragment color. This will be written to the framebuffer's color attachment.
layout( location = 0 ) out vec4 FragColor;
// Output fragment normal. This will be written to the framebuffer's normal attachment.
layout( location = 1 ) out vec4 FragNorm;

void main()
{
	// Simply output a solid white color.
	// This creates a white silhouette of the object in the first rendering pass.
	// This white color is then used by the JFA initialization pass to identify
	// pixels belonging to the object.
	FragColor = vec4( 1.0, 1.0, 1.0, 1.0 );
	// Output the normalized view-space normal.
    // We store it in a vec4 and normalize it here to ensure it's a unit vector.
    // Store it as a color by mapping the [-1, 1] range to [0, 1].
    FragNorm = vec4( normalize( v_norm ) * 0.5 + 0.5, 1.0 );
}
