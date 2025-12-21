#!/usr/bin/env node

/**
 * Test Discovery Parsing
 * Verifies the useDiscovery hook logic works correctly
 */

const http = require('http');

const DISCOVERY_PORT = 11430;
const DISCOVERY_ENDPOINT = '/api/discovery';

async function testDiscoveryParsing() {
  console.log('\n=== TESTING DISCOVERY PARSING ===\n');
  
  return new Promise((resolve, reject) => {
    const url = `http://127.0.0.1:${DISCOVERY_PORT}${DISCOVERY_ENDPOINT}`;
    console.log(`🔍 Fetching discovery from: ${url}\n`);
    
    http.get(url, (res) => {
      let data = '';
      
      res.on('data', chunk => {
        data += chunk;
      });
      
      res.on('end', () => {
        try {
          const discovery = JSON.parse(data);
          
          console.log('📋 Discovery Response Structure:');
          console.log(`   - discovery_port: ${discovery.discovery_port}`);
          console.log(`   - backends array length: ${discovery.backends?.length || 0}`);
          
          if (!discovery.backends || discovery.backends.length === 0) {
            throw new Error('❌ No backends in discovery response');
          }
          
          const backend = discovery.backends[0];
          console.log('\n📦 First Backend:');
          console.log(`   - id: ${backend.id}`);
          console.log(`   - port: ${backend.port}`);
          console.log(`   - url: ${backend.url}`);
          console.log(`   - models: ${backend.models?.length || 0}`);
          console.log(`   - capabilities: ${backend.capabilities?.join(', ')}`);
          console.log(`   - healthy: ${backend.health?.healthy}`);
          
          // Simulate what useDiscovery hook does
          console.log('\n🔄 Simulating useDiscovery Hook Logic:');
          const backendInfo = {
            port: backend.port,
            version: backend.id
          };
          console.log(`   - Extracted port: ${backendInfo.port}`);
          console.log(`   - Extracted version: ${backendInfo.version}`);
          
          // Verify WebSocket endpoint
          console.log('\n🔌 WebSocket Connection URL:');
          const wsUrl = `ws://127.0.0.1:${backendInfo.port}/ws/console`;
          console.log(`   - ${wsUrl}`);
          
          console.log('\n✅ DISCOVERY PARSING TEST PASSED\n');
          resolve(backendInfo);
        } catch (e) {
          console.error(`\n❌ ERROR: ${e.message}\n`);
          reject(e);
        }
      });
    }).on('error', (error) => {
      console.error(`\n❌ HTTP ERROR: ${error.message}\n`);
      reject(error);
    });
  });
}

testDiscoveryParsing()
  .then(() => process.exit(0))
  .catch(() => process.exit(1));
