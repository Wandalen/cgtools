import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { COLORS } from '../config/colors.js';

const DEFAULT_CAMERA_POSITION = new THREE.Vector3(-288.71, 281.43, -57.4);

// Owns the scene, camera, renderer, controls and top-level groups that the
// rest of the app populates and animates.
export class World {
    constructor(container) {
        this.container = container;

        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(COLORS.spaceBg);

        this.camera = new THREE.PerspectiveCamera(
            45,
            window.innerWidth / window.innerHeight,
            0.1,
            2000
        );
        this.camera.position.copy(DEFAULT_CAMERA_POSITION);

        this.renderer = new THREE.WebGLRenderer({ antialias: true, powerPreference: 'high-performance' });
        this.renderer.setSize(window.innerWidth, window.innerHeight);
        this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
        this.renderer.shadowMap.enabled = true;
        this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
        this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
        this.renderer.toneMappingExposure = 1.4;
        this.container.appendChild(this.renderer.domElement);

        this.controls = new OrbitControls(this.camera, this.renderer.domElement);
        this.controls.enableDamping = true;
        this.controls.dampingFactor = 0.05;
        this.controls.maxPolarAngle = Math.PI / 2 - 0.02; // Keep camera above grid
        this.controls.minDistance = 20;
        this.controls.maxDistance = 500;
        this.controls.target.set(0, 0, 0);

        this.groups = {
            grid: new THREE.Group(),
            trajectory: new THREE.Group(),
            sensorRing: new THREE.Group(),
            asteroids: new THREE.Group(),
            ships: new THREE.Group(),
            station: new THREE.Group(),
        };
        for (const group of Object.values(this.groups)) {
            this.scene.add(group);
        }

        this.addLighting();

        window.addEventListener('resize', () => this.onWindowResize());
    }

    addLighting() {
        const hemiLight = new THREE.HemisphereLight(0x2a4460, 0x050a0f, 2.0);
        this.scene.add(hemiLight);

        const sunLight = new THREE.DirectionalLight(0xffeedd, 4.0);
        sunLight.position.set(300, 150, -250);
        sunLight.castShadow = true;
        sunLight.shadow.mapSize.width = 2048;
        sunLight.shadow.mapSize.height = 2048;
        sunLight.shadow.camera.near = 10;
        sunLight.shadow.camera.far = 1000;
        const d = 200;
        sunLight.shadow.camera.left = -d;
        sunLight.shadow.camera.right = d;
        sunLight.shadow.camera.top = d;
        sunLight.shadow.camera.bottom = -d;
        this.scene.add(sunLight);

        // Visual representation of the Star (Sun)
        const starGeo = new THREE.SphereGeometry(30, 32, 32);
        const starMat = new THREE.MeshBasicMaterial({ color: 0xffffff });
        const starMesh = new THREE.Mesh(starGeo, starMat);
        starMesh.position.copy(sunLight.position);
        this.scene.add(starMesh);

        const starGlowGeo = new THREE.SphereGeometry(45, 32, 32);
        const starGlowMat = new THREE.MeshBasicMaterial({
            color: 0xffa500,
            transparent: true,
            opacity: 0.15,
            blending: THREE.AdditiveBlending,
        });
        starMesh.add(new THREE.Mesh(starGlowGeo, starGlowMat));

        const starGlowOuterGeo = new THREE.SphereGeometry(85, 32, 32);
        const starGlowOuterMat = new THREE.MeshBasicMaterial({
            color: 0xff6600,
            transparent: true,
            opacity: 0.05,
            blending: THREE.AdditiveBlending,
        });
        starMesh.add(new THREE.Mesh(starGlowOuterGeo, starGlowOuterMat));

        // Secondary Cool Fill Light (ambient starlight/nebula reflection)
        const fillLight = new THREE.DirectionalLight(0x0088cc, 1.5);
        fillLight.position.set(-150, -50, -200);
        this.scene.add(fillLight);
    }

    resetCamera() {
        this.camera.position.copy(DEFAULT_CAMERA_POSITION);
        this.controls.target.set(0, 0, 0);
    }

    onWindowResize() {
        this.camera.aspect = window.innerWidth / window.innerHeight;
        this.camera.updateProjectionMatrix();
        this.renderer.setSize(window.innerWidth, window.innerHeight);
    }

    render() {
        this.renderer.render(this.scene, this.camera);
    }
}
