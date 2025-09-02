#version 300 es
// Set the precision for float calculations. mediump is usually sufficient for color.
precision mediump float;
// Output fragment color. This will be written to the framebuffer's color attachment.
out vec4 FragColor;

void main()
{
	// Simply output a solid white color.
	// This creates a white silhouette of the object in the first rendering pass.
	// This white color is then used by the JFA initialization pass to identify
	// pixels belonging to the object.
	FragColor = vec4( 1.0, 1.0, 1.0, 1.0 );
}
