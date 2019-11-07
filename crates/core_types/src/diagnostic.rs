use im::hashmap::{HashMap,Iter};
use std::{time::{Duration,Instant},hash::Hash,thread,sync::Arc,fmt::Debug};
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
    pub fn end_capture(&mut self,futures_name:S)
    {
        let old_diagnostic = self.futures_queue.get(&futures_name).unwrap_or_else(||  panic!("future {:?} not found should have called start_capture",futures_name.clone()));
        self.futures_queue = self.futures_queue.update(futures_name,Diagnostic
        {
            poll_count : old_diagnostic.poll_count + 1,
            total_polling_time : Some(old_diagnostic.current_running_time.expect("Make sure too call capture() method before you record diagnostic").elapsed()),
            current_running_time : None
        });
    }

    pub fn diagnostic_iter<'a>(&'a self) -> Iter<'a,S,Diagnostic>
    {
        
        self.futures_queue.iter()
    }

    pub fn start_capture(&mut self,futures_name:S)
    {
         let new_diagnostic = Diagnostic{
            poll_count : 1,
            total_polling_time: None,
            current_running_time : Some(Instant::now())
        };
        self.futures_queue = self.futures_queue.update(futures_name,new_diagnostic);
    }

    pub fn update_all_running_time(&mut self)
    {
        let updated_diagnostics = self
        .futures_queue
        .iter()
        .filter(|(_,f)|f.total_polling_time.is_none())
        .map(|(s,old_diagnostic)|(s.clone(),Diagnostic{
            poll_count : old_diagnostic.poll_count,
            total_polling_time : Some(old_diagnostic.current_running_time.expect("current running time Should have previously been set for future").elapsed()),
            current_running_time : None

        })).collect::<HashMap::<S,Diagnostic>>();
        self.futures_queue = self.futures_queue.clone().union(updated_diagnostics);
    }


    pub fn run(diagnostics : Arc<RwLock<FuturesDiagnosticTrace<S>>>)
    {
        thread::spawn(move || {
            loop
            {
                diagnostics.write().unwrap().update_all_running_time();
                match diagnostics.read().unwrap().panic_configuration
                {
                    FuturesPanicConfiguration::Panic(duration) =>
                    {
                       diagnostics
                       .read()
                       .unwrap()
                       .diagnostic_iter()
                       .map(|(f,s)|{
                           debug!("Future {:?} last polled at {:?} for the {:?} time",f,s.total_polling_time,s.poll_count);
                           (f,s)
                       })
                       .filter(|(_,s)|{
                           s.total_polling_time.map(|total| total > Duration::from_secs(60)).unwrap_or(false)
                       })
                       .map(|(f,s)|{
                           warn!("Future {:?} has been polling for over 1 minute at {:?}",f,s.total_polling_time.unwrap());
                           (f,s)
                       })
                       .filter(|(_,s)|{
                           s.total_polling_time.map(|total| total > duration).unwrap_or(false)
                       })
                       .map(|(f,s)|{
                           error!("Future : {:?} has been polling for over 1 minute at {:?}",f,s.total_polling_time.unwrap());
                           panic!("ERROR : PANIC INITIATED FOR FUTURE")
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
    poll_count : u64,
    total_polling_time : Option<Duration>,
    current_running_time: Option<Instant>
}

