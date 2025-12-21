#!/usr/bin/env node
/**
 * Persistent Playwright Session Server
 * 
 * Keeps a browser session alive and accepts commands via HTTP.
 * This allows multi-step interactions without losing page state.
 * 
 * Usage:
 *   Start server:  node session-server.js start <url> [port]
 *   
 * Commands via HTTP (default port 9222):
 *   GET  /screenshot?name=foo.png
 *   POST /click       { "selector": "text=TinyLlama" }
 *   POST /type        { "selector": "textarea", "text": "Hello" }
 *   POST /click-type  { "clickSelector": "textarea", "text": "Hello", "sendSelector": "button" }
 *   GET  /status
 *   POST /navigate    { "url": "http://localhost:8080/chat" }
 *   POST /stop
 */

import { chromium } from 'playwright';
import http from 'http';
import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const screenshotDir = path.join(__dirname, 'screenshots');

if (!fs.existsSync(screenshotDir)) {
  fs.mkdirSync(screenshotDir, { recursive: true });
}

const args = process.argv.slice(2);
const command = args[0];
const targetUrl = args[1] || 'http://localhost:8080';
const serverPort = parseInt(args[2]) || 9222;

if (command !== 'start') {
  console.log(`
Persistent Playwright Session Server

Usage:
  node session-server.js start <url> [port]

Examples:
  node session-server.js start http://localhost:8080
  node session-server.js start http://localhost:8080 9222

Then send commands via HTTP:
  curl http://localhost:9222/screenshot?name=test.png
  curl -X POST http://localhost:9222/click -d '{"selector":"text=TinyLlama"}'
  curl -X POST http://localhost:9222/type -d '{"selector":"textarea","text":"Hello"}'
  curl http://localhost:9222/status
  curl -X POST http://localhost:9222/stop
`);
  process.exit(1);
}

let browser = null;
let page = null;
let consoleLogs = [];

async function initBrowser() {
  console.log(`🚀 Starting browser session for: ${targetUrl}`);
  
  browser = await chromium.launch({ 
    headless: true,
    timeout: 30000
  });
  
  page = await browser.newPage();
  
  // Capture console logs
  page.on('console', msg => {
    const entry = `[${msg.type()}] ${msg.text()}`;
    consoleLogs.push(entry);
    // Keep only last 100 logs
    if (consoleLogs.length > 100) consoleLogs.shift();
  });
  
  await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });
  console.log(`✅ Browser ready at: ${targetUrl}`);
}

function sendJson(res, statusCode, data) {
  res.writeHead(statusCode, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify(data, null, 2));
}

function parseBody(req) {
  return new Promise((resolve, reject) => {
    let body = '';
    req.on('data', chunk => body += chunk);
    req.on('end', () => {
      try {
        resolve(body ? JSON.parse(body) : {});
      } catch (e) {
        reject(new Error('Invalid JSON'));
      }
    });
    req.on('error', reject);
  });
}

