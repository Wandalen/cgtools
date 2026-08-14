import { gridTuning, FADE_CURVES } from './gridTuning.js';

function buildSlider({ id, label, min, max, step, value, format }) {
    const row = document.createElement('div');
    row.className = 'space-y-1';
    row.innerHTML = `
        <div class="flex justify-between text-[10px] text-cyan-300">
            <span>${label}</span>
            <span data-value class="font-mono-tech text-cyan-400">${format(value)}</span>
        </div>
        <input type="range" min="${min}" max="${max}" step="${step}" value="${value}"
            class="w-full h-1 accent-cyan-400">
    `;
    const input = row.querySelector('input');
    const valueLabel = row.querySelector('[data-value]');
    input.id = id;
    return { row, input, valueLabel };
}

function buildSelect({ id, label, options, value }) {
    const row = document.createElement('div');
    row.className = 'space-y-1';
    const opts = options.map((o) => `<option value="${o.index}" ${o.index === value ? 'selected' : ''}>${o.label}</option>`).join('');
    row.innerHTML = `
        <div class="text-[10px] text-cyan-300">${label}</div>
        <select class="w-full bg-slate-900 border border-cyan-900 rounded text-cyan-300 text-[10px] py-1 px-1">
            ${opts}
        </select>
    `;
    const select = row.querySelector('select');
    select.id = id;
    return { row, select };
}

function buildColorPicker({ id, label, value }) {
    const row = document.createElement('div');
    row.className = 'flex items-center justify-between text-[10px] text-cyan-300';
    row.innerHTML = `
        <span>${label}</span>
        <input type="color" value="${value}" class="w-10 h-5 bg-transparent border border-cyan-900 rounded cursor-pointer">
    `;
    const input = row.querySelector('input');
    input.id = id;
    return { row, input };
}

function subheading(text) {
    const el = document.createElement('div');
    el.className = 'text-[10px] text-cyan-500 uppercase tracking-wider pt-1';
    el.textContent = text;
    return el;
}

function curveLabel(index, options) {
    const found = options.find((o) => o.index === index);
    return found ? found.label : String(index);
}

// Reads every live uniform value off the material and formats it as a
// paste-back-able text block, so tuned settings don't have to be copied by
// hand off the sliders.
function buildTuningSummary(gridMaterial) {
    const u = gridMaterial.uniforms;
    const lines = [
        '===== Grid tuning config =====',
        '(paste this whole block back to update the grid defaults)',
        '',
        `outside opacity: ${u.uDimAlpha.value.toFixed(2)}`,
        `inside opacity: ${u.uBrightAlpha.value.toFixed(2)}`,
        '',
        `ribbon gap: ${u.uRibbonGap.value.toFixed(2)}`,
        `ribbon width outer: ${u.uRibbonWidthOuter.value.toFixed(2)}`,
        `ribbon width inner: ${u.uRibbonWidthInner.value.toFixed(2)}`,
        `ribbon opacity: ${u.uRibbonOpacity.value.toFixed(2)}`,
        `ribbon color core: #${u.uRingColorCore.value.getHexString()}`,
        `ribbon color edge: #${u.uRingColorEdge.value.getHexString()}`,
        '',
        `inside fade width (grid squares): ${u.uInsideFadeWidth.value.toFixed(2)}`,
        `inside fade curve: ${curveLabel(u.uInsideFadeMode.value, FADE_CURVES)}`,
        `inside fade gamma: ${u.uInsideFadeGamma.value.toFixed(2)}`,
        '',
        `camera fade start: ${u.uCameraFadeStart.value}`,
        `camera fade end: ${u.uCameraFadeEnd.value}`,
        `camera fade curve: ${curveLabel(u.uCameraFadeMode.value, FADE_CURVES)}`,
        `camera fade gamma: ${u.uCameraFadeGamma.value.toFixed(2)}`,
        '',
        `view radius override: ${gridTuning.viewRadiusOverride ?? 'none (follows selected unit)'}`,
        '',
        `asteroid glow alpha: ${u.uAsteroidGlowAlpha.value.toFixed(2)}`,
        `asteroid glow width (grid squares): ${u.uAsteroidGlowWidth.value.toFixed(2)}`,
        `asteroid glow curve: ${curveLabel(u.uAsteroidGlowMode.value, FADE_CURVES)}`,
        `asteroid glow gamma: ${u.uAsteroidGlowGamma.value.toFixed(2)}`,
    ];
    return lines.join('\n');
}

