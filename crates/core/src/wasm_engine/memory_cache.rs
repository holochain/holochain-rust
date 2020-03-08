use lazy_static::lazy_static;
use wasmer_runtime::Module;
use wasmer_runtime::cache::WasmHash;
use wasmer_runtime::error::CacheError;
use wasmer_runtime::Backend;
use wasmer_runtime::cache::Cache;
use std::path::PathBuf;
use wasmer_runtime::cache::FileSystemCache;
use std::collections::HashMap;
use std::io;

use std::sync::Mutex;

lazy_static! {
    static ref HASHCACHE: Mutex<HashMap<String, HashMap<WasmHash, Module>>> = {
        Mutex::new(HashMap::new())
    };
}

pub struct MemoryFallbackFileSystemCache{
    fs_fallback: FileSystemCache,
}

impl MemoryFallbackFileSystemCache {
    pub fn new<P: Into<PathBuf>>(fallback_path: P) -> io::Result<MemoryFallbackFileSystemCache> {
        Ok(MemoryFallbackFileSystemCache {
            fs_fallback: unsafe { FileSystemCache::new(fallback_path) }?
        })
    }

    fn store_mem(&self, key: WasmHash, module: Module) -> Result<(), CacheError> {
        let mut cache = HASHCACHE.lock().unwrap();
        let backend_key = module.info().backend.to_string();
        let backend_map = cache.entry(backend_key).or_insert(HashMap::new());
        backend_map.entry(key).or_insert(module.clone());
        Ok(())
    }

    fn store_fs(&mut self, key: WasmHash, module: Module) -> Result<(), CacheError> {
        self.fs_fallback.store(key, module)?;
        Ok(())
    }
}

impl Cache for MemoryFallbackFileSystemCache {
    type LoadError = CacheError;
    type StoreError = CacheError;

    fn load(&self, key: WasmHash) -> Result<Module, CacheError> {
        self.load_with_backend(key, Backend::default())
    }

    fn load_with_backend(&self, key: WasmHash, backend: Backend) -> Result<Module, CacheError> {
        // local scope to keep mutex happy
        {
            let cache = HASHCACHE.lock().unwrap();
            let backend_key = backend.to_string();
            match cache.get(backend_key) {
                Some(module_cache) => {
                    match module_cache.get(&key) {
                        // short circuit with what we found in memory :D
                        Some(module) => return Ok(module.to_owned()),
                        _ => (),
                    }
                },
                _ => (),
            };
        }
        // we did not find anything in memory so fallback to fs
        match self.fs_fallback.load_with_backend(key, backend) {
            Ok(module) => {
                // update the memory cache so we load faster next time
                self.store_mem(key, module.clone())?;
                Ok(module)
            },
            Err(e) => Err(e),
        }
    }

    fn store(&mut self, key: WasmHash, module: Module) -> Result<(), CacheError> {
        // store in depth first order
        self.store_fs(key, module.clone())?;
        self.store_mem(key, module)?;

        Ok(())
    }
}
