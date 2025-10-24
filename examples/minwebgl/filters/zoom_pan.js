// Zoom and Pan Controller for Canvas
// Pure CSS transforms - no WebGL manipulation

let state = {
  scale: 1.0,
  translateX: 0,
  translateY: 0,
  isDragging: false,
  dragStartX: 0,
  dragStartY: 0,
  startTranslateX: 0,
  startTranslateY: 0
};

const MIN_SCALE = 0.1;
const MAX_SCALE = 10;
const ZOOM_SPEED = 0.1;

function updateTransform() {
  const wrapper = document.getElementById('canvas-wrapper');
  if (wrapper) {
    wrapper.style.transform = `translate(${state.translateX}px, ${state.translateY}px) scale(${state.scale})`;
  }
  updateZoomInfo();
}

function updateZoomInfo() {
  const info = document.getElementById('zoom-info');
  if (info) {
    info.textContent = `${Math.round(state.scale * 100)}%`;
  }
}

function zoomIn() {
  state.scale = Math.min(MAX_SCALE, state.scale * (1 + ZOOM_SPEED));
  updateTransform();
}

function zoomOut() {
  state.scale = Math.max(MIN_SCALE, state.scale * (1 - ZOOM_SPEED));
  updateTransform();
}

function resetView() {
  state.scale = 1.0;
  state.translateX = 0;
  state.translateY = 0;
  updateTransform();
}

function zoomAtPoint(deltaY, clientX, clientY) {
  const wrapper = document.getElementById('canvas-wrapper');
  if (!wrapper) return;

  const rect = wrapper.getBoundingClientRect();

  // Get mouse position relative to wrapper center
  const mouseX = clientX - rect.left - rect.width / 2;
  const mouseY = clientY - rect.top - rect.height / 2;

  // Calculate new scale
  const oldScale = state.scale;
  const zoomFactor = deltaY > 0 ? (1 - ZOOM_SPEED) : (1 + ZOOM_SPEED);
  const newScale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, oldScale * zoomFactor));

  // Adjust translation to zoom towards mouse position
  const scaleDiff = newScale - oldScale;
  state.translateX -= (mouseX / oldScale) * scaleDiff;
  state.translateY -= (mouseY / oldScale) * scaleDiff;
  state.scale = newScale;

  updateTransform();
}

export function setupZoomPan() {
  const wrapper = document.getElementById('canvas-wrapper');
  const container = document.querySelector('.canvas-container');
  const zoomInBtn = document.getElementById('zoom-in');
  const zoomOutBtn = document.getElementById('zoom-out');
  const zoomResetBtn = document.getElementById('zoom-reset');

  if (!wrapper || !container) {
    console.warn('Canvas wrapper or container not found');
    return;
  }

  // Button controls
  if (zoomInBtn) zoomInBtn.addEventListener('click', zoomIn);
  if (zoomOutBtn) zoomOutBtn.addEventListener('click', zoomOut);
  if (zoomResetBtn) zoomResetBtn.addEventListener('click', resetView);

  // Mouse wheel zoom
  container.addEventListener('wheel', (e) => {
    e.preventDefault();
    zoomAtPoint(e.deltaY, e.clientX, e.clientY);
  }, { passive: false });

  // Double-click to reset
  wrapper.addEventListener('dblclick', resetView);

  // Pan with drag
  wrapper.addEventListener('mousedown', (e) => {
    state.isDragging = true;
    state.dragStartX = e.clientX;
    state.dragStartY = e.clientY;
    state.startTranslateX = state.translateX;
    state.startTranslateY = state.translateY;
    wrapper.style.transition = 'none';
  });

  document.addEventListener('mousemove', (e) => {
    if (state.isDragging) {
      const dx = e.clientX - state.dragStartX;
      const dy = e.clientY - state.dragStartY;
      state.translateX = state.startTranslateX + dx;
      state.translateY = state.startTranslateY + dy;
      updateTransform();
    }
  });

  document.addEventListener('mouseup', () => {
    if (state.isDragging) {
      state.isDragging = false;
      wrapper.style.transition = 'transform 0.1s ease-out';
    }
  });

  // Touch support for mobile
  let touchStartDistance = 0;
  let touchStartScale = 1;

  wrapper.addEventListener('touchstart', (e) => {
    if (e.touches.length === 1) {
      // Single touch - pan
      state.isDragging = true;
      state.dragStartX = e.touches[0].clientX;
      state.dragStartY = e.touches[0].clientY;
      state.startTranslateX = state.translateX;
      state.startTranslateY = state.translateY;
    } else if (e.touches.length === 2) {
      // Two touches - pinch zoom
      const dx = e.touches[1].clientX - e.touches[0].clientX;
      const dy = e.touches[1].clientY - e.touches[0].clientY;
      touchStartDistance = Math.sqrt(dx * dx + dy * dy);
      touchStartScale = state.scale;
    }
    wrapper.style.transition = 'none';
  }, { passive: true });

  wrapper.addEventListener('touchmove', (e) => {
    if (e.touches.length === 1 && state.isDragging) {
      // Pan
      const dx = e.touches[0].clientX - state.dragStartX;
      const dy = e.touches[0].clientY - state.dragStartY;
      state.translateX = state.startTranslateX + dx;
      state.translateY = state.startTranslateY + dy;
      updateTransform();
    } else if (e.touches.length === 2) {
      // Pinch zoom
      const dx = e.touches[1].clientX - e.touches[0].clientX;
      const dy = e.touches[1].clientY - e.touches[0].clientY;
      const distance = Math.sqrt(dx * dx + dy * dy);
      const scale = (distance / touchStartDistance) * touchStartScale;
      state.scale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, scale));
      updateTransform();
    }
  }, { passive: true });

  wrapper.addEventListener('touchend', () => {
    state.isDragging = false;
    wrapper.style.transition = 'transform 0.1s ease-out';
  }, { passive: true });

  // Keyboard shortcuts
  document.addEventListener('keydown', (e) => {
    // Don't trigger if user is typing in an input
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;

    switch (e.key) {
      case '+':
      case '=':
        e.preventDefault();
        zoomIn();
        break;
      case '-':
      case '_':
        e.preventDefault();
        zoomOut();
        break;
      case '0':
        e.preventDefault();
        resetView();
        break;
      case 'ArrowUp':
        e.preventDefault();
        state.translateY += 50;
        updateTransform();
        break;
      case 'ArrowDown':
        e.preventDefault();
        state.translateY -= 50;
        updateTransform();
        break;
      case 'ArrowLeft':
        e.preventDefault();
        state.translateX += 50;
        updateTransform();
        break;
      case 'ArrowRight':
        e.preventDefault();
        state.translateX -= 50;
        updateTransform();
        break;
    }
  });

  // Initialize
  updateZoomInfo();
  console.log('🔍 Zoom & Pan initialized');
}
