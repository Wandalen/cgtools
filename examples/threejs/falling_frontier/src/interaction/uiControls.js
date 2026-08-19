import { playbackState } from '../state.js';

function bindLayerToggle(buttonId, group) {
    document.getElementById(buttonId).addEventListener('click', (e) => {
        group.visible = !group.visible;
        e.currentTarget.classList.toggle('active');
    });
}

function bindScanlinesToggle() {
    document.getElementById('toggle-scanlines').addEventListener('click', (e) => {
        document.getElementById('scanline-overlay').classList.toggle('hidden');

        const btn = e.currentTarget;
        btn.classList.toggle('active');

        const statusTag = btn.querySelector('span:last-child');
        if (btn.classList.contains('active')) {
            statusTag.innerText = '[ON]';
            statusTag.classList.remove('text-cyan-700');
            statusTag.classList.add('text-cyan-400');
        } else {
            statusTag.innerText = '[OFF]';
            statusTag.classList.remove('text-cyan-400');
            statusTag.classList.add('text-cyan-700');
        }
    });
}

function bindAnimateToggleAndTimeControls() {
    const toggleAnimateBtn = document.getElementById('toggle-animate');
    const animateStatusTag = toggleAnimateBtn.querySelector('span:last-child');

    function updateAnimateBtnUI(active) {
        toggleAnimateBtn.classList.toggle('active', active);
        if (animateStatusTag) animateStatusTag.innerText = active ? '[ACTIVE]' : '[PAUSED]';
    }

    toggleAnimateBtn.addEventListener('click', () => {
        playbackState.isAnimating = !playbackState.isAnimating;
        updateAnimateBtnUI(playbackState.isAnimating);
    });

    const btnPause = document.getElementById('btn-pause');
    const btnPlay = document.getElementById('btn-play');
    const btnFast = document.getElementById('btn-fast');

    btnPause.addEventListener('click', () => {
        playbackState.isAnimating = false;
        btnPause.classList.add('active');
        btnPlay.classList.remove('active');
        btnFast.classList.remove('active');
        updateAnimateBtnUI(false);
    });

    btnPlay.addEventListener('click', () => {
        playbackState.isAnimating = true;
        playbackState.shipSpeedMultiplier = 1;
        btnPause.classList.remove('active');
        btnPlay.classList.add('active');
        btnFast.classList.remove('active');
        updateAnimateBtnUI(true);
    });

    btnFast.addEventListener('click', () => {
        playbackState.isAnimating = true;
        playbackState.shipSpeedMultiplier = 2.5;
        btnPause.classList.remove('active');
        btnPlay.classList.remove('active');
        btnFast.classList.add('active');
        updateAnimateBtnUI(true);
    });
}

// Wires every tactical UI button (layer toggles, scanlines, playback, camera reset).
export function setupUIControls(world) {
    bindLayerToggle('toggle-grid', world.groups.grid);
    bindLayerToggle('toggle-trajectories', world.groups.trajectory);
    bindLayerToggle('toggle-ranges', world.groups.sensorRing);
    bindScanlinesToggle();
    bindAnimateToggleAndTimeControls();

    document.getElementById('btn-reset-cam').addEventListener('click', () => {
        world.resetCamera();
    });
}
