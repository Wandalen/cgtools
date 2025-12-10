// Custom controls module for filter parameters

let currentControls = [];
let changeCallback = null;

// Clear all controls
export function clearControls() {
  const container = document.getElementById('controls-container');
  if (container) {
    container.innerHTML = '';
  }
  currentControls = [];
  changeCallback = null;
}

// Add a slider control
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

  currentControls.push({ type: 'slider', property, element: slider });
}

// Add a dropdown control
export function addDropdown(label, property, value, options) {
  const container = document.getElementById('controls-container');
  if (!container) return;

  const group = document.createElement('div');
  group.className = 'control-group';

  const labelEl = document.createElement('div');
  labelEl.className = 'control-label';
  labelEl.textContent = label;

  const select = document.createElement('select');
  select.className = 'control-dropdown';
  select.dataset.property = property;

  options.forEach(opt => {
    const option = document.createElement('option');
    option.value = opt;
    option.textContent = opt;
    if (opt === value) {
      option.selected = true;
    }
    select.appendChild(option);
  });

  select.addEventListener('change', (e) => {
    if (changeCallback) {
      changeCallback(getValues());
    }
  });

  group.appendChild(labelEl);
  group.appendChild(select);
  container.appendChild(group);

  currentControls.push({ type: 'dropdown', property, element: select });
}

// Set the callback for when any control changes
export function onChange(callback) {
  changeCallback = callback;
}

// Get all current values as an object
export function getValues() {
  const values = {};
  currentControls.forEach(control => {
    if (control.type === 'slider') {
      values[control.property] = parseFloat(control.element.value);
    } else if (control.type === 'dropdown') {
      values[control.property] = control.element.value;
    }
  });
  return values;
}

// Show the controls bar and buttons
export function show() {
  const bar = document.getElementById('bottom-controls');
  if (bar) {
    bar.classList.add('visible');
  }
}

// Hide the controls bar and buttons
export function hide() {
  const bar = document.getElementById('bottom-controls');
  if (bar) {
    bar.classList.remove('visible');
  }
}

// Format value for display
function formatValue(value) {
  if (Number.isInteger(value)) {
    return value.toString();
  }
  // Show up to 3 decimal places, removing trailing zeros
  return parseFloat(value.toFixed(3)).toString();
}
