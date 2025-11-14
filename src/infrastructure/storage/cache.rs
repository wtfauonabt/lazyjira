use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Simple in-memory cache with TTL
#[allow(dead_code)] // Will be used for caching API responses
pub struct Cache<K, V> {
    data: HashMap<K, CacheEntry<V>>,
    #[allow(dead_code)] // Field used internally
    default_ttl: Duration,
}

struct CacheEntry<V> {
    value: V,
    #[allow(dead_code)] // Field used internally
    expires_at: Instant,
}

impl<K, V> Cache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
{
    #[allow(dead_code)] // Will be used for caching API responses
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            data: HashMap::new(),
            default_ttl,
        }
    }

    #[allow(dead_code)] // Will be used for caching API responses
    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key).and_then(|entry| {
            if entry.expires_at > Instant::now() {
                Some(&entry.value)
            } else {
                None
            }
        })
    }

    #[allow(dead_code)] // Will be used for caching API responses
    pub fn insert(&mut self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl);
    }

    #[allow(dead_code)] // Will be used for caching API responses
    pub fn insert_with_ttl(&mut self, key: K, value: V, ttl: Duration) {
        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + ttl,
        };
        self.data.insert(key, entry);
    }

    #[allow(dead_code)] // Will be used when cache management is implemented
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.data.remove(key).map(|entry| entry.value)
    }

    #[allow(dead_code)] // Will be used when cache management is implemented
    pub fn clear(&mut self) {
        self.data.clear();
    }

    #[allow(dead_code)] // Will be used for caching API responses
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.data.retain(|_, entry| entry.expires_at > now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = Cache::new(Duration::from_secs(60));
        cache.insert("key1", "value1");
        
        assert_eq!(cache.get(&"key1"), Some(&"value1"));
        assert_eq!(cache.get(&"key2"), None);
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = Cache::new(Duration::from_millis(100));
        cache.insert("key1", "value1");
        
        assert_eq!(cache.get(&"key1"), Some(&"value1"));
        
        std::thread::sleep(Duration::from_millis(150));
        
        assert_eq!(cache.get(&"key1"), None);
    }

    #[test]
    fn test_cache_cleanup() {
        let mut cache = Cache::new(Duration::from_millis(50));
        cache.insert("key1", "value1");
        cache.insert_with_ttl("key2", "value2", Duration::from_millis(200));
        
        std::thread::sleep(Duration::from_millis(100));
        cache.cleanup_expired();
        
        assert_eq!(cache.get(&"key1"), None);
        assert_eq!(cache.get(&"key2"), Some(&"value2"));
    }
}
