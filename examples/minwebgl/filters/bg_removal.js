// Show/hide loading overlay
function showLoading() {
  let overlay = document.getElementById("bg-loading-overlay");
  if (!overlay) {
    overlay = document.createElement("div");
    overlay.id = "bg-loading-overlay";
    overlay.innerHTML = `<div class="bg-loading-text">Removing background...</div>`;
    document.body.appendChild(overlay);
  }
  overlay.classList.add("visible");
}

function hideLoading() {
  const overlay = document.getElementById("bg-loading-overlay");
  if (overlay) {
    overlay.classList.remove("visible");
  }
}

// Worker code as string for inline creation
const workerCode = `
import { removeBackground } from "https://esm.sh/@imgly/background-removal@1.4.5";

self.onmessage = async (e) => {
  const { blob } = e.data;
  try {
    const resultBlob = await removeBackground(blob);
    self.postMessage({ success: true, blob: resultBlob });
  } catch (error) {
    console.error("Worker: background removal error:", error);
    self.postMessage({ success: false, error: error.message });
  }
};
`;

// Lazy-init worker
let worker = null;
function getWorker() {
  if (!worker) {
    const blob = new Blob([workerCode], { type: "application/javascript" });
    const url = URL.createObjectURL(blob);
    worker = new Worker(url, { type: "module" });
    URL.revokeObjectURL(url);
  }
  return worker;
}

export function removeBg(imageInput) {
  showLoading();

  return new Promise((resolve) => {
    const w = getWorker();

    // Fix(UX/DX-9): a worker-level error (thrown outside `self.onmessage`'s own
    // try/catch -- e.g. the `import` at the top of `workerCode` failing because the
    // esm.sh CDN is unreachable) fires an "error" event on the worker, not a
    // "message" event. Without this listener, `errorHandler` below never ran,
    // `handler` never fired either, `hideLoading()` was never called (overlay stuck
    // visible forever), and this Promise never resolved -- hanging the Rust-side
    // `await` in `bg_removal_bindgen::image_process` and leaving `is_processing`
    // stuck `true` forever (blocking all further background-removal attempts, see
    // `main.rs`'s `is_processing` guard).
    // Root cause: only the async message handler's own internal errors were caught;
    // errors originating outside it (module load, syntax errors) were unhandled.
    // Pitfall: a Worker's top-level/import errors surface as an "error" event on the
    // Worker object, never as a "message" -- a message-only handler is blind to them.
    const errorHandler = (e) => {
      w.removeEventListener("message", handler);
      w.removeEventListener("error", errorHandler);
      hideLoading();
      console.error("Background removal worker error:", e.message || e);
      resolve(null);
    };

    const handler = (e) => {
      w.removeEventListener("message", handler);
      w.removeEventListener("error", errorHandler);
      hideLoading();

      if (e.data.success) {
        resolve(e.data.blob);
      } else {
        console.error("Background removal failed:", e.data.error);
        resolve(null);
      }
    };

    w.addEventListener("message", handler);
    w.addEventListener("error", errorHandler);
    w.postMessage({ blob: imageInput });
  });
}
