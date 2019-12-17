use crate::{
    stats::{GroupingKey, OnlineStats, StatsByMetric},
    Metric,
};
use regex::Regex;
use std::collections::{HashMap, HashSet};
/// A stream id represents a unique entity which provided a metric.
//use chrono::prelude::*;
use std::convert::{TryFrom, TryInto};

#[derive(Shrinkwrap, Clone, Debug, Hash, Eq, PartialEq)]
pub struct StreamId(pub String);

impl StreamId {
    pub fn new<S: Into<String>>(s: S) -> Self {
        StreamId(s.into())
    }

    pub fn group_by_regex<I: IntoIterator<Item = Metric>>(
        re: &Regex,
        metrics: I,
    ) -> StatsByMetric<OnlineStats> {
        StatsByMetric(metrics.into_iter().fold(HashMap::new(), |mut map, metric| {
            let metric_name = metric.name.clone();
            let stream_id = metric.stream_id.clone();
            stream_id
                .and_then(|stream_id| {
                    re.captures_iter(stream_id.as_str()).next().map(|captured| {
                        let key = GroupingKey::new(captured[1].to_string(), metric_name);
                        let entry = map.entry(key);
                        let stats: &mut OnlineStats = entry.or_insert_with(OnlineStats::empty);
                        stats.add(metric.value)
                    })
                })
                .unwrap_or_else(|| {});
            map
        }))
    }
}
/*
impl TryInto<(DateTime<FixedOffset>, String)> for StreamId {

   type Error = chrono::ParseError;

   fn try_into(&self) -> Result<(DateTime<FixedOffset>, String), Self::Error> {

       let date_str : String = self.0.clone();
       let date = DateTime::parse_from_str(date_str.as_str(), "%Y-%m-%d_%H:%M:%S")?;

       Ok((date, date_str))
   }
}
*/

const LOG_STREAM_SEPARATOR: &str = ".";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScenarioData {
    run_name: String,
    net_name: String,
    dna_name: String,
    scenario_name: String,
    player_name: String,
    instance_id: String,
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
            instance_id: split[5].into(),
        })
    }
}

impl TryFrom<rusoto_logs::LogStream> for ScenarioData {
    type Error = String;
    fn try_from(log_stream: rusoto_logs::LogStream) -> Result<Self, Self::Error> {
        let result: Result<Self, Self::Error> = log_stream
            .log_stream_name
            .map(|x| Ok(x))
            .unwrap_or_else(|| Err("Log stream name missing".into()))
            .and_then(TryFrom::try_from);
        result
    }
}

// Eg. "2019-12-06_01-54-47_stress_10_1_2.sim2h.smoke.9"
// Default pattern agggregate by the entire stream id:
// semantically: run_name.net_type.dna.scenario.conductor_id.instance_id
// Define !p to indicate substitution of regex p into an expression
// regex rule: p = [\\w\\d\\-_]+
// regex: (!p\\.!p\\.!p\\.!p\\.!p\\.!p)
// By conductor (over all instances)
// run_name.net_type.dna.sceanrio.conductor_id.*
// regex: (!p\\.!p\\.!p\\.!p\\.!p)\\.!p
// By scenario (over all conductors and instances)
// run_name.net_type.dna.scenario.*
// regex: (!p\\.!p\\.!p\\.!p)\\.!p\\.!p

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
