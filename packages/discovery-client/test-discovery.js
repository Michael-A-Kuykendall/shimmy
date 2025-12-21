#!/usr/bin/env node

// Quick test script to verify discovery client functionality
const { DiscoveryClient } = require('./dist/client');

async function testDiscovery() {
  console.log('🔍 Testing Shimmy Discovery Client...\n');
  
  const client = new DiscoveryClient({
    baseUrl: 'http://localhost',
    portRange: [11430, 11435],
    retryAttempts: 2,
    retryDelay: 1000,
  });

  try {
    console.log('📡 Starting discovery...');
    const backends = await client.discoverBackends();
    console.log(`✅ Found ${backends.length} backend(s):`);
    
    backends.forEach((backend, i) => {
      console.log(`  ${i + 1}. Port ${backend.port} - ${backend.models.length} models (${backend.health})`);
      backend.models.forEach((model, j) => {
        console.log(`     ${j + 1}. ${model.name} (${model.backend_type})`);
      });
    });

    const allModels = client.getAllModels();
    console.log(`\n📊 Total models available: ${allModels.length}`);
    
    if (allModels.length > 0) {
      console.log('✅ Discovery client is working correctly!');
      console.log('✅ Note: Message streaming is handled by React hook, not the raw client');
    } else {
      console.log('❌ No models found - check if shimmy is running');
    }
    
  } catch (error) {
    console.error('❌ Discovery failed:', error.message);
    if (client.getLastError()) {
      console.error('Last client error:', client.getLastError());
    }
    process.exit(1);
  }
  
  console.log('\n🏁 Test completed');
}

testDiscovery().catch(console.error);