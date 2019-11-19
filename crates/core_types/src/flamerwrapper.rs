#[cfg(not(target_arch = "wasm32"))]
extern crate flame;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
use std::env;
const FLAME_ENV : &str = "COMPILE_WITH_FLAME";
const FLAME_PATH : &str = "FLAME_GRAPH_PATH" ;
pub struct FlamerWrapper;


impl FlamerWrapper
{
    #[allow(unused_variables)]
    pub fn start(&self, guard_name:&'static str )
    {
        if let Ok(setting) = env::var(FLAME_ENV)
        {
            if setting =="YES"
            {
                debug!("adding guard {:?}",guard_name.clone());
                #[cfg(not(target_arch = "wasm32"))]
                flame::start(guard_name);
                #[cfg(target_arch = "wasm32")]
                warn!("cannot compile flamer for wasm, will not start guard");
            }
        }
    }

    #[allow(unused_variables)]
    pub fn end(&self,guard_name:&'static str)
    {
        if let Ok(setting) = env::var(FLAME_ENV)
        {
            if setting =="YES"
            {
                debug!("ending guard {:?}",guard_name.clone());
                #[cfg(not(target_arch = "wasm32"))]
                flame::end(guard_name);
                #[cfg(target_arch = "wasm32")]
                warn!("cannot compile flamer for wasm, will not start guard");
            }
        }
    }
    
    #[allow(unused_variables)]
    pub fn dump_html() 
    {
        if let Ok(setting) = env::var(FLAME_ENV)
        {
            if setting =="YES"
            {
                if let Ok(path) = env::var(FLAME_PATH)
                {
                    #[cfg(not(target_arch = "wasm32"))]
                    File::create(path).map(|file_for_flame|{
                        debug!("about to dump to flame graph");
                        flame::dump_html(file_for_flame).unwrap_or_else(|_|{
                            warn!("Flame graph enabled but cannot print to path")
                        })
                    }).unwrap_or_else(|_|{
                        warn!("Path provided for flame graph not valid will not be able to dump html")
                    });
                    #[cfg(target_arch = "wasm32")]
                    warn!("flame graphs not enabled for wasm")
                   
                    
                }
                else 
                {
                    warn!("target not set for flame grap")
                }
            }
        }
    }
  
}