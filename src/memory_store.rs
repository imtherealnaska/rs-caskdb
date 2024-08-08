use std::collections::HashMap;

use crate::store::Store;

pub struct MemoryStore {
    data: HashMap<String, String>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Store for MemoryStore {
    fn get(&self, key: &String) -> Option<&String> {
        self.data.get(key)
    }

    fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    fn close(&self) -> bool {
        return true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store_get() {
        let mut store = MemoryStore::new();
        store.set("name".to_string(), "jojo".to_string());
        assert_eq!(
            store.get(&"name".to_string()),
            Some(&"jojo".to_string()),
            "Get() returned unexpected value"
        );
    }

    #[test]
    fn test_memory_store_invalid_get() {
        let store = MemoryStore::new();
        assert_eq!(
            store.get(&"some rando key".to_string()),
            None,
            "Get returned unexpected value"
        )
    }

    #[test]
    fn test_memory_store_close() {
        let store = MemoryStore::new();
        assert_eq!(store.close(), true, "close() failed");
    }
}
