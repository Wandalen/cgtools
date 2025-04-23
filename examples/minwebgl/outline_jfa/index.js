import init, { App } from './pkg/outline_jfa.js';

async function run() {
    await init();

    const canvas = document.getElementById('canvas');
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    try {
        const app = new App('canvas', canvas.width, canvas.height);

        function renderLoop(timestamp) {
            app.tick(timestamp);
            requestAnimationFrame(renderLoop);
        }

        requestAnimationFrame(renderLoop);

        window.addEventListener('resize', () => {
            canvas.width = window.innerWidth;
            canvas.height = window.innerHeight;
            app.resize(canvas.width, canvas.height);
        });

    } catch (error) {
        console.error("Failed to initialize WasmApp:", error);
    }
}

run();