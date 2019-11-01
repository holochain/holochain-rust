# Configuring an App

As mentioned in [Intro to DNA: Configuration](./intro_to_dna_config.md) at the top level of a Holochain app source code folder there should be a file named `app.json`. This file is useful for two primary things:

1. When executing your application, Holochain can adopt specific behaviours, that can be configured in the `app.json` file. These mostly relate to how the Distributed Hash Table and P2P gossip functions.
2. You can give app users, and other developers background info about your application, such as the name of the app, and the author.

Here are the properties currently in use:

| Property                  | Description                                                                                                                                                                                                                                          |
|---------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| name                      | Give this application or service a name.                                                                                                                                                                                                             |
| description               | Describe this application or service for other people to read.                                                                                                                                                                                       |
| authors                   | Optionally provide contact details for the app developer(s). It is an array, so multiple people can be referenced.                                                                                                                                   |
| authors.identifier        | A string including a name, and a public email for the contact person.                                                                                                                                                                                |
| authors.public_key_source | Can reference a publicly hosted cryptographic "public key" from a private-public key-pair.                                                                                                                                                           |
| authors.signature         | The app developer can optionally add a string that is signed by their private key, so that app users could verify the authenticity of the application.                                                                                               |
| version                   | Provides a version number for this application. Version numbers are incredibly important for distributed apps, so use this property wisely.                                                                                                          |
| dht                       | This is a placeholder for the configuration options that Holochain will implement, regarding the Distributed Hash Table. It will provide a number of ways that the DHT behaviour can be customized.                                                  |
| properties                | Properties, if used, can be an object which implements numerous app specific configuration values. These can be up to the app developer to define, and, when implemented, will be able to be called using the [property]() function of the Zome API. |

The minimum recommended values to set when you initialize a new project folder are:

1. name
2. description
3. authors
4. version

To edit them, just open `app.json` in a text editor (preferably one with syntax highlighting for JSON), change the values, and save the file.
