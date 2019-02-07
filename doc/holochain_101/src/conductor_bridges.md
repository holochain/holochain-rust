# Bridges
`bridges` is an array of configuration instances that are configured to be able to make calls to Zome functions of another instance. You can think of this of as configuring an internal direct interface between DNA instances.  The [section on bridging](bridging.md) provides more information on how this ability is used to to compose complex applications out of many DNA instances.

**Optional**

### Properties

#### `caller_id`: `string`
A reference to the given ID of a defined [instance](./conductor_instances.md) that calls the other one. This instance depends on the callee.


#### `callee_id`: `string`
A reference to the given ID of a defined [instance](./conductor_instances.md) that exposes capabilities through this bridge. This instance is used by the caller.

#### `handle`: `string`
The caller's local handle for this bridge and the callee. A caller can have many bridges to other DNAs and those DNAs could by bound dynamically. Callers reference callees by this arbitrary but unique local name.

### Example
```toml
[[bridges]]
caller_id = "app1"
callee_id = "app2"
handle = "happ-store"
```
