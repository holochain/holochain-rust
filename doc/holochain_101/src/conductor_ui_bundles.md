# UI Bundles

`ui_bundles` is an array of configurations for "UI Bundles", meaning there can be multiple within one Conductor. A UI Bundle is a folder containing static assets, like HTML, CSS, and Javascript files, that will be accessed through the browser and used as a user interface for one or more DNA instances. These are served via [UI Interfaces](./conductor_ui_interfaces.md), which is covered next.

**Optional**

### Properties

#### `id`: `string`
Give an ID of your choice to this UI Bundle

#### `root_dir`: `string`
Path to the folder containing the static files to serve

#### `hash`: `string`
A hash has to be provided for __FIXME "sanity check"??__

### Example
```toml
[[ui_bundles]]
id = "bundle1"
root_dir = "ui"
hash = "Qm000"
```
