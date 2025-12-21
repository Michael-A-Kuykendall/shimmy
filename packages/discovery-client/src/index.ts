/**
 * @shimmy/discovery-client
 * 
 * Simple discovery client for Shimmy backends.
 * Hides IPC complexity from theme developers.
 */

// Main hook
export { useShimmy } from './hooks/useShimmy';

// Types for theme developers
export type {
  ShimmyBackend,
  ShimmyModel,
  ShimmyConfig,
  UseShimmyResult,
  SendMessageOptions,
  BackendCapabilities,
} from './types';

// Advanced usage - discovery client class
export { DiscoveryClient } from './client';

// Auto-startup service for theme makers
export { autoStartShimmy, ShimmyStartupService } from './startup-service';