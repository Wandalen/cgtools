// Minimal slider control panel for shader_chunk_preview's tunable
// uniforms: addSlider() creates one labeled range input, onChange()
// registers the single callback fired (with every slider's current value,
// keyed by property) whenever any slider moves. No dropdown/show/hide --
// every param here is a numeric range and the panel is always visible (see
// index.html / style.css) -- ported from examples/minwebgl/filters/
// controls.js, trimmed to what this example actually uses.

let sliders = [];
let changeCallback = null;

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
