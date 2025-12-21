import { useState, useEffect, useRef, useCallback } from 'react';
import { resolveShimmyBackend } from '@/utils/discovery';

export interface ShimmyMetrics {
  cpu_usage_percent: number;
  memory_usage_percent: number;
  memory_used_gb: number;
  memory_total_gb: number;
  gpu_usage_percent?: number;
  gpu_memory_used_gb?: number;
  gpu_memory_total_gb?: number;
  tokens_per_second?: number;
  total_tokens_generated: number;
  active_sessions: number;
  uptime_seconds: number;
  timestamp: number;
}

export interface ShimmyModel {
  name: string;
  display_name?: string;
  parameter_count?: string;
  quantization?: string;
  context_length?: number;
  size_bytes?: number;
  model_type?: string;
  path?: string;
  loaded: boolean;
  supported_features: string[];
  source: string;
}

export interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

export function useShimmy() {
  const [metrics, setMetrics] = useState<ShimmyMetrics | null>(null);
  const [models, setModels] = useState<ShimmyModel[]>([]);
  const [messages, setMessages] = useState<Message[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);
  const [backendUrl, setBackendUrl] = useState<string>('');
  
  const wsRef = useRef<WebSocket | null>(null);
  const currentMessageRef = useRef('');
  const reconnectTimeoutRef = useRef<number>();
  const generationTimeoutRef = useRef<number>(); // Safety timeout

  // Discover backend on mount
  useEffect(() => {
    resolveShimmyBackend()
      .then(backend => {
        setBackendUrl(backend.url);
        console.log('✅ Discovered Shimmy backend:', backend.url);
      })
      .catch(err => {
        console.error('❌ Failed to discover backend:', err);
      });
  }, []);

  // Poll metrics when backend URL is available
  useEffect(() => {
    if (!backendUrl) return;
    
    const fetchMetrics = async () => {
      try {
        const response = await fetch(`${backendUrl}/api/metrics`);
        const data = await response.json();
        setMetrics(data);
      } catch (error) {
        console.error('Failed to fetch metrics:', error);
      }
    };

    fetchMetrics();
    const interval = setInterval(fetchMetrics, 1000);
    return () => clearInterval(interval);
  }, [backendUrl]);

  // WebSocket connection with auto-reconnect
  const connectWebSocket = useCallback(() => {
    if (!backendUrl || wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    try {
      const wsUrl = backendUrl.replace('http', 'ws') + '/ws/console';
      const ws = new WebSocket(wsUrl);

      ws.onopen = () => {
        console.log('✅ WebSocket connected');
        setIsConnected(true);
        
        // Request models per contract
        ws.send(JSON.stringify({ type: 'get_models' }));
      };

      ws.onmessage = (event) => {
        console.log('📨 WebSocket message received:', event.data);
        
        try {
          const data = JSON.parse(event.data);
          console.log('✅ Parsed JSON:', data);
          
          // Handle models response
          if (data.type === 'models_response' && data.success) {
            setModels(data.models || []);
            return;
          }
          
          // Handle model selection confirmation
          if (data.type === 'model_selected') {
            console.log('✅ Model selected:', data.model_name);
            return;
          }
          
          // Handle streaming tokens
          if (data.token) {
            currentMessageRef.current += data.token;
            setMessages(prev => {
              const updated = [...prev];
              if (updated[updated.length - 1]?.role === 'assistant') {
                updated[updated.length - 1].content = currentMessageRef.current;
              }
              return updated;
            });
            return;
          }
          
          // Handle generation complete
          if (data.type === 'generation_complete') {
            console.log('✅ Generation complete');
            if (generationTimeoutRef.current) {
              clearTimeout(generationTimeoutRef.current);
            }
            setIsGenerating(false);
            currentMessageRef.current = '';
            return;
          }
        } catch (err) {
          // Raw text response (non-JSON) - treat as complete response
          console.log('✅ Raw text response received:', event.data);
          console.log('🔓 Setting isGenerating to false');
          if (generationTimeoutRef.current) {
            clearTimeout(generationTimeoutRef.current);
          }
          setMessages(prev => {
            const updated = [...prev];
            if (updated[updated.length - 1]?.role === 'assistant') {
              updated[updated.length - 1].content = event.data;
            }
            return updated;
          });
          setIsGenerating(false);
          currentMessageRef.current = '';
        }
      };

      ws.onerror = (error) => {
        console.error('❌ WebSocket error:', error);
      };

      ws.onclose = () => {
        console.log('🔌 WebSocket disconnected');
        setIsConnected(false);
        wsRef.current = null;
        
        reconnectTimeoutRef.current = window.setTimeout(() => {
          console.log('🔄 Attempting to reconnect...');
          connectWebSocket();
        }, 3000);
      };

      wsRef.current = ws;
    } catch (error) {
      console.error('Failed to connect WebSocket:', error);
      setIsConnected(false);
    }
  }, [backendUrl]);

  useEffect(() => {
    connectWebSocket();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (generationTimeoutRef.current) {
        clearTimeout(generationTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [connectWebSocket]);

  const sendMessage = useCallback((content: string) => {
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) {
      console.error('❌ WebSocket not connected, cannot send message');
      return;
    }

    console.log('📤 Sending message:', content);

    const userMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content,
      timestamp: new Date(),
    };
    setMessages(prev => [...prev, userMessage]);

    const assistantMessage: Message = {
      id: (Date.now() + 1).toString(),
      role: 'assistant',
      content: '',
      timestamp: new Date(),
    };
    setMessages(prev => [...prev, assistantMessage]);

    currentMessageRef.current = '';
    setIsGenerating(true);
    console.log('🔒 Set isGenerating = true');

    // Safety timeout: re-enable input after 60 seconds even if no response
    if (generationTimeoutRef.current) {
      clearTimeout(generationTimeoutRef.current);
    }
    generationTimeoutRef.current = window.setTimeout(() => {
      console.log('⚠️ Generation timeout - force re-enabling input');
      setIsGenerating(false);
    }, 60000);

    // Send message to backend (contract-compliant format)
    const messagePayload = {
      type: 'chat',
      message: content
    };
    console.log('📨 Sending WebSocket message:', JSON.stringify(messagePayload));
    wsRef.current.send(JSON.stringify(messagePayload));
  }, []);

  const selectModel = useCallback((modelName: string) => {
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) {
      console.error('WebSocket not connected');
      return;
    }

    // Send select_model message per contract
    wsRef.current.send(JSON.stringify({
      type: 'select_model',
      model_name: modelName
    }));
  }, []);

  return {
    metrics,
    models,
    messages,
    isConnected,
    isGenerating,
    sendMessage,
    selectModel,
  };
}
