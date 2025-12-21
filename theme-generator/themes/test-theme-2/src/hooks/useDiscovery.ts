import { useState, useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'

// Generated from schema
interface BackendInfo {

  health_check: boolean

  websocket_port?: number
  version?: string
  models_endpoint?: string
  discovery_timestamp?: string
}

const DISCOVERY_CONFIG = {
  endpoint: '/api/discovery',
  port: 11430,
  timeout: 5000,
  validationFields: [
  &#34;health_check&#34;
]
}

/**
 * Custom hook for Shimmy backend discovery
 * Implements auto-discovery protocol from schema
 */
export function useDiscovery() {
  const [discoveryUrl, setDiscoveryUrl] = useState<string>(`http://127.0.0.1:${DISCOVERY_CONFIG.port}${DISCOVERY_CONFIG.endpoint}`)
  
  const {
    data: backendInfo,
    error,
    isLoading,
    refetch
  } = useQuery<BackendInfo>({
    queryKey: ['shimmy-discovery', discoveryUrl],
    queryFn: async (): Promise<BackendInfo> => {
      console.log(`🔍 Attempting discovery at: ${discoveryUrl}`)
      
      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), DISCOVERY_CONFIG.timeout)
      
      try {
        const response = await fetch(discoveryUrl, {
          method: 'GET',
          headers: {
            'Accept': 'application/json',
            'User-Agent': 'shimmy-theme/test-theme-2'
          },
          signal: controller.signal
        })
        
        clearTimeout(timeoutId)
        
        if (!response.ok) {
          throw new Error(`Discovery failed: ${response.status} ${response.statusText}`)
        }
        
        const data = await response.json()
        console.log('✅ Discovery successful:', data)
        
        // Validate required fields from schema
        const missingFields = DISCOVERY_CONFIG.validationFields.filter(
          field => data[field] === undefined
        )
        
        if (missingFields.length > 0) {
          throw new Error(`Missing required fields: ${missingFields.join(', ')}`)
        }
        
        return data
      } catch (error) {
        clearTimeout(timeoutId)
        
        if (error.name === 'AbortError') {
          throw new Error(`Discovery timeout after ${DISCOVERY_CONFIG.timeout}ms`)
        }
        
        throw error
      }
    },
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 10000),
    staleTime: 1000 * 60 * 5, // 5 minutes
    refetchOnWindowFocus: false
  })
  
  // Try alternative ports if default fails
  useEffect(() => {
    if (error && discoveryUrl.includes(`:${DISCOVERY_CONFIG.port}`)) {
      console.log('🔄 Trying alternative discovery ports...')
      
      const alternativePorts = [11431, 11432, 11433, 11434, 11435]
      for (const port of alternativePorts) {
        if (port !== DISCOVERY_CONFIG.port) {
          setDiscoveryUrl(`http://127.0.0.1:${port}${DISCOVERY_CONFIG.endpoint}`)
          break
        }
      }
    }
  }, [error])
  
  return {
    backendInfo,
    isLoading,
    error: error as Error | null,
    retry: refetch,
    discoveryUrl
  }
}