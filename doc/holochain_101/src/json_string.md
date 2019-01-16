# Serialization and JsonString

## Why serialize anything? Why JSON?

### Holochain zomes are written in WASM.

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

### Holochain aims to support all WASM languages not just Rust/JS

The official Holochain HDKs are Rust and AssemblyScript. The Rust HDK will
always be the most tightly integrated HDK with core simply because Holochain
itself is Rust based.

Generally though, we are hoping and expecting many different WASM zome
languages build an ecosystem over time. Personally I'm hoping for a decent LISP
to appear ;)

To encourage as many languages as possible we want to keep the minimum
requirements for interacting with holochain core as minimal as possible.

Currently the two requirements for writing zomes in `<your favourite language>`:

- Must compile to WASM
- Must be able to serialize UTF-8 data and allocate to memory read by core

We can't do much about the first requirement but here are some lists to watch:

- https://github.com/appcypher/awesome-wasm-langs
- https://github.com/mbasso/awesome-wasm

The second requirement means that we must be very mindful of choosing a
serialization format that can round trip through as many languages as possible.

In the end, this is the main reason we chose JSON for communication with core.

Note that at the time of writing, the AssemblyScript (ostentisbly JavaScript)
WASM implementation does not even provide a native `JSON.parse()` method!
To do something as apparently simple as serialize JSON in JavaScript we have
had to implement a custom JSON parser. At least JSON (naturally) maps very well
to JavaScript native data, other serialization/language combinations are even
further from maturity.

WASM is very promising but very immature so esoteric serialization options are
not really viable options right now, even if `serde` supports them in Rust.

### JSON serialization only pertains to communication with core

Holochain often makes a distinction between "app data" and "core data".
Following the biomimicry theme we sometimes call this "conscious" vs.
"subconscious" when this data is used in zomes or core logic respectively.

The most obvious example of this is the `Entry` enum that has an `Entry::App`
variant explicitly for app data, and other variants for system logic.

The `Entry` enum itself is serialized via JSON so that is has maximal
compatibility across all zome languages (see above) across the core/wasm
boundary. However, the _contents_ of `Entry::App(..)` are treated as an opaque
UTF-8 string by Holochain core. Naturally the HDK macros we offer provide sugar
to work with the value of app entries but this is not enforced anywhere within
core. Because the Rust serialization round tripping must work across both core
and the HDK it must work equally well while treating the app entry values as
opaque in the subconscious and meaningful structs in the conscious. This is
achieved through a healthy dose of compiler and macro magic.

This means that zome developers can implement their own serialization logic for
their own data if they wish. Simply by wrapping a zome-serialized app entry
value in `"\"...\""` it becomes a string primitive from core's perspective. The
zome can do anything needed with this, including custom validation logic, etc.
The `RawString` type handles this automatically with `JsonString` (see below).

## Serialization through Rust types

### How Rust serializes: serde from 1000m

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

- We no longer have an `unwrap` so there is slightly less boilerplate to type
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

### JSON structure, the Rust compiler and you

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
- Force developers to work with a very leaky abstraction over the Rust compiler

As the first option requires a lot of boilerplate and isn't interoperable
across all languages anyway (e.g. kebab case, snake case, etc.) we currently
are pushing this sugar down to container/HDK implementations. Additionally, the
serialized form of entries is used to calculate `Address` values for storage
and retrieval from the local chain and DHT so we need to be very careful here
as it will be hard to change in the future.

That said, we are open to constructive feedback on what this sugar looks like
and how it works! Ideally zome development is as idiomatic as possible across
as many languages as possible 🕶

### Binary data as base64

We recommend base64 encoding binary data straight into an app entry string that
you can use in your zome logic directly (see above).

