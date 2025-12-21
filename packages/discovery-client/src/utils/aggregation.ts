/**
 * Multi-backend aggregation and load balancing
 */

import { ShimmyBackend, ShimmyModel } from '../types';

/**
 * Aggregate models from multiple backends, removing duplicates
 */
export function aggregateModels(backends: ShimmyBackend[]): ShimmyModel[] {
  const modelMap = new Map<string, ShimmyModel>();
  
  for (const backend of backends) {
    if (backend.health !== 'Ok') continue;
    
    for (const model of backend.models) {
      // Use display name as the key to deduplicate
      const key = model.displayName;
      
      if (!modelMap.has(key)) {
        modelMap.set(key, {
          ...model,
          backend: backend.id, // Track which backend has this model
        });
      }
    }
  }
  
  return Array.from(modelMap.values()).sort((a, b) => 
    a.displayName.localeCompare(b.displayName)
  );
}

/**
 * Find the best backend for a specific model
 */
export function findBestBackendForModel(
  modelName: string, 
  backends: ShimmyBackend[],
  preferredBackends: string[] = []
): ShimmyBackend | null {
  
  const healthyBackends = backends.filter(b => b.health === 'Ok');
  const backendsWithModel = healthyBackends.filter(backend =>
    backend.models.some(m => 
      m.name === modelName || m.displayName === modelName
    )
  );
  
  if (backendsWithModel.length === 0) {
    return null;
  }
  
  // Prefer backends in the preferred list
  for (const preferredId of preferredBackends) {
    const preferred = backendsWithModel.find(b => b.id === preferredId);
    if (preferred) return preferred;
  }
  
  // Return the first available backend
  return backendsWithModel[0];
}

/**
 * Load balance requests across multiple backends
 */
export class LoadBalancer {
  private roundRobinIndex = 0;
  
  /**
   * Get next backend using round-robin strategy
   */
  getNextBackend(backends: ShimmyBackend[]): ShimmyBackend | null {
    const healthyBackends = backends.filter(b => b.health === 'Ok');
    
    if (healthyBackends.length === 0) {
      return null;
    }
    
    const backend = healthyBackends[this.roundRobinIndex % healthyBackends.length];
    this.roundRobinIndex++;
    
    return backend;
  }
  
  /**
   * Get backend with least load (by model count - simple heuristic)
   */
  getLeastLoadedBackend(backends: ShimmyBackend[]): ShimmyBackend | null {
    const healthyBackends = backends.filter(b => b.health === 'Ok');
    
    if (healthyBackends.length === 0) {
      return null;
    }
    
    return healthyBackends.reduce((least, current) => 
      current.models.length < least.models.length ? current : least
    );
  }
}

/**
 * Health check and monitoring utilities
 */
export class BackendHealthMonitor {
  private healthHistory = new Map<string, boolean[]>();
  private readonly maxHistoryLength = 10;
  
  /**
   * Record a health check result for a backend
   */
  recordHealthCheck(backendId: string, isHealthy: boolean): void {
    if (!this.healthHistory.has(backendId)) {
      this.healthHistory.set(backendId, []);
    }
    
    const history = this.healthHistory.get(backendId)!;
    history.push(isHealthy);
    
    // Keep only recent history
    if (history.length > this.maxHistoryLength) {
      history.shift();
    }
  }
  
  /**
   * Get health score for a backend (0-1, higher is better)
   */
  getHealthScore(backendId: string): number {
    const history = this.healthHistory.get(backendId);
    if (!history || history.length === 0) {
      return 0.5; // Unknown, assume neutral
    }
    
    const healthyCount = history.filter(h => h).length;
    return healthyCount / history.length;
  }
  
  /**
   * Get backends sorted by health score
   */
  getRankedBackends(backends: ShimmyBackend[]): ShimmyBackend[] {
    return backends
      .filter(b => b.health === 'Ok')
      .sort((a, b) => this.getHealthScore(b.id) - this.getHealthScore(a.id));
  }
}

/**
 * Simple failover manager
 */
export class FailoverManager {
  private failedBackends = new Set<string>();
  private retryTimeouts = new Map<string, number>();
  private readonly retryDelay = 30000; // 30 seconds
  
  /**
   * Mark a backend as failed
   */
  markBackendFailed(backendId: string): void {
    this.failedBackends.add(backendId);
    
    // Schedule retry
    const existingTimeout = this.retryTimeouts.get(backendId);
    if (existingTimeout) {
      clearTimeout(existingTimeout);
    }
    
    const timeout = window.setTimeout(() => {
      this.failedBackends.delete(backendId);
      this.retryTimeouts.delete(backendId);
    }, this.retryDelay);
    
    this.retryTimeouts.set(backendId, timeout);
  }
  
  /**
   * Check if a backend is currently marked as failed
   */
  isBackendFailed(backendId: string): boolean {
    return this.failedBackends.has(backendId);
  }
  
  /**
   * Get available backends (excluding failed ones)
   */
  getAvailableBackends(backends: ShimmyBackend[]): ShimmyBackend[] {
    return backends.filter(b => 
      b.health === 'Ok' && !this.isBackendFailed(b.id)
    );
  }
  
  /**
   * Clear all failure records
   */
  reset(): void {
    this.failedBackends.clear();
    this.retryTimeouts.forEach(timeout => clearTimeout(timeout));
    this.retryTimeouts.clear();
  }
}