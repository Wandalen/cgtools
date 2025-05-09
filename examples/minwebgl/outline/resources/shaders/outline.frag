#version 300 es
precision mediump float;
in vec2 v_tex_coord;
out vec4 FragColor;
uniform sampler2D u_object_texture;   // Input: Rendered 3D object silhouette
uniform sampler2D u_jfa_texture;      // Input: Final JFA result (nearest seed coords)
uniform vec2 u_resolution;
uniform float u_outline_thickness;  // Outline thickness in pixels
uniform vec4 u_outline_color;       // Color of the outline
uniform vec4 u_object_color;        // Fill color for the object itself
uniform vec4 u_background_color;    // Background color

// Check if a seed coordinate is valid (not the sentinel)
bool is_valid_seed(vec2 seed_coord) {
    return seed_coord.x >= 0.0;
}

void main() {
    // Check if the current pixel belongs to the object silhouette
    float object_present = texture(u_object_texture, v_tex_coord).r;

    if (object_present > 0.1) {
        // Pixel belongs to the object, draw with object color
        FragColor = u_object_color;
    } else {
        // Pixel is background, check distance to nearest object pixel using JFA result
        vec2 nearest_seed_norm = texture(u_jfa_texture, v_tex_coord).xy;

        if (is_valid_seed(nearest_seed_norm)) {
            // Calculate distance in pixel space
            vec2 current_pixel_coord = v_tex_coord * u_resolution;
            vec2 nearest_seed_pixel = nearest_seed_norm * u_resolution;
            float dist = distance(current_pixel_coord, nearest_seed_pixel);

            // If the distance is within the outline thickness, draw the outline color
            if (dist < u_outline_thickness) {
                FragColor = u_outline_color;
            } else {
                // Pixel is background, outside the outline
                FragColor = u_background_color;
            }
        } else {
             // Pixel is far background (JFA didn't find a nearby seed)
             FragColor = u_background_color;
        }
    }
}