async function handleRequest(req, res) {
  const url = new URL(req.url, `http://localhost:${serverPort}`);
  const pathname = url.pathname;
  
  try {
    // GET /status
    if (pathname === '/status' && req.method === 'GET') {
      sendJson(res, 200, {
        success: true,
        browserOpen: !!browser,
        pageUrl: page ? page.url() : null,
        recentLogs: consoleLogs.slice(-10)
      });
      return;
    }
    
    // GET /screenshot?name=foo.png
    if (pathname === '/screenshot' && req.method === 'GET') {
      const name = url.searchParams.get('name') || `screenshot-${Date.now()}.png`;
      const filepath = path.join(screenshotDir, name);
      
      await page.screenshot({ path: filepath, fullPage: true });
      
      // Save logs
      const logPath = path.join(screenshotDir, name.replace('.png', '.log'));
      fs.writeFileSync(logPath, consoleLogs.join('\n'));
      
      sendJson(res, 200, {
        success: true,
        screenshot: filepath,
        logs: logPath
      });
      return;
    }
    
    // GET /logs
    if (pathname === '/logs' && req.method === 'GET') {
      sendJson(res, 200, {
        success: true,
        logs: consoleLogs
      });
      return;
    }
    
    // POST /click { selector }
    if (pathname === '/click' && req.method === 'POST') {
      const body = await parseBody(req);
      const { selector, timeout = 10000 } = body;
      
      if (!selector) {
        sendJson(res, 400, { success: false, error: 'selector required' });
        return;
      }
      
      await page.waitForSelector(selector, { timeout });
      await page.click(selector);
      
      // Brief wait for any navigation/state change
      await page.waitForTimeout(500);
      
      sendJson(res, 200, {
        success: true,
        action: 'click',
        selector,
        currentUrl: page.url()
      });
      return;
    }
    
    // POST /type { selector, text }
    if (pathname === '/type' && req.method === 'POST') {
      const body = await parseBody(req);
      const { selector, text, timeout = 10000 } = body;
      
      if (!selector || text === undefined) {
        sendJson(res, 400, { success: false, error: 'selector and text required' });
        return;
      }
      
      await page.waitForSelector(selector, { timeout });
      await page.fill(selector, text);
      
      sendJson(res, 200, {
        success: true,
        action: 'type',
        selector,
        text
      });
      return;
    }
    
    // POST /press { key } - press a key like Enter
    if (pathname === '/press' && req.method === 'POST') {
      const body = await parseBody(req);
      const { key } = body;
      
      if (!key) {
        sendJson(res, 400, { success: false, error: 'key required' });
        return;
      }
      
      await page.keyboard.press(key);
      await page.waitForTimeout(500);
      
      sendJson(res, 200, {
        success: true,
        action: 'press',
        key
      });
      return;
    }
    
    // POST /type-and-send { selector, text, sendSelector } - type and click send
    if (pathname === '/type-and-send' && req.method === 'POST') {
      const body = await parseBody(req);
      const { selector, text, sendSelector, timeout = 10000 } = body;
      
      if (!selector || text === undefined || !sendSelector) {
        sendJson(res, 400, { success: false, error: 'selector, text, and sendSelector required' });
        return;
      }
      
      await page.waitForSelector(selector, { timeout });
      await page.fill(selector, text);
      
      await page.waitForSelector(sendSelector, { timeout });
      await page.click(sendSelector);
      
      // Wait a bit for response to start
      await page.waitForTimeout(1000);
      
      sendJson(res, 200, {
        success: true,
        action: 'type-and-send',
        selector,
        text,
        sendSelector
      });
      return;
    }
    
    // POST /wait { selector, timeout } - wait for element
    if (pathname === '/wait' && req.method === 'POST') {
      const body = await parseBody(req);
      const { selector, timeout = 60000 } = body;
      
      if (!selector) {
        sendJson(res, 400, { success: false, error: 'selector required' });
        return;
      }
      
      await page.waitForSelector(selector, { timeout });
      
      sendJson(res, 200, {
        success: true,
        action: 'wait',
        selector
      });
      return;
    }
    
    // POST /navigate { url }
    if (pathname === '/navigate' && req.method === 'POST') {
      const body = await parseBody(req);
      const { url: navUrl } = body;
      
      if (!navUrl) {
        sendJson(res, 400, { success: false, error: 'url required' });
        return;
      }
      
      await page.goto(navUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });
      
      sendJson(res, 200, {
        success: true,
        action: 'navigate',
        url: navUrl
      });
      return;
    }
    
    // POST /stop - shutdown
    if (pathname === '/stop' && req.method === 'POST') {
      sendJson(res, 200, { success: true, message: 'Shutting down...' });
      
      setTimeout(async () => {
        if (browser) await browser.close();
        process.exit(0);
      }, 500);
      return;
    }
    
    // 404
    sendJson(res, 404, { success: false, error: `Unknown endpoint: ${pathname}` });
    
  } catch (error) {
    console.error(`❌ Error handling ${pathname}:`, error.message);
    sendJson(res, 500, { success: false, error: error.message });
  }
}

async function main() {
  await initBrowser();
  
  const server = http.createServer(handleRequest);
  
  server.listen(serverPort, () => {
    console.log(`📡 Session server listening on http://localhost:${serverPort}`);
    console.log(`
Commands:
  curl http://localhost:${serverPort}/status
  curl "http://localhost:${serverPort}/screenshot?name=test.png"
  curl -X POST http://localhost:${serverPort}/click -H "Content-Type: application/json" -d '{"selector":"text=TinyLlama"}'
  curl -X POST http://localhost:${serverPort}/type -H "Content-Type: application/json" -d '{"selector":"textarea","text":"Hello"}'
  curl -X POST http://localhost:${serverPort}/type-and-send -H "Content-Type: application/json" -d '{"selector":"textarea","text":"Hello","sendSelector":"button:has-text(Send)"}'
  curl -X POST http://localhost:${serverPort}/stop
`);
  });
  
  // Graceful shutdown
  process.on('SIGINT', async () => {
    console.log('\n🛑 Shutting down...');
    if (browser) await browser.close();
    server.close();
    process.exit(0);
  });
}

main().catch(err => {
  console.error('❌ Failed to start:', err);
  process.exit(1);
});
