#!/usr/bin/env node

/**
 * Full Theme Shakedown Test
 * Tests all phases: Discovery, WebSocket, Chat, Metrics, Tools
 */

const http = require('http');
const WebSocket = require('ws');

const DISCOVERY_PORT = 11430;
const DISCOVERY_ENDPOINT = '/api/discovery';
const THEME_PORT = 5173;

let backendPort = null;
let ws = null;

async function testDiscovery() {
  console.log('\n=== PHASE 1: DISCOVERY ===\n');
  
  return new Promise((resolve, reject) => {
    const url = `http://127.0.0.1:${DISCOVERY_PORT}${DISCOVERY_ENDPOINT}`;
    console.log(`🔍 Querying: ${url}`);
    
    http.get(url, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const discovery = JSON.parse(data);
          if (discovery.backends && discovery.backends.length > 0) {
            backendPort = discovery.backends[0].port;
            console.log(`✅ Discovery successful`);
            console.log(`   Backend port: ${backendPort}`);
            console.log(`   Capabilities: ${discovery.backends[0].capabilities?.join(', ')}`);
            resolve(true);
          } else {
            reject(new Error('No backends in discovery'));
          }
        } catch (e) {
          reject(e);
        }
      });
    }).on('error', reject);
  });
}

async function testWebSocketConnection() {
  console.log('\n=== PHASE 2: WEBSOCKET CONNECTION ===\n');
  
  return new Promise((resolve, reject) => {
    const wsUrl = `ws://127.0.0.1:${backendPort}/ws/console`;
    console.log(`🔌 Connecting to: ${wsUrl}`);
    
    ws = new WebSocket(wsUrl);
    
    ws.on('open', () => {
      console.log('✅ WebSocket connected');
      resolve(true);
    });
    
    ws.on('error', (error) => {
      console.error(`❌ WebSocket error: ${error.message}`);
      reject(error);
    });
  });
}

async function testGetModels() {
  console.log('\n=== PHASE 3: GET MODELS ===\n');
  
  return new Promise((resolve, reject) => {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
      reject(new Error('WebSocket not connected'));
      return;
    }
    
    console.log('📤 Sending: get_models');
    ws.send(JSON.stringify({ type: 'get_models' }));
    
    const timeout = setTimeout(() => {
      reject(new Error('get_models timeout'));
    }, 5000);
    
    const handler = (data) => {
      try {
        const message = JSON.parse(data);
        if (message.type === 'models_response') {
          clearTimeout(timeout);
          ws.removeEventListener('message', handler);
          console.log(`✅ Models received: ${message.models?.length || 0} models`);
          if (message.models && message.models.length > 0) {
            console.log(`   First model: ${message.models[0].name}`);
          }
          resolve(message.models);
        }
      } catch (e) {
        // Ignore parse errors
      }
    };
    
    ws.on('message', handler);
  });
}

async function testSelectModel(models) {
  console.log('\n=== PHASE 4: SELECT MODEL ===\n');
  
  if (!models || models.length === 0) {
    console.log('⚠️  No models available, skipping');
    return null;
  }
  
  return new Promise((resolve, reject) => {
    const modelName = models[0].name;
    console.log(`📤 Sending: select_model (${modelName})`);
    ws.send(JSON.stringify({ type: 'select_model', model_name: modelName }));
    
    const timeout = setTimeout(() => {
      reject(new Error('select_model timeout'));
    }, 5000);
    
    const handler = (data) => {
      try {
        const message = JSON.parse(data);
        if (message.type === 'model_selected') {
          clearTimeout(timeout);
          ws.removeEventListener('message', handler);
          console.log(`✅ Model selected: ${message.model_name}`);
          resolve(modelName);
        }
      } catch (e) {
        // Ignore parse errors
      }
    };
    
    ws.on('message', handler);
  });
}

async function testChatMessage(modelName) {
  console.log('\n=== PHASE 5: CHAT MESSAGE ===\n');
  
  if (!modelName) {
    console.log('⚠️  No model selected, skipping');
    return;
  }
  
  return new Promise((resolve, reject) => {
    const message = 'Hello, what is 2+2?';
    console.log(`📤 Sending: chat ("${message}")`);
    ws.send(JSON.stringify({ type: 'chat', message }));
    
    let response = '';
    let tokenCount = 0;
    const timeout = setTimeout(() => {
      reject(new Error('chat timeout'));
    }, 120000);
    
    const handler = (data) => {
      try {
        const msg = JSON.parse(data);
        if (msg.type === 'chat_token') {
          tokenCount++;
          response += msg.token || '';
          if (tokenCount % 10 === 0) {
            process.stdout.write('.');
          }
        } else if (msg.type === 'generation_complete') {
          clearTimeout(timeout);
          ws.removeEventListener('message', handler);
          console.log(`\n✅ Chat complete`);
          console.log(`   Tokens: ${tokenCount}`);
          console.log(`   Response: "${response.substring(0, 100)}${response.length > 100 ? '...' : ''}"`);
          resolve(response);
        }
      } catch (e) {
        // Ignore parse errors
      }
    };
    
    ws.on('message', handler);
  });
}

async function testMetrics() {
  console.log('\n=== PHASE 6: METRICS ===\n');
  
  return new Promise((resolve, reject) => {
    const url = `http://127.0.0.1:${backendPort}/api/metrics`;
    console.log(`📊 Querying: ${url}`);
    
    http.get(url, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const metrics = JSON.parse(data);
          console.log('✅ Metrics received:');
          console.log(`   CPU: ${metrics.cpu_percent?.toFixed(1)}%`);
          console.log(`   Memory: ${metrics.memory_mb?.toFixed(0)}MB`);
          console.log(`   Tokens/sec: ${metrics.tokens_per_second?.toFixed(2)}`);
          resolve(metrics);
        } catch (e) {
          reject(e);
        }
      });
    }).on('error', reject);
  });
}

async function testThemeLoad() {
  console.log('\n=== PHASE 7: THEME LOAD ===\n');
  
  return new Promise((resolve, reject) => {
    const url = `http://localhost:${THEME_PORT}/index.html`;
    console.log(`🌐 Loading: ${url}`);
    
    http.get(url, (res) => {
      if (res.statusCode === 200) {
        console.log('✅ Theme loaded successfully');
        resolve(true);
      } else {
        reject(new Error(`Theme returned ${res.statusCode}`));
      }
    }).on('error', reject);
  });
}

async function runFullShakedown() {
  try {
    console.log('🚀 STARTING FULL THEME SHAKEDOWN\n');
    
    // Phase 1: Discovery
    await testDiscovery();
    
    // Phase 2: WebSocket Connection
    await testWebSocketConnection();
    
    // Phase 3: Get Models
    const models = await testGetModels();
    
    // Phase 4: Select Model
    const selectedModel = await testSelectModel(models);
    
    // Phase 5: Chat Message
    await testChatMessage(selectedModel);
    
    // Phase 6: Metrics
    await testMetrics();
    
    // Phase 7: Theme Load
    await testThemeLoad();
    
    console.log('\n✅ ALL PHASES PASSED\n');
    
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.close();
    }
    
    process.exit(0);
  } catch (error) {
    console.error(`\n❌ SHAKEDOWN FAILED: ${error.message}\n`);
    
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.close();
    }
    
    process.exit(1);
  }
}

runFullShakedown();
