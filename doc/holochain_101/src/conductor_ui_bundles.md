# UI Bundles

`ui_bundles` is an array of configurations of folders containing static assets, like HTML, CSS, and Javascript files, that will be accessed through a browser and used as a user interface for one or more DNA instances. These are served via [UI Interfaces](./conductor_ui_interfaces.md), which is covered next.

**Optional**

### Properties

#### `id`: `string`
Give an ID of your choice to this UI Bundle

#### `root_dir`: `string`
Path to the folder containing the static files to serve

#### `hash`: `string` Optional
A hash can optionally be provided, which could be used to validate that the UI being installed is the UI bundle that was intended to be installed.

### Example
```toml
[[ui_bundles]]
id = "bundle1"
root_dir = "ui"
```
