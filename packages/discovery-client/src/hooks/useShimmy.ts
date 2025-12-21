/**
 * useShimmy React Hook
 * 
 * The main interface that theme developers will use. Handles all discovery,
 * connection management, and provides a simple API for sending messages.
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { DiscoveryClient } from '../client';
import { ShimmyConfig, UseShimmyResult, ShimmyBackend, ShimmyModel, SendMessageOptions, ModelChooserData } from '../types';

export function useShimmy(config: ShimmyConfig = {}): UseShimmyResult {
  const [models, setModels] = useState<ShimmyModel[]>([]);
  const [backends, setBackends] = useState<ShimmyBackend[]>([]);
  const [selectedModel, setSelectedModelState] = useState<ShimmyModel | null>(null);
  const [isReady, setIsReady] = useState(false);
  const [isValidating, setIsValidating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Persistent model selection with localStorage
  const setSelectedModel = useCallback((model: ShimmyModel | null) => {
    setSelectedModelState(model);
    if (model) {
      try {
        localStorage.setItem('shimmy_selected_model', model.name);
      } catch (e) {
        // Ignore localStorage errors
      }
    } else {
      try {
        localStorage.removeItem('shimmy_selected_model');
      } catch (e) {
        // Ignore localStorage errors
      }
    }
  }, []);
  
  const clientRef = useRef<DiscoveryClient | null>(null);
  const refreshTimerRef = useRef<NodeJS.Timeout | null>(null);

  // Initialize discovery client
  useEffect(() => {
    clientRef.current = new DiscoveryClient(config);
    
    // Subscribe to connection events
    const unsubscribeConnecting = clientRef.current.on('connecting', () => {
      setError(null);
      setIsReady(false);
    });
    
    const unsubscribeConnected = clientRef.current.on('connected', (event, data) => {
      console.log('[useShimmy] 🎉 Connected event received:', data);
      setError(null);
      setIsReady(true);
      // Models already loaded during connection, no need to refresh again
    });
    
    const unsubscribeDisconnected = clientRef.current.on('disconnected', () => {
      setIsReady(false);
      setModels([]);
      setBackends([]);
    });
    
    const unsubscribeError = clientRef.current.on('error', (event, data) => {
      setError(data?.message || 'Connection error');
      setIsReady(false);
    });
    
    const unsubscribeModelsUpdated = clientRef.current.on('models_updated', (event, newModels) => {
      // Models updated event fired - will be picked up by refresh()
    });
    
    if (config.autoConnect !== false) {
      refresh();
    }

    return () => {
      // Cleanup event listeners
      unsubscribeConnecting();
      unsubscribeConnected();
      unsubscribeDisconnected();
      unsubscribeError();
      unsubscribeModelsUpdated();
      
      if (refreshTimerRef.current) {
        clearInterval(refreshTimerRef.current);
      }
      if (clientRef.current) {
        clientRef.current.disconnect();
      }
    };
  }, []);

  // Set up auto-refresh timer
  useEffect(() => {
    if (refreshTimerRef.current) {
      clearInterval(refreshTimerRef.current);
    }

    const interval = config.refreshInterval || 5000;
    refreshTimerRef.current = setInterval(() => {
      refresh();
    }, interval);

    return () => {
      if (refreshTimerRef.current) {
        clearInterval(refreshTimerRef.current);
      }
    };
  }, [config.refreshInterval]);

  /**
   * Refresh backend discovery
   */
  const refresh = useCallback(async () => {
    if (!clientRef.current) return;

    try {
      setError(null);
      console.log('[useShimmy] Refresh called, isConnected:', clientRef.current.isConnected());
      
      // Only call discoverBackends if not already connected
      // The heartbeat will handle connection monitoring
      if (!clientRef.current.isConnected()) {
        console.log('[useShimmy] Not connected, calling discoverBackends()');
        const discoveredBackends = await clientRef.current.discoverBackends();
        console.log('[useShimmy] discoverBackends returned:', discoveredBackends.length, 'backends');
        setBackends(discoveredBackends);
        setIsReady(discoveredBackends.length > 0);
      } else {
        // Already connected - just update models
        console.log('[useShimmy] Already connected, skipping discoverBackends');
        setIsReady(true);
      }
      
      // Always refresh models list (in case new models added)
      const allModels = clientRef.current.getAllModels();
      console.log('[useShimmy] getAllModels returned:', allModels.length, 'models');
      setModels(allModels);
      
      // Auto-select model if none selected and models are available
      if (!selectedModel && allModels.length > 0) {
        let modelToSelect: ShimmyModel | null = null;
        
        // Try to restore from localStorage first
        try {
          const savedModelName = localStorage.getItem('shimmy_selected_model');
          if (savedModelName) {
            modelToSelect = allModels.find(m => m.name === savedModelName || m.displayName === savedModelName) || null;
          }
        } catch (e) {
          // Ignore localStorage errors
        }
        
        // If no saved model or saved model not found, select first ready model
        if (!modelToSelect) {
          modelToSelect = allModels.find(m => m.health_status === 'ready') || null;
        }
        
        // If no ready models, select first model (user can validate it later)
        if (!modelToSelect && allModels.length > 0) {
          modelToSelect = allModels[0];
        }
        
        if (modelToSelect) {
          setSelectedModel(modelToSelect);
        }
      }
      
      const clientError = clientRef.current.getLastError();
      if (clientError) {
        setError(clientError);
        setIsReady(false);
      }
      
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Discovery failed';
      setError(errorMessage);
      setIsReady(false);
      setBackends([]);
      setModels([]);
    }
  }, []);

  /**
   * Send a message to the selected model or specified model
   */
  const sendMessage = useCallback(async (
    message: string, 
    modelName?: string,
    options: SendMessageOptions = {}
  ): Promise<AsyncIterable<string>> => {
    
    if (!clientRef.current || !isReady) {
      throw new Error('Shimmy is not ready. Please wait for discovery to complete.');
    }

    // Determine which model to use
    let targetModelName: string;
    let targetBackend: ShimmyBackend | null = null;
    
    if (modelName) {
      // Use explicitly specified model
      targetModelName = modelName;
      targetBackend = clientRef.current.findBackendForModel(modelName);
      if (!targetBackend) {
        throw new Error(`Model "${modelName}" not found in any backend`);
      }
    } else if (selectedModel) {
      // Use currently selected model
      targetModelName = selectedModel.name;
      targetBackend = clientRef.current.findBackendForModel(selectedModel.name);
      if (!targetBackend) {
        throw new Error(`Selected model "${selectedModel.name}" not found in any backend`);
      }
      
      // Ensure selected model is ready
      if (selectedModel.health_status !== 'ready') {
        throw new Error(`Selected model "${selectedModel.displayName}" is not ready (${selectedModel.health_status}). Please select a different model.`);
      }
    } else {
      // No model specified and none selected - use first ready model
      const readyModels = clientRef.current.getReadyModels();
      if (readyModels.length === 0) {
        throw new Error('No ready models available. Please select a model first.');
      }
      
      const firstReadyModel = readyModels[0];
      targetModelName = firstReadyModel.name;
      targetBackend = clientRef.current.findBackendForModel(firstReadyModel.name);
      
      if (!targetBackend) {
        throw new Error('No healthy backends available');
      }
    }

    // Build the request payload
    const requestPayload = {
      model: targetModelName,
      prompt: message,
      stream: options.stream !== false, // Default to streaming
      temperature: options.temperature,
      max_tokens: options.max_tokens,
    };

    // CRITICAL: The discovery WebSocket (/ws/console) is ONLY for discovery operations
    // (get_models, select_model, etc.). Chat streaming requires /ws/generate endpoint
    // which has a different protocol. For now, always use HTTP for chat to ensure
    // compatibility with shimmy's /api/generate endpoint.
    
    // Use HTTP for chat with timeout (discovery WebSocket not suitable for chat)
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 30000); // 30 second timeout
    
    try {
      const response = await fetch(`${targetBackend.url}/api/generate`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(requestPayload),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(`Backend request failed: ${response.status} ${response.statusText}`);
      }
      
      return createStreamingIterable(response);
    } catch (error) {
      clearTimeout(timeoutId);
      if (error instanceof Error && error.name === 'AbortError') {
        throw new Error('Request timed out after 30 seconds. The model may be too large or the backend is unresponsive.');
      }
      throw error;
    }
  }, [isReady, selectedModel]);

  /**
   * Select a model and validate it if needed
   */
  const selectModel = useCallback(async (modelName: string): Promise<void> => {
    if (!clientRef.current) {
      throw new Error('Discovery client not initialized');
    }

    try {
      setError(null);
      const allModels = clientRef.current.getAllModels();
      const targetModel = allModels.find(m => m.name === modelName || m.displayName === modelName);
      
      if (!targetModel) {
        throw new Error(`Model "${modelName}" not found in available models`);
      }

      // If model hasn't been validated, validate it now
      if (targetModel.health_status === 'unknown') {
        setIsValidating(true);
        
        try {
          const result = await clientRef.current.validateModel(targetModel.name);
          
          // Properly update model status through state management
          await refresh(); // Refresh to get updated model states
          
          // Get the updated model after refresh
          const updatedModels = clientRef.current.getAllModels();
          const updatedTargetModel = updatedModels.find(m => m.name === modelName || m.displayName === modelName);
          
          if (!updatedTargetModel || updatedTargetModel.health_status !== 'ready') {
            throw new Error(`Model validation failed: ${updatedTargetModel?.health_error || 'Unknown error'}`);
          }
          
          // Successfully validated - select it
          setSelectedModel(updatedTargetModel);
          
        } finally {
          setIsValidating(false);
        }
      } else if (targetModel.health_status === 'ready') {
        // Model already validated and ready
        setSelectedModel(targetModel);
      } else if (targetModel.health_status === 'failed') {
        throw new Error(`Model "${modelName}" failed validation: ${targetModel.health_error || 'Model is not working'}`);
      } else if (targetModel.health_status === 'checking') {
        throw new Error(`Model "${modelName}" is currently being validated. Please wait and try again.`);
      } else {
        throw new Error(`Model "${modelName}" is in unknown state: ${targetModel.health_status}`);
      }
      
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to select model');
      setIsValidating(false);
      throw error; // Re-throw so caller can handle it
    }
  }, [refresh]);

  // Computed values
  const readyModels = models.filter(m => m.health_status === 'ready');
  
  const modelChooserData = models.map(model => ({
    model,
    displayName: model.displayName,
    subtitle: createModelSubtitle(model),
    status: model.health_status,
    statusColor: getStatusColor(model.health_status),
    statusText: getStatusText(model.health_status),
    isRecommended: model.health_status === 'ready' && (model.response_time_ms || 0) < 2000,
    metadata: {
      backend: model.backend,
      responseTime: model.response_time_ms ? `~${(model.response_time_ms / 1000).toFixed(1)}s` : undefined,
      errorMessage: model.health_error,
    },
  }));

  return {
    models,
    readyModels,
    selectedModel,
    backends,
    sendMessage,
    selectModel,
    modelChooserData,
    isReady,
    isValidating,
    error,
    refresh,
    backendCount: backends.filter((b: ShimmyBackend) => b.health === 'Ok').length,
  };
}

