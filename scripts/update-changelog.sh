#!/bin/bash
# Manual Changelog Update Script
# Updates CHANGELOG.md with new version entry

set -e

echo "📝 Shimmy Changelog Updater"
echo "=========================="

# Check if version provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <version> [release-notes-file]"
    echo "Example: $0 1.3.3 release-notes.md"
    echo "Example: $0 1.3.3 (will prompt for release notes)"
    exit 1
fi

VERSION="$1"
RELEASE_NOTES_FILE="$2"
DATE=$(date +%Y-%m-%d)

echo "📋 Version: $VERSION"
echo "📅 Date: $DATE"
echo ""

# Get release notes
if [ -n "$RELEASE_NOTES_FILE" ] && [ -f "$RELEASE_NOTES_FILE" ]; then
    echo "📖 Reading release notes from: $RELEASE_NOTES_FILE"
    RELEASE_NOTES=$(cat "$RELEASE_NOTES_FILE")
else
    echo "✏️ Enter release notes (press Ctrl+D when done):"
    echo "   Use standard format: ### Added, ### Changed, ### Fixed, etc."
    echo ""
    RELEASE_NOTES=$(cat)
fi

echo ""
echo "🔍 Preview of changelog entry:"
echo "=============================="
echo "## [$VERSION] - $DATE"
echo ""
echo "$RELEASE_NOTES"
echo ""
echo "=============================="
echo ""

read -p "👍 Does this look correct? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Cancelled"
    exit 1
fi

# Create backup
cp CHANGELOG.md CHANGELOG.md.bak
echo "💾 Created backup: CHANGELOG.md.bak"

# Create temporary file with new entry
cat > new_entry.md << EOF
## [$VERSION] - $DATE

$RELEASE_NOTES

EOF

# Insert new entry after "## [Unreleased]" line
awk '
/^## \[Unreleased\]/ {
    print $0
    print ""
    while ((getline line < "new_entry.md") > 0) {
        print line
    }
    close("new_entry.md")
    next
}
{print}
' CHANGELOG.md > CHANGELOG_new.md

# Replace original file
mv CHANGELOG_new.md CHANGELOG.md
rm -f new_entry.md

# Add version link at the end
echo "" >> CHANGELOG.md
echo "[$VERSION]: https://github.com/Michael-A-Kuykendall/shimmy/releases/tag/v$VERSION" >> CHANGELOG.md

echo "✅ CHANGELOG.md updated successfully!"
echo ""
echo "🔍 Diff of changes:"
echo "==================="
git diff --no-index CHANGELOG.md.bak CHANGELOG.md || true
echo ""
echo "💡 Next steps:"
echo "  1. Review the changes: git diff CHANGELOG.md"
echo "  2. Commit the changes: git add CHANGELOG.md && git commit -m 'docs: Update CHANGELOG.md for v$VERSION'"
echo "  3. Or restore backup: mv CHANGELOG.md.bak CHANGELOG.md"