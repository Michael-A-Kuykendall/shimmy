use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Lightweight request routing and connection intelligence
/// Constitutional compliance: <50MB memory, <1ms routing overhead
pub struct RouteManager {
    instances: Arc<RwLock<HashMap<String, ModelInstance>>>,
    config: RoutingConfig,
}

#[derive(Debug, Clone)]
pub struct RoutingConfig {
    pub health_check_interval: Duration,
    pub failure_threshold: u32,
    pub recovery_threshold: u32,
    pub request_timeout: Duration,
    pub max_retries: u32,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(30),
            failure_threshold: 3,
            recovery_threshold: 2,
            request_timeout: Duration::from_secs(30),
            max_retries: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInstance {
    pub id: String,
    pub model_name: String,
    pub endpoint: String,
    pub status: InstanceStatus,
    pub last_check: SystemTime,
    pub response_time: Duration,
    pub error_count: u32,
    pub success_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstanceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Serialize)]
pub struct RoutingStats {
    pub total_instances: usize,
    pub healthy_instances: usize,
    pub total_requests: u64,
    pub successful_routes: u64,
    pub failed_routes: u64,
    pub average_route_time: Duration,
}

impl RouteManager {
    pub fn new() -> Self {
        Self::with_config(RoutingConfig::default())
    }

    pub fn with_config(config: RoutingConfig) -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Register a model instance for routing
    pub async fn register_instance(&self, instance: ModelInstance) -> Result<()> {
        let mut instances = self.instances.write().await;
        instances.insert(instance.id.clone(), instance.clone());
        info!("Registered instance: {} for model: {}", instance.id, instance.model_name);
        Ok(())
    }

    /// Unregister a model instance
    pub async fn unregister_instance(&self, instance_id: &str) -> Result<bool> {
        let mut instances = self.instances.write().await;
        let removed = instances.remove(instance_id).is_some();
        if removed {
            info!("Unregistered instance: {}", instance_id);
        }
        Ok(removed)
    }

    /// Find the best instance for a model request
    pub async fn route_request(&self, model_name: &str) -> Option<ModelInstance> {
        let instances = self.instances.read().await;
        
        // Find healthy instances for the requested model
        let candidates: Vec<_> = instances
            .values()
            .filter(|instance| {
                instance.model_name == model_name && instance.status == InstanceStatus::Healthy
            })
            .collect();

        if candidates.is_empty() {
            warn!("No healthy instances available for model: {}", model_name);
            return None;
        }

        // Simple load balancing: choose instance with best performance
        let best_instance = candidates
            .iter()
            .min_by_key(|instance| instance.response_time)
            .copied()
            .cloned();

        if let Some(ref instance) = best_instance {
            info!("Routed request for model '{}' to instance '{}'", model_name, instance.id);
        }

        best_instance
    }

    /// Check health of all instances
    pub async fn health_check(&self) -> Result<()> {
        let mut instances = self.instances.write().await;
        
        for (id, instance) in instances.iter_mut() {
            let start_time = SystemTime::now();
            
            // Simplified health check - in production this would make HTTP requests
            let is_healthy = self.check_instance_health(instance).await;
            
            let check_duration = start_time.elapsed().unwrap_or_default();
            instance.last_check = SystemTime::now();
            
            if is_healthy {
                instance.success_count += 1;
                instance.response_time = check_duration;
                
                if instance.status != InstanceStatus::Healthy && 
                   instance.success_count >= self.config.recovery_threshold {
                    instance.status = InstanceStatus::Healthy;
                    info!("Instance {} recovered to healthy status", id);
                }
            } else {
                instance.error_count += 1;
                
                if instance.error_count >= self.config.failure_threshold {
                    instance.status = InstanceStatus::Unhealthy;
                    warn!("Instance {} marked as unhealthy", id);
                } else {
                    instance.status = InstanceStatus::Degraded;
                }
            }
        }
        
        Ok(())
    }