// Helper functions for model chooser data
function createModelSubtitle(model: ShimmyModel): string {
  const parts = [];
  
  if (model.backend_type) {
    parts.push(model.backend_type);
  }
  
  if (model.health_status === 'ready' && model.response_time_ms) {
    parts.push(`~${(model.response_time_ms / 1000).toFixed(1)}s`);
  }
  
  parts.push(getStatusText(model.health_status));
  
  return parts.join(' • ');
}

function getStatusColor(status: string): 'gray' | 'blue' | 'green' | 'red' {
  switch (status) {
    case 'ready': return 'green';
    case 'checking': return 'blue';
    case 'failed': return 'red';
    default: return 'gray';
  }
}

function getStatusText(status: string): string {
  switch (status) {
    case 'ready': return 'Ready';
    case 'checking': return 'Checking...';
    case 'failed': return 'Failed';
    default: return 'Unknown';
  }
}

/**
 * Create an async iterable from a WebSocket message
 */
async function* createWebSocketIterable(client: any, requestPayload: any): AsyncIterable<string> {
  // For now, let's use the chat message type from the WebSocket handler
  const chatRequest = {
    type: 'chat',
    message: requestPayload.prompt,
    model: requestPayload.model,
    stream: requestPayload.stream,
    timestamp: new Date().toISOString(),
  };

  // Send the message via WebSocket (this will be handled by the ConsoleWebSocketHandler)
  const response = await client.sendMessage(chatRequest);
  
  // For streaming, we'd need to handle WebSocket message streaming
  // For now, let's return the complete response as a single token
  if (typeof response === 'string') {
    yield response;
  } else if (response.content) {
    yield response.content;
  } else if (response.response) {
    yield response.response;
  }
}

/**
 * Create an async iterable from a streaming response
 */
async function* createStreamingIterable(response: Response): AsyncIterable<string> {
  const reader = response.body?.getReader();
  if (!reader) {
    throw new Error('Response body is not readable');
  }

  const decoder = new TextDecoder();
  let buffer = '';

  try {
    while (true) {
      const { done, value } = await reader.read();
      
      if (done) break;
      
      buffer += decoder.decode(value, { stream: true });
      
      // Process Server-Sent Events format
      const lines = buffer.split('\n');
      buffer = lines.pop() || '';  // Keep incomplete line in buffer
      
      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const data = line.slice(6).trim();
          
          if (data === '[DONE]') {
            return;
          }
          
          try {
            const parsed = JSON.parse(data);
            if (parsed.token) {
              yield parsed.token;
            } else if (typeof parsed === 'string') {
              yield parsed;
            }
          } catch {
            // If not JSON, yield as plain text
            if (data) {
              yield data;
            }
          }
        }
      }
    }
  } finally {
    reader.releaseLock();
  }
}