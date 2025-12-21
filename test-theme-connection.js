#!/usr/bin/env node

/**
 * Theme Connection Test Script
 * Simulates browser discovery and WebSocket connection
 */

const http = require('http');
const WebSocket = require('ws');

const DISCOVERY_PORT = 11430;
const DISCOVERY_ENDPOINT = '/api/discovery';
const THEME_PORT = 5173;

async function testDiscovery() {
  console.log('\n=== PHASE 1: DISCOVERY TEST ===\n');
  
  return new Promise((resolve, reject) => {
    const url = `http://127.0.0.1:${DISCOVERY_PORT}${DISCOVERY_ENDPOINT}`;
    console.log(`🔍 Querying discovery at: ${url}`);
    
    http.get(url, (res) => {
      let data = '';
      
      res.on('data', chunk => {
        data += chunk;
      });
      
      res.on('end', () => {
        try {
          const discovery = JSON.parse(data);
          console.log('✅ Discovery response received');
          console.log(`   Status: ${res.statusCode}`);
          console.log(`   Backends: ${discovery.backends?.length || 0}`);
          
          if (discovery.backends && discovery.backends.length > 0) {
            const backend = discovery.backends[0];
            console.log(`   Backend Port: ${backend.port}`);
            console.log(`   Backend URL: ${backend.url}`);
            console.log(`   Capabilities: ${backend.capabilities?.join(', ')}`);
            resolve(backend.port);
          } else {
            reject(new Error('No backends in discovery response'));
          }
        } catch (e) {
          reject(new Error(`Failed to parse discovery response: ${e.message}`));
        }
      });
    }).on('error', reject);
  });
}

async function testWebSocket(port) {
  console.log('\n=== PHASE 2: WEBSOCKET CONNECTION TEST ===\n');
  
  return new Promise((resolve, reject) => {
    const wsUrl = `ws://127.0.0.1:${port}/ws/console`;
    console.log(`🔌 Connecting to WebSocket: ${wsUrl}`);
    
    try {
      const ws = new WebSocket(wsUrl);
      
      ws.on('open', () => {
        console.log('✅ WebSocket connected');
        
        // Send get_models request
        console.log('📤 Sending get_models request...');
        ws.send(JSON.stringify({ type: 'get_models' }));
      });
      
      ws.on('message', (data) => {
        try {
          const message = JSON.parse(data);
          console.log(`📥 Received message type: ${message.type}`);
          
          if (message.type === 'models_response') {
            console.log(`   Models count: ${message.models?.length || 0}`);
            if (message.models && message.models.length > 0) {
              console.log(`   First model: ${message.models[0].name}`);
            }
            ws.close();
            resolve(true);
          }
        } catch (e) {
          console.error(`❌ Failed to parse message: ${e.message}`);
        }
      });
      
      ws.on('error', (error) => {
        console.error(`❌ WebSocket error: ${error.message}`);
        reject(error);
      });
      
      ws.on('close', () => {
        console.log('🔌 WebSocket closed');
      });
      
      // Timeout after 5 seconds
      setTimeout(() => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.close();
          reject(new Error('WebSocket test timeout'));
        }
      }, 5000);
      
    } catch (error) {
      reject(error);
    }
  });
}

async function testThemeLoad() {
  console.log('\n=== PHASE 3: THEME LOAD TEST ===\n');
  
  return new Promise((resolve, reject) => {
    const url = `http://localhost:${THEME_PORT}/index.html`;
    console.log(`🌐 Loading theme at: ${url}`);
    
    http.get(url, (res) => {
      console.log(`✅ Theme HTTP response: ${res.statusCode}`);
      
      if (res.statusCode === 200 || res.statusCode === 304) {
        resolve(true);
      } else {
        reject(new Error(`Theme returned status ${res.statusCode}`));
      }
    }).on('error', reject);
  });
}

async function runTests() {
  try {
    console.log('🚀 Starting Theme Connection Tests\n');
    
    // Test 1: Discovery
    const backendPort = await testDiscovery();
    
    // Test 2: WebSocket
    await testWebSocket(backendPort);
    
    // Test 3: Theme Load
    await testThemeLoad();
    
    console.log('\n✅ ALL TESTS PASSED\n');
    process.exit(0);
  } catch (error) {
    console.error(`\n❌ TEST FAILED: ${error.message}\n`);
    process.exit(1);
  }
}

runTests();
