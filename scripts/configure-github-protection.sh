#!/bin/bash
# Configure GitHub Branch Protection and Quality Gates
# Sets up professional merge protection for main branch

echo "🛡️ Configuring GitHub Branch Protection"
echo "======================================="

# Check if gh CLI is available
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is required but not installed"
    echo "   Install from: https://cli.github.com/"
    echo "   Or run: winget install GitHub.cli"
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

# Configure main branch protection
echo "🔒 Configuring main branch protection..."
gh api repos/$REPO_OWNER/$REPO_NAME/branches/main/protection \
  --method PUT \
  --field required_status_checks='{"strict":true,"contexts":["PPT Contract Tests","Test Suite","Code Coverage","Security Audit","Code Quality","Build Verification","Professional Quality Gate"]}' \
  --field enforce_admins=true \
  --field required_pull_request_reviews='{"required_approving_review_count":1,"dismiss_stale_reviews":true,"require_code_owner_reviews":false}' \
  --field restrictions=null \
  --field allow_force_pushes=false \
  --field allow_deletions=false

if [ $? -eq 0 ]; then
    echo "✅ Main branch protection configured successfully"
else
    echo "❌ Failed to configure branch protection"
    echo "   Note: This requires admin permissions on the repository"
    exit 1
fi

echo ""
echo "📋 Professional Quality Gates Configured:"
echo "  ✅ PPT Contract Tests - Critical quality gate"
echo "  ✅ Test Suite - Comprehensive test coverage"
echo "  ✅ Code Coverage - Professional standards (≥95%)"
echo "  ✅ Security Audit - Vulnerability scanning"
echo "  ✅ Code Quality - Formatting and linting"
echo "  ✅ Build Verification - Cross-platform builds"
echo "  ✅ Quality Gate Summary - Final validation"
echo ""
echo "🛡️ Branch Protection Rules:"
echo "  • Require PR reviews (minimum 1 approval)"
echo "  • Dismiss stale reviews on new commits"
echo "  • Require status checks to be up to date"
echo "  • No force pushes allowed"
echo "  • No branch deletions allowed"
echo "  • Enforce restrictions for administrators"
echo ""
echo "🎯 Professional development workflow now enforced!"
echo ""
echo "💡 Next steps:"
echo "  1. All pull requests must pass quality gates"
echo "  2. Use './scripts/dev-test.sh' locally before pushing"
echo "  3. Pre-commit hooks will catch issues early"
echo "  4. Coverage reports available via Codecov integration"