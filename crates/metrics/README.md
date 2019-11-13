# holochain_metrics


[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.net)

[![Twitter Follow](https://img.shields.io/twitter/follow/holochain.svg?style=social&label=Follow)](https://twitter.com/holochain)

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)

Provides instrumentation to measure, publish, collect, aggregate, and analyze performance metrics from log files or cloudwatch.
## Usage

### Instrumentation

Use the `with_latency_publishing!` macro to wrap existing rust functions with latency calculations. Or more generally just invoke
the publisher with your own metrics.

```Rust
use holochain_metrics::{Metric, LoggerMetricPublisher};

fn main() {

    let publisher = LoggerMetricPublisher::default();
    let metric = Metric:new("request_size", 1000.0);
    publisher.publish(&metric);

}
```

### Command interface
```shell
$ hc-metrics --help
```
```shell
metrics 0.0.37-alpha12
Holochain metric utilities

USAGE:
    holochain_metrics <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    cloudwatch-test           Runs a simple smoke test of cloudwatch publishing features
    help                      Prints this message or the help of the given subcommand(s)
    print-cloudwatch-stats    Prints descriptive stats in csv form over a time range from a cloudwatch datasource
    print-log-stats           Prints descriptive stats in csv form over a time range
```

## License
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)

Copyright (C) 2019, Holochain Foundation

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

[http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0)

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