Yes this uses more space than binary data, 33% more to be specific :(

But there are benefits:

- It is UTF-8 and web (e.g. data URI) friendly
- Simply wrapped in `"\"..\""` it becomes valid JSON (see `RawString` below)
- It has wide language support (see above for why this is important)
- It will be supported by all persistence backends for the forseeable future
  - At least these storage systems require base64 encoded data at some point:
    - Browser based localStorage
    - MongoDB
    - Elasticsearch
    - Amazon SimpleDB
    - Amazon DynamoDB.

The performance penalty can be minimal:

https://lemire.me/blog/2018/01/17/ridiculously-fast-base64-encoding-and-decoding/

### JSON is lame! Can Holochain support `<my favourite serialization format>`?

Yes... and no...

It depends what you mean by "support".

Right now, most serialization formats are supported in app/zome data simply by
wrapping the output in double quotes so core sees it as a JSON string literal.
Holochain core won't try to interpret/mangle any of that data so the zome can
theoretically do whatever it wants at that point without a performance hit.

In practise, there are some limitations as mentioned in this doc:

- WASM languages tend to have no or limited serialization options
  - you may need to roll your own parse/stringify logic
  - seriously... e.g. we pushed our own `JSON.parse` implementation upstream
    for the AssemblyScript team, that's _JSON parsing in JavaScript_!
  - don't underestimate how bleeding edge and limited the WASM tooling still is
    - to work directly with WASM you must be prepared to bleed
- If you don't use JSON you can't use `hdk` macros for that part of your zome
- Only valid UTF-8 strings are supported (may change in the future)

If you're looking for a way to provide core data in non-JSON format then NO
that is not supported and won't be in the short-mid term future.

Yes, `serde` supports many serialization options but:

- Not all data in core uses default `serde` serialization logic
  - e.g. this document explaining non-default serde serialization logic
- Swapping to a different serializer in serde is not just a matter of passing
  config to serde
  - we'd have to centralise/`match` everywhere and swap out `serde_json` for
    analogous crates in each other format we'd want to use
  - even using a `SerialString` instead of `JsonString` (see below) would not
    clear out every implementation without a lot of work
- Serde is already quite heavy in compilation/WASM files so we don't want to
  bloat that more with edge-case serialization needs
  - every new format is a new crate
- We don't (yet) have any use-cases showing that JSON is a problem/bottleneck
- Adding more serialization options would exacerbate non-idiomatic container
  and HDK data structure mapping issues (see above)

## JsonString

### The problem and our solution

Sometimes we want to _nest_ serialization (e.g. `hdk::call`) and sometimes we
want to _wrap_ serialization (e.g. `Entry::App`), sometimes converting to a
string uses entirely different logic (e.g. error values). Ideally we want the
compiler to guide us through this process as mistakes are common and difficult
to debug. We also want serialization logic to be as invisible as possible to
zome developers using our HDKs.

Serde will serialize anything that implements `Serialize`, including `String`
so we added a type `JsonString` that does not _automatically_ round trip to act
as a logical "checkpoint" in our code.

`JsonString` doesn't "do" anything beyond giving ourselves and the compiler a
shared target while stepping through the serialization round trip.

Essentially we trade this:

```rust
// foo_a is a Foo
// foo_json is a String
// Foo implements Serialize and Deserialize
let foo_json = serde_json::to_string(&foo_a)?;
let foo_b: Foo = serde_json::from_str(&foo_json)?;
```

for this:

```rust
// foo_a is a Foo
// JsonString implements From<Foo>
// Foo implements TryFrom<JsonString>
let foo_json = JsonString::from(foo_a);
let foo_b = Foo::try_from(hopefully_foo_json)?;
```

Which looks very similar but protects us from this bug:

```rust
let foo_json = serde_json::to_string(&foo_a)?;
let foo_json = serde_json::to_string(&foo_json)?; // <-- double serialized :/
let foo_b: Foo = serde_json::from_str(&foo_json)?; // <-- will fail :(
```

Because nesting `JsonString::from()` calls is a compiler error:

```rust
let foo_json = JsonString::from(JsonString::from(foo_a)); // <-- compiler saves us :)
```

and this bug:

```rust
let foo_a: Foo = serde_json::from_str(&string_but_not_json)?; // <-- runtime error :(
```

Because calling `Foo::try_from(String)` is (probably) a compiler error:

```rust
let foo_a = Foo::try_from(string_but_not_json)?; // <-- compiler saves us again :)
```

and this bug:

```rust
type Foo = Result<String, String>;
let foo_json_a = json!({"Err": some_error.to_string()}); // <-- good key `Err`
// somewhere else... maybe a different crate or old crate version...
let foo_json_b = json!({"error": some_error.to_string()}); // <-- bad key `error` :/

let foo: Foo = serde_json::from(&foo_json_a)?; // <-- works, key matches variant name
let foo: Foo = serde_json::from(&foo_json_b)?; // <-- runtime error! :(
```

Because the structure of the JSON data is defined centrally at compile time:

```rust
// Result<Into<JsonString>, Into<JsonString>> is implemented for you by HC core
let foo_json_a = JsonString::from(Err(some_error.to_string()));
// only one way to do things, automatically consistent across all crates
// doing anything different is a compiler issue
let foo_json_b = JsonString::from(Err(some_error.to_string()));
```

Which is great for the majority of data that needs serializing. There are some
important edge cases that we need to cover with additional techniques/tooling.

#### String handling

`JsonString::from` assumes any `String` or `&str` passed to it is already a
serialized JSON value.

We can use `serde_json::to_string` and `json!` to create JSON data that we can
then wrap in `JsonString`.

```rust
// same end result for both of these...
let foo_json = JsonString::from(serde_json::to_string(&foo));
let foo_json = JsonString::from(foo);
```

More commonly useful, we can move back and forward between `String` and
`JsonString` without incurring serialization overhead or human error:

```rust
// this does a round trip through types without triggering any serde
JsonString::from(String::from(JsonString::from(foo)));
```

This is helpful when a function signature requires a `String` or `JsonString`
argument and we have the inverse type. It also helps when manually building
JSON data by _wrapping_ already serialized data e.g. with `format!`.

An example taken from core:

```rust
impl<T: Into<JsonString>, E: Into<JsonString> + JsonError> From<Result<T, E>> for JsonString {
    fn from(result: Result<T, E>) -> JsonString {
        let is_ok = result.is_ok();
        let inner_json: JsonString = match result {
            Ok(inner) => inner.into(),
            Err(inner) => inner.into(),
        };
        let inner_string = String::from(inner_json);
        format!(
            "{{\"{}\":{}}}",
            if is_ok { "Ok" } else { "Err" },
            inner_string
        )
        .into()
    }
}
```

Which looks like this:

```rust
let result: Result<String, HolochainError> =
    Err(HolochainError::ErrorGeneric("foo".into()));

assert_eq!(
    JsonString::from(result),
    JsonString::from("{\"Err\":{\"ErrorGeneric\":\"foo\"}}"),
);
```

When given a `Result` containing any value that can be turned into a
`JsonString` (see below), we can _convert_ it first, then _wrap_ it with
`String::from` + `format!`.

### String serialization

Sometimes we _want_ a `String` to be serialized as a JSON string primitive
rather than simply wrapped in a `JsonString` struct. `JsonString::from` won't
do what we need because it always wraps strings, we need to _nest_ the `String`
serialization.

```rust
let foo = String::from(JsonString::from("foo")); // "foo" = not what we want
let foo = ???; // "\"foo\"" = what we want
```

To keep the type safety from `JsonString` and nest String serialization use
`RawString` wrapped in `JsonString`. `RawString` wraps `String` and serializes
it to a JSON string primitive when `JsonString`ified.

```rust
// does what we need :)
let foo = String::from(JsonString::from(RawString::from("foo"))); // "\"foo\""
```

An example of this can be seen in the core version of the `Result`
serialization from above that deals with `String` error values:

```rust
impl<T: Into<JsonString>> From<Result<T, String>> for JsonString {
    fn from(result: Result<T, String>) -> JsonString {
        let is_ok = result.is_ok();
        let inner_json: JsonString = match result {
            Ok(inner) => inner.into(),
            // strings need this special handling c.f. Error
            Err(inner) => RawString::from(inner).into(), // <-- RawString here!
        };
        let inner_string = String::from(inner_json);
        format!(
            "{{\"{}\":{}}}",
            if is_ok { "Ok" } else { "Err" },
            inner_string
        )
        .into()
    }
}
```

Which looks like this:

```rust
let result: Result<String, String> = Err(String::from("foo"));

assert_eq!(
    JsonString::from(result),
    JsonString::from("{\"Err\":\"foo\"}"),
)
```

If we didn't do this then the `format!` would return invalid JSON data with the
String error value missing the wrapping double quotes.

`RawString` is useful when working with types that have a `.to_string()` method
or similar where the returned string is _not_ valid JSON.

Examples of when `RawString` could be useful:

- Error descriptions that return plain text in a string
- Base64 encoded binary data
- Enum variants with custom string representations
- "Black boxing" JSON data that Rust should not attempt to parse

### Implementing `JsonString` for custom types

As mentioned above, there are two trait implementations that every struct or
enum should implement to be compatible with core serialization logic:

- `impl From<MyType> for JsonString` to serialize `MyType`
- `impl TryFrom<JsonString> for MyType` to attempt to deserialize into `MyType`

Note that `TryFrom` is currently an unstable Rust feature. To enable it add
`!#[feature(try_from)]` to your crate/zome.

Based on discussions in the Rust community issue queues/forums, we expect this
feature to eventually stabilise and no longer require feature flags to use.

The `TryFrom` trait will need to be added as `use std::convert::TryFrom` to
each module/zome implementing it for a struct/enum.

#### Boilerplate

To defer all the logic to standard `serde` defaults with some sensible
debug logic in the case of an error, there are two utility functions in core,
`default_to_json` and `default_try_from_json`.

The standard minimal boilerplate looks like this:

```rust
struct MyType {}

impl From<MyType> for JsonString {
  fn from(my_type: MyType) -> Self {
    default_to_json(my_type)
  }
}

impl TryFrom<JsonString> for MyType {
  type Error = HolochainError;
  fn try_from(json_string: JsonString) -> Result<Self, Self::Error> {
    default_try_from_json(json_string)
  }
}
```

#### Automatic derive

The standard boilerplate has been implemented as a derive macro in the
`holochain_core_types_derive` crate.

Simply `#[derive(DefaultJson)]` to add the above boilerplate plus some extra
conveniences (e.g. for references) to your type.

`DefaultJson` requires:

- `JsonString` is included
- `HolochainError` is included
- `MyType` implements `Serialize`, `Deserialize` and `Debug` from serde/std

```rust
use holochain_core_types::json::JsonString;
use holochain_core_types::error::HolochainError;

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct MyType {}
```

### Using JsonString as the property of a struct/enum

Because `JsonString` cannot _automatically_ be round tripped with `Serialize`
and `Deserialize`, the following can cause difficulty:

```rust
#[derive(Serialize, Deserialize)]
struct Foo {
  bar: JsonString,
}
```

The compiler will complain about this because anything deriving `Serialize`
recursively must consist only of values that also implement `Serialize`.

There are a few approaches here, each with benefits and tradeoffs.

0. Swap out `JsonString` with `String`
0. Use a serde attribute to manually serialize `Bar`
0. Use a serde attribute to skip `Bar`
0. Create a "new type" or wrapper/conversion struct

#### Swap `JsonString` with `String`

This approach is quick and dirty. Simply change the type of `Bar` to `String`.
When prototyping or on deadline, this might be the most attractive option ;)

