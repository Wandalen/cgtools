#!/bin/bash

# Headless execution smoke for built browser examples (terminal harness, option C).
#
# Per example: trunk-build the crate, serve its dist/ on an ephemeral port,
# load the page in headless chromium via `browsee .run`, and fail on wasm
# panics or uncaught JS exceptions in the piped console. This is execution
# proof (page boots, wasm runs, no panics) — not pixel proof: WebGPU frame
# presentation is impossible in headless chromium on this host (see
# examples/scene_script/sun_grid_lines/readme.md), and WebGL2 runs on
# SwiftShader, so visual verdicts stay with windowed `browsee .launch`
# sessions and per-example showcase images.
#
# usage:
#   script/example_smoke.sh [example_dir ...]
# defaults: both sun_grid_lines twins plus the canonical WebGPU-path pair
# (hello_triangle, renderer_pbr_scene)
#
# Verify any verdict yourself by replaying the printed browsee line, e.g.:
#   browsee .run http://127.0.0.1:<port>/ features::software_gl timeout::40 dom::1

set -u

root=$( cd "$( dirname "$0" )/.." && pwd )
cd "$root" || exit 2

examples=( "$@" )
if [ "${#examples[@]}" -eq 0 ]
then
  examples=(
    examples/minwebgl/sun_grid_lines
    examples/scene_script/sun_grid_lines
    examples/minwebgpu/hello_triangle
    examples/minwebgpu/renderer_pbr_scene
  )
fi

command -v browsee > /dev/null || { echo "example_smoke: browsee not on PATH" >&2 ; exit 2 ; }
command -v trunk > /dev/null || { echo "example_smoke: trunk not on PATH (cargo install trunk --locked)" >&2 ; exit 2 ; }

failures=0
for dir in "${examples[@]}"
do
  name=$( basename "$( dirname "$dir" )" )/$( basename "$dir" )
  if [ ! -f "$dir/index.html" ]
  then
    echo "❌ $name: no index.html at $dir"
    failures=$(( failures + 1 ))
    continue
  fi

  # minwebgpu-dependent examples need the WebGPU flag preset; everything
  # else renders through WebGL2, which only needs the software-GL opt-in on
  # this host. Detected from the crate's own Cargo.toml, not the parent
  # directory name, so this stays correct across category moves.
  features=software_gl
  grep -q "minwebgpu" "$dir/Cargo.toml" && features=webgpu,software_gl

  echo "=== $name (features::$features) ==="
  if ! ( cd "$dir" && trunk build --release > /dev/null 2>&1 )
  then
    echo "❌ $name: trunk build failed (rerun: cd $dir && trunk build --release)"
    failures=$(( failures + 1 ))
    continue
  fi

  port=$( python3 -c 'import socket; s = socket.socket(); s.bind( ( "127.0.0.1", 0 ) ); print( s.getsockname()[ 1 ] ); s.close()' )
  python3 -m http.server "$port" --bind 127.0.0.1 --directory "$dir/dist" > /dev/null 2>&1 &
  server=$!
  for _ in $( seq 40 )
  do
    curl -sf -o /dev/null "http://127.0.0.1:$port/" && break
    sleep 0.25
  done

  out=$( browsee .run "http://127.0.0.1:$port/" features::"$features" timeout::40 dom::1 2>&1 )
  status=$?
  kill "$server" 2> /dev/null
  wait "$server" 2> /dev/null

  defects=$( printf '%s\n' "$out" | grep -E 'panicked at|Uncaught |RuntimeError' | head -3 )
  if [ "$status" -ne 0 ]
  then
    echo "❌ $name: browsee .run exit $status"
    printf '%s\n' "$out" | tail -5
    failures=$(( failures + 1 ))
  elif [ -n "$defects" ]
  then
    echo "❌ $name: console defects:"
    printf '%s\n' "$defects"
    failures=$(( failures + 1 ))
  elif ! printf '%s' "$out" | grep -qi '<canvas'
  then
    echo "❌ $name: page produced no <canvas> in DOM"
    failures=$(( failures + 1 ))
  else
    echo "✅ $name: booted headless, no panics, canvas present"
  fi
done

exit $(( failures > 0 ))
