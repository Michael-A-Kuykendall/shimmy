#!/usr/bin/env node

/**
 * Mock Discovery Server for testing the theme validator
 * Provides a fake discovery response that matches the expected schema
 */

const http = require('http');

const mockDiscoveryResponse = {
    discovery_port: 11430,
    last_updated: new Date().toISOString(),
    epoch: 12345,
    backends: [
        {
            id: "shimmy-backend-01",
            url: "http://127.0.0.1:62345",
            port: 62345,
            models: [
                {
                    name: "test-model",
                    display_name: "Test Model",
                    size_bytes: 1024000,
                    parameter_count: "7B",
                    quantization: "Q4_K_M",
                    context_length: 4096,
                    model_type: "llama",
                    source: "registered",
                    backend: "shimmy-backend-01",
                    health_status: "ready",
                    health_error: null
                }
            ],
            capabilities: ["chat", "completion", "streaming"],
            health: {
                healthy: true,
                last_check: new Date().toISOString()
            },
            started_at: new Date().toISOString(),
            pid: 12345
        }
    ]
};

const server = http.createServer((req, res) => {
    // Handle CORS
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
    
    if (req.method === 'OPTIONS') {
        res.writeHead(200);
        res.end();
        return;
    }
    
    if (req.url === '/api/discovery' && req.method === 'GET') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(mockDiscoveryResponse, null, 2));
        console.log(`📡 Served discovery response to ${req.connection.remoteAddress}`);
    } else {
        res.writeHead(404);
        res.end('Not Found');
    }
});

const PORT = 11430;
const HOST = '127.0.0.1';

server.listen(PORT, HOST, () => {
    console.log(`🚀 Mock Discovery Server running on http://${HOST}:${PORT}/api/discovery`);
    console.log('📋 Response includes 1 healthy backend with test model');
    console.log('⏱️  Server will automatically stop after 30 seconds...');
    
    // Auto-stop after 30 seconds
    setTimeout(() => {
        console.log('⏹️  Stopping mock server...');
        server.close();
        process.exit(0);
    }, 30000);
});

// Handle graceful shutdown
process.on('SIGINT', () => {
    console.log('\n⏹️  Stopping mock server...');
    server.close();
    process.exit(0);
});