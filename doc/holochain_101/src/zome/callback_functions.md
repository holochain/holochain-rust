# Callback functions

## Overview

A callback function is implemented in the Zome language and called by Holochain.

Contrast this to a Zome API function that is implemented by Holochain and called
by the Zome.

As per Zome API functions, the names of the callback functions may be slightly
different depending on the language. The canonical name follows Rust naming
conventions but other languages may vary these (e.g. camel casing).

To implement a callback function in a Zome simply define it and Holochain will
call it automatically during standard internal workflows.

## Reference

### Genesis

Canonical name: `genesis`
Parameters: none

Called the first time an agent launches an instance of a DNA with Holochain. Within genesis an app develop has the ability whether the given agent should be allowed to successfully join the Holochain network for this particular DNA, by implenting rules, or preconditions that must be met. If `genesis` comes back from the Zome with a fail, the agent will not be able to join.

[View it in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/macro.define_zome.html)