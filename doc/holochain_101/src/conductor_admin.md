# Administering Conductors

It is possible to dynamically configure a Conductor via a JSON-RPC interface connection. There is a powerful API that is exposed for doing so.

To do this, first, recall that the `admin = true` [property needs to be set](./conductor_interfaces.md#admin-bool-optional) for the interface that should allow admin access. Second, it is helpful to review and understand the behaviours around the [`persistence_dir` property](./conductor_persistence_dir.md) for the Conductor.

Finally, to view the API for this functionality, check out the full [API reference material](https://developer.holochain.org/api/latest/holochain_container_api/interface/struct.ContainerApiBuilder.html) for it and scroll to view the `with_admin_dna_functions` comment block and the `with_admin_ui_functions` comment block. Calling these functions works exactly the same way as the other [JSON-RPC API calls that were discussed](./conductor_json_rpc_api.md).

As mentioned in [production Conductor](./production_conductor.md), there is a GUI in development that will cover all this functionality, so that it does not have to be done programmatically, but can be done by any user simply point and click.
