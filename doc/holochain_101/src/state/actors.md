# Internal actors

Actors are discussed in two contexts:

- Each Holochain agent as an actor in a networking context
- Riker actors as an implemenation detail in the Holochain core lib

This article is about the latter.

## Actor model

The [actor model](https://en.wikipedia.org/wiki/Actor_model) is a relatively safe approach to co-ordinating concurrency.

At a high level:

- An actor is the "primitive", like objects are the primitive of the OO paradigm
- Actors are stateful but this state is never exposed to the rest of the system
- Actors manage their internal state
- Actors maintain a message queue or "inbox"
- Messages can be received concurrently but must be processed sequentially in FIFO order
- The messages have a preset format
- Actors update their internal state in response to messages
- Actors can send messages to each other
- Messages are always processed at most once
- Actors can "supervise" each other to create a fault tolerent system
- A supervisor can restart or stop a failed actor, or escalate the decision to another supervisor

The guarantees provided by the message queue allow actors to use stateful logic
that would not be safe otherwise in a concurrent context.

For example, we can implement logic that reads/writes to the file system without
locks or other co-ordination. Then put an actor in front of this logic and only
interact with the file system through the relevant actor.

## Riker

[Riker](http://riker.rs/) is an actor library for Rust.

The actor implementation in Riker has a few key concepts:

- protocol: a set of valid messages that can be sent (e.g. an enum)
- actor system: manages and co-ordinates all actors
- actor: anything implementing the `Actor` trait to create new actor instances and handle receiving messages
- actor instance: an instance of the actor struct that has internal state and is tracked by the actor system
- actor ref(erence): an ActorRef<MyProtocol> that can tell messages to the actor instance it references via. the actor system

The actor reference is a "killer feature" of Riker for us.

- known size at compile, safe as properties of structs/enums
- small size, almost free to clone
- safe to share across threads and copy, no Arc reference counting, no locks, etc.
- safe to drop (the actor system maintains a URI style lookup)
- known type, no onerous generic trait handling
- no onerous lifetimes
