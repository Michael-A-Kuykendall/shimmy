#!/usr/bin/env node

// Ensure dist/ipc-client.js is present and returns a stubbed IPC backend list.
const fs = require('fs');
const path = require('path');

const distDir = path.resolve(__dirname, '..', 'dist');
const ipcClientPath = path.join(distDir, 'ipc-client.js');

// write a small IPC client module that will be dynamically imported by dist/client.js
fs.writeFileSync(ipcClientPath, `
exports.queryIPCDiscovery = async function() {
  return [
    {
      id: 'local-ipc-1',
      url: 'http://127.0.0.1:51061',
      port: 51061,
      models: [ { name: 'test', display_name: 'Test Model', source: 'local', backend: 'llama', health_status: 'ready' } ],
      capabilities: ['chat'],
      health: { healthy: true, last_check: new Date().toISOString() },
      started_at: new Date().toISOString(),
      pid: 1234
    }
  ];
};
`, 'utf8');

const { DiscoveryClient } = require(path.join(__dirname, '..', 'dist', 'client'));

(async function() {
  // Simulate HTTP discovery failure by overriding the method
  DiscoveryClient.prototype.discoverViaHTTP = async function() { throw new Error('Simulated HTTP failure'); };

  const client = new DiscoveryClient({ discoveryPorts: [11430] });

  console.log('Running IPC fallback test (HTTP will fail, IPC module present)');
  const backends = await client.discoverBackends();
  if (!backends || backends.length === 0) {
    console.error('❌ Expected IPC fallback to return backends, got none');
    process.exit(2);
  }

  console.log('✅ IPC fallback returned backends:', backends.map(b => b.port));

  // cleanup
  fs.unlinkSync(ipcClientPath);
  process.exit(0);
})();