    /// Simplified health check - placeholder for actual HTTP health checks
    async fn check_instance_health(&self, _instance: &ModelInstance) -> bool {
        // In a real implementation, this would make HTTP requests to the instance
        // For now, we'll simulate health checks
        true
    }

    /// Get routing statistics
    pub async fn get_stats(&self) -> RoutingStats {
        let instances = self.instances.read().await;
        
        let healthy_count = instances
            .values()
            .filter(|i| i.status == InstanceStatus::Healthy)
            .count();

        let total_requests = instances.values().map(|i| i.success_count + i.error_count).sum::<u32>() as u64;
        let successful_requests = instances.values().map(|i| i.success_count).sum::<u32>() as u64;

        let avg_response_time = if !instances.is_empty() {
            let total_time: u128 = instances.values().map(|i| i.response_time.as_millis()).sum();
            Duration::from_millis((total_time / instances.len() as u128) as u64)
        } else {
            Duration::from_millis(0)
        };

        RoutingStats {
            total_instances: instances.len(),
            healthy_instances: healthy_count,
            total_requests,
            successful_routes: successful_requests,
            failed_routes: total_requests - successful_requests,
            average_route_time: avg_response_time,
        }
    }

    /// Start background health checking task
    pub fn start_health_checker(&self) {
        let manager = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(manager.config.health_check_interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = manager.health_check().await {
                    warn!("Health check failed: {}", e);
                }
            }
        });
    }

    /// List all registered instances
    pub async fn list_instances(&self) -> Vec<ModelInstance> {
        let instances = self.instances.read().await;
        instances.values().cloned().collect()
    }
}

impl Clone for RouteManager {
    fn clone(&self) -> Self {
        Self {
            instances: self.instances.clone(),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_instance(id: &str, model: &str, endpoint: &str) -> ModelInstance {
        ModelInstance {
            id: id.to_string(),
            model_name: model.to_string(),
            endpoint: endpoint.to_string(),
            status: InstanceStatus::Healthy,
            last_check: SystemTime::now(),
            response_time: Duration::from_millis(100),
            error_count: 0,
            success_count: 1,
        }
    }

    #[tokio::test]
    async fn test_instance_registration() {
        let manager = RouteManager::new();
        let instance = create_test_instance("test-1", "phi3-mini", "http://localhost:11435");

        assert!(manager.register_instance(instance.clone()).await.is_ok());
        
        let instances = manager.list_instances().await;
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].id, "test-1");
    }

    #[tokio::test]
    async fn test_request_routing() {
        let manager = RouteManager::new();
        
        // Register instances for different models
        let instance1 = create_test_instance("test-1", "phi3-mini", "http://localhost:11435");
        let instance2 = create_test_instance("test-2", "llama-7b", "http://localhost:11436");
        
        manager.register_instance(instance1).await.unwrap();
        manager.register_instance(instance2).await.unwrap();

        // Test routing to specific model
        let route = manager.route_request("phi3-mini").await;
        assert!(route.is_some());
        assert_eq!(route.unwrap().id, "test-1");

        // Test routing to non-existent model
        let no_route = manager.route_request("non-existent").await;
        assert!(no_route.is_none());
    }

    #[tokio::test]
    async fn test_health_stats() {
        let manager = RouteManager::new();
        let instance = create_test_instance("test-1", "phi3-mini", "http://localhost:11435");
        
        manager.register_instance(instance).await.unwrap();
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_instances, 1);
        assert_eq!(stats.healthy_instances, 1);
    }

    #[tokio::test]
    async fn test_instance_unregistration() {
        let manager = RouteManager::new();
        let instance = create_test_instance("test-1", "phi3-mini", "http://localhost:11435");

        manager.register_instance(instance).await.unwrap();
        assert_eq!(manager.list_instances().await.len(), 1);

        let removed = manager.unregister_instance("test-1").await.unwrap();
        assert!(removed);
        assert_eq!(manager.list_instances().await.len(), 0);

        // Try to remove non-existent instance
        let not_removed = manager.unregister_instance("non-existent").await.unwrap();
        assert!(!not_removed);
    }
}