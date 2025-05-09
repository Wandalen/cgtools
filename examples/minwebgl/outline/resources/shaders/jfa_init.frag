#version 300 es
precision highp float; // Need high precision for coordinates
in vec2 v_tex_coord;
out vec4 FragColor; // Outputting vec4 for RGBA32F texture
uniform sampler2D u_object_texture; // Input: Rendered 3D object silhouette

void main() {
    // Check if the object is present (we rendered it white)
    float object_present = texture(u_object_texture, v_tex_coord).r;
    if (object_present > 0.1) { // If pixel is part of the object
        // Store normalized texture coordinates of this object pixel
        FragColor = vec4(v_tex_coord, 0.0, 1.0);
    } else {
        // Mark background pixels with a sentinel value
        FragColor = vec4(-1.0, -1.0, -1.0, 1.0);
    }
}