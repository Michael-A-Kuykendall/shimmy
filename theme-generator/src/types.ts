export interface ShimmySchema {
  websocket_messages: MessageSpec[];
  discovery: DiscoverySpec;
  streaming: StreamingSpec;
  required_behaviors: BehaviorSpec[];
  components: ComponentSpec[];
}

export interface MessageSpec {
  type: string;
  direction: 'in' | 'out' | 'both';
  required_fields: string[];
  optional_fields: string[];
  description?: string;
}

export interface DiscoverySpec {
  port: number;
  endpoint: string;
  required_fields: string[];
  validation_fields: string[];
}

export interface StreamingSpec {
  enabled: boolean;
  token_field: string;
  completion_marker: string;
}

export interface BehaviorSpec {
  name: string;
  required: boolean;
  description: string;
}

export interface ComponentSpec {
  name: string;
  props: ComponentProp[];
  hooks: string[];
  required: boolean;
}

export interface ComponentProp {
  name: string;
  type: string;
  required: boolean;
  description?: string;
}

export interface GeneratorConfig {
  themeName: string;
  template: 'react-vite' | 'react-nextjs' | 'vanilla';
  outputDir: string;
  contractPath?: string;
  skipInstall?: boolean;
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

export interface WebSocketMessage {
  type: string;
  [key: string]: any;
}

export interface ModelsResponse {
  type: 'models_response';
  success: boolean;
  models: ShimmyModel[];
  selected_model?: string | null;
  timestamp: string;
  error?: string;
}

export interface SelectModelRequest {
  type: 'select_model';
  model_name: string;
  session_id?: string;
}

export interface ModelSelectionResponse {
  type: 'model_selected';
  success: boolean;
  model_name: string;
  session_id?: string | null;
  model_info?: any;
  error?: string | null;
  timestamp: string;
  note?: string;
}

export interface ChatRequest {
  type: 'chat';
  message: string;
  model?: string;
  session_id?: string;
}

export interface ChatTokenResponse {
  type: 'chat_token';
  token: string;
  session_id?: string;
}

export interface GenerationCompleteResponse {
  type: 'generation_complete';
  session_id?: string;
  total_tokens: number;
  generation_time_ms: number;
  tokens_per_second: number;
}

export interface ShimmyMetrics {
  cpu_usage_percent: number;
  memory_usage_percent: number;
  memory_used_gb: number;
  memory_total_gb: number;
  gpu_usage_percent?: number;
  gpu_memory_used_gb?: number;
  gpu_memory_total_gb?: number;
  gpu_temperature_c?: number;
  tokens_per_second?: number;
  total_tokens_generated: number;
  active_sessions: number;
  queue_length: number;
  average_response_time_ms: number;
  requests_per_minute: number;
  uptime_seconds: number;
  model_memory_usage_gb?: number;
  context_usage_percent?: number;
  timestamp: number;
}

export interface TemplateContext {
  themeName: string;
  schema: ShimmySchema;
  messageTypes: string[];
  componentNames: string[];
  hookNames: string[];
  discoveryConfig: DiscoverySpec;
  streamingConfig: StreamingSpec;
  packageName: string;
  capitalizedName: string;
}