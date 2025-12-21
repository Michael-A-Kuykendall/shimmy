#!/usr/bin/env node

// Test the actual HTTP streaming endpoint that the discovery client uses
const { default: fetch } = require('node-fetch');

async function testStreaming() {
  console.log('🧪 Testing HTTP streaming endpoint...\n');
  
  try {
    const response = await fetch('http://localhost:11435/api/generate', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        model: 'phi3-lora',
        prompt: 'Say hello world',
        stream: true,
        max_tokens: 5,
      }),
    });

    console.log('Response status:', response.status);
    console.log('Response headers:', Object.fromEntries(response.headers));
    
    if (!response.ok) {
      console.error('❌ HTTP error:', response.status, response.statusText);
      const text = await response.text();
      console.error('Error body:', text);
      return;
    }

    console.log('\n📡 Reading streaming response...');
    
    // Read the stream
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let tokenCount = 0;
    
    while (tokenCount < 10) { // Limit to first 10 tokens
      const { done, value } = await reader.read();
      if (done) break;
      
      const chunk = decoder.decode(value, { stream: true });
      console.log('Raw chunk:', JSON.stringify(chunk));
      
      // Parse SSE format
      const lines = chunk.split('\n');
      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const data = line.slice(6).trim();
          console.log('SSE data:', data);
          
          if (data === '[DONE]') {
            console.log('✅ Stream completed with [DONE]');
            return;
          }
          
          try {
            const parsed = JSON.parse(data);
            if (parsed.token) {
              process.stdout.write(parsed.token);
              tokenCount++;
            }
          } catch (e) {
            console.log('Non-JSON data:', data);
          }
        }
      }
    }
    
    console.log('\n✅ Streaming test completed');
    
  } catch (error) {
    console.error('❌ Test failed:', error.message);
  }
}

testStreaming();