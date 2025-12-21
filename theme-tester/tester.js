#!/usr/bin/env node
// tester.js - Minimal Playwright orchestrator
// Just: take screenshot OR click button
// That's it. Nothing else.

import { chromium } from 'playwright';
import path from 'path';
import { fileURLToPath } from 'url';
import fs from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const screenshotDir = path.join(__dirname, 'screenshots');

if (!fs.existsSync(screenshotDir)) {
  fs.mkdirSync(screenshotDir, { recursive: true });
}

const args = process.argv.slice(2);
const action = args[0];
const url = args[1];
const param = args[2];
const param2 = args[3];
const param3 = args[4];

if (!action || !url) {
  console.log(`
Usage:
  node tester.js screenshot <url> [filename]
  node tester.js click <url> <selector>
  node tester.js type-send <url> <input-selector> <send-selector> <text>

Examples:
  node tester.js screenshot http://localhost:8080
  node tester.js screenshot http://localhost:8080 my-screenshot.png
  node tester.js click http://localhost:8080 "button:has-text('CONNECT')"
  node tester.js type-send http://localhost:8080 "input[type='text']" "button:has-text('Send')" "Hello world"
`);
  process.exit(1);
}

// Hard timeout wrapper - kill process after a reasonable upper bound so long-running
// operations (model loading / streaming responses) have time to start.
// Long-running CI or model generations can exceed 15s (common), use 2 minutes here.
const HARD_TIMEOUT = 2 * 60 * 1000; // 120s
setTimeout(() => {
  console.error(`❌ Hard timeout reached (${HARD_TIMEOUT/1000}s) - forcing exit`);
  process.exit(1);
}, HARD_TIMEOUT);

async function main() {
  const browser = await chromium.launch({ 
    headless: true,
    timeout: 10000 // 10s browser launch timeout
  });
  const page = await browser.newPage();
  
  try {
    // Capture console logs
    const logs = [];
    page.on('console', msg => {
      const type = msg.type();
      const text = msg.text();
      logs.push(`[${type}] ${text}`);
    });

    // Use 'domcontentloaded' instead of 'networkidle' to avoid hanging on continuous polling
    // (metrics polling, heartbeats, etc. never stop, so networkidle would hang forever)
    await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30000 });
    
    // Helper to save logs
    const saveLogs = (basename) => {
      const logPath = path.join(screenshotDir, `${basename}.log`);
      fs.writeFileSync(logPath, logs.join('\n'));
      console.log(`📝 Logs: ${logPath}`);
    };

    if (action === 'screenshot') {
      const filename = param || `screenshot-${Date.now()}.png`;
      const basename = path.basename(filename, '.png');
      const filepath = path.join(screenshotDir, filename);
      
      await page.screenshot({ path: filepath, fullPage: true });
      saveLogs(basename);
      console.log(`✅ Screenshot: ${filepath}`);
    } 
    else if (action === 'click') {
      if (!param) {
        console.error('❌ selector required');
        process.exit(1);
      }
      // param2 can be filename now
      const filename = param2 || `after-click-${Date.now()}.png`;
      const basename = path.basename(filename, '.png');
      const filepath = path.join(screenshotDir, filename);

      // Wait for selector to appear before clicking to reduce fragile label failures
      try {
        await page.waitForSelector(param, { timeout: 60000 });
      } catch (e) {
        console.warn(`⚠️ waitForSelector timeout for '${param}'; attempting click anyway`);
      }
      await page.click(param);
      console.log(`✅ Clicked: ${param}`);
      
      await page.screenshot({ path: filepath, fullPage: true });
      saveLogs(basename);
      console.log(`✅ Screenshot: ${filepath}`);
    }
    else if (action === 'type-send') {
      if (!param || !param2 || !param3) {
        console.error('❌ type-send requires: input-selector, send-selector, and text');
        process.exit(1);
      }
      
      const inputSelector = param;
      const sendSelector = param2;
      const text = param3;
      // param4 can be filename
      const filename = args[5] || `after-send-${Date.now()}.png`;
      const basename = path.basename(filename, '.png');
      const filepath = path.join(screenshotDir, filename);
      
      // Wait for input to appear and fill it — avoid typing into missing fields
      try {
        await page.waitForSelector(inputSelector, { timeout: 60000 });
      } catch (e) {
        console.warn(`⚠️ Input selector not found: ${inputSelector} — continuing`);
      }
      // Type text into input
      await page.fill(inputSelector, text);
      console.log(`✅ Typed into ${inputSelector}: "${text}"`);
      
      // Wait for send control to appear and be clickable
      try {
        await page.waitForSelector(sendSelector, { timeout: 60000 });
      } catch (e) {
        console.warn(`⚠️ Send selector not found: ${sendSelector} — attempting click`);
      }
      // Click send button
      await page.click(sendSelector);
      console.log(`✅ Clicked send button: ${sendSelector}`);
      
      // Wait a bit for response to start
      await page.waitForTimeout(2000);
      
      await page.screenshot({ path: filepath, fullPage: true });
      saveLogs(basename);
      console.log(`✅ Screenshot: ${filepath}`);
    }
  } catch (error) {
    console.error(`❌ Error: ${error.message}`);
    throw error; // rethrow so the outer handler exits non-zero and callers can fallback
  } finally {
    try {
      await browser.close();
    } catch (e) {
      // Ignore browser close errors
    }
  }
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(`❌ Fatal error: ${err.message}`);
    process.exit(1);
  });
