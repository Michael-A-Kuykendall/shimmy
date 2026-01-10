#!/usr/bin/env bash
# Wait for v1.9.0-test build to complete, then show download command

REPO="Michael-A-Kuykendall/shimmy-private"

echo "⏳ Waiting for v1.9.0-test build to complete..."
echo ""

while true; do
    STATUS=$(gh run list --repo "$REPO" --workflow=release.yml --limit 1 --json status | jq -r '.[].status')
    
    if [[ "$STATUS" == "completed" ]]; then
        CONCLUSION=$(gh run list --repo "$REPO" --workflow=release.yml --limit 1 --json conclusion | jq -r '.[].conclusion')
        
        if [[ "$CONCLUSION" == "success" ]]; then
            printf "\n✅ BUILD SUCCESS!\n\n"
            printf "Download binaries:\n"
            printf "  gh release download v1.9.0-test --repo %s\n\n" "$REPO"
            printf "Or download to test directory:\n"
            printf "  mkdir -p test-binaries && cd test-binaries\n"
            printf "  gh release download v1.9.0-test --repo %s\n\n" "$REPO"
            exit 0
        else
            printf "\n❌ BUILD FAILED: %s\n\n" "$CONCLUSION"
            gh run list --repo "$REPO" --workflow=release.yml --limit 1 --json url | jq -r '.[].url'
            exit 1
        fi
    fi
    
    printf "."
    sleep 60  # Check every minute
done
