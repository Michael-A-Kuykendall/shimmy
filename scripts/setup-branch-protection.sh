#!/bin/bash
# Simple GitHub Branch Protection Setup
# Protects main branch with essential quality gates

echo "🛡️ Setting up GitHub Branch Protection"
echo "====================================="

# Check if gh CLI is available
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is required but not installed"
    echo "   Install: https://cli.github.com/"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "❌ GitHub CLI not authenticated"
    echo "   Run: gh auth login"
    exit 1
fi

# Get repository information
REPO_OWNER=$(gh repo view --json owner --jq .owner.login)
REPO_NAME=$(gh repo view --json name --jq .name)

echo "📋 Repository: $REPO_OWNER/$REPO_NAME"
echo ""

# Configure main branch protection with essential rules
echo "🔒 Configuring main branch protection..."

gh api repos/$REPO_OWNER/$REPO_NAME/branches/main/protection \
  --method PUT \
  --input - << 'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["CI", "DCO"]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": {
    "required_approving_review_count": 1,
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false
  },
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false
}
EOF

if [ $? -eq 0 ]; then
    echo "✅ Main branch protection configured successfully"
else
    echo "❌ Failed to configure branch protection"
    echo "   Note: This requires admin permissions on the repository"
    exit 1
fi

echo ""
echo "📋 Protection Rules Applied:"
echo "  ✅ Require pull request reviews (1 approval minimum)"
echo "  ✅ Dismiss stale reviews on new commits"
echo "  ✅ Require CI checks to pass"
echo "  ✅ Require DCO sign-off on commits"
echo "  ✅ No force pushes to main"
echo "  ✅ No deletion of main branch"
echo "  ✅ Maintainer can bypass (for emergency fixes)"
echo ""
echo "🎯 Essential quality gates now enforced!"
echo ""
echo "💡 What this means:"
echo "  • All changes must go through pull requests"
echo "  • CI must pass before merging"
echo "  • All commits must be signed off (DCO)"
echo "  • Code review required for all changes"
echo "  • Emergency fixes possible via maintainer bypass"