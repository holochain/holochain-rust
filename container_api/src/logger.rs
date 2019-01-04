use holochain_core::{logger::{ChannelLogger, Sender}};
use std::thread;
use chrono::{Local, DateTime};
use colored::*;
use regex::Regex;

#[derive(Deserialize, Serialize, Clone)]
pub struct LogRule {
    #[serde(with = "serde_regex")]
    pub pattern: Regex,
    #[serde(default)]
    pub exclude: bool,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct LogRules {
    rules:  Vec<LogRule>
}

impl LogRules {
    pub fn new()->Self {
        LogRules{rules: Vec::new()}
    }
    pub fn add_rule(&mut self,pattern: &str,exclude: bool) {
        let regex = Regex::new(pattern).unwrap();
        self.rules.push(LogRule{pattern:regex,exclude});
    }
    pub fn run(&self, id: String, msg: String) -> Option<LogMessage> {
        let mut message = LogMessage{
            date: Local::now(),
            id: "".to_string(),
            msg: msg.clone(),
        };
        if self.rules.len() == 0 {
            message.id = id.green().to_string();
            Some(message)
        } else {
            for r in &self.rules {
                if r.pattern.is_match(&msg) {
                    if r.exclude {
                        return None;
                    }
                    message.id = id.red().to_string();
                    return Some(message);
                }
            }
            None
        }
    }
}
pub struct DebugLogger {
    sender: Sender,
}

impl DebugLogger {
    pub fn new(rules: LogRules) -> Self {
        let(tx,rx) = ChannelLogger::setup();
        let logger = DebugLogger{
            sender: tx.clone(),
        };

        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok((id,msg)) => {
                        debug(&rules,id,msg)
                    },
                    Err(_) => break,
                }
            }
        });
        logger
    }
    pub fn get_sender(&self) -> Sender {
        self.sender.clone()
    }
}

pub fn debug(rules: &LogRules, id: String,msg: String) {
    match rules.run(id, msg) {
        Some(message) => render(message),
        None=>(),
    }
}

pub fn render(msg: LogMessage) {
    let x = format!("{}:{}:{}", msg.date.format("%Y-%m-%d %H:%M:%S"),msg.id,msg.msg);
    println!("{}",x);
}

#[derive(Debug,PartialEq)]
pub struct LogMessage {
    date: DateTime<Local>,
    id: String,
    msg: String,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use toml;

    #[test]
    fn test_log_rules() {
        let mut rules = LogRules::new();
        rules.add_rule("foo",false);
        let id = "instance".to_string();
        assert_eq!(rules.run(id.clone(),"bar".to_string()),None);
        let m = rules.run(id.clone(),"xfooy".to_string()).unwrap();
        assert_eq!(m.msg,"xfooy");
        rules.add_rule("baz",true); // rule to reject anything with baz
        rules.add_rule("b",false);  // rule to accept anything with b
        assert_eq!(rules.run(id.clone(),"baz".to_string()),None);
        let m = rules.run(id.clone(),"xboy".to_string()).unwrap();
        assert_eq!(m.msg,"xboy");
    }

    #[test]
    fn test_rules_serialization() {
        let toml = r#"[[rules]]
pattern = "foo"
exclude = false

[[rules]]
pattern = "bar"
exclude = true
"#;
        let mut rules = LogRules::new();
        rules.add_rule("foo",false);
        rules.add_rule("bar",true);
        let toml1 = toml::to_string(&rules).unwrap();
        assert_eq!(toml1,toml);

        let rules1 = toml::from_str::<LogRules>(toml).unwrap();
        assert!(rules1.rules[0].pattern.is_match("foo"));
        assert!(rules1.rules[1].pattern.is_match("bar"));
    }
}
