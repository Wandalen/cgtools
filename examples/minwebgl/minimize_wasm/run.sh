#!/bin/bash
trunk build --release
for i in $(ls dist/*.wasm); do
  echo "Compressing file $i";
  wasm-strip $i
  wasm-opt -Os -o $i $i
  echo "Splitting file $i";
  split -b 16k --numeric-suffixes $i $i
  filename=$(basename ${i})
  echo "Patching index.html for loading splitted chunks instead of single file"
  sed -i -e "s/const wasm = await init('\/${filename}');/const wasmChunks = [\n\
  \"\/${filename}00\",\n\
  \"\/${filename}01\",\n\
  \"\/${filename}02\",\n\
];\n\
\n\
async function loadWasmChunks(urls) {\n\
  const chunks = await Promise.all(\n\
    urls.map((url) => fetch(url).then((res) => res.arrayBuffer()))\n\
  );\n\
\n\
  const totalSize = chunks.reduce(\n\
    (acc, chunk) => acc + chunk.byteLength,\n\
    0\n\
  );\n\
  const fullBuffer = new Uint8Array(totalSize);\n\
\n\
  let offset = 0;\n\
  for (const chunk of chunks) {\n\
    fullBuffer.set(new Uint8Array(chunk), offset);\n\
    offset += chunk.byteLength;\n\
  }\n\
\n\
  console.log(\"WASM file has been joined successfully\");\n\
  return fullBuffer.buffer;\n\
}\n\
\n\
async function bufferToResponse() {\n\
  const wasmBuffer = await loadWasmChunks(wasmChunks);\n\
\n\
  const response = new Response(wasmBuffer, {\n\
    headers: { \"Content-Type\": \"application\/wasm\" },\n\
  });\n\
\n\
  console.log(\"Response has been created:\", response);\n\
  return response;\n\
}\n\
let response = await bufferToResponse();\n\
const wasm = await init(response);/g" dist/index.html
  echo "Removing link to single file from index.html"
  sed -i -e "s/<link rel=\"preload\" href=\"\/${filename}\".*type=\"application\/wasm\">//g" dist/index.html
done
