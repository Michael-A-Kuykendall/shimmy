#!/bin/bash

# Script to extract chat logs from Copilot session files using grep/sed
# Usage: ./extract_chat_logs_v2.sh <session_file> <date> <session_id>

SESSION_FILE="$1"
DATE="$2"
SESSION_ID="$3"

echo "## Session: $DATE - $SESSION_ID"
echo ""

# Extract human messages
echo "# Human Messages:"
grep -A 5 '"text":' "$SESSION_FILE" | grep '"text":' | sed 's/.*"text": "\(.*\)".*/\1/' | sed 's/\\r\\n/\n/g' | sed 's/\\n/\n/g' | sed 's/\\"/"/g' | head -10
echo ""

# Extract assistant responses  
echo "# Assistant Responses:"
grep -A 10 '"value":' "$SESSION_FILE" | grep '"value":' | sed 's/.*"value": "\(.*\)".*/\1/' | sed 's/\\r\\n/\n/g' | sed 's/\\n/\n/g' | sed 's/\\"/"/g' | head -10
echo ""

echo "---"
echo ""
