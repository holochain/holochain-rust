use im::hashmap::{HashMap,Iter};
use std::{time::{Duration,Instant},hash::Hash,ops::Add,thread,sync::Arc,fmt::Debug};
use holochain_locksmith::RwLock;

pub enum FuturesPanicConfiguration
{
    Panic(Duration),
    NotPanic
}
pub struct FuturesDiagnosticTrace<S:Into<String> + Clone +Eq + Hash + Debug + Send + Sync>
{
    pub futures_queue : HashMap<S,Diagnostic>,
    pub panic_configuration : FuturesPanicConfiguration,
    current_instant_time : Option<Instant>
}

impl <S : Into<String> + Clone +Eq + Hash+ Debug + Send + Sync + 'static> FuturesDiagnosticTrace<S>
{
    pub fn new() -> Self
    {
        FuturesDiagnosticTrace
        {
            futures_queue : HashMap::new(),
            panic_configuration : FuturesPanicConfiguration::NotPanic,
            current_instant_time : None
        }
    }
    pub fn record_diagnostic(&mut self,futures_name:S)
    {
        let new_diagnostic = Diagnostic{
            poll_count : 1,
            total_polling_time: self.current_instant_time.expect("Make sure too call capture() method before you record diagnosticf").elapsed()
        };
        let diagnostic = self.futures_queue.get(&futures_name).map(|diagnostic_found| diagnostic_found.to_owned() +  new_diagnostic.clone()).unwrap_or(new_diagnostic);
        self.futures_queue.insert(futures_name,diagnostic);
    }

    pub fn diagnostic_iter<'a>(&'a self) -> Iter<'a,S,Diagnostic>
    {
        self.futures_queue.iter()
    }

    pub fn capture(&mut self)
    {
        self.current_instant_time = Some(Instant::now());
    }

    pub fn run(diagnostics : Arc<RwLock<FuturesDiagnosticTrace<S>>>)
    {
        thread::spawn(move || {
            loop
            {
                match diagnostics.read().unwrap().panic_configuration
                {
                    FuturesPanicConfiguration::NotPanic =>
                    {
                        diagnostics.read().unwrap().diagnostic_iter().for_each(|(futures_key,value)|
                        {
                            println!("FUTURE AT {:?} has polled {:?} and has been running for {:?} total",futures_key,value.poll_count,value.total_polling_time);
                        });
                    }
                    _=>unimplemented!("This has not been implemented yet")
                }
                thread::sleep(Duration::from_millis(1000))
            }
            
        });
    }

}




#[derive(Clone)]
pub struct Diagnostic
{
    poll_count : u64,
    total_polling_time : Duration
}

impl Add for Diagnostic
{
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            poll_count: self.poll_count + other.poll_count,
            total_polling_time: self.total_polling_time + other.total_polling_time,
        }
    }
}