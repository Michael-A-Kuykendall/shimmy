/// <reference types="vite/client" />

// Shimmy contract type definitions
declare global {
  const __SHIMMY_CONTRACT__: {
    version: string
    discoveryEndpoint: string
    websocketEndpoint: string
    streamingTypes: string[]
    messageTypes: string[]
  }
}

// Environment variables
interface ImportMetaEnv {
  readonly VITE_SHIMMY_BACKEND_URL?: string
  readonly VITE_SHIMMY_WS_URL?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}