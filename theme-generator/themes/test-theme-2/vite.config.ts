import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    host: true,
    proxy: {
      // Proxy Shimmy backend requests
      '/api/discovery': {
        target: 'http://localhost:11430',
        changeOrigin: true,
        secure: false,
      },
      '/ws/console': {
        target: 'ws://localhost:11430',
        changeOrigin: true,
        ws: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    sourcemap: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
      },
    },
  },
  define: {
    // Inject Shimmy contract configuration
    __SHIMMY_CONTRACT__: {
  &#34;discoveryEndpoint&#34;: &#34;/api/discovery:11430&#34;,
  &#34;websocketEndpoint&#34;: &#34;/ws/console&#34;,
  &#34;messageTypes&#34;: []
},
  },
})