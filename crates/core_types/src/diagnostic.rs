use im::hashmap::{HashMap,Iter};
use std::{time::{Duration},hash::Hash,thread,sync::Arc,fmt::Debug};
use holochain_locksmith::RwLock;



pub enum FuturesPanicConfiguration
{
    Panic(Duration),
    NotPanic
}

pub struct FuturesDiagnosticTrace<S:Into<String> + Clone +Eq + Hash + Debug + Send + Sync>
 
{
    pub futures_queue : HashMap<S,Diagnostic>,
    pub panic_configuration : FuturesPanicConfiguration
}

impl <S : Into<String> + Clone +Eq + Hash+ Debug + Send + Sync + 'static> FuturesDiagnosticTrace<S>
{
    pub fn new() -> Self
    {
        FuturesDiagnosticTrace
        {
            futures_queue : HashMap::new(),
            panic_configuration : FuturesPanicConfiguration::Panic(Duration::from_secs(70)),
        }
    }
    pub fn capture(&mut self,futures_name:S,polling_time:Duration)
    {
        self.futures_queue = self.futures_queue.update(futures_name.clone(),Diagnostic
        {
            total_polling_time : polling_time
        });
    }

    pub fn diagnostic_iter<'a>(&'a self) -> Iter<'a,S,Diagnostic>
    {
        
        self.futures_queue.iter()
    }

    pub fn run(diagnostics : Arc<RwLock<FuturesDiagnosticTrace<S>>>)
    {
        thread::spawn(move || {
            loop
            {
                match diagnostics.read().unwrap().panic_configuration
                {
                    FuturesPanicConfiguration::Panic(duration) =>
                    {
                       diagnostics
                       .read()
                       .unwrap()
                       .diagnostic_iter()
                       .map(|(f,s)|{
                           debug!("Future {:?} last polled at {:?}",f,s.total_polling_time);
                           (f,s)
                       })
                       .filter(|(_,s)|{
                           s.clone().total_polling_time> Duration::from_secs(60)
                       })
                       .map(|(f,s)|{
                           warn!("Future {:?} has been polling for over 1 minute at {:?}",f,s.total_polling_time);
                           (f,s)
                       })
                       .filter(|(_,s)|{
                           s.clone().total_polling_time > duration
                       })
                       .map(|(f,s)|{
                           error!("Future : {:?} has been polling for over {:?} seconds minute at {:?}",f,duration,s.total_polling_time);
                       })
                       .for_each(drop)
                    }
                    _=>
                    {
                        
                    }
                };
                thread::sleep(Duration::from_secs(70))
            }
            
        });
    }

}




#[derive(Clone)]
pub struct Diagnostic
{
    total_polling_time : Duration,
}