function buildCopyButton(gridMaterial) {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'w-full py-1.5 rounded btn-tactical text-cyan-300 font-bold text-[11px]';
    btn.textContent = 'Copy Settings';
    btn.addEventListener('click', async () => {
        const text = buildTuningSummary(gridMaterial);
        try {
            await navigator.clipboard.writeText(text);
            btn.textContent = 'Copied!';
        } catch (err) {
            console.log(text);
            btn.textContent = 'Copy failed - see console';
        }
        setTimeout(() => {
            btn.textContent = 'Copy Settings';
        }, 1500);
    });
    return btn;
}

// Floating dev panel with live sliders for the tactical grid's view-zone
// circle (radius, ribbon gap/width) and grid opacity (dim/bright). Only
// meant for tuning defaults during development - not part of the in-game UI.
export function setupGridTuningPanel(gridMaterial) {
    const panel = document.createElement('div');
    panel.className =
        'ui-panel rounded-lg p-3 fixed bottom-3 right-3 z-30 w-64 max-h-[85vh] flex flex-col pointer-events-auto text-xs';

    const header = document.createElement('button');
    header.type = 'button';
    header.className =
        'w-full flex justify-between items-center font-bold text-white uppercase text-xs border-b border-cyan-900 pb-1 mb-2 cursor-pointer';
    header.innerHTML = '<span>Grid Tuning (dev)</span><span data-caret>&#9662;</span>';
    panel.appendChild(header);

    const body = document.createElement('div');
    body.className = 'space-y-3 overflow-y-auto pr-1 min-h-0';
    panel.appendChild(body);

    body.appendChild(buildCopyButton(gridMaterial));

    const caret = header.querySelector('[data-caret]');
    header.addEventListener('click', () => {
        const collapsed = body.classList.toggle('hidden');
        caret.innerHTML = collapsed ? '&#9656;' : '&#9662;';
        header.classList.toggle('mb-2', !collapsed);
        header.classList.toggle('mb-0', collapsed);
    });

    // View radius override, with a checkbox to opt in (default: follow the
    // selected unit's own view radius).
    const radiusRow = document.createElement('div');
    radiusRow.className = 'flex items-center space-x-2 text-[10px] text-cyan-300 mb-1';
    radiusRow.innerHTML = `
        <input type="checkbox" id="grid-radius-override" class="accent-cyan-400">
        <label for="grid-radius-override">Override view radius</label>
    `;
    body.appendChild(radiusRow);
    const overrideCheckbox = radiusRow.querySelector('input');

    const radius = buildSlider({
        id: 'grid-radius',
        label: 'View Radius',
        min: 10,
        max: 300,
        step: 1,
        value: 160,
        format: (v) => v,
    });
    radius.input.disabled = true;
    radius.input.addEventListener('input', () => {
        const v = Number(radius.input.value);
        radius.valueLabel.textContent = v;
        gridTuning.viewRadiusOverride = overrideCheckbox.checked ? v : null;
    });
    overrideCheckbox.addEventListener('change', () => {
        radius.input.disabled = !overrideCheckbox.checked;
        gridTuning.viewRadiusOverride = overrideCheckbox.checked ? Number(radius.input.value) : null;
    });
    body.appendChild(radius.row);

    const ribbonGap = buildSlider({
        id: 'grid-ribbon-gap',
        label: 'Ribbon Gap',
        min: 0,
        max: 10,
        step: 0.1,
        value: gridTuning.ribbonGap,
        format: (v) => Number(v).toFixed(1),
    });
    ribbonGap.input.addEventListener('input', () => {
        const v = Number(ribbonGap.input.value);
        ribbonGap.valueLabel.textContent = v.toFixed(1);
        gridMaterial.uniforms.uRibbonGap.value = v;
    });
    body.appendChild(ribbonGap.row);

    const ribbonWidthOuter = buildSlider({
        id: 'grid-ribbon-width-outer',
        label: 'Ribbon Width (Outer)',
        min: 0.2,
        max: 5,
        step: 0.1,
        value: gridTuning.ribbonWidthOuter,
        format: (v) => Number(v).toFixed(1),
    });
    ribbonWidthOuter.input.addEventListener('input', () => {
        const v = Number(ribbonWidthOuter.input.value);
        ribbonWidthOuter.valueLabel.textContent = v.toFixed(1);
        gridMaterial.uniforms.uRibbonWidthOuter.value = v;
    });
    body.appendChild(ribbonWidthOuter.row);

    const ribbonWidthInner = buildSlider({
        id: 'grid-ribbon-width-inner',
        label: 'Ribbon Width (Inner)',
        min: 0.2,
        max: 5,
        step: 0.1,
        value: gridTuning.ribbonWidthInner,
        format: (v) => Number(v).toFixed(1),
    });
    ribbonWidthInner.input.addEventListener('input', () => {
        const v = Number(ribbonWidthInner.input.value);
        ribbonWidthInner.valueLabel.textContent = v.toFixed(1);
        gridMaterial.uniforms.uRibbonWidthInner.value = v;
    });
    body.appendChild(ribbonWidthInner.row);

    const ribbonOpacity = buildSlider({
        id: 'grid-ribbon-opacity',
        label: 'Ribbon Opacity',
        min: 0,
        max: 1,
        step: 0.01,
        value: gridTuning.ribbonOpacity,
        format: (v) => Number(v).toFixed(2),
    });
    ribbonOpacity.input.addEventListener('input', () => {
        const v = Number(ribbonOpacity.input.value);
        ribbonOpacity.valueLabel.textContent = v.toFixed(2);
        gridMaterial.uniforms.uRibbonOpacity.value = v;
    });
    body.appendChild(ribbonOpacity.row);

    const ribbonColorCore = buildColorPicker({
        id: 'grid-ribbon-color-core',
        label: 'Ribbon Core Color',
        value: gridTuning.ribbonColorCore,
    });
    ribbonColorCore.input.addEventListener('input', () => {
        gridMaterial.uniforms.uRingColorCore.value.set(ribbonColorCore.input.value);
    });
    body.appendChild(ribbonColorCore.row);

    const ribbonColorEdge = buildColorPicker({
        id: 'grid-ribbon-color-edge',
        label: 'Ribbon Edge Color',
        value: gridTuning.ribbonColorEdge,
    });
    ribbonColorEdge.input.addEventListener('input', () => {
        gridMaterial.uniforms.uRingColorEdge.value.set(ribbonColorEdge.input.value);
    });
    body.appendChild(ribbonColorEdge.row);

    const dimAlpha = buildSlider({
        id: 'grid-dim-alpha',
        label: 'Outside Opacity',
        min: 0,
        max: 1,
        step: 0.01,
        value: gridTuning.dimAlpha,
        format: (v) => Number(v).toFixed(2),
    });
    dimAlpha.input.addEventListener('input', () => {
        const v = Number(dimAlpha.input.value);
        dimAlpha.valueLabel.textContent = v.toFixed(2);
        gridMaterial.uniforms.uDimAlpha.value = v;
    });
    body.appendChild(dimAlpha.row);

    const brightAlpha = buildSlider({
        id: 'grid-bright-alpha',
        label: 'Inside Opacity',
        min: 0,
        max: 1,
        step: 0.01,
        value: gridTuning.brightAlpha,
        format: (v) => Number(v).toFixed(2),
    });
    brightAlpha.input.addEventListener('input', () => {
        const v = Number(brightAlpha.input.value);
        brightAlpha.valueLabel.textContent = v.toFixed(2);
        gridMaterial.uniforms.uBrightAlpha.value = v;
    });
    body.appendChild(brightAlpha.row);

    body.appendChild(subheading('View Zone Fade'));

    const insideFadeWidth = buildSlider({
        id: 'grid-inside-fade-width',
        label: 'Fade Width (grid squares)',
        min: 0,
        max: 8,
        step: 0.1,
        value: gridTuning.insideFadeWidth,
        format: (v) => Number(v).toFixed(1),
    });
    insideFadeWidth.input.addEventListener('input', () => {
        const v = Number(insideFadeWidth.input.value);
        insideFadeWidth.valueLabel.textContent = v.toFixed(1);
        gridMaterial.uniforms.uInsideFadeWidth.value = v;
    });
    body.appendChild(insideFadeWidth.row);

    const insideFadeCurve = buildSelect({
        id: 'grid-inside-fade-curve',
        label: 'Fade Curve',
        options: FADE_CURVES,
        value: gridTuning.insideFadeCurve,
    });
    insideFadeCurve.select.addEventListener('change', () => {
        gridMaterial.uniforms.uInsideFadeMode.value = Number(insideFadeCurve.select.value);
    });
    body.appendChild(insideFadeCurve.row);

    const insideFadeGamma = buildSlider({
        id: 'grid-inside-fade-gamma',
        label: 'Fade Gamma',
        min: 0.2,
        max: 3,
        step: 0.05,
        value: gridTuning.insideFadeGamma,
        format: (v) => Number(v).toFixed(2),
    });
    insideFadeGamma.input.addEventListener('input', () => {
        const v = Number(insideFadeGamma.input.value);
        insideFadeGamma.valueLabel.textContent = v.toFixed(2);
        gridMaterial.uniforms.uInsideFadeGamma.value = v;
    });
    body.appendChild(insideFadeGamma.row);

    body.appendChild(subheading('Camera Distance Fade'));

    const fadeStart = buildSlider({
        id: 'grid-fade-start',
        label: 'Camera Fade Start',
        min: 0,
        max: 1000,
        step: 10,
        value: gridTuning.cameraFadeStart,
        format: (v) => v,
    });
    fadeStart.input.addEventListener('input', () => {
        const v = Number(fadeStart.input.value);
        fadeStart.valueLabel.textContent = v;
        gridMaterial.uniforms.uCameraFadeStart.value = v;
    });
    body.appendChild(fadeStart.row);

    const fadeEnd = buildSlider({
        id: 'grid-fade-end',
        label: 'Camera Fade End',
        min: 0,
        max: 2000,
        step: 10,
        value: gridTuning.cameraFadeEnd,
        format: (v) => v,
    });
    fadeEnd.input.addEventListener('input', () => {
        const v = Number(fadeEnd.input.value);
        fadeEnd.valueLabel.textContent = v;
        gridMaterial.uniforms.uCameraFadeEnd.value = v;
    });
    body.appendChild(fadeEnd.row);

    const fadeCurve = buildSelect({
        id: 'grid-fade-curve',
        label: 'Fade Curve',
        options: FADE_CURVES,
        value: gridTuning.cameraFadeCurve,
    });
    fadeCurve.select.addEventListener('change', () => {
        gridMaterial.uniforms.uCameraFadeMode.value = Number(fadeCurve.select.value);
    });
    body.appendChild(fadeCurve.row);

    const fadeGamma = buildSlider({
        id: 'grid-fade-gamma',
        label: 'Fade Gamma',
        min: 0.2,
        max: 3,
        step: 0.05,
        value: gridTuning.cameraFadeGamma,
        format: (v) => Number(v).toFixed(2),
    });
    fadeGamma.input.addEventListener('input', () => {
        const v = Number(fadeGamma.input.value);
        fadeGamma.valueLabel.textContent = v.toFixed(2);
        gridMaterial.uniforms.uCameraFadeGamma.value = v;
    });
    body.appendChild(fadeGamma.row);

    body.appendChild(subheading('Asteroid Glow'));

    const glowAlpha = buildSlider({
        id: 'grid-glow-alpha',
        label: 'Glow Opacity',
        min: 0,
        max: 1,
        step: 0.01,
        value: gridTuning.asteroidGlowAlpha,
        format: (v) => Number(v).toFixed(2),
    });
    glowAlpha.input.addEventListener('input', () => {
        const v = Number(glowAlpha.input.value);
        glowAlpha.valueLabel.textContent = v.toFixed(2);
        gridMaterial.uniforms.uAsteroidGlowAlpha.value = v;
    });
    body.appendChild(glowAlpha.row);

    const glowWidth = buildSlider({
        id: 'grid-glow-width',
        label: 'Glow Width (grid squares)',
        min: 0,
        max: 8,
        step: 0.1,
        value: gridTuning.asteroidGlowWidth,
        format: (v) => Number(v).toFixed(1),
    });
    glowWidth.input.addEventListener('input', () => {
        const v = Number(glowWidth.input.value);
        glowWidth.valueLabel.textContent = v.toFixed(1);
        gridMaterial.uniforms.uAsteroidGlowWidth.value = v;
    });
    body.appendChild(glowWidth.row);

    const glowCurve = buildSelect({
        id: 'grid-glow-curve',
        label: 'Glow Curve',
        options: FADE_CURVES,
        value: gridTuning.asteroidGlowCurve,
    });
    glowCurve.select.addEventListener('change', () => {
        gridMaterial.uniforms.uAsteroidGlowMode.value = Number(glowCurve.select.value);
    });
    body.appendChild(glowCurve.row);

    const glowGamma = buildSlider({
        id: 'grid-glow-gamma',
        label: 'Glow Gamma',
        min: 0.2,
        max: 3,
        step: 0.05,
        value: gridTuning.asteroidGlowGamma,
        format: (v) => Number(v).toFixed(2),
    });
    glowGamma.input.addEventListener('input', () => {
        const v = Number(glowGamma.input.value);
        glowGamma.valueLabel.textContent = v.toFixed(2);
        gridMaterial.uniforms.uAsteroidGlowGamma.value = v;
    });
    body.appendChild(glowGamma.row);

    document.body.appendChild(panel);
}
