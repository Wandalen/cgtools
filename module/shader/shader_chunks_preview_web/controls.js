// Minimal slider control panel for the preview runner's tunable uniforms:
// addSlider() creates one labeled range input, onChange() registers the
// single callback fired (with every slider's current value, keyed by
// property) whenever any slider moves. No dropdown/show/hide -- every
// bundle parameter is a numeric range and the panel is always visible (see
// index.html / style.css) -- ported from examples/minwebgl/filters/
// controls.js, trimmed to what this runner actually uses.
//
// Also hosts the Shadertoy-style live source editor: initEditor() seeds the
// textarea and wires a debounced input listener, onEdit() registers the
// callback that listener fires, and setDiagnostics()/clearDiagnostics()
// show or hide the last compile/pipeline error underneath it. setChunkTitle()
// sets the page title and on-page heading to the previewed chunk's name.

let sliders = [];
let changeCallback = null;

// Set the page title and the on-page heading to the previewed chunk's name,
// so it's always clear which chunk a given preview session is showing.
export function setChunkTitle(name) {
  document.title = `${name} — Shader Chunk Preview`;
  const el = document.getElementById('chunk-title');
  if (el) el.textContent = name;
}

let editorTextarea = null;
let editCallback = null;
let editDebounceTimer = null;
const EDIT_DEBOUNCE_MS = 500;

export function addSlider(label, property, value, min, max, step) {
  const container = document.getElementById('controls-container');
  if (!container) return;

  const group = document.createElement('div');
  group.className = 'control-group';

  const labelEl = document.createElement('div');
  labelEl.className = 'control-label';
  labelEl.textContent = label;

  const sliderContainer = document.createElement('div');
  sliderContainer.className = 'control-slider-container';

  const slider = document.createElement('input');
  slider.type = 'range';
  slider.className = 'control-slider';
  slider.min = min;
  slider.max = max;
  slider.step = step;
  slider.value = value;
  slider.dataset.property = property;

  const valueDisplay = document.createElement('div');
  valueDisplay.className = 'control-value';
  valueDisplay.textContent = formatValue(value);

  slider.addEventListener('input', (e) => {
    const val = parseFloat(e.target.value);
    valueDisplay.textContent = formatValue(val);
    if (changeCallback) {
      changeCallback(getValues());
    }
  });

  sliderContainer.appendChild(slider);
  sliderContainer.appendChild(valueDisplay);
  group.appendChild(labelEl);
  group.appendChild(sliderContainer);
  container.appendChild(group);

  sliders.push({ property, element: slider });
}

// Set the callback fired with the full current value set on every slider move.
export function onChange(callback) {
  changeCallback = callback;
}

function getValues() {
  const values = {};
  sliders.forEach((s) => {
    values[s.property] = parseFloat(s.element.value);
  });
  return values;
}

// Show up to 3 decimal places, removing trailing zeros.
function formatValue(value) {
  if (Number.isInteger(value)) {
    return value.toString();
  }
  return parseFloat(value.toFixed(3)).toString();
}

// Seed the source textarea and wire its debounced input listener. Safe to
// call once at startup; a missing #shader-source element is a no-op, same
// guard style as addSlider().
export function initEditor(initialSource) {
  const textarea = document.getElementById('shader-source');
  if (!textarea) return;

  textarea.value = initialSource;
  editorTextarea = textarea;

  textarea.addEventListener('input', () => {
    if (editDebounceTimer) clearTimeout(editDebounceTimer);
    editDebounceTimer = setTimeout(() => {
      if (editCallback) editCallback(editorTextarea.value);
    }, EDIT_DEBOUNCE_MS);
  });
}

// Set the callback fired (with the textarea's current text) ~500ms after
// the last keystroke.
export function onEdit(callback) {
  editCallback = callback;
}

// Show a compile/pipeline diagnostic message under the editor.
export function setDiagnostics(text) {
  const el = document.getElementById('shader-diagnostics');
  if (!el) return;
  el.textContent = text;
  el.hidden = false;
}

// Hide the diagnostics panel -- called once a recompile succeeds.
export function clearDiagnostics() {
  const el = document.getElementById('shader-diagnostics');
  if (!el) return;
  el.textContent = '';
  el.hidden = true;
}
