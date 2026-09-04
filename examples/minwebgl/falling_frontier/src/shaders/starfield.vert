#version 300 es

layout ( location = 0 ) in vec3 a_position;
layout ( location = 1 ) in vec3 a_color;
layout ( location = 2 ) in float a_size;
layout ( location = 3 ) in float a_alpha;

uniform mat4 u_view_proj;

out vec3 v_color;
out float v_alpha;

void main()
{
  v_color = a_color;
  v_alpha = a_alpha;
  gl_Position = u_view_proj * vec4( a_position, 1.0 );

  // Deliberately *not* three.js's PointsMaterial({ sizeAttenuation: true })
  // (`gl_PointSize *= scale / -mvPosition.z`, which this had originally).
  // The star box is centered on the camera's own orbit target, and the
  // camera's eye position sits well inside the box's bounds too, so a
  // meaningful fraction of the (uniformly-distributed - checked live via a
  // CPU-side position dump) points are genuinely close to the camera at
  // any moment. Distance attenuation makes exactly those points render
  // largest/brightest, and since they cluster directionally toward
  // wherever the camera is looking (the target), that reads as "stars
  // clustered in the middle" even though nothing about the underlying data
  // is uneven. A per-star fixed size (randomized on the CPU - see
  // starfield.rs's MIN_SIZE/MAX_SIZE) avoids that bias entirely - no star's
  // size ever depends on camera position, and stars read as individually
  // distinct instead of one uniform haze.
  gl_PointSize = a_size;
}