This will likely cause problems upstream and downstream of what you are doing,
or may be symptomatic of poorly handled JSON somewhere. This is _roughly_ how
`Entry` used to work, with a `String` valued `SerializedEntry`and `JsonString`
valued `Entry` that could be swapped between using a `From` implementation.

Done correctly we can "onboard" values to `Foo` by simply carefully wrapping
and unwrapping the `String`. Done badly, we reintroduce the possibility for
invalid wrap/nest/etc. logic to creep in.

This works best when the fields on `Foo` are private and immutable, exposed
only through getter/setter/new style methods that internally convert between
`JsonString` and `String`.

This option is less suitable if we _want_ to double serialize the nested JSON
data when serializing `Foo`. For an example of where we preserve JSON rather
than trying to automatically deserialize or wrap it with structs, see the
return values from `hdk::call` (not using structs, but similar ideas).

Also consider that somebody reading your code might entirely miss the fact that
`Foo::bar` is JSON data if all they read is the struct definition.

It may be worthwhile adding methods to `Foo` to enforce this:

```rust
#[derive(Serialize, Deserialize)]
pub struct Foo {
  bar: String,
}

impl Foo {
  pub fn new(bar: JsonString) -> Foo {
    Foo {bar: String::from(bar)}
  }

  pub fn bar(&self) -> JsonString {
    JsonString::from(self.bar.clone())
  }
}
```

