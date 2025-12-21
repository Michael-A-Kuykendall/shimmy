#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

const distDir = path.resolve(__dirname, '..', 'dist');
const ipcClientPath = path.join(distDir, 'ipc-client.js');

// Write an ipc-client that would throw if invoked (to ensure HTTP is used)
fs.writeFileSync(ipcClientPath, `
exports.queryIPCDiscovery = async function() { throw new Error('IPC should not be used when HTTP succeeds'); };
`, 'utf8');

const { DiscoveryClient } = require(path.join(__dirname, '..', 'dist', 'client'));

(async function() {
  // Simulate HTTP discovery success by overriding discoverViaHTTP
  DiscoveryClient.prototype.discoverViaHTTP = async function() {
    return [ { id: 'http-1', port: 11430, models: [ { name: 'via-http', display_name: 'Via HTTP', source: 'http', backend: 'llama', health_status: 'ready' } ] } ];
  };

  const client = new DiscoveryClient({ discoveryPorts: [11430] });

  console.log('Running HTTP preferred test (HTTP returns results, IPC should NOT be used)');
  const backends = await client.discoverBackends();
  if (!backends || backends.length === 0) {
    console.error('❌ Expected HTTP discovery to return backends, got none');
    process.exit(2);
  }

  console.log('✅ HTTP discovery returned backends:', backends.map(b => b.port));

  // cleanup
  fs.unlinkSync(ipcClientPath);
  process.exit(0);
})();
