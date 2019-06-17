<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Contents**

- [Scenario Testing](#scenario-testing)
      - [Import Example](#import-example)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

# Scenario Testing

`Scenario` is a class that is exported from `holochain-nodejs` and can be imported into your code.
It can be used to run tests individually for a single node, or to orchestrate multi-node tests, which is why it
is called `Scenario`. It does all the work of starting and stopping conductors and integrating with various test harnesses.

#### Import Example
```javascript
const { Scenario } = require('@holochain/holochain-nodejs')
```