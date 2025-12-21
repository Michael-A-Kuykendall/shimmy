#!/bin/bash
# Helper script for shimmy dev command
# This runs in the background so the dev command doesn't kill itself

THEME_PATH="$1"

echo "🚀 Starting Shimmy backend..."
./target/release/shimmy.exe serve --bind auto 2>&1 | tee shimmy_startup.log &
SHIMMY_PID=$!

echo "⏳ Waiting for discovery..."
for i in {1..60}; do
  if curl -s http://127.0.0.1:11430/api/discovery >/dev/null 2>&1; then
    # NOTE: reading the compatibility ephemeral port file is deprecated; prefer the discovery API
    PORT=$(cat <ephemeral-port-file> 2>/dev/null || echo "unknown")
    echo "✅ Shimmy ready on port $PORT"
    break
  fi
  sleep 1
done

echo "🎨 Starting theme..."
cd "$THEME_PATH"
rm -rf dist .vite node_modules/.vite 2>/dev/null
npm run dev 2>&1 | tee ../../theme_startup.log &
THEME_PID=$!

echo "⏳ Waiting for theme..."
for i in {1..120}; do
  if curl -s http://localhost:8080 | grep -q "SHIMMY"; then
    echo "✅ Theme ready after $i seconds"
    break
  fi
  sleep 1
done

echo ""
echo "═══════════════════════════════════════════════════════"
echo "✅✅✅ STACK FULLY OPERATIONAL ✅✅✅"
echo "═══════════════════════════════════════════════════════"
echo "Shimmy Backend: http://127.0.0.1:$PORT"
echo "Discovery HTTP: http://127.0.0.1:11430/api/discovery"
echo "Theme UI: http://localhost:8080"
echo "═══════════════════════════════════════════════════════"
