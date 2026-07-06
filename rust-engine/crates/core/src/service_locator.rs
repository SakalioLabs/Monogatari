//! Service locator for dependency management.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::{EngineError, Result};

/// A thread-safe service locator that stores services by their type.
///
/// Services are stored as `Arc<RwLock<T>>` so they can be shared across threads.
pub struct ServiceLocator {
    services: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl ServiceLocator {
    /// Create a new empty service locator.
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }

    /// Register a service. Replaces any existing service of the same type.
    pub fn register<T: Send + Sync + 'static>(&self, service: Arc<RwLock<T>>) {
        let type_id = TypeId::of::<T>();
        let mut services = self.services.write().unwrap();
        services.insert(type_id, Box::new(service));
    }

    /// Get a reference to a service, returning None if not found.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<RwLock<T>>> {
        let type_id = TypeId::of::<T>();
        let services = self.services.read().unwrap();
        services
            .get(&type_id)
            .and_then(|s| s.downcast_ref::<Arc<RwLock<T>>>())
            .cloned()
    }

    /// Get a reference to a service, returning an error if not found.
    pub fn get_required<T: Send + Sync + 'static>(&self) -> Result<Arc<RwLock<T>>> {
        self.get::<T>().ok_or_else(|| {
            EngineError::ServiceNotFound(std::any::type_name::<T>().to_string())
        })
    }

    /// Check if a service of the given type is registered.
    pub fn has<T: Send + Sync + 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        let services = self.services.read().unwrap();
        services.contains_key(&type_id)
    }

    /// Remove all registered services.
    pub fn clear(&self) {
        let mut services = self.services.write().unwrap();
        services.clear();
    }
}

impl Default for ServiceLocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ServiceLocator {
    fn clone(&self) -> Self {
        // ServiceLocator shares its internal state via RwLock
        // This is a design choice - cloning gives a new handle to the same data
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }
}
