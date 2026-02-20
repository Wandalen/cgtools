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

    const handler = (e) => {
      w.removeEventListener("message", handler);
      hideLoading();

      if (e.data.success) {
        resolve(e.data.blob);
      } else {
        console.error("Background removal failed:", e.data.error);
        resolve(null);
      }
    };

    w.addEventListener("message", handler);
    w.postMessage({ blob: imageInput });
  });
}
