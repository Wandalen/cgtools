function round(n, decimals = 2) {
    const factor = 10 ** decimals;
    return Math.round(n * factor) / factor;
}

function vec3ToArray(v) {
    return [round(v.x), round(v.y), round(v.z)];
}

// Exposes window.dumpSceneConfig() in the browser console: it prints the
// current camera, station and ship placements (as edited via the transform
// gizmo) in a format that can be pasted back to update the scene defaults.
// Note: only base position/rotationY are captured, not the patrol paths.
export function exposeSceneConfigDump({ world, animatedShips, station, asteroidsGroup }) {
    window.dumpSceneConfig = () => {
        const lines = [];
        lines.push('===== Falling Frontier scene config =====');
        lines.push('(paste this whole block back to update the scene defaults)');
        lines.push('');
        lines.push(`background color: 0x${world.scene.background.getHexString()}`);
        lines.push('');
        lines.push(`camera: position ${JSON.stringify(vec3ToArray(world.camera.position))}, target ${JSON.stringify(vec3ToArray(world.controls.target))}`);
        lines.push('');
        lines.push(`station "${station.userData.name}": position ${JSON.stringify(vec3ToArray(station.position))}, rotationY ${round(station.rotation.y)}`);
        lines.push('');
        lines.push('fleet:');
        animatedShips.forEach(({ mesh }) => {
            lines.push(`  ${mesh.userData.name.padEnd(14)} position ${JSON.stringify(vec3ToArray(mesh.position))}, rotationY ${round(mesh.rotation.y)}`);
        });
        lines.push('');
        lines.push('asteroids:');
        asteroidsGroup.children.forEach((asteroid) => {
            lines.push(`  ${asteroid.userData.name.padEnd(14)} position ${JSON.stringify(vec3ToArray(asteroid.position))}, rotationY ${round(asteroid.rotation.y)}`);
        });

        const text = lines.join('\n');
        console.log(text);
        return text;
    };

    console.log('%cScene editor: click a ship/station to select, G = move (XZ), R = rotate (Y), Escape = deselect. Run dumpSceneConfig() here to print the current layout.', 'color:#00e5ff');
}