Treat `bar` as though it was going to be stored as a `JsonString` right until
the last moment.

Avoid this:

```rust
let bar_json = json!({"bar": bar.inner()}).to_string();
// somwhere later...
let foo = Foo{bar: bar_json};
```

Because then _everything_ that needs to use `Foo` must consistently implement
the manual jsonification logic. This is especially important if `Foo` and/or
bar is to be used across multiple crates.

Instead, prefer this:

```rust
#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct Bar {
  bar: ..
}

let bar_json = JsonString::from(Bar{bar: ..});
let foo = Foo::new(bar); // assuming impl Foo::new from above
```

The result is still a raw `String` in `Foo` but the validity and consistency of
the JSON data is enforced across all crates by `JsonString::from(bar)`.

It is even possible to internalise the `JsonString` completely within the `Foo`
methods using `Into<JsonString>`. This is covered in more detail below.

#### Using serde attributes

Serde allows us to set serialization logic at the field level for structs.

The best example of this is handling of `AppEntryValue` in core.
As all zome data is treated as JSON, assumed to line up with internal structs
in the HDK but potentially opaque string primitives (see above) we simply alias
`AppEntryValue` to `JsonString`.

The `Entry` enum needs to be serialized for many reasons in different contexts,
including for system entries that zome logic never handles directly.

