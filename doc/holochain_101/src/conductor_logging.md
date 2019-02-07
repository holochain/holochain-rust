# Logging

`logger` is a table for the configuration of how logging should behave in the Conductor. Select between types of loggers and setup rules for nicer display of the logs. There is only one logger per Conductor.

**Optional**

### Properties

#### `type`: `enum` Optional
Select which type of logger to use for the Conductor. If you leave this off, "simple" logging is the default
- `debug`: enables more sophisticated logging with color coding and filters
- `simple`: a most minimal logger, no color coding or filtering

#### `rules`: `LogRules` Optional
A table for optionally adding a set of rules to the logger

#### `LogRules.rules`: `LogRule`
An array of tables containing the rules for the logger

#### `LogRule.pattern`: `Regex string`
A Regex pattern as a string to match a log message against, to see whether this rule should apply to it.

#### `LogRule.exclude`: `bool` Optional
Whether to use this pattern to exclude things that match from the logs. Defaults to `false`. This option is useful for when the logs seem noisy.

#### `LogRule.color`: `enum` Optional
What color to use in the terminal output for logs that match this pattern. Options:
```
black, red, green, yellow, blue, magenta, cyan, white
```

### Example
```toml
[logger]
type = "debug"
[[logger.rules.rules]]
color = "red"
exclude = false
pattern = "^err/"

[[logger.rules.rules]]
color = "white"
exclude = false
pattern = "^debug/dna"

[[logger.rules.rules]]
exclude = true
pattern = "^debug/reduce"

[[logger.rules.rules]]
exclude = false
pattern = ".*"
```
