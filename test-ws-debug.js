// Quick WebSocket debugging script
const WebSocket = require('ws');

async function testWebSocket() {
    const discoveryUrl = 'http://127.0.0.1:11430/api/discovery/shimmy';
    
    console.log('📍 Fetching discovery info from:', discoveryUrl);
    const response = await fetch(discoveryUrl);
    const discovery = await response.json();
    console.log('✅ Discovery response:', JSON.stringify(discovery, null, 2));
    
    const wsUrl = `ws://127.0.0.1:${discovery.port}/ws/console`;
    console.log('🔌 Connecting to:', wsUrl);
    
    const ws = new WebSocket(wsUrl);
    
    ws.on('open', () => {
        console.log('✅ WebSocket connected');
        const msg = JSON.stringify({ type: 'get_models' });
        console.log('📤 Sending:', msg);
        ws.send(msg);
    });
    
    ws.on('message', (data) => {
        console.log('📥 Received:', data.toString());
    });
    
    ws.on('error', (error) => {
        console.error('❌ WebSocket error:', error);
    });
    
    ws.on('close', () => {
        console.log('🔴 WebSocket closed');
        process.exit(0);
    });
    
    // Timeout after 10 seconds
    setTimeout(() => {
        console.log('⏱️  Timeout - no response received');
        ws.close();
    }, 10000);
}

testWebSocket().catch(console.error);
