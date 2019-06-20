//! Tagging capability: add capability to filter by regex and attribute some color to them.

use regex::Regex;
use std::default::Default;


#[derive(Clone, Debug)]
pub struct TagFilter {
    pub pattern: Option<String>,
    exclude: bool,
    color: Option<String>,
    re: Regex,
}

impl TagFilter {
    pub fn new(pattern: &str, exclude: bool, color: &str) -> Self {
        Self {
            pattern: Some(pattern.to_owned()),
            exclude,
            color: Some(color.to_owned()),
            re: Regex::new(&pattern).expect("Fail to init TagFilter's regex."),
        }
    }
    /// Returns if we should excluse this matter or not.
    pub fn exclude(&self) -> bool {
        self.exclude
    }

    /// Returns the color of the tag.
    pub fn tag_color(&self) -> String {
        match &self.color {
            Some(color) => color.clone(),
            None => String::default()
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

impl Default for TagFilter {
    fn default() -> Self {
        Self {
            pattern: Some(String::default()),
            exclude: false,
            color: Some(String::from("white")),
            re: Regex::new(&String::default()).expect("Fail to init TagFilter's regex."),
        }
    }
}

pub struct TagFilterBuilder {
    pattern: String,
    exclude: bool,
    color: String,
}

impl TagFilterBuilder {
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
        self.color = color.to_owned();
        self
    }

    pub fn build(&self) -> TagFilter {
        let pattern = match self.pattern.len() {
            0 => None,
            _ => Some(self.pattern.clone()),
        };

        TagFilter {
            pattern,
            exclude: self.exclude,
            color: Some(self.color.to_owned()),
            re: Regex::new(&self.pattern).expect("Fail to init TagFilter's regex."),
        }
    }
}

impl Default for TagFilterBuilder {
    fn default() -> Self {
        Self {
            pattern: String::default(),
            exclude: false,
            color: String::from("white"),
        }
    }
}

#[test]
fn should_log_test() {
    let tag_filter = TagFilterBuilder::new()
        .set_pattern("foo")
        .set_exclusion(false)
        .build();

    assert_eq!(tag_filter.should_log("bar"), false);
    assert_eq!(tag_filter.should_log("xfooy"), true);
}

#[test]
fn is_match_test() {
    let tag_filter = TagFilterBuilder::new()
        .set_pattern("foo")
        .set_exclusion(false)
        .build();

    assert_eq!(tag_filter.is_match("bar"), false);
    assert_eq!(tag_filter.is_match("xfooy"), true);

}
