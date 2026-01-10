use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Configuration for response caching
#[derive(Debug, Clone)]
pub struct ResponseCacheConfig {
    pub enabled: bool,
    pub max_entries: usize,
    pub max_size_mb: usize,
    pub default_ttl: Duration,
    pub max_prompt_length: usize,
}

impl Default for ResponseCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 1000,
            max_size_mb: 512,                       // 512MB cache
            default_ttl: Duration::from_secs(3600), // 1 hour
            max_prompt_length: 8192,                // Don't cache very long prompts
        }
    }
}

/// Cache key for responses - includes prompt and generation parameters
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub prompt_hash: String,
    pub model_name: String,
    pub max_tokens: usize,
    pub temperature: String, // Store as string to handle floating point comparison
    pub top_p: String,
    pub stop_sequences: Vec<String>,
}

impl CacheKey {
    pub fn new(
        prompt: &str,
        model_name: &str,
        max_tokens: usize,
        temperature: f32,
        top_p: f32,
        stop_sequences: &[String],
    ) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        prompt.hash(&mut hasher);
        let prompt_hash = format!("{:x}", hasher.finish());

        Self {
            prompt_hash,
            model_name: model_name.to_string(),
            max_tokens,
            temperature: format!("{:.3}", temperature),
            top_p: format!("{:.3}", top_p),
            stop_sequences: stop_sequences.to_vec(),
        }
    }
}

/// Cached response entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResponse {
    pub response: String,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub access_count: u64,
    pub response_time: Duration,
    pub size_bytes: usize,
}

impl CachedResponse {
    pub fn new(response: String, response_time: Duration) -> Self {
        let now = SystemTime::now();
        let size_bytes = response.len();

        Self {
            response,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            response_time,
            size_bytes,
        }
    }

    pub fn access(&mut self) {
        self.last_accessed = SystemTime::now();
        self.access_count += 1;
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or_default()
            > ttl
    }
}

/// Response cache with LRU eviction and TTL
pub struct ResponseCache {
    cache: Arc<RwLock<HashMap<CacheKey, CachedResponse>>>,
    config: ResponseCacheConfig,
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub entries: usize,
    pub total_size_bytes: usize,
    pub average_response_time_saved: Duration,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            self.hits as f64 / (self.hits + self.misses) as f64
        }
    }
}

impl Default for ResponseCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseCache {
    pub fn new() -> Self {
        Self::with_config(ResponseCacheConfig::default())
    }

    pub fn with_config(config: ResponseCacheConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get cached response if available and not expired
    pub async fn get(&self, key: &CacheKey) -> Option<String> {
        if !self.config.enabled {
            return None;
        }

        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = cache.get_mut(key) {
            if entry.is_expired(self.config.default_ttl) {
                // Remove expired entry
                cache.remove(key);
                stats.misses += 1;
                debug!("Cache miss: expired entry");
                None
            } else {
                // Hit! Update access info
                entry.access();
                stats.hits += 1;
                stats.average_response_time_saved = Duration::from_millis(
                    (stats.average_response_time_saved.as_millis() as u64 * (stats.hits - 1)
                        + entry.response_time.as_millis() as u64)
                        / stats.hits,
                );
                debug!("Cache hit for key: {:?}", key);
                Some(entry.response.clone())
            }
        } else {
            stats.misses += 1;
            debug!("Cache miss: entry not found");
            None
        }
    }

    /// Store response in cache
    pub async fn put(&self, key: CacheKey, response: String, response_time: Duration) {
        if !self.config.enabled || response.len() > self.config.max_prompt_length {
            return;
        }

        let entry = CachedResponse::new(response, response_time);
        let entry_size = entry.size_bytes;

        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        // Check if we need to evict entries
        while cache.len() >= self.config.max_entries
            || stats.total_size_bytes + entry_size > self.config.max_size_mb * 1024 * 1024
        {
            if let Some(lru_key) = self.find_lru_key(&cache).await {
                if let Some(removed) = cache.remove(&lru_key) {
                    stats.total_size_bytes -= removed.size_bytes;
                    stats.evictions += 1;
                    debug!("Evicted LRU cache entry");
                }
            } else {
                break; // No entries to evict
            }
        }

        // Insert new entry
        cache.insert(key.clone(), entry);
        stats.entries = cache.len();
        stats.total_size_bytes += entry_size;

        info!("Cached response for key: {:?}", key);
    }

    /// Find least recently used key for eviction
    async fn find_lru_key(&self, cache: &HashMap<CacheKey, CachedResponse>) -> Option<CacheKey> {
        cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(key, _)| key.clone())
    }

