/**
 * Express.js integration example for Shimmy AI inference
 */
const express = require('express');
const axios = require('axios');
const cors = require('cors');

const app = express();
const PORT = process.env.PORT || 3000;

// Configuration
const SHIMMY_BASE_URL = process.env.SHIMMY_URL || 'http://localhost:11435';
const SHIMMY_API_KEY = process.env.SHIMMY_API_KEY || 'sk-local';

// Middleware
app.use(cors());
app.use(express.json({ limit: '10mb' }));

// Create axios instance for Shimmy
const shimmyClient = axios.create({
  baseURL: SHIMMY_BASE_URL,
  headers: {
    'Authorization': `Bearer ${SHIMMY_API_KEY}`,
    'Content-Type': 'application/json'
  },
  timeout: 300000 // 5 minutes
});

// Health check
app.get('/', (req, res) => {
  res.json({
    status: 'healthy',
    service: 'shimmy-express',
    version: '1.0.0'
  });
});

// List available models
app.get('/models', async (req, res) => {
  try {
    const response = await shimmyClient.get('/v1/models');
    res.json(response.data);
  } catch (error) {
    console.error('Error fetching models:', error.message);
    res.status(503).json({
      error: 'Shimmy service unavailable',
      details: error.message
    });
  }
});

// Chat completions - OpenAI compatible
app.post('/chat/completions', async (req, res) => {
  try {
    const { model, messages, max_tokens = 100, temperature = 0.7, stream = false } = req.body;

    if (!model || !messages) {
      return res.status(400).json({
        error: 'Missing required fields: model and messages'
      });
    }

    const response = await shimmyClient.post('/v1/chat/completions', {
      model,
      messages,
      max_tokens,
      temperature,
      stream
    });

    res.json(response.data);
  } catch (error) {
    console.error('Error in chat completion:', error.message);

    if (error.response) {
      res.status(error.response.status).json({
        error: 'Shimmy API error',
        details: error.response.data
      });
    } else {
      res.status(503).json({
        error: 'Shimmy service unavailable',
        details: error.message
      });
    }
  }
});

// Simplified chat endpoint
app.post('/simple-chat', async (req, res) => {
  try {
    const { prompt, model = 'phi3-mini', max_tokens = 150 } = req.body;

    if (!prompt) {
      return res.status(400).json({ error: 'Missing prompt' });
    }

    const response = await shimmyClient.post('/v1/chat/completions', {
      model,
      messages: [{ role: 'user', content: prompt }],
      max_tokens
    });

    res.json(response.data);
  } catch (error) {
    console.error('Error in simple chat:', error.message);
    res.status(503).json({
      error: 'Chat failed',
      details: error.message
    });
  }
});

// Streaming chat endpoint
app.post('/chat/stream', async (req, res) => {
  try {
    const { model, messages, max_tokens = 100, temperature = 0.7 } = req.body;

    // Set up SSE headers
    res.writeHead(200, {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      'Connection': 'keep-alive',
      'Access-Control-Allow-Origin': '*'
    });

    const response = await shimmyClient.post('/v1/chat/completions', {
      model,
      messages,
      max_tokens,
      temperature,
      stream: true
    }, {
      responseType: 'stream'
    });

    response.data.on('data', (chunk) => {
      res.write(`data: ${chunk}\n\n`);
    });

    response.data.on('end', () => {
      res.write('data: [DONE]\n\n');
      res.end();
    });

  } catch (error) {
    console.error('Error in streaming chat:', error.message);
    res.write(`data: {"error": "${error.message}"}\n\n`);
    res.end();
  }
});

// Error handling middleware
app.use((error, req, res, next) => {
  console.error('Unhandled error:', error);
  res.status(500).json({
    error: 'Internal server error',
    details: error.message
  });
});

// Start server
app.listen(PORT, async () => {
  console.log(`🚀 Shimmy Express server running on port ${PORT}`);

  // Test Shimmy connection
  try {
    await shimmyClient.get('/v1/models');
    console.log(`✅ Connected to Shimmy at ${SHIMMY_BASE_URL}`);
  } catch (error) {
    console.log(`⚠️  Could not connect to Shimmy: ${error.message}`);
  }
});

module.exports = app;
