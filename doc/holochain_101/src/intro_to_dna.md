# Introduction to DNA

As a developer, though you will not have to interact directly with the contents of a DNA file that often, it is quite important to grasp its' role and structure.

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

[TODO link to command line tools 'package' command]

## Configuration

For the configuration related parts of your DNA, they will come from actual JSON files stored in your application folder. There will be multiple JSON files nested in the folder structure. An application folder should have a file in its' root called `app.json`.

This file should define various properties of your application. Some of these properties Holochain fully expects and will not work without, others can be customised to your application. To see the full details of which properties you can set in `app.json`, [go here TODO: link to DNA properties reference].

## Code

Obviously, the logic and functionality of your application will be written in code. Here is a quick overview of how that works.

Holochain allows you to break down the functionality of your application into smaller, logical modules, called "Zomes". Each Zome should have its own sub-folder within the main application folder, in which the configuration and code for that particular Zome should be placed. 

These Zomes can call and access the functionality of the others, but they are written independently. When the DNA file is being built, the code for these Zomes is encoded using Base64 encoding and combined with the configuration file associated with the Zome. 

The configuration file should be a JSON file [TODO: what is the naming convention for this file] stored in the Zome folder. To see the full details of a Zome JSON file, [go here TODO: link to Zome properties reference]. 

The only coding language that Holochain knows how to execute is WebAssembly. However, it is unlikely to write WebAssembly code by hand, more likely is to write code in a language that can compile to WebAssembly, such as Rust, or Assemblyscript.

It is likely within each Zome to store code written in another language, and to define a build step in which it is compiled to WebAssembly, and then included in your DNA. If this is sounding complex, don't worry, you won't have to confront a lot of this complexity yourself, there are tools supplied to make this easy, and you'll be writing in a language that's familiar, or easy to learn.

With this overview in mind, the details of app development can be explored.