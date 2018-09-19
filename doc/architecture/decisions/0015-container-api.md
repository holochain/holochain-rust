# 15. Container API

Date: 2018-09-17

## Status

Proposed

## Context

* We have several different roles for using a Holochain app (through a *container*):
  * local, QML based UI components
  * as a special case of the above: administration UI
  * externally located (living in a browser) web UIs connected through some form of IPC
  * *bridged* apps, i.e. Holochain app as the user of another Holochain app
  * Services hosting multiple Holochain apps and exposing their zome functions to UIs selectively
* With HoloSqape and the Holochain-nodejs, we already have two container implementations

A Holochain app cannot be used directly. Holochain is built as a library that exposes an interface for
creating Holochain instances and calling their zome functions (see [core_api](/core_api/src/lib.rs)).
This *core_api* requires its client to provide a context, which holds the representation of the agent
(name and keys), a logger and persistence and will also include a networking proxy in the future.

While it is possible to use this library directly in a monolithic app where the UI is tightly
coupled to the app and everything linked to the Holochain library into the same executable, we regard this
as a special case, since this does not allow for app composability (which we think is a crucial
concept for the Holochain app ecosystem). 

Instead, in the context of the Rust iteration of Holochain, we have been following the notion of
providing a *Container* as a relevant concept (and implemention) for the deployment of Holochain apps:
we provide a Container (i.e. [HoloSqape](https://github.com/holochain/holosqape)) for each supported platform (Linux, MacOS, Windows, iOS, Android)
that gets installed on a host machine. Holochain apps get installed into the container.
The Container:  
 * manages installed hApps (Holochain Apps),
 * manages agent identities (keys) ,
 * should also enable hApps to call functions of other hApps - what we call *bridging*,
 * has to implement access permissions to installed hApps.

So far, the interface our Container implementation provides was growing organically
in [container.h](https://github.com/holochain/holosqape/blob/master/bindings/container.h).

With upcoming alternative container implementations (based on [Holochain-nodejs](https://github.com/holochain/holochain-nodejs)
or a custom one for HoloPorts) we should drive the process of building this Container API
consciously and with more coherence and visibility amongst our separate dev teams.

We need a protocol for communication with a Holochain container and a specification of what upcoming
containers have to implement, so that apps and UIs can be build against a standardised interface.

## Decision

We establish the **Container as a fundamental module of the Holochain architecture/stack**
by specifying its **API**, that can be assumed by UI components, Holochain apps (i.e. zome
functions in the case of bridging)
and remote processes alike to be implemented by the context a Holochain app is executed in.

Fundamental to this API is **user/client roles and permissions**.
Clients will be able to use different subsets of the Container's API depending on their specific permissions. 
So an implicit aspect of this API is that every potential API call happens in the context of a known
client identified through a client ID that the API manages and returns to the admin client as handles. 

We will specify this API in a separate *specification document*.
The following subsection provide examples for how this *could* look like:

### Example API

Every client (of the Container API, i.e. QML root widgets, admin UI in the case of HoloSqape, 
and network services built on top of the Container) 
will have these functions available, though functions can return with a permission denied
error in case an app or capability was given that the current client is not allowed to use.

* `call(app_hash, capability_name, function_name, parameters) -> {results}`
* `connect(app_hash, capability_name, signal_name, callback_function)`
* `installed_apps() -> [hash]`
* `capabilities_of(app_hash) -> [String]`
* `functions_of(app_hash, capability_name) -> [String]`
* `request_access(app_hash, capability)`
* `can_access?(app_hash, capability) -> bool`


### Admin

Only the admin user can manage apps and permissions:

* `install_app(Dna) -> hash`
* `uninstall_app(hash)`
* `start_app(hash)`
* `stop_app(hash)`
* `promote_to_admin(client_id)`
* `retract_admin(client_id)`
* `grant_capability(client_id, app_hash, capability)`
* `deny_capability(client_id, app_hash, capability)`

### Extensible
More API capabilities might be added in the future.

## Consequences

* We can build several separate pieces independently against the specified API
* We can build different container implementations for different contexts but have components that
  rely on the existence of a container decoupled from those specific implementations.
  Components that rely on and could use this API:
  * any kind of GUI
  * app composition code like bridging
  * external web UI
  * Services orchastrating several hosted apps