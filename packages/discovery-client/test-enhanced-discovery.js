#!/usr/bin/env node

// Test the enhanced discovery client with health checking
const { DiscoveryClient } = require('./dist/client');

async function testEnhancedDiscovery() {
  console.log('🔍 Testing Enhanced Shimmy Discovery Client...\n');
  
  const client = new DiscoveryClient({
    discoveryPorts: [11430, 11431, 11432, 11433, 11434, 11435],
    refreshInterval: 5000,
    autoConnect: true,
  });

  try {
    console.log('📡 Starting discovery...');
    const backends = await client.discoverBackends();
    console.log(`✅ Found ${backends.length} backend(s)\n`);

    const allModels = client.getAllModels();
    console.log(`📊 Total models discovered: ${allModels.length}`);
    
    // Show models before health check
    console.log('\n🔍 Models before health checking:');
    allModels.forEach((model, i) => {
      console.log(`  ${i + 1}. ${model.displayName} (${model.name}) - Status: ${model.health_status}`);
    });

    if (allModels.length > 0) {
      console.log('\n🧪 Starting model health validation...');
      await client.validateAllModels();
      
      console.log('\n✅ Health validation completed!');
      
      // Show results
      const updatedModels = client.getAllModels();
      console.log('\n📊 Health Check Results:');
      updatedModels.forEach((model, i) => {
        const status = model.health_status;
        const time = model.response_time_ms ? ` (${model.response_time_ms}ms)` : '';
        const error = model.health_error ? ` - ${model.health_error}` : '';
        
        console.log(`  ${i + 1}. ${model.displayName}`);
        console.log(`     Status: ${status}${time}${error}`);
      });

      const readyModels = client.getReadyModels();
      console.log(`\n✅ Ready models: ${readyModels.length}/${allModels.length}`);
      
      if (readyModels.length > 0) {
        console.log('\n🚀 Ready models:');
        readyModels.forEach((model, i) => {
          const time = model.response_time_ms ? ` (~${(model.response_time_ms / 1000).toFixed(1)}s)` : '';
          console.log(`  ${i + 1}. ${model.displayName}${time}`);
        });
      }
      
      if (readyModels.length !== allModels.length) {
        const failedModels = allModels.filter(m => m.health_status === 'failed');
        console.log(`\n❌ Failed models: ${failedModels.length}`);
        failedModels.forEach((model, i) => {
          console.log(`  ${i + 1}. ${model.displayName} - ${model.health_error}`);
        });
      }
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
  
  console.log('\n🏁 Enhanced discovery test completed');
}

testEnhancedDiscovery().catch(console.error);