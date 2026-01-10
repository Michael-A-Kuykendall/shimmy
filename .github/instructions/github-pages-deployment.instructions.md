---
applyTo: "**"
---
# GitHub Pages Deployment Instructions

## Overview
Instructions for deploying the Shimmy Vision frontend to GitHub Pages.

## Repository Setup
- Repository: michaelallenkuykendall/shimmy-vision
- Branch: main (or gh-pages for deployment)
- GitHub Pages enabled in repository settings

## Build Process
```bash
cd shimmy-vision  # Assuming frontend code is in this directory
npm install
npm run build
```

## Deployment
```bash
npm run deploy  # If configured with gh-pages package
# OR manually copy dist/ to docs/ and push
```

## Configuration
- Update API endpoints in code:
  - Test: `shimmy-license-webhook-test.michaelallenkuykendall.workers.dev`
  - Live: `shimmy-license-webhook.michaelallenkuykendall.workers.dev`

## Verification
- URL: https://michael-a-kuykendall.github.io/shimmy-vision/
- Check console for errors
- Test purchase flow (redirects to Stripe)

## Previous Usage
- Frontend deployed to test mode
- API endpoints switched manually in code
- Site loads but may need live endpoint update