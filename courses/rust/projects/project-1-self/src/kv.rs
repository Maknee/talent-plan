use std::collections::HashMap;
/// The `KvStore` stores string key/value pairs.
/// Example:
///
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key".to_owned(), "value".to_owned());
/// let val = store.get("key".to_owned());
/// assert_eq!(val, Some("value".to_owned()));
/// ```
pub struct KvStore {
    kv: HashMap<String, String>,
}

impl KvStore {
    /// Create
    pub fn new() -> Self {
        Self {
            kv: Default::default(),
        }
    }

    // /// Set
    // pub fn set(&mut self, k: &str, v: &str)  {
    //     self.kv.insert(k.to_string(), v.to_string());
    // }

    /// Set
    pub fn set(&mut self, k: String, v: String)  {
        self.kv.insert(k, v);
    }
    /// Set
    // pub fn get(&self, k: &str) -> Option<String>  {
    //     self.kv.get(k).cloned()
    // }
    
    /// Set
    pub fn get(&self, k: String) -> Option<String>  {
        self.kv.get(&k).cloned()
}
    
    /// Set
    // pub fn remove(&mut self, k: &str)  {
    //     self.kv.remove(k);
    // }

    /// Set
    pub fn remove(&mut self, k: String)  {
        self.kv.remove(&k);
    }
}