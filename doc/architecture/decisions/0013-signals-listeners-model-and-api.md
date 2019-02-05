# 13. Signal/Listener model and API

Date: 2018-05-16

## Status

Accepted

## Context

In the go code we used polling from the UI to discover local state changes, be they in the source-chain or in the DHT.  We want to change this to an event based model where UI (or other clients, like bridged apps) can become listeners to signals.  Additionally there are system level signals (like warrants) as well as other app level definable signals that we want consumers of exposed holochain app APIs to be able to access on a push (i.e. listening) basis.

Note that this is only about signals that can be listened to by conductor context, i.e. by the client of the core_api, NOT by the app nodes themselves.

## Decision

We will extend the API in a way that's roughly equivalent to the [Signal-slot pattern](https://en.wikipedia.org/wiki/Signals_and_slots), i.e. in the following ways:

1. In the DNA you can declare observable signals anywhere you can declare functions.  You can think of this almost identically to declaring a function except that it "goes the other way," i.e. a function def exposes an entry point where an signal def exposes an exit point.  It would look like this:

``` javascript
          "signals": [
            {
              "name": "Post",
              "description": "signal emmited when a post is committed",
              "config-params": null,
              "arguments": {
                  "name": "hash",
                  "type": "hash"
              },
            // ...
          ],
```

The above declaration defines what arguments the signal will send to the listeners and additionally a config-param object to be passed in on the listen request which may be useful for qualifying some aspect of what to listen for.

2. App developers can emit signals from their code via a new `emit()` function to be added to the api, e.g. like this:

``` javascript
postHash = commit("Post",{content:"foo"})
emit("Post",postHash)
```

3. Finally, just as you can call any function using the `core_api::call()`, you can register a listener with `core_api::listen()` and you and unregister a listener with `core_api::unlisten()`

## Consequences

- We need to clarify when you can use a `emit()` call, i.e. is it valid during validations and genesis?

- We need to make sure that app developers understand that this doesn't happen globally, but rather just to the clients that subscribed to the signal from the conductor.

- We need to think through what system signals we want to emit.

- We need to remember (and make a separate ADR) for signals that happen and can be listened for between nodes at the DHT level.

- Only clients that got permission to access the capability the signal is associated with can register a listener.

- We need to decide if this will be a full pub/sub model where the producer is fully decoupled from the subscriber with an intermediate handler, or if it will follow the signal/slot pattern where the producer has a reference to the subscriber and sends directly.
