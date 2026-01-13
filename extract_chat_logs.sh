#!/bin/bash

# Script to extract chat logs from Copilot session files
# Usage: ./extract_chat_logs.sh <session_file> <date> <session_id>

SESSION_FILE="$1"
DATE="$2"
SESSION_ID="$3"

echo "## Session: $DATE - $SESSION_ID"
echo ""

# Extract messages using jq
jq -r '.requests[] | 
  "# Human\n" + (.message.text | sub("\r\n"; "\n"; "g") | sub("\r"; ""; "g")) + "\n",
  if (.response | length > 0) then
    "# Assistant\n" + (.response[0].value | sub("\r\n"; "\n"; "g") | sub("\r"; ""; "g")) + "\n"
  else
    ""
  end' "$SESSION_FILE" 2>/dev/null || echo "Error processing $SESSION_FILE"

echo "---"
echo ""
