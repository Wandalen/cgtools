gsap.registerPlugin(ScrollTrigger);

// =====================================================================
//  Color swatch buttons
// =====================================================================

function makeButtons(container, colors, propName) {
  colors.forEach((color, i) => {
    const [r, g, b] = color.rgb;
    const btn = document.createElement('button');
    btn.className = 'color-btn' + (i === 0 ? ' active' : '');
    btn.style.backgroundColor = `rgb(${r},${g},${b})`;
    btn.title = color.name;
    btn.onclick = () => {
      container.querySelectorAll('.color-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      const arr = window.jewelryParams[propName];
      arr[0] = r / 255;
      arr[1] = g / 255;
      arr[2] = b / 255;
      window.jewelryGui?.controllersRecursive().forEach(c => c.updateDisplay());
    };
    container.appendChild(btn);
  });
}

(function poll() {
  if (window.colorsJson) {
    const { metal, gem } = JSON.parse(window.colorsJson);
    makeButtons(document.getElementById('metal-panel'), metal, 'metal_color');
    makeButtons(document.getElementById('gem-panel'), gem, 'gem_color');
  } else {
    requestAnimationFrame(poll);
  }
})();

// =====================================================================
//  Screenshot
// =====================================================================

async function downloadScreenshot() {
  const params = window.jewelryParams;
  if (!params) return;

  const savedBloom = params.bloom_strength;
  params.bloom_strength = 0;
  await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));

  const canvas = document.getElementById('canvas');
  const offscreen = document.createElement('canvas');
  offscreen.width = canvas.width;
  offscreen.height = canvas.height;
  const ctx = offscreen.getContext('2d');
  ctx.drawImage(canvas, 0, 0);

  const imageData = ctx.getImageData(0, 0, offscreen.width, offscreen.height);
  const data = imageData.data;
  const bgR = data[0], bgG = data[1], bgB = data[2];
  const tolerance = 20;
  for (let i = 0; i < data.length; i += 4) {
    const dr = Math.abs(data[i] - bgR);
    const dg = Math.abs(data[i + 1] - bgG);
    const db = Math.abs(data[i + 2] - bgB);
    if (dr + dg + db < tolerance * 3) data[i + 3] = 0;
  }
  ctx.putImageData(imageData, 0, 0);
  params.bloom_strength = savedBloom;

  const link = document.createElement('a');
  link.download = `jewelry_${Date.now()}.png`;
  link.href = offscreen.toDataURL('image/png');
  link.click();
}

document.getElementById('btn-screenshot').onclick = downloadScreenshot;
document.addEventListener('keydown', e => {
  if ((e.key === 's' || e.key === 'S') && !e.ctrlKey && !e.metaKey) downloadScreenshot();
});

// =====================================================================
//  Camera animation — coordinates from original jewelry_3d_site
// =====================================================================

// Keyframes (eye + center) from the original index.js
const KEY = {
  init:      { eye: [0.6373576,  1.1441559, -0.9127405],  center: [0.55595696, 0.55741394, -1.0331136] },
  hero:      { eye: [0.6858612,  2.7440538, -0.026622068], center: [0.36420232, 0.8480059,  -0.36873266] },
  brilliant: { eye: [-0.40259048, 2.6242757, -0.18104002], center: [-0.23794234, 0.49070162, -0.32702705] },
  choose:    { eye: [-0.39456308, 2.431139,   0.23367776], center: [0.2921338,  0.9732934,  -0.18001612] },
  config:    { eye: [-1.2621417,  4.005461,   1.2621417],  center: [0.0, 0.6, 0.0] },
};

// Camera state object that GSAP animates
const cam = {
  ex: KEY.init.eye[0],    ey: KEY.init.eye[1],    ez: KEY.init.eye[2],
  cx: KEY.init.center[0], cy: KEY.init.center[1], cz: KEY.init.center[2],
};

function applyCam() {
  if (window.renderer) {
    renderer.set_camera(cam.ex, cam.ey, cam.ez, cam.cx, cam.cy, cam.cz);
  }
}

function eyeCenter(key) {
  return {
    ex: key.eye[0], ey: key.eye[1], ez: key.eye[2],
    cx: key.center[0], cy: key.center[1], cz: key.center[2],
  };
}

// --- Wait for WASM renderer to be ready, then play intro ---
(function waitForRenderer() {
  if (!window.renderer) { requestAnimationFrame(waitForRenderer); return; }

  Object.assign(cam, eyeCenter(KEY.init));
  applyCam();

  // 1) Intro animation — fly from init to hero position
  gsap.timeline({
    onUpdate: applyCam,
    onComplete: setupScrollAnimation,
  })
  .to(cam, {
    ...eyeCenter(KEY.hero),
    duration: 4,
    ease: 'power2.inOut',
  });
})();

// 2) Scroll-driven camera animation — single timeline, single ScrollTrigger
function setupScrollAnimation() {
  document.documentElement.style.overflowY = 'auto';
  document.body.style.overflowY = 'auto';

  const scrollTl = gsap.timeline({
    scrollTrigger: {
      trigger: '.scroll-container',
      start: 'top top',
      end: 'bottom bottom',
      scrub: true,
      invalidateOnRefresh: true,
    },
    onUpdate: applyCam,
  });

  // First half of scroll: Hero → Brilliant (ring shifts right)
  scrollTl.to(cam, { ...eyeCenter(KEY.brilliant), duration: 1, ease: 'none' });
  // Second half of scroll: Brilliant → Choose (ring centers)
  scrollTl.to(cam, { ...eyeCenter(KEY.choose), duration: 1, ease: 'none' });

  // Fade scroll hint
  gsap.to('#scroll-hint', {
    opacity: 0,
    scrollTrigger: {
      trigger: '.scroll-container',
      start: 'top top',
      end: '+=200',
      scrub: true,
    },
  });

  ScrollTrigger.refresh();
}

// 3) "Customize" button — transition to interactive configurator mode
document.getElementById('btn-customize').addEventListener('click', () => {
  document.documentElement.style.overflowY = 'hidden';
  document.body.style.overflowY = 'hidden';
  window.scrollTo(0, 0);
  ScrollTrigger.getAll().forEach(st => st.kill());

  document.getElementById('scroll-hint').style.display = 'none';
  document.getElementById('btn-customize').style.display = 'none';

  gsap.to(cam, {
    ...eyeCenter(KEY.config),
    duration: 2.5,
    ease: 'power4.out',
    onUpdate: applyCam,
    onComplete: () => {
      renderer.release_camera();
      document.getElementById('configurator-ui').classList.add('active');
    },
  });
});
