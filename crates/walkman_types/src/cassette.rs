use crate::event::WalkmanLogItem;
use regex::Regex;
use std::io::{BufRead, Read};

#[derive(Serialize, Deserialize)]
pub struct Cassette {
    logs: Vec<WalkmanLogItem>,
}

impl Cassette {
    pub fn logs(&self) -> &Vec<WalkmanLogItem> {
        self.logs.as_ref()
    }

    pub fn from_log_data<R: Read>(reader: std::io::BufReader<R>) -> Cassette {
        Cassette {
            logs: reader
                .lines()
                .map(|line| line.expect("IO error while parsing log for walkman"))
                .filter_map(parse_line)
                .collect(),
        }
    }
}

fn parse_line(line: String) -> Option<WalkmanLogItem> {
    let re_tag = Regex::new(r"<walkman>(.*?)</walkman>").unwrap();
    re_tag.captures(&line).and_then(|caps| {
        caps.get(1).and_then(|m| {
            serde_json::from_str(m.as_str()).expect("Couldn't parse serialized walkman log item")
        })
    })
}
