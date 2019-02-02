# Introduction to DNA: Configuration

As a developer, you won't have to interact directly with the contents of a DNA file that often. However, it is quite important to grasp its role and structure.

Holochain DNA files are written in a data format known as JSON. It stores sets of key-value pairs, and allows a nested tree structure. It looks like this:

```json
{
  "property_name": "property_value",
  "nest_name": {
    "nested_property_name": "nested_property_value"
  }
}
```

JSON is usually used for configuration and static data, but in the case of Holochain, these DNA files also contain compiled code, which is executable by Holochain.

As previously mentioned, you do not need to edit this "master" DNA file directly. Holochain command line tools can be used to build it from your raw files.

[Learn more about the package command which fulfills this function](https://github.com/holochain/holochain-rust/tree/develop/cli#usage)

## Configuration

For the configuration-related parts of your DNA, they will come from actual JSON files stored in your application folder. There will be multiple JSON files nested in the folder structure. An application folder should have a file in its root called `app.json`.

This file should define various properties of your application. Some of these properties Holochain fully expects and will not work without, others can be customised to your application.

### app.json Properties

A default `app.json` file looks roughly like this:

```json
{
  "name": "Holochain App Name",
  "description": "A Holochain app",
  "authors": [
    {
      "indentifier": "Author Name <author@name.com>",
      "public_key_source": "",
      "signature": ""
    }
  ],
  "version": "0.0.1",
  "dht": {},
  "properties": null
}
```
