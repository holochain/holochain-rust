# Emitting Signals

Holochain provides a mechanism for clients to subscribe to various signals that are emitted by different parts the system, some of them low level, and other explicitly by zome developers.  Low level signals are used by our testing frame work.  High level signals produced by zomes can be used by UIs to receive "Application" level notifications of events.

To emit a signal in your zome code, use the hdk `emit_signal` function, like this:

``` rust
hdk::emit_signal("message_received", JsonString::from_json(&format!(
    "{{message: {}}}", message)));
```
