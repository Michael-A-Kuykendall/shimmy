const express = require('express');
const axios = require('axios');

const app = express();
const port = process.env.PORT || 3000;
const SHIMMY_BASE_URL = process.env.SHIMMY_BASE_URL || 'http://localhost:11435';

app.use(express.json());

app.get('/', (req, res) => {
  res.json({ message: 'Shimmy Express Integration' });
});

app.post('/v1/chat/completions', async (req, res) => {
  try {
    const response = await axios.post(`${SHIMMY_BASE_URL}/v1/chat/completions`, req.body);
    res.json(response.data);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

app.get('/v1/models', async (req, res) => {
  try {
    const response = await axios.get(`${SHIMMY_BASE_URL}/v1/models`);
    res.json(response.data);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

app.listen(port, () => {
  console.log(`Shimmy Express integration listening at http://localhost:${port}`);
});