# Serialization and JsonString

## Serialization in Holochain

### Why serialize anything? Why JSON?

#### Holochain zomes are written in WASM.

WASM only supports working directly with integers and manually allocating
memory. This means that sharing any data between holochain core and zome
functions must be serialized. There is no way that WASM functions can
understand the Rust type system natively. Serialized data can be allocated for
WASM to read out and deserialize into Rust types/structs/enums.

Any developers using the Rust HDK get the serialization/deserialization and
type handling _almost_ "for free". The macros for defining entities and zomes
automatically wrap the memory work and serialization round trips for anything
that implements `Into<JsonString>` and `TryFrom<JsonString>` (see below).

We use `serde` for our serialization round trips as it is by far the most
popular and mature option for Rust. Many serialization formats other than JSON
are supported by `serde` but JSON is a solid option. JSON allows us to easily
bring the Rust type system across to WASM with decent performance.

From the `serde_json` github repository README:

> It is fast. You should expect in the ballpark of 500 to 1000 megabytes per
> second deserialization and 600 to 900 megabytes per second serialization,
> depending on the characteristics of your data. This is competitive with the
> fastest C and C++ JSON libraries or even 30% faster for many use cases.
> Benchmarks live in the serde-rs/json-benchmark repo.

#### Holochain aims to support all WASM languages not just Rust/JS

The official Holochain HDKs are Rust and AssemblyScript. The Rust HDK will
always be the most tightly integrated HDK with core simply because Holochain
itself is Rust based.

Generally though, we are hoping and expecting many different WASM zome
languages build an ecosystem over time. Personally I'm hoping for a decent LISP
to appear ;)

To encourage as many languages as possible we want to keep the minimum
requirements for interacting with holochain core as minimal as possible.

Currently the two requirements for writing zomes in <your language of choice>:

- Must compile to WASM
- Must be able to serialize UTF-8 data and allocate to memory read by core

We can't do much about the first requirement but here are some lists to watch:

- https://github.com/appcypher/awesome-wasm-langs
- https://github.com/mbasso/awesome-wasm

The second requirement means that we must be very mindful of choosing a
serialization format that can round trip through as many languages as possible.

In the end, this is the main reason we chose JSON for communication with core.

Note that when we started on an AssemblyScript (ostentisbly JavaScript) HDK
there was not even a `JSON.parse()` method in AssemblyScript itself!

WASM is very promising and very immature, so esoteric serialization options are
not really options unfortunately, even if `serde` might support them.

#### JSON serialization only pertains to communication with core

Holochain often makes a distinction between "app data" and "core data".
Following the biomimicry theme we sometimes call this "conscious" vs.
"subconscious" when this data is used in core logic or exposed to zomes.

The most obvious example of this is the `Entry` enum that has an `Entry::App`
variant explicitly for app data, and other variants for system logic.

The `Entry` enum itself is serialized via JSON so that is has maximal
compatibility across all zome languages (see above) across the core/wasm
boundary. However, the _contents_ of `Entry::App(..)` are treated as an opaque
UTF-8 string by Holochain core. Naturally the HDK macros we offer provide sugar
to work with the value of app entries but this is not enforce anywhere within
core.

This means that zome developers can implement their own serialization logic for
their own data if they wish. Simply by wrapping a zome-serialized app entry
value in `"\"...\""` it becomes a string primitive from core's perspective. The
zome can do anything needed with this, including custom validation logic, etc.

### Serialization through Rust types

#### How Rust serializes: serde from 1000m

The `serde` crate leans heavily on the Rust compiler for serialization round
tripping.

Using the "vanilla" `serde_json` crate affords this logic on the way in:

```rust
let foo_json = serde_json::to_string(foo).unwrap();
```

Notes:

- There is an `unwrap` but this can't fail for simple structs/enums in practise
  - The `unwrap` can fail e.g. serializing streams but we don't do that
  - The compiler enforces that everything we pass to `serde` can `Serialize`
- `foo` can be anything that implements `Serialize`
- we have no direct control over the structure of the JSON output
  - the `Serialize` implementation of `foo` decides this for us
  - in the case of nested data e.g. hash maps, `Serialize` works recursively

OR using the manual `json!` macro:

```rust
let foo_json = json!({"foo": foo.inner()});
```

Notes:

- We no longer have an `unwrap` to deal with
- We have a lot of direct control over the structure of our output JSON
- For better or worse we avoid what the compiler says about `Serialize` on `Foo`
- We must now manually ensure that `"{\"foo\":...}"` is handled _everywhere_
  - Including in crates we don't control
  - Including when we change our JSON structure across future releases
  - Including across WASM boundaries in HDK consumers

AND on the way out:

```rust
let foo: Foo = serde_json::from_str(&hopefully_foo_json).unwrap();
```

Notes:

- Serde relies on compiler info, the type `Foo` on the left, to deserialize
- Serde requires that `hopefully_foo_json` makes sense as `Foo`
  - This _definitely can_ fail as the json is just a `String` to the compiler
  - In real code do not `unwrap` this, handle the `Err` carefully!

#### JSON structure and the compiler

All this means that our JSON data MUST closely align with the types we define
for the compiler. There is a lot of flexibility offered by `serde` for tweaking
the output (e.g. lowercasing names of things, modifying strings, etc.) but the
tweaks involve a lot of boilerplate and have limits.

For example this can be awkard when handling `Result` values. The `Result` enum
has two variants in Rust, `Ok` and `Err`. Both of these, like all enum variants
in Rust, follow the title case convention.

This means that in a JS container/HDK consuming JSON values returned from zome
functions that return a `Result` (a good idea!) we see this JavaScript:

```javascript
const result = app.call(...)
const myVar = result.Ok...
```

We get a `result.Ok` rather than the `result.ok` that we'd expect from
idiomatic JavaScript.

As the JSON structure comes from the Rust compiler, we have two options:

- Force serde to output JSON that follows the conventions of another language
- Force containers/HDKs to provide sugar to map between Rust/XXX idioms

As the first option requires a lot of boilerplate and isn't interoperable
across all languages anyway (e.g. kebab case, snake case, etc.) we currently
are pushing this sugar down to container/HDK implementations.
