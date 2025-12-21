#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

console.log('🚀 Starting orchestrator...');
console.log('⏳ This will take 45 seconds to 4 minutes. Waiting for completion...\n');

const orchestrator = spawn('cargo', [
  'run',
  '--bin', 'shimmy',
  '--',
  'dev',
  'shimmy-default',
  '--verify'
], {
  cwd: process.cwd(),
  stdio: 'inherit',
  shell: true
});

orchestrator.on('close', (code) => {
  if (code === 0) {
    console.log('\n✅ Orchestrator completed successfully');
    process.exit(0);
  } else {
    console.log(`\n❌ Orchestrator failed with code ${code}`);
    process.exit(code);
  }
});

orchestrator.on('error', (err) => {
  console.error(`❌ Failed to start orchestrator: ${err.message}`);
  process.exit(1);
});
