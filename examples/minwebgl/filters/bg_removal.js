import { removeBackground } from "https://esm.sh/@imgly/background-removal";

// [TEMP] Show/hide loading overlay
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

export async function removeBg(imageInput) {
  showLoading();
  try {
    const blob = await removeBackground(imageInput);
    hideLoading();
    return blob;
  } catch (e) {
    console.error("JS removeBG wrapper error:", e);
    hideLoading();
    return null;
  }
}

// Hugginface transformers approach

/*
import {
  pipeline,
  RawImage,
  env,
} from "https://cdn.jsdelivr.net/npm/@huggingface/transformers";

env.allowLocalModels = false;
env.allowRemoteModels = true;

export async function removeBg(imageInput) {
  const segmenter = await pipeline("image-segmentation", "briaai/RMBG-1.4", {
    dtype: "q8",
    device: "wasm",
  });

  const output = await segmenter(imageInput);
  const maskRawImage = output[0].mask;

  const originalImage = await RawImage.fromBlob(imageInput);
  return await applyMask(originalImage, maskRawImage);
}

function applyMask(image, maskRawImage) {
  const width = image.width;
  const height = image.height;

  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d");
  ctx.drawImage(image.toCanvas(), 0, 0);
  const originalImageData = ctx.getImageData(0, 0, width, height);

  const maskCanvas = document.createElement("canvas");
  maskCanvas.width = width;
  maskCanvas.height = height;
  const maskCtx = maskCanvas.getContext("2d");
  maskCtx.drawImage(maskRawImage.toCanvas(), 0, 0, width, height);
  const maskImageData = maskCtx.getImageData(0, 0, width, height);

  for (let i = 0; i < originalImageData.data.length; i += 4) {
    const maskValue = maskImageData.data[i];
    originalImageData.data[i + 3] = maskValue;
  }

  ctx.putImageData(originalImageData, 0, 0);

  return new Promise((resolve) => {
    canvas.toBlob((blob) => {
      resolve(blob);
    }, "image/png");
  });
}
*/
