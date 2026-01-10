---
applyTo: "**"
---
# Post-Launch Monitoring Instructions

## Overview
Instructions for monitoring Shimmy Vision after production launch.

## Key Metrics to Monitor
- API response times
- Error rates
- License creation success
- Payment completion rates
- User feedback

## Tools
- Stripe dashboard (payments, webhooks)
- Keygen dashboard (licenses)
- Cloudflare Worker logs (`wrangler tail`)
- GitHub Pages analytics
- User support channels

## Alert Setup
- Webhook failures
- API timeouts
- High error rates
- Payment declines

## Response Protocols
- Critical issues: Immediate rollback
- Minor issues: Monitor and fix
- User reports: Respond within 24 hours

## Timeframes
- First 24 hours: Hourly checks
- First week: Daily checks
- Ongoing: Weekly reviews

## Previous Usage
- No monitoring setup established
- Manual checks performed
- No automated alerts configured