It looks something like this (at the time of writing):

```rust
#[derive(Clone, Debug, Serialize, Deserialize, DefaultJson)]
pub enum Entry {
    #[serde(serialize_with = "serialize_app_entry")]
    #[serde(deserialize_with = "deserialize_app_entry")]
    App(AppEntryType, AppEntryValue),

    Dna(Dna),
    AgentId(AgentId),
    Delete(Delete),
    LinkAdd(LinkAdd),
    LinkRemove(LinkRemove),
    LinkList(LinkList),
    ChainHeader(ChainHeader),
    ChainMigrate(ChainMigrate),
}
```

Note that `Entry`:

- Derives `Serialize` and `Deserialize` and even `DefaultJson`!
- Contains `AppEntryValue` in a tuple, which is a `JsonString`
- Uses some serde serialization attributes

This works because the serialization attributes tell serde how to handle the
`JsonString` _in this context_. This is a double edged sword. We have explicit
control over the serialization so we can never accidentally wrap/nest/etc. JSON
data in an invalid way. We also only define the serialization for this type in
this one place. If `AppEntryValue` was used in some other struct/enum, we would
have to manually remember to use the same or compatible serialize/deserialize
callbacks.

This approach also gives a lot of control over the final JSON structure. We can
avoid stutters and reams of redundant data in the final output. This can
mitigate the verbosity and awkwardness of compiler-driven JSON structures
when sending data to other languages (see above).

The serde documentation explains in great (technical) detail how to implement
custom serialization and deserialization logic for many different data types:

https://serde.rs/field-attrs.html

For reference, the callbacks used in `Entry` above look like this:

```rust
pub type AppEntryValue = JsonString;

fn serialize_app_entry<S>(
    app_entry_type: &AppEntryType,
    app_entry_value: &AppEntryValue,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = serializer.serialize_tuple(2)?;
    state.serialize_element(&app_entry_type.to_string())?;
    state.serialize_element(&app_entry_value.to_string())?;
    state.end()
}

fn deserialize_app_entry<'de, D>(deserializer: D) -> Result<(AppEntryType, AppEntryValue), D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct SerializedAppEntry(String, String);

    let serialized_app_entry = SerializedAppEntry::deserialize(deserializer)?;
    Ok((
        AppEntryType::from(serialized_app_entry.0),
        AppEntryValue::from(serialized_app_entry.1),
    ))
}
```

Obviously this is a lot of boilerplate for one tuple, and is really only the
tip of the iceberg for how complex custom serde implementations can get. Use
this for surgical implementations along critical path type safety/ergonomics.

#### Skip the attribute

Serde also allows for attributes to be completely skipped during serialization.

In the context of a `JsonString` this is unlikely to be the desired behaviour.
If we are serializing the outer struct we _probably_ want the inner JSON data
to also be serialized, but not necessarily, or perhaps we don't _need_ it and
so can live without it.

This option has very clear tradeoffs. We lose the JSON data when the outer
struct is serialized but also don't have to worry about how it might be
represented.

This option is very handy during development/prototyping/debugging when you
want to sketch out a larger idea without immediately tackling serde logic.

Simply add the `#[serde(skip)]` attribute to your struct.

```rust
#[derive(Serialize, Deserialize)]
struct Foo {
  #[serde(skip)]
  bar: JsonString,
}
```

#### Wrap/convert to a new type or struct

If it is possible to create a struct that better represents the data, or a new
type to hold it, then _that_ struct can implement to/try_from `JsonString`.

This is very similar to the first option where we put a `String` into `Foo` but
it provides semantics, information for the compiler and somewhere to hook
`into()` for our code.

```rust
// Bar as a new type
#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct Bar(String)

#[derive(Serialize, Deserialize)]
struct Foo {
  bar: Bar,
}

impl Foo {
  fn new(bar: Bar) -> Foo {
    Foo { bar }
  }

  fn bar(&self) -> Bar {
    self.bar.clone()
  }
}

// somewhere else...
let json = JsonString::from(..);
let bar = Bar::from(json);
let foo = Foo::new(bar);

// or...
let json = JsonString::from(..);
let foo = Foo::new(json.into());
```

The biggest drawback to this approach is the potential for stutter. With lots
of nested types we give the compiler more power but also can incidentally bloat
the JSON output a lot.

Many ad-hoc/once-off types can also become confusing for humans and lead to
duplicated/redundant code over time.

It is easy to end up with JSON like `{"Foo":{"bar":{"Bar":[".."]}}}` with a
poorly chosen combination of enum variants and tuples.

As per all the considerations outlined for using `String` directly on `Foo`,
avoid using `json!` or similar to build up the internal `String` of `Bar`.

## Hiding JsonString with `Into<JsonString>`

It is possible in function signatures to simply leave an argument open to
anything that can be converted to `JsonString`.

This is exactly like using `Into<String>` but for JSON data. An even looser
option is to only require `TryInto<JsonString>` but this makes little or no
difference to us in practise.

An example of this is the `store_as_json` used to pass native Rust typed data
across the WASM boundary. This is used internally by the `define_zome!` macro
for all zome funtions:

```rust
pub fn store_as_json<J: TryInto<JsonString>>(
    stack: &mut SinglePageStack,
    jsonable: J,
) -> Result<SinglePageAllocation, RibosomeErrorCode> {
    let j: JsonString = jsonable
        .try_into()
        .map_err(|_| RibosomeErrorCode::ArgumentDeserializationFailed)?;

    let json_bytes = j.into_bytes();
    let json_bytes_len = json_bytes.len() as u32;
    if json_bytes_len > U16_MAX {
        return Err(RibosomeErrorCode::OutOfMemory);
    }
    write_in_wasm_memory(stack, &json_bytes, json_bytes_len as u16)
}
```

The relevant `into()` or `try_into()` method is called _internally_ by the
function accepting `Into<JsonString>`, meaning the caller needs to know almost
nothing about _how_ the serialization is done. Additionally, the caller _could_
do its own custom serialization, passing a `String` through, which would be
wrapped as-is into a `JsonString`.

Unfortunately this doesn't work as well for structs because of the way trait
bounds work (or don't work) without complex boxing etc. See above for simple
strategies to cope with nested/wrapped serialization in nested native data
structures.

This approach can be combined with the "quick and dirty" `Foo` with private
`String` internals to create a `Foo` that can store _anything_ that round trips
through `JsonString`:

```rust
struct Foo {
  bar: String,
}

impl Foo {
  fn new<J: Into<JsonString>> (bar: J) -> Foo {
    Foo{ bar: String::from(JsonString::from(bar)) }
  }

  fn bar<T: TryFrom<JsonString>>(&self) -> Result<T, HolochainError> {
    Ok(JsonString::from(self.bar.clone()).try_into()?)
  }
}

// somewhere later..
// we can build MyBar ad-hoc to send to Foo as long as it implements JsonString
// we could create MyOtherBar in the same way and send to Foo in the same way
#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct MyBar { .. }

let my_bar = MyBar::new(..);
// auto stores as String via. JsonString internally
let foo = Foo::new(my_bar);
// note we must provide the MyBar type at restore time because we destroyed
// that type info during the serialization process
let restored_bar: MyBar = foo.bar()?;
```

This is how the `ContentAddressableStorage` trait used to work. It would
"magically" restore the correct `Content` from storage based on an `Address`
and type alone, provided the compiler had the type info available at compile
time.

We had to sacrifice this neat trick due to incompatible constraints from the
type system elsewhereon the CAS, but it should work well in most scenarios :)
