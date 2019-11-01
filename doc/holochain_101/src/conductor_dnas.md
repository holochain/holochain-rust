# DNAs

`dnas` is an array of configurations for "DNAs" that are available to be instantiated in the Conductor. A DNA is a packaged JSON file containing a valid DNA configuration including the WASM code for the Zomes. How to package DNA from source files can be read about [here](./packaging.md).

**Optional**

### Properties

#### `id`: `string`
Give an ID of your choice to this DNA

#### `file`: `string`
Path to the packaged DNA file

#### `hash`: `string` Optional
A hash can optionally be provided, which could be used to validate that the DNA being installed is the DNA that was intended to be installed.

### Example
```toml
[[dnas]]
id = "app spec rust"
file = "example-config/app_spec.dna.json"
```
