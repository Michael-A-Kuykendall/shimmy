import { useState, useEffect, useRef, useCallback } from 'react'

// Generated WebSocket message types from schema


type WebSocketMessage = 

  | { type: 'get_models' }
  | { type: 'select_model'; model_name: string }
  | { type: 'chat_request'; message: string }
  | { type: 'get_metrics' }


interface UseWebSocketReturn {
  socket: WebSocket | null
  isConnected: boolean
  isConnecting: boolean
  error: string | null
  sendMessage: (message: WebSocketMessage) => void
  reconnect: () => void
  disconnect: () => void
}

const WEBSOCKET_CONFIG = {
  endpoint: '/ws/console',
  reconnectInterval: 5000,
  maxReconnectAttempts: 10,
  pingInterval: 30000
}

/**
 * Enhanced WebSocket hook with reconnection logic
 * Generated from Shimmy schema
 */
export function useWebSocket(port?: number): UseWebSocketReturn {
  const [socket, setSocket] = useState<WebSocket | null>(null)
  const [isConnected, setIsConnected] = useState(false)
  const [isConnecting, setIsConnecting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  
  const reconnectAttempts = useRef(0)
  const reconnectTimeout = useRef<NodeJS.Timeout | null>(null)
  const pingInterval = useRef<NodeJS.Timeout | null>(null)
  const shouldReconnect = useRef(true)
  
  const connect = useCallback(() => {
    if (!port) {
      setError('WebSocket port not available from discovery')
      return
    }
    
    setIsConnecting(true)
    setError(null)
    
    const wsUrl = `ws://127.0.0.1:${port}${WEBSOCKET_CONFIG.endpoint}`
    console.log(`🔌 Connecting to WebSocket: ${wsUrl}`)
    
    try {
      const ws = new WebSocket(wsUrl)
      
      ws.onopen = () => {
        console.log('✅ WebSocket connected')
        setIsConnected(true)
        setIsConnecting(false)
        setError(null)
        reconnectAttempts.current = 0
        
        // Start ping interval to keep connection alive
        pingInterval.current = setInterval(() => {
          if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: 'ping' }))
          }
        }, WEBSOCKET_CONFIG.pingInterval)
      }
      
      ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data)
          
          // Handle pong responses
          if (message.type === 'pong') {
            return
          }
          
          // Dispatch custom event for message handling
          window.dispatchEvent(
            new CustomEvent('shimmy-websocket-message', { 
              detail: message 
            })
          )
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error)
        }
      }
      
      ws.onerror = (error) => {
        console.error('WebSocket error:', error)
        setError('WebSocket connection error')
      }
      
      ws.onclose = (event) => {
        console.log('🔌 WebSocket disconnected:', event.code, event.reason)
        setIsConnected(false)
        setIsConnecting(false)
        
        // Clear ping interval
        if (pingInterval.current) {
          clearInterval(pingInterval.current)
          pingInterval.current = null
        }
        
        // Attempt to reconnect if not intentionally closed
        if (shouldReconnect.current && event.code !== 1000) {
          if (reconnectAttempts.current < WEBSOCKET_CONFIG.maxReconnectAttempts) {
            reconnectAttempts.current++
            setError(`Connection lost. Reconnecting... (${reconnectAttempts.current}/${WEBSOCKET_CONFIG.maxReconnectAttempts})`)
            
            reconnectTimeout.current = setTimeout(() => {
              connect()
            }, WEBSOCKET_CONFIG.reconnectInterval)
          } else {
            setError('Max reconnection attempts reached')
          }
        }
      }
      
      setSocket(ws)
    } catch (error) {
      console.error('Failed to create WebSocket:', error)
      setError('Failed to create WebSocket connection')
      setIsConnecting(false)
    }
  }, [port])
  
  const disconnect = useCallback(() => {
    shouldReconnect.current = false
    
    if (reconnectTimeout.current) {
      clearTimeout(reconnectTimeout.current)
      reconnectTimeout.current = null
    }
    
    if (pingInterval.current) {
      clearInterval(pingInterval.current)
      pingInterval.current = null
    }
    
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.close(1000, 'User disconnected')
    }
    
    setSocket(null)
    setIsConnected(false)
    setIsConnecting(false)
  }, [socket])
  
  const reconnect = useCallback(() => {
    disconnect()
    shouldReconnect.current = true
    reconnectAttempts.current = 0
    setTimeout(() => connect(), 100)
  }, [connect, disconnect])
  
  const sendMessage = useCallback((message: WebSocketMessage) => {
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(message))
      console.log('📤 Sent message:', message)
    } else {
      console.warn('Cannot send message: WebSocket not connected')
      setError('Cannot send message: Not connected')
    }
  }, [socket])
  
  // Connect when port becomes available
  useEffect(() => {
    if (port && shouldReconnect.current) {
      connect()
    }
    
    return () => {
      disconnect()
    }
  }, [port, connect, disconnect])
  
  // Cleanup on unmount
  useEffect(() => {
    return () => {
      shouldReconnect.current = false
      disconnect()
    }
  }, [disconnect])
  
  return {
    socket,
    isConnected,
    isConnecting,
    error,
    sendMessage,
    reconnect,
    disconnect
  }
}