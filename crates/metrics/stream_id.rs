/// A stream id represents a unique entity which provided a metric.
use chrono::prelude::*;
use std::time::SystemTime;
#[derive(Shrinkwrap, Clone, Debug, Hash, Eq, PartialEq)]
pub struct StreamId(String);

impl StreamId {
    
    pub fn new<S:Into<String>>(s:S) -> Self {
        StreamId(s.into()) 
    }
}

impl TryInto<(SystemTime, String)> for StreamId {

   type Error = ParseResult<DateTime<FixedOffset>>; 

   fn try_into(&self) -> Result<(SystemTime, String), Self::Error> {

       let date_str = self.0;
       let date = DateTime::parse_from_str(date_str, "%Y-%m-%d_%H:%M:%S")?;

       Ok((date.to_time(), date_str))
   }
}

const LOG_STREAM_SEPARATOR: &str = ".";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScenarioData {
    run_name: String,
    net_name: String,
    dna_name: String,
    scenario_name: String,
    player_name: String,
}

impl Into<String> for ScenarioData {
    fn into(self: Self) -> String {
        let s = self;
        format!(
            "{}.{}.{}.{}.{}",
            s.run_name, s.net_name, s.dna_name, s.scenario_name, s.player_name
        )
    }
}

impl TryFrom<String> for ScenarioData {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let split = s.split(LOG_STREAM_SEPARATOR).collect::<Vec<_>>();
        if split.len() < 4 {
            return Err(format!(
                "Log stream name doesn't have at least 4 path elements: {:?}",
                split
            ));
        }
        Ok(Self {
            run_name: split[0].into(),
            net_name: split[1].into(),
            dna_name: split[2].into(),
            scenario_name: split[3].into(),
            player_name: split[4].into(),
        })
    }
}

impl TryFrom<LogStream> for ScenarioData {
    type Error = String;
    fn try_from(log_stream: LogStream) -> Result<Self, Self::Error> {
        let result: Result<Self, Self::Error> = log_stream
            .log_stream_name
            .map(|x| Ok(x))
            .unwrap_or_else(|| Err("Log stream name missing".into()))
            .and_then(TryFrom::try_from);
        result
    }
}

impl ScenarioData {
    /// Groups by everything _but_ the player name
    fn without_player_name(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.run_name, self.net_name, self.dna_name, self.scenario_name
        )
    }


    /// Groups a log stream name by its scenario name, which is by convention is the 2nd to last field.
    /// Eg. "2019-12-06_01-54-47_stress_10_1_2.sim2h.smoke.9"
    pub fn group_by_scenario(
        log_stream_names: &mut dyn Iterator<Item = String>,
    ) -> HashMap<String, HashSet<ScenarioData>> {
        log_stream_names.fold(HashMap::new(), |mut grouped, log_stream_name| {
            let scenario_data: Result<ScenarioData, _> = log_stream_name.try_into();
            if let Ok(scenario_data) = scenario_data {
                grouped
                    .entry(scenario_data.without_player_name())
                    .or_insert_with(HashSet::new)
                    .insert(scenario_data);
            }
            grouped
        })
}


}

