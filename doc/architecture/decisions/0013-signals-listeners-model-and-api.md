# 13. Signal/Listener model and API

Date: 2018-05-16

## Status

Proposed

## Context

In the go code we used polling from the UI to discover local state changes, be they in the source-chain or in the DHT.  We want to change this to an event based model where UI (or other clients, like bridged apps) can become listeners to signals.  Additionally there are system level signals (like warrants) as well as other app level definable signals that we want consumers of exposed holochain app APIs to be able to access on a push (i.e. listening) basis.

Note that this is only about signals that can be listened to by container/composer context, i.e. by the client of the core_api, NOT by the app nodes themselves.

## Decision

We will extend the API in a way that's roughly equivalent to the [Signal-slot pattern](https://en.wikipedia.org/wiki/Signals_and_slots), i.e. in the following ways:

1. In the DNA you can declare observable signals anywhere you can declare functions.  You can think of this almost identically to declaring a function except that it "goes the other way," i.e. a function def exposes an entry point where an signal def exposes an exit point.  It would look like this:

``` javascript
          "signal_declarations": [
            {
              "name": "Post",
              "description: "signal emmited when a post is committed",
              "params": null,
              "sends": {
                  "hash": "hash",
              },
            // ...
          ],
```

Note that in the example above we are using the attribute method of declaring the signature, and it declares what the signal will send to the listeners and what must be passed in as "params" on the listen request which may be useful for qualifying some aspect of what to listen for.  See [#134](https://waffle.io/holochain/org/cards/5b4cd03d0df367001d6d12a6) for details.

2. App developers can emit signals from their code via a new `emit()` function to be added to the api, e.g. like this:

``` javascript
postHash = commit("Post",{content:"foo"})
emit("Post",postHash)
```

3. Finally, just as you can call any function using the `core_api::call()`, you can register a listener with `core_api::listen()` and you and unregister a listener with `core_api::unlisten()`

## Consequences

- We need to clarify when you can use a `emit()` call, i.e. is it valid during validations and genesis?

- We need to make sure that app developers understand that this doesn't happen globally, but rather just to the clients that subscribed to the signal from the container.

- We need to think through what system signals we want to emit.

- We need to remember (and make a separate ADR) for signals that happen and can be listened for between nodes at the DHT level.

-Only clients that got permission to access the capability the signal is associated with can register a listener.
