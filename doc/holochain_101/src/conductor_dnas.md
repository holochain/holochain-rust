# DNAs

`dnas` is an array of configurations for "DNAs", meaning there can be multiple within one Conductor. A DNA is a packaged JSON file containing a valid DNA configuration including the WASM code for the Zomes. How to package DNA from source files can be read about [here](./packaging.md).

**Optional**

### Properties

#### `id`: `string`
Give an ID of your choice to this DNA

#### `file`: `string`
Path to the packaged DNA file

#### `hash`: `string`
A hash has to be provided for __FIXME "sanity check"??__

### Example
```toml
[[dnas]]
id = "app spec rust"
file = "example-config/app_spec.hcpkg"
hash = "Qm328wyq38924y"
```
