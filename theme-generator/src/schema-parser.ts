import fetch from 'node-fetch';
import { ShimmySchema, MessageSpec, DiscoverySpec, StreamingSpec, BehaviorSpec, ComponentSpec } from './types.js';

export class SchemaParser {
  private schema: ShimmySchema | null = null;

  async loadSchema(contractPath?: string): Promise<ShimmySchema> {
    if (contractPath) {
      // Load from file
      const fs = await import('fs-extra');
      const schemaData = await fs.readFile(contractPath, 'utf8');
      this.schema = JSON.parse(schemaData);
    } else {
      // Auto-fetch from running shimmy
      this.schema = await this.fetchSchemaFromRunningShimmy();
    }

    return this.schema!;
  }

  private async fetchSchemaFromRunningShimmy(): Promise<ShimmySchema> {
    // Try discovery service ports
    const discoveryPorts = [11430, 11431, 11432, 11433, 11434];
    
    for (const port of discoveryPorts) {
      try {
        const discoveryResponse = await fetch(`http://127.0.0.1:${port}/api/discovery`);
        if (discoveryResponse.ok) {
          const data = await discoveryResponse.json() as any;
          const backends = Array.isArray(data.backends) ? data.backends : [];
          const validBackend = backends.find((b: any) => 
            b?.validation?.health_check && 
            b?.validation?.models_endpoint && 
            b?.validation?.websocket_endpoint
          ) || backends[0];
          
          if (validBackend?.port) {
            const schemaResponse = await fetch(`http://127.0.0.1:${validBackend.port}/__shimmy__/schema`);
            if (schemaResponse.ok) {
              return await schemaResponse.json();
            }
          }
        }
      } catch (e) {
        // Continue to next port
      }
    }

    // If no running shimmy found, return default schema based on contract
    return this.getDefaultSchema();
  }

  private getDefaultSchema(): ShimmySchema {
    return {
      websocket_messages: [
        {
          type: 'get_models',
          direction: 'out',
          required_fields: ['type'],
          optional_fields: [],
          description: 'Request available models'
        },
        {
          type: 'models_response',
          direction: 'in',
          required_fields: ['type', 'success', 'models', 'timestamp'],
          optional_fields: ['selected_model', 'error'],
          description: 'Response with model list'
        },
        {
          type: 'select_model',
          direction: 'out',
          required_fields: ['type', 'model_name'],
          optional_fields: ['session_id'],
          description: 'Select a model for use'
        },
        {
          type: 'model_selected',
          direction: 'in',
          required_fields: ['type', 'success', 'model_name', 'timestamp'],
          optional_fields: ['session_id', 'model_info', 'error', 'note'],
          description: 'Confirm model selection'
        },
        {
          type: 'chat',
          direction: 'out',
          required_fields: ['type', 'message'],
          optional_fields: ['model', 'session_id'],
          description: 'Send chat message for generation'
        },
        {
          type: 'chat_token',
          direction: 'in',
          required_fields: ['type', 'token'],
          optional_fields: ['session_id'],
          description: 'Streaming token response'
        },
        {
          type: 'generation_complete',
          direction: 'in',
          required_fields: ['type', 'total_tokens', 'generation_time_ms', 'tokens_per_second'],
          optional_fields: ['session_id'],
          description: 'Generation completion notification'
        },
        {
          type: 'get_metrics',
          direction: 'out',
          required_fields: ['type'],
          optional_fields: [],
          description: 'Request system metrics'
        },
        {
          type: 'metrics_response',
          direction: 'in',
          required_fields: ['type', 'success', 'cpu_usage', 'memory_usage', 'timestamp'],
          optional_fields: ['gpu_usage', 'active_model', 'error'],
          description: 'Response with system metrics'
        }
      ],
      discovery: {
        port: 11430,
        endpoint: '/api/discovery',
        required_fields: ['backends'],
        validation_fields: ['health_check', 'models_endpoint', 'websocket_endpoint']
      },
      streaming: {
        enabled: true,
        token_field: 'token',
        completion_marker: 'generation_complete'
      },
      required_behaviors: [
        {
          name: 'model_selection',
          required: true,
          description: 'Must implement model chooser with selection capability'
        },
        {
          name: 'chat_streaming',
          required: true,
          description: 'Must handle streaming token responses'
        },
        {
          name: 'connection_resilience',
          required: true,
          description: 'Must handle reconnection and error states'
        },
        {
          name: 'discovery_integration',
          required: true,
          description: 'Must use discovery service to resolve backend'
        }
      ],
      components: [
        {
          name: 'ModelChooser',
          props: [
            { name: 'models', type: 'ShimmyModel[]', required: true },
            { name: 'selectedModel', type: 'ShimmyModel | null', required: false },
            { name: 'onModelSelect', type: '(model: ShimmyModel) => void', required: true },
            { name: 'loading', type: 'boolean', required: false },
            { name: 'error', type: 'string | null', required: false }
          ],
          hooks: ['useModels', 'useWebSocket'],
          required: true
        },
        {
          name: 'Chat',
          props: [
            { name: 'messages', type: 'ChatMessage[]', required: true },
            { name: 'onSendMessage', type: '(message: string) => void', required: true },
            { name: 'isGenerating', type: 'boolean', required: false },
            { name: 'selectedModel', type: 'string | null', required: false }
          ],
          hooks: ['useChat', 'useWebSocket'],
          required: true
        },
        {
          name: 'Metrics',
          props: [
            { name: 'metrics', type: 'ShimmyMetrics | null', required: false },
            { name: 'loading', type: 'boolean', required: false },
            { name: 'error', type: 'string | null', required: false }
          ],
          hooks: ['useMetrics'],
          required: false
        }
      ]
    };
  }

  extractMessageTypes(): string[] {
    if (!this.schema) {
      throw new Error('Schema not loaded. Call loadSchema() first.');
    }
    return this.schema.websocket_messages.map(msg => msg.type);
  }

  extractComponentNames(): string[] {
    if (!this.schema) {
      throw new Error('Schema not loaded. Call loadSchema() first.');
    }
    return this.schema.components.map(comp => comp.name);
  }

  extractHookNames(): string[] {
    if (!this.schema) {
      throw new Error('Schema not loaded. Call loadSchema() first.');
    }
    const hooks = new Set<string>();
    this.schema.components.forEach(comp => {
      comp.hooks.forEach(hook => hooks.add(hook));
    });
    return Array.from(hooks);
  }

  getSchema(): ShimmySchema {
    if (!this.schema) {
      throw new Error('Schema not loaded. Call loadSchema() first.');
    }
    return this.schema;
  }
}