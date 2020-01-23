# sim1h

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.net)

[![Twitter Follow](https://img.shields.io/twitter/follow/holochain.svg?style=social&label=Follow)](https://twitter.com/holochain)

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)

A simulator/emulator for [lib3h](https://github.com/holochain/lib3h).

## Why?

rrDHT is agent centric by design.

This makes certain things difficult debug.

There is no "global store" to audit to see what has been published.

Which is great for scalability/decentralisation but problematic for:

- Holochain Core devs trying to distinguishing a networking error from a core error
- Conductor devs verifying the crossover between wasm and network workflows
- Zome devs reviewing how data is published and recieved across nodes
- Hardware (e.g. holoport) devs wanting to test OS level concerns decoupled from DHT network health etc.

It's also nice to have another implementation of `Lib3hProtocol`.
One small step towards this being more "protocol" than "implementation detail".

## How?

Sim1h is a sandbox/centralised network implementation.

It implements the same `Lib3hProtocol` interface as `lib3h` so that conductors
can send all the same data through, but handles them centrally.

Notably send/receive is handled by recipients writing to and polling the
database rather than senders pushing over direct network connections.

This means everything hitting "the network" gets dumped into a database where
it can be inspected by devs. It also delays our requirements of solving the
tough challenges of NAT traversal, gossip, node discovery, and DHT sharding.

All operations are idempotent/append-only meaning all data written is available
for review at all times.

Devs can open up the database with a GUI like [dynamodb-admin](https://github.com/aaronshaf/dynamodb-admin) to inspect what happened "globally".

## What?

Currently wrapping dynamodb from AWS for the key/value store because:

- has a cloud option to support nodes in different locations
- has a local/self-install option for local development/CI/testing
- has a 25GB free tier with no monthly fees
- it's pretty popular and does what you'd expect for basic key/value stuff
- provides a stream client that shows an ordered history of all recent writes

## Implementation Details

### Entry data

All entry data published to the network is represented as "entry aspects".

This includes:

- Entry CRUD operations
- Link add/removal
- Entry headers

Aspect data all implements the `AddressableContent` trait from [`holochain-persistence`](https://github.com/holochain/holochain-persistence).

This means all aspects can be (and are) added to the database as simple key/value pairs.

The database accepts all aspects verbatim from agents, serialized using
standard `aspect.content()` calls.

Once an aspect is written to the database a reference to its address is
appended to a list under the entry address.

Queries for an entry involve the conductor fetching all aspects for a given
entry address from the database then "reflecting" these back at itself _as though_
the query came from a third party.

### Direct messaging

Send/receive handling is modeled as a simple append-only inbox for each agent.

Every agent has 2 "folders" in their inbox, "all messages" and "seen messages".

When A wants to send a message to B:

- A stores the message content in the database under its networking `request_id`
- A appends the `request_id` to B's "all messages" folder
- B periodically polls and diffs both "all messages" and "seen messages"
- B fetches unseen messages visible in the diff and records the IDs as "seen"
- B completes the same process in reverse to notify A of message receipt

### Connect/join space

Agents simply touch their own Address in a table representing the space they
want to join.

Any agent with an address in the space is assumed to be "connected".

Spaces and table names are currently the same thing.
Tables are created by the first agent that discovers it doesn't exist.

This is likely to change "soon" as creating tables on the fly works great
locally in the `dynamodb` jar but is tricky on the managed AWS service.

## Scaling

For the purposes of testing hApps with large numbers of "nodes" sim1h should
scale well because Dynamodb itself is very scalable in terms of raw storage/read/write
metrics (e.g. 25GB storage in the free tier).

There are some details to be aware of.

String based attribute values are [limited to 400kb total size](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.NamingRulesDataTypes.html#HowItWorks.DataTypes.String).

This means that agent inboxes and entry aspect lists will blow up at some point.

Sending hundreds or thousands of messages to a node is fine, but 10's of thousands will break.

Same situation for entry links.

This could be fixed relatively easily by modelling messaging/linking differently.

For example, incoming messages could be a linked list with a single "last ID seen"
value to act as a cursor for each agent.

There are plenty of other ways to do this.

First person to blow up inboxes/links can implement a solution ;)

Also note that [we are](https://github.com/holochain/sim1h/pull/24):

- enabling strong consistency for all reads
- using dynamodb scans for entry aspects
- avoiding write transactions
- recursively brute forcing several failure modes for puts, such as rate limits
- scaling a bunch of reads linearly with number of agents due to polling

Some of these things could potentially be refactored to be more polite (e.g. exponential backoff for failures and avoiding scans)
and some are unlikely to change (e.g. the read consistency model).

## Security model

**There is no security in the Sim1h layer.**

**Sim1h assumes local validation was honest and successful for all aspects.**

**Sim1h trusts all authors of all data with no further validation.**

For well written zomes and the standard conductor software this works fine.

Zomes always validate data locally before committing it to the local source
chain so it is safe in the sense that honest nodes will always send valid data.

As long as every node with AWS credentials runs up-to-date conductors and zomes
the security is comparable to traditional server infrastructure.

**It is not safe to hand out AWS credentials to anyone with modified conductor
software or who might write to the database in any way outside what is managed
by the conductor + sim1h.**

For example, if Alice was sending Bob direct messages to Bob's inbox, and Carol
wanted to block them, Carol could simply record "seen" IDs directly to Bob's
inbox then Bob would never retrieve them.

In summary, Sim1h is for "internal use only", whatever that means to you.

## Usage

Sim1h is available in the v0.0.31 of the #[holochain-rust conductor](https://github.com/holochain/holochain-rust)
as is a new `sim1h` network type.  Also, there is a nix command to run a local dynamodb instance.

In your conductor config, use the following for the `network` config section:

```
[network]
type = 'sim1h'
dynamo_url = 'http://localhost:8000' # URL of running dynamodb instance
```

You can run a local dynamodb instance at port 8000 by [entering a nix-shell](https://docs.holochain.love/) and running:

```shell
    dynamodb
```
or
```shell
    dynamodb-memory
```
The latter is for a non-persistent instance of the database.

If you want to expose your local dynamodb instance over the internet, we suggest using a tunneling service like [ngrok](https://ngrok.com/) to map a public URL to your local port. Then, your friends can use that public URL as their `dynamo_url` instead of localhost.

Ngrok is also included in the `sim1h` nix-shell.

Please note that you must define `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` environment variables when running your conductor which are passed to [rusoto](https://github.com/rusoto/rusoto) (the underlying library we use to access dynamodb).  When running your own instance of dynamodb, these two values can be what ever you want, but they must be set.  Additionally in this case you can use the value of the `AWS_ACCESS_KEY_ID` to create completely different name-spaces for different conductor sets.

That's it!

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
