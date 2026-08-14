import * as THREE from 'three';

const PARTICLE_COUNT = 2000;

// Cosmic dust particle field scattered through the whole scene volume.
export function createStarfield(scene) {
    const geometry = new THREE.BufferGeometry();
    const positions = new Float32Array(PARTICLE_COUNT * 3);
    const colors = new Float32Array(PARTICLE_COUNT * 3);

    for (let i = 0; i < PARTICLE_COUNT; i++) {
        positions[i * 3] = (Math.random() - 0.5) * 1200;
        positions[i * 3 + 1] = (Math.random() - 0.5) * 600;
        positions[i * 3 + 2] = (Math.random() - 0.5) * 1200;

        // Cyan, blue and white tinted dust particles
        const color = new THREE.Color();
        if (Math.random() > 0.4) {
            color.setHSL(0.55 + Math.random() * 0.1, 0.8, 0.6 + Math.random() * 0.3);
        } else {
            color.setHex(0xffffff);
        }
        colors[i * 3] = color.r;
        colors[i * 3 + 1] = color.g;
        colors[i * 3 + 2] = color.b;
    }

    geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
    geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));

    const material = new THREE.PointsMaterial({
        size: 1.5,
        vertexColors: true,
        transparent: true,
        opacity: 0.75,
        sizeAttenuation: true,
    });

    const starField = new THREE.Points(geometry, material);
    scene.add(starField);
    return starField;
}
