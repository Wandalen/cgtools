import './style.css';
import { World } from './scene/world.js';
import { createStarfield } from './world/starfield.js';
import { createTacticalGrid, setGridFocus, updateGridBoundary, updateAsteroidGlow } from './world/tacticalGrid.js';
import { createAsteroidBelt, spinAsteroids } from './world/asteroidBelt.js';
import { createSpaceStation, spinStations } from './world/spaceStation.js';
import { createFleetAndTrajectories, updateFleetMotion } from './world/fleet.js';
import { setupSelection } from './interaction/selection.js';
import { setupUIControls } from './interaction/uiControls.js';
import { createGizmo } from './interaction/transform.js';
import { exposeSceneConfigDump } from './debug/sceneConfig.js';
import { setupGridTuningPanel } from './debug/gridTuningPanel.js';
import { gridTuning } from './debug/gridTuning.js';
import { playbackState } from './state.js';

function init() {
    const container = document.getElementById('canvas-container');
    const world = new World(container);
    const { groups } = world;

    // Routes/sensor rings are being reworked - hidden by default for now.
    // The "Vector Trajectories" / "Sensor Ranges & Rings" buttons still
    // toggle them back on.
    groups.trajectory.visible = false;
    groups.sensorRing.visible = false;

    createStarfield(world.scene);
    const gridMaterial = createTacticalGrid(groups.grid);
    createAsteroidBelt(groups.asteroids);
    const station = createSpaceStation(groups.station, groups.trajectory, groups.grid);
    const animatedShips = createFleetAndTrajectories(groups);

    const gizmo = createGizmo(world);
    setupSelection(world, gizmo);
    setupUIControls(world);
    exposeSceneConfigDump({ world, animatedShips, station, asteroidsGroup: groups.asteroids });
    setupGridTuningPanel(gridMaterial);

    function animate() {
        requestAnimationFrame(animate);

        world.controls.update();
        spinStations(groups.station, gizmo.object);
        spinAsteroids(groups.asteroids, gizmo.object);

        if (playbackState.isAnimating) {
            updateFleetMotion(animatedShips, playbackState.shipSpeedMultiplier, gizmo.object);
        }

        // The grid's "view zone" circle follows whatever is currently
        // selected (and has a view radius); nothing selected -> whole grid
        // stays dim. The tuning panel's radius override, if enabled, wins.
        const focusTarget = gizmo.object?.userData.viewRadius !== undefined ? gizmo.object : null;
        const focusRadius = gridTuning.viewRadiusOverride ?? focusTarget?.userData.viewRadius;
        setGridFocus(gridMaterial, focusTarget?.position, focusRadius);
        updateGridBoundary(gridMaterial, groups.asteroids);
        updateAsteroidGlow(gridMaterial, groups.asteroids);

        world.render();
    }

    animate();
}

window.addEventListener('load', init);
