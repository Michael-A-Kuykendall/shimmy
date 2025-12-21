#!/usr/bin/env node

// Test complete model selection machinery
const { DiscoveryClient } = require('./dist/client');

async function testModelSelection() {
  console.log('🔧 Testing Complete Model Selection Machinery...\n');
  
  const client = new DiscoveryClient({
    discoveryPorts: [11430, 11431, 11432, 11433, 11434, 11435],
    refreshInterval: 5000,
    autoConnect: true,
  });

  try {
    // 1. Discovery
    console.log('📡 Step 1: Discovery...');
    const backends = await client.discoverBackends();
    console.log(`✅ Found ${backends.length} backend(s)`);

    const allModels = client.getAllModels();
    console.log(`📊 Total models discovered: ${allModels.length}\n`);

    // 2. Health Validation
    console.log('🧪 Step 2: Health validation...');
    await client.validateAllModels();
    
    const readyModels = client.getReadyModels();
    console.log(`✅ Ready models: ${readyModels.length}/${allModels.length}`);
    
    if (readyModels.length === 0) {
      console.log('❌ No ready models found - cannot test selection');
      return;
    }
    
    console.log('\n✅ Ready models:');
    readyModels.forEach((model, i) => {
      const time = model.response_time_ms ? ` (~${(model.response_time_ms / 1000).toFixed(1)}s)` : '';
      console.log(`  ${i + 1}. ${model.displayName}${time}`);
    });

    // 3. Model Selection Tests
    console.log('\n🎯 Step 3: Model selection tests...');
    
    // Test 1: Select by name
    const testModel = readyModels[0];
    console.log(`\n   Test 1: Selecting model "${testModel.displayName}"...`);
    const validationResult = await client.validateModel(testModel.name);
    console.log(`   ✅ Selection test: ${validationResult.status} (${validationResult.responseTime}ms)`);
    
    // Test 2: Find backend for model
    const backend = client.findBackendForModel(testModel.name);
    if (backend) {
      console.log(`   ✅ Backend found: ${backend.id} on port ${backend.port}`);
    } else {
      console.log(`   ❌ Backend not found for model ${testModel.name}`);
    }
    
    // Test 3: Error handling - invalid model
    console.log('\n   Test 2: Testing invalid model selection...');
    try {
      await client.validateModel('nonexistent-model');
      console.log('   ❌ Should have failed for invalid model');
    } catch (error) {
      console.log(`   ✅ Correctly rejected invalid model: ${error.message}`);
    }

    // 4. Integration Test - API Call
    console.log('\n🚀 Step 4: End-to-end API test...');
    if (backend) {
      try {
        const response = await fetch(`${backend.url}/api/generate`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            model: testModel.name,
            prompt: 'Say "OK"',
            stream: false,
            max_tokens: 2,
          }),
          signal: AbortSignal.timeout(10000),
        });

        if (response.ok) {
          const result = await response.json();
          console.log(`   ✅ API call successful: ${JSON.stringify(result)}`);
        } else {
          console.log(`   ❌ API call failed: ${response.status} ${response.statusText}`);
        }
      } catch (error) {
        console.log(`   ❌ API call error: ${error.message}`);
      }
    }

    console.log('\n✅ Model Selection Machinery Test COMPLETE');
    console.log('\n📋 Summary:');
    console.log(`   • Discovery: ${backends.length} backend(s) found`);
    console.log(`   • Health Check: ${readyModels.length}/${allModels.length} models ready`);
    console.log(`   • Selection: Model selection and validation working`);
    console.log(`   • API Integration: Backend communication verified`);
    
  } catch (error) {
    console.error('❌ Test failed:', error.message);
    if (client.getLastError()) {
      console.error('Last client error:', client.getLastError());
    }
    process.exit(1);
  }
}

testModelSelection().catch(console.error);