import { WebSocket } from 'ws';

async function testToolAwareness() {
    console.log('🧠 Testing Tool Awareness...');

    return new Promise((resolve, reject) => {
        const ws = new WebSocket('ws://127.0.0.1:61455/ws/console');

        ws.on('open', () => {
            console.log('✅ WebSocket connected');

            // Send a message asking about tools
            const message = {
                type: 'chat',
                id: Date.now().toString(),
                content: 'What tools do you have access to? Please list them clearly.'
            };

            console.log('📤 Sending:', JSON.stringify(message, null, 2));
            ws.send(JSON.stringify(message));
        });

        ws.on('message', (data) => {
            try {
                const message = JSON.parse(data.toString());
                console.log('📨 Received:', JSON.stringify(message, null, 2));

                if (message.type === 'response') {
                    console.log('💬 AI Response:', message.content);
                    ws.close();
                    resolve(message.content);
                }
            } catch (e) {
                console.log('📨 Raw message:', data.toString());
            }
        });

        ws.on('error', (error) => {
            console.error('❌ WebSocket error:', error);
            reject(error);
        });

        ws.on('close', () => {
            console.log('🔌 WebSocket closed');
        });

        // Timeout after 2 minutes
        setTimeout(() => {
            ws.close();
            reject(new Error('Timeout waiting for response'));
        }, 120000);
    });
}

// Run the test
testToolAwareness()
    .then(response => {
        console.log('\n✅ Test completed successfully!');
        console.log('Response analysis:');
        if (response.includes('tool') || response.includes('Tool')) {
            console.log('✅ AI mentioned tools - basic awareness working');
        } else {
            console.log('❌ AI did not mention tools - may need better prompting');
        }
    })
    .catch(error => {
        console.error('❌ Test failed:', error);
    });