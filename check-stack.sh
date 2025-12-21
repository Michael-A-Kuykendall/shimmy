#!/bin/bash
# Standardized stack status checker

echo "═══════════════════════════════════════════════════════"
echo "🔍 SHIMMY STACK STATUS CHECK"
echo "═══════════════════════════════════════════════════════"

# Check backend
# Prefer discovery API for backend port resolution; fallback to compatibility ephemeral port file when necessary
PORT=$(curl -s http://127.0.0.1:11430/api/discovery 2>/dev/null | jq -r '.backends[0].port // empty')
if [ -z "$PORT" ]; then
    PORT=$(cat <ephemeral-port-file> 2>/dev/null || echo "NOT_RUNNING")
fi
if [ "$PORT" = "NOT_RUNNING" ]; then
    echo "❌ Backend: NOT RUNNING"
else
    echo "✅ Backend: http://127.0.0.1:$PORT"
fi

# Check discovery
BACKENDS=$(curl -s http://127.0.0.1:11430/api/discovery 2>/dev/null | jq -r '.backends | length' 2>/dev/null)
if [ -z "$BACKENDS" ] || [ "$BACKENDS" = "null" ]; then
    echo "❌ Discovery: NOT RESPONDING"
else
    echo "✅ Discovery: $BACKENDS backend(s) registered"
fi

# Check theme
THEME_TITLE=$(curl -s http://localhost:8080 2>/dev/null | grep -o '<title>.*</title>' | sed 's/<[^>]*>//g')
if [ -z "$THEME_TITLE" ]; then
    echo "❌ Theme: NOT RESPONDING"
else
    echo "✅ Theme: $THEME_TITLE"
fi

echo "═══════════════════════════════════════════════════════"