    /// Clear expired entries
    pub async fn cleanup_expired(&self) {
        if !self.config.enabled {
            return;
        }

        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        let initial_count = cache.len();
        let initial_size = stats.total_size_bytes;

        cache.retain(|_, entry| !entry.is_expired(self.config.default_ttl));

        // Recalculate size
        stats.total_size_bytes = cache.values().map(|e| e.size_bytes).sum();
        stats.entries = cache.len();

        let removed_count = initial_count - cache.len();
        let size_freed = initial_size - stats.total_size_bytes;

        if removed_count > 0 {
            info!(
                "Cleaned up {} expired cache entries, freed {} bytes",
                removed_count, size_freed
            );
        }
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        cache.clear();
        stats.entries = 0;
        stats.total_size_bytes = 0;
        stats.evictions += stats.entries as u64;

        info!("Cache cleared");
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let stats_guard = self.stats.read().await;
        let mut stats = (*stats_guard).clone();
        stats.entries = cache.len();
        stats.total_size_bytes = cache.values().map(|e| e.size_bytes).sum();
        stats
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(&self) {
        let cache = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;
                cache.cleanup_expired().await;
            }
        });
    }
}

impl Clone for ResponseCache {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            config: self.config.clone(),
            stats: self.stats.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    fn create_test_key(prompt: &str, model: &str) -> CacheKey {
        CacheKey::new(prompt, model, 100, 0.7, 0.9, &[])
    }

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = ResponseCache::new();
        let key = create_test_key("test prompt", "test-model");

        // Miss first
        assert!(cache.get(&key).await.is_none());

        // Store and hit
        cache
            .put(
                key.clone(),
                "test response".to_string(),
                Duration::from_millis(100),
            )
            .await;
        assert_eq!(cache.get(&key).await, Some("test response".to_string()));

        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = ResponseCacheConfig {
            default_ttl: Duration::from_millis(50),
            ..Default::default()
        };
        let cache = ResponseCache::with_config(config);
        let key = create_test_key("expire test", "test-model");

        cache
            .put(
                key.clone(),
                "response".to_string(),
                Duration::from_millis(10),
            )
            .await;
        assert!(cache.get(&key).await.is_some());

        // Wait for expiration
        sleep(Duration::from_millis(60)).await;
        assert!(cache.get(&key).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let config = ResponseCacheConfig {
            max_entries: 2,
            ..Default::default()
        };
        let cache = ResponseCache::with_config(config);

        // Fill cache
        cache
            .put(
                create_test_key("1", "model"),
                "resp1".to_string(),
                Duration::from_millis(10),
            )
            .await;
        cache
            .put(
                create_test_key("2", "model"),
                "resp2".to_string(),
                Duration::from_millis(10),
            )
            .await;

        // Access first entry to make it more recent
        cache.get(&create_test_key("1", "model")).await;

        // Add third entry, should evict second (LRU)
        cache
            .put(
                create_test_key("3", "model"),
                "resp3".to_string(),
                Duration::from_millis(10),
            )
            .await;

        assert!(cache.get(&create_test_key("1", "model")).await.is_some());
        assert!(cache.get(&create_test_key("2", "model")).await.is_none());
        assert!(cache.get(&create_test_key("3", "model")).await.is_some());
    }

    #[test]
    fn test_cache_key_generation() {
        let key1 = CacheKey::new("hello", "model1", 100, 0.7, 0.9, &[]);
        let key2 = CacheKey::new("hello", "model1", 100, 0.7, 0.9, &[]);
        let key3 = CacheKey::new("hello", "model2", 100, 0.7, 0.9, &[]);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = ResponseCache::new();
        let key = create_test_key("stats test", "model");

        // Miss
        cache.get(&key).await;

        // Hit
        cache
            .put(
                key.clone(),
                "response".to_string(),
                Duration::from_millis(100),
            )
            .await;
        cache.get(&key).await;
        cache.get(&key).await;

        let stats = cache.stats().await;
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 2.0 / 3.0);
    }
}
