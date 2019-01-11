use chrono::{DateTime, Local};
use colored::*;
use holochain_core::logger::{ChannelLogger, Sender};
use holochain_core_types::error::HolochainError;
use regex::Regex;
use std::thread;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LogRule {
    #[serde(with = "serde_regex")]
    pub pattern: Regex,
    #[serde(default)]
    pub exclude: bool,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LogRules {
    pub rules: Vec<LogRule>,
}

impl Default for LogRules {
    fn default() -> LogRules {
        let mut rules = LogRules::new();
        rules
            .add_rule("^err/", false, Some("red".to_string()))
            .expect("rule is valid");
        rules
            .add_rule("^debug/dna", false, Some("white".to_string()))
            .expect("rule is valid");
        rules.add_rule(".*", false, None).expect("rule is valid");
        rules
    }
}

impl LogRules {
    pub fn new() -> Self {
        LogRules { rules: Vec::new() }
    }

    // add a new rule to the rules list
    pub fn add_rule(
        &mut self,
        pattern: &str,
        exclude: bool,
        color: Option<String>,
    ) -> Result<(), HolochainError> {
        let regex = Regex::new(pattern).map_err(|e| HolochainError::new(&e.to_string()))?;
        self.rules.push(LogRule {
            pattern: regex,
            exclude,
            color,
        });
        Ok(())
    }

    // run the rules on a message, returning None if the message is rejected, or Some(LogMessage)
    pub fn run(&self, id: String, msg: String) -> Option<LogMessage> {
        let mut message = LogMessage {
            date: Local::now(),
            id: id,
            msg: msg.clone(),
            color: None,
        };
        if self.rules.len() == 0 {
            None
        } else {
            for r in &self.rules {
                if r.pattern.is_match(&msg) {
                    if r.exclude {
                        return None;
                    }
                    message.color = r.color.clone();
                    return Some(message);
                }
            }
            None
        }
    }
}

// The DebugLogger implements a receiver for the instance ChannelLogger
// which allows for configurable colorization and filtering of log messages.
pub struct DebugLogger {
    sender: Sender,
}

impl DebugLogger {
    pub fn new(rules: LogRules) -> Self {
        let (tx, rx) = ChannelLogger::setup();
        let logger = DebugLogger { sender: tx.clone() };

        thread::spawn(move || loop {
            match rx.recv() {
                Ok((id, msg)) => run(&rules, id, msg),
                Err(_) => break,
            }
        });
        logger
    }
    pub fn get_sender(&self) -> Sender {
        self.sender.clone()
    }
}

// run checks a message against the rules and renders it if it matches
pub fn run(rules: &LogRules, id: String, msg: String) {
    match rules.run(id, msg) {
        Some(message) => render(message),
        None => (),
    }
}

static ID_COLORS: &'static [&str] = &["green", "yellow", "blue", "magenta", "cyan"];

// TODO this is actually silly and we should allocate colors to IDs so they aren't likely to collide
fn pick_color(text: &str) -> &str {
    let mut total : u16 = 0;
    for b in text.to_string().into_bytes() {
        total += b as u16;
    }
    ID_COLORS[(total as usize) % ID_COLORS.len()]
}

// renders a log message, using the id color if no color specified for the message.
pub fn render(msg: LogMessage) {
    let id_color = pick_color(&msg.id);
    let msg_color = match msg.color {
        None => id_color.to_string(),
        Some(color) => color,
    };
    let x = format!(
        "{}:{}: {}",
        msg.date.format("%Y-%m-%d %H:%M:%S"),
        msg.id.color(id_color),
        msg.msg.color(msg_color)
    );
    println!("{}", x);
}

#[derive(Debug, PartialEq)]
pub struct LogMessage {
    date: DateTime<Local>,
    id: String,
    msg: String,
    color: Option<String>,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use toml;

    #[test]
    fn test_log_rules() {
        let mut rules = LogRules::new();
        rules.add_rule("foo", false, None).unwrap();
        let id = "instance".to_string();
        assert_eq!(rules.run(id.clone(), "bar".to_string()), None);
        let m = rules.run(id.clone(), "xfooy".to_string()).unwrap();
        assert_eq!(m.msg, "xfooy");
        rules.add_rule("baz", true, None).unwrap(); // rule to reject anything with baz
        rules.add_rule("b", false, None).unwrap(); // rule to accept anything with b
        assert_eq!(rules.run(id.clone(), "baz".to_string()), None);
        let m = rules.run(id.clone(), "xboy".to_string()).unwrap();
        assert_eq!(m.msg, "xboy");
    }

    #[test]
    fn test_log_rules_default() {
        let rules = LogRules::default();
        assert_eq!(rules.rules.len(), 3);
        assert_eq!(format!("{:?}",rules),"LogRules { rules: [LogRule { pattern: ^err/, exclude: false, color: Some(\"red\") }, LogRule { pattern: ^debug/dna, exclude: false, color: Some(\"white\") }, LogRule { pattern: .*, exclude: false, color: None }] }".to_string());
    }

    #[test]
    fn test_bad_log_rules() {
        let mut rules = LogRules::new();
        assert_eq!(
            rules.add_rule("foo[", false, None),
            Err(HolochainError::new(
                "regex parse error:\n    foo[\n       ^\nerror: unclosed character class"
            ))
        );
    }

    #[test]
    fn test_rules_serialization() {
        let toml = r#"[[rules]]
pattern = "foo"
exclude = false

[[rules]]
pattern = "bar"
exclude = true
color = "blue"
"#;
        let mut rules = LogRules::new();
        rules.add_rule("foo", false, None).unwrap();
        rules
            .add_rule("bar", true, Some("blue".to_string()))
            .unwrap();
        let toml1 = toml::to_string(&rules).unwrap();
        assert_eq!(toml1, toml);

        let rules1 = toml::from_str::<LogRules>(toml).unwrap();
        assert!(rules1.rules[0].pattern.is_match("foo"));
        assert_eq!(rules1.rules[0].color, None);
        assert!(rules1.rules[1].pattern.is_match("bar"));
        assert_eq!(rules1.rules[1].color, Some("blue".to_string()));
    }
}
