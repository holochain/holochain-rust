extern crate flame;
use std::{fs::File,env};
const FLAME_ENV : &str = "COMPILE_WITH_FLAME";
const FLAME_PATH : &str = "FLAME_GRAPH_PATH" ;
pub struct FlamerWrapper;

impl FlamerWrapper
{
    pub fn start(guard_name:&'static str )
    {
        if let Ok(setting) = env::var(FLAME_ENV)
        {
            if setting =="YES"
            {
                flame::start(guard_name);
            }
        }
    }

    pub fn end(guard_name:&'static str)
    {
        if let Ok(setting) = env::var(FLAME_ENV)
        {
            if setting =="YES"
            {
                flame::end(guard_name);
            }
        }
    }

    pub fn dump_html() 
    {
        if let Ok(setting) = env::var(FLAME_ENV)
        {
            if setting =="YES"
            {
                if let Ok(path) = env::var(FLAME_PATH)
                {
                    File::create(path).map(|file_for_flame|{
                        flame::dump_html(file_for_flame).unwrap_or_else(|_|{
                            warn!("Flame graph enabled but cannot print to path")
                        })
                    }).unwrap_or_else(|_|{
                        warn!("Path provided for flame graph not valid will not be able to dump html")
                    })
                    
                }
            }
        }
    }
}