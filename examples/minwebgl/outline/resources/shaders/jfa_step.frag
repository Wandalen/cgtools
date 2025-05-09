#version 300 es
precision highp float;
in vec2 v_tex_coord;
out vec4 FragColor; // Outputting vec4 for RGBA32F texture
uniform sampler2D u_jfa_texture; // Input: JFA texture from previous step
uniform vec2 u_resolution;
uniform float u_step_size; // Current jump distance

// Check if a seed coordinate is valid (not the sentinel)
bool is_valid_seed(vec2 seed_coord) {
    return seed_coord.x >= 0.0; // Check only x is enough
}

void main() {
    vec2 current_pixel_coord = v_tex_coord * u_resolution; // Current pixel in pixel space

    // Sample current pixel's stored seed coordinate (normalized)
    vec2 current_seed_norm = texture(u_jfa_texture, v_tex_coord).xy;
    vec2 best_seed_norm = current_seed_norm; // Start with the current seed
    float min_dist_sq = 1e20; // Initialize with a large value

    // If the current pixel already has a valid seed, calculate its distance
    if (is_valid_seed(current_seed_norm)) {
        min_dist_sq = distance(current_pixel_coord, current_seed_norm * u_resolution);
    }

    // Sample neighbors with the current step size
    vec2 step_offset_pixels = vec2(u_step_size);
    for (int y = -1; y <= 1; ++y) {
        for (int x = -1; x <= 1; ++x) {
            if (x == 0 && y == 0) continue; // Skip center pixel

            vec2 offset_pixels = vec2(float(x), float(y)) * step_offset_pixels;
            vec2 sample_coord_norm = (current_pixel_coord + offset_pixels) / u_resolution;

            // Clamp sample coordinates to stay within texture bounds (important!)
            // Using clamp avoids branching and potential out-of-bounds reads
            sample_coord_norm = clamp(sample_coord_norm, 0.0, 1.0);

            vec2 neighbor_seed_norm = texture(u_jfa_texture, sample_coord_norm).xy;

            // If the neighbor has a valid seed, check its distance
            if (is_valid_seed(neighbor_seed_norm)) {
                float d_sq = distance(current_pixel_coord, neighbor_seed_norm * u_resolution);
                // If this neighbor's seed is closer, update the best seed
                if (d_sq < min_dist_sq) {
                    min_dist_sq = d_sq;
                    best_seed_norm = neighbor_seed_norm;
                }
            }
        }
    }

    // Output the best seed found (or the initial sentinel value if none found)
    FragColor = vec4(best_seed_norm, 0.0, 1.0);
}