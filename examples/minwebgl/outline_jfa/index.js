import init, { App } from './pkg/outline_jfa.js';

const state = new InputState();

async function run() {
    await init();

    const canvas = document.getElementById('canvas');
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    try {
        const app = new App('canvas', canvas.width, canvas.height);

        function renderLoop(timestamp) {
            state.set_timestamp(timestamp);
            app.tick(state);
            state.reset();
            requestAnimationFrame(renderLoop);
        }

        requestAnimationFrame(renderLoop);

        window.addEventListener("resize", () => {
            canvas.width = window.innerWidth;
            canvas.height = window.innerHeight;
            app.resize(canvas.width, canvas.height);
        });

        window.addEventListener(
            "keydown",
            (event) => {
                state.add_keyboard_event(event);
            },
            false,
        );
        
        window.addEventListener(
            "keyup", 
            (event) => {
                state.add_keyboard_event(event);
            },
            false,
        );

        myPics.addEventListener("mousedown", (event) => {
            state.add_mouse_event(event);
        });
          
        myPics.addEventListener("mousemove", (event) => {
            state.add_mouse_event(event);
        });
        
        window.addEventListener("mouseup", (event) => {
            state.add_mouse_event(event);
        });

    } catch (error) {
        console.error("Failed to initialize WasmApp:", error);
    }
}

run();