use lazy_static::lazy_static;
use wasmer_runtime::Module;
use wasmer_runtime::cache::WasmHash;
use wasmer_runtime::error::CacheError;
use wasmer_runtime::Backend;
use wasmer_runtime::cache::Cache;
use std::collections::HashMap;

use std::sync::Mutex;

lazy_static! {
    static ref HASHCACHE: Mutex<HashMap<String, HashMap<WasmHash, Module>>> = {
        Mutex::new(HashMap::new())
    };
}

pub struct MemoryCache{}

impl MemoryCache {
    pub fn new() -> MemoryCache {
        MemoryCache{}
    }
}

impl Cache for MemoryCache {
    type LoadError = CacheError;
    type StoreError = CacheError;

    fn load(&self, key: WasmHash) -> Result<Module, CacheError> {
        self.load_with_backend(key, Backend::default())

    }

    fn load_with_backend(&self, key: WasmHash, backend: Backend) -> Result<Module, CacheError> {
        let cache = HASHCACHE.lock().unwrap();
        let backend_key = backend.to_string();
        match cache.get(backend_key) {
            Some(module_cache) => {
                match module_cache.get(&key) {
                    Some(module) => Ok(module.to_owned()),
                    None => Err(CacheError::InvalidatedCache),
                }
            },
            None => Err(CacheError::InvalidatedCache),
        }
    }

    fn store(&mut self, key: WasmHash, module: Module) -> Result<(), CacheError> {
        let mut cache = HASHCACHE.lock().unwrap();
        let backend_key = module.info().backend.to_string();
        let backend_map = cache.entry(backend_key).or_insert(HashMap::new());
        backend_map.entry(key).or_insert(module);
        Ok(())
    }
}
