//! Event bus for decoupled communication between engine systems.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

type Callback = Box<dyn Fn(&dyn Any) + Send + Sync>;

/// A type-erased event bus that supports publish/subscribe patterns.
///
/// Events are identified by their Rust type. Subscribers register callbacks
/// for specific event types and receive a reference to the event data.
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<TypeId, Vec<Callback>>>>,
}

impl EventBus {
    /// Create a new empty event bus.
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to events of type `T`.
    ///
    /// The callback will be called with a reference to the event data
    /// whenever an event of type `T` is published.
    pub fn subscribe<T: 'static>(&self, callback: impl Fn(&T) + Send + Sync + 'static) {
        let type_id = TypeId::of::<T>();
        let wrapped: Callback = Box::new(move |event: &dyn Any| {
            if let Some(event) = event.downcast_ref::<T>() {
                callback(event);
            }
        });

        let mut subs = self.subscribers.write().unwrap();
        subs.entry(type_id).or_default().push(wrapped);
    }

    /// Publish an event to all subscribers of its type.
    pub fn publish<T: 'static>(&self, event: &T) {
        let type_id = TypeId::of::<T>();
        let subs = self.subscribers.read().unwrap();
        if let Some(callbacks) = subs.get(&type_id) {
            for callback in callbacks {
                callback(event);
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            subscribers: Arc::clone(&self.subscribers),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};

    #[test]
    fn test_subscribe_and_publish() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);

        bus.subscribe(move |_: &i32| {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        bus.publish(&42i32);
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicI32::new(0));

        for _ in 0..3 {
            let c = Arc::clone(&counter);
            bus.subscribe(move |_: &String| {
                c.fetch_add(1, Ordering::Relaxed);
            });
        }

        bus.publish(&String::from("test"));
        assert_eq!(counter.load(Ordering::Relaxed), 3);
    }
}
