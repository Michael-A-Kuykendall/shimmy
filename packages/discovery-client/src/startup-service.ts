/**
 * Shimmy Auto-Startup Service (Browser-Compatible)
 * 
 * Automatically ensures shimmy is running when themes need it.
 * Theme makers just start their theme and everything works.
 */

export interface StartupConfig {
  preferredPorts?: number[];
  maxWaitTime?: number;
  startupEndpoint?: string;
}

export class ShimmyStartupService {
  private config: Required<StartupConfig>;
  private startupPromise: Promise<boolean> | null = null;

  constructor(config: StartupConfig = {}) {
    this.config = {
      preferredPorts: config.preferredPorts || [],  // REMOVED: No port probing! Use IPC only!
      maxWaitTime: config.maxWaitTime || 30000, // 30 seconds
      startupEndpoint: config.startupEndpoint || 'http://127.0.0.1:11440/startup', // Future shimmy management API
    };
  }

  /**
   * Ensure shimmy is running - start it if needed
   */
  async ensureShimmyRunning(): Promise<boolean> {
    // If already starting, wait for that process
    if (this.startupPromise) {
      return this.startupPromise;
    }

    // Check if shimmy is already running
    if (await this.isShimmyRunning()) {
      return true;
    }

    // Start shimmy
    this.startupPromise = this.startShimmy();
    const result = await this.startupPromise;
    this.startupPromise = null;
    return result;
  }

  /**
   * Check if shimmy is already running via IPC (Node.js) or HTTP API (browser)
   * NOTE: No port probing fallback - themes MUST use IPC!
   */
  private async isShimmyRunning(): Promise<boolean> {
    // For Node.js themes: use IPC discovery
    try {
      const ipcModule = await import('./ipc-client');
      if (ipcModule.isIPCAvailable()) {
        const backends = await ipcModule.queryIPCDiscovery();
        return backends && backends.length > 0;
      }
    } catch {
      // IPC not available or failed
    }

    // For browser or other environments: port probing is NOT supported
    // User must have shimmy running manually
    return false;
  }

  /**
   * Request shimmy startup via management API (future feature)
   */
  private async startShimmy(): Promise<boolean> {
    console.log('🚀 Requesting shimmy startup...');
    
    try {
      // Try to request startup via management API
      const response = await fetch(this.config.startupEndpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          action: 'start',
          config: {
            model: 'champion', // Use Champion model by default
            bind: 'auto',
          }
        }),
        signal: AbortSignal.timeout(5000),
      });

      if (response.ok) {
        return this.waitForShimmyReady();
      }
    } catch (error) {
      // Management API not available - user needs to start shimmy manually
      console.warn('⚠️ Shimmy management API not available');
    }

    // For now, show helpful message to user  
    console.error('❌ Please start shimmy manually with: shimmy serve --bind auto');
    return false;
  }

  /**
   * Wait for shimmy to be ready and responding
   */
  private async waitForShimmyReady(): Promise<boolean> {
    const startTime = Date.now();
    const checkInterval = 1000; // Check every second

    while (Date.now() - startTime < this.config.maxWaitTime) {
      if (await this.isShimmyRunning()) {
        console.log('✅ Shimmy is ready!');
        return true;
      }
      
      // Wait before next check
      await new Promise(resolve => setTimeout(resolve, checkInterval));
    }

    console.error('❌ Shimmy failed to start within timeout');
    return false;
  }

}

/**
 * Global startup service instance
 */
let globalStartupService: ShimmyStartupService | null = null;

/**
 * Get or create the global startup service
 */
export function getStartupService(config?: StartupConfig): ShimmyStartupService {
  if (!globalStartupService) {
    globalStartupService = new ShimmyStartupService(config);
  }
  return globalStartupService;
}

/**
 * Auto-start shimmy (call this from themes/discovery client)
 */
export async function autoStartShimmy(config?: StartupConfig): Promise<boolean> {
  const service = getStartupService(config);
  return service.ensureShimmyRunning();
}