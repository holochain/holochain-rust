# Emitting Signals

Holochain provides a mechanism for clients to subscribe to various signals 
that are emitted by different parts the system, 
some of them low level, and other explicitly by zome developers.  

Low level signals are used by our [testing frame work](https://github.com/holochain/diorama).
High level signals produced by zomes can be used by UIs to receive 
"Application" level notifications of events.

A signal consists of a name and a content represented as a `JsonString`.
Best practice for creating type safe `JsonString`s is by having a struct 
derive from `DefaultJson` like so: 

``` rust
#[derive(Debug, Serialize, Deserialize, DefaultJson)]
struct SignalPayload {
    message: String
}
let message = "Hello World".to_string();
hdk::emit_signal("message_received", SignalPayload{message});
```

This will send a signal with the following JSON representation:
```json
{
  signal_type: 'User', 
  name: 'message_received', 
  arguments: '{"message":"Hello World"}',
}
```

User signals (those emitted by `hdk::emit_signal`) are sent over
all websocket interfaces that include the instance that is emitting
a signal.

[hc-web-client](https://github.com/holochain/hc-web-client) enables
UIs to easily listen for signals by registering a callback through
`onSignal`:

```javascript
const { onSignal } = await this.webClientConnect({url})

onSignal((msg) => {
  console.log(msg.signal) // -> { signal_type: 'User', name: 'test-signal', arguments: '{"message":"test message"}' }
})
```

### Outlook
The current implementation of signals is at MVP stage - there is everything
needed to not force UIs to poll for new data, given that the DNA uses
callbacks (mainly node-2-node messages and `receive`) to emit signals
as needed.

Future additions will be:
* Signal signature description in the DNA
  [ADR 13](https://github.com/holochain/holochain-rust/blob/develop/doc/architecture/decisions/0013-signals-listeners-model-and-api.md)
  describes signals as statically defined properties of a DNA which 
  would enable conductor level binding/connecting of signals with
  slotes (i.e. zome functions) similar to bridges but with looser 
  coupling.
* DHT level signals/hooks
  Local changes like authoring a new entry could already emit a signal
  and thus trigger a reload/re-render of the UI.
  But what if a remote agent authored a chat message in a channel
  I'm following?
  It would be very handy to have DNA hooks (i.e. callbacks) that get
  called when a DHT node has validated and starts holding a new entry
  or link, such that either a node-2-node message can be sent to a set
  of observers, or introduce the concept of network level signals that
  do exactly that automatically.