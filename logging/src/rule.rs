//! Log filtering facility: add the capability to filter out by regex log messages and/or colorize them.

use regex::Regex;
use serde_derive::Deserialize;
use std::default::Default;

/// This structure is a helper for toml deserialization.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct Rule {
    pub pattern: String,
    pub exclude: Option<bool>,
    pub color: Option<String>,
}

/// This is our main way to filter out or colorize log messages.
#[derive(Clone, Debug)]
pub struct RuleFilter {
    pub pattern: Option<String>,
    exclude: bool,
    color: Option<String>,
    re: Regex,
}

impl RuleFilter {
    pub fn new(pattern: &str, exclude: bool, color: &str) -> Self {
        Self {
            pattern: Some(pattern.to_owned()),
            exclude,
            color: Some(color.to_owned()),
            re: Regex::new(&pattern).expect("Fail to init RuleFilter's regex."),
        }
    }
    /// Returns if we should exclude this log entry or not.
    pub fn exclude(&self) -> bool {
        self.exclude
    }

    /// Returns the color of the log entry.
    pub fn get_color(&self) -> String {
        match &self.color {
            Some(color) => color.clone(),
            None => String::default(),
        }
    }

    /// Returns true if we should log this sentence.
    pub fn should_log(&self, args: &str) -> bool {
        self.re.is_match(args) && !self.exclude
    }

    pub fn regex(&self) -> Regex {
        self.re.clone()
    }
    pub fn is_match(&self, args: &str) -> bool {
        self.re.is_match(args)
    }
}

impl Default for RuleFilter {
    fn default() -> Self {
        Self {
            pattern: Some(String::default()),
            exclude: false,
            color: None,
            re: Regex::new(&String::default()).expect("Fail to init RuleFilter's regex."),
        }
    }
}

impl From<Rule> for RuleFilter {
    fn from(rule: Rule) -> Self {
        let tf = RuleFilter::default();
        RuleFilter::new(
            &rule.pattern,
            rule.exclude.unwrap_or_else(|| tf.exclude()),
            &rule.color.unwrap_or_else(|| tf.get_color()),
        )
    }
}

impl From<RuleFilter> for Rule {
    fn from(rule_filter: RuleFilter) -> Self {
        Rule {
            pattern: rule_filter.pattern.unwrap_or_default(),
            exclude: Some(rule_filter.exclude),
            color: Some(rule_filter.color.unwrap_or_default()),
        }
    }
}

/// [RuleFilter] builder following the builder pattern.
pub struct RuleFilterBuilder {
    pattern: String,
    exclude: bool,
    color: Option<String>,
}

impl RuleFilterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_pattern(&mut self, pattern: &str) -> &mut Self {
        self.pattern = pattern.to_owned();
        self
    }

    pub fn set_exclusion(&mut self, exclude: bool) -> &mut Self {
        self.exclude = exclude;
        self
    }

    pub fn set_color(&mut self, color: &str) -> &mut Self {
        self.color = Some(color.to_owned());
        self
    }

    pub fn build(&self) -> RuleFilter {
        let pattern = match self.pattern.len() {
            0 => None,
            _ => Some(self.pattern.clone()),
        };

        RuleFilter {
            pattern,
            exclude: self.exclude,
            color: self.color.clone(),
            re: Regex::new(&self.pattern).expect("Fail to init RuleFilter's regex."),
        }
    }
}

impl Default for RuleFilterBuilder {
    fn default() -> Self {
        Self {
            pattern: String::default(),
            exclude: false,
            color: None,
        }
    }
}

#[test]
fn should_log_test() {
    let rule_filter = RuleFilterBuilder::new()
        .set_pattern("foo")
        .set_exclusion(false)
        .build();

    assert_eq!(rule_filter.should_log("bar"), false);
    assert_eq!(rule_filter.should_log("xfooy"), true);
}

#[test]
fn is_match_test() {
    let rule_filter = RuleFilterBuilder::new()
        .set_pattern("foo")
        .set_exclusion(false)
        .build();

    assert_eq!(rule_filter.is_match("bar"), false);
    assert_eq!(rule_filter.is_match("xfooy"), true);
}
