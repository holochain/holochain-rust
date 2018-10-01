
# hc command line holochain service app reference

# This command has been deprecated

## hc --help
To get help from the command line, type ```$ hc --help```

You'll see this response:
```
NAME:
   hc - holochain peer command line interface

USAGE:
   hc [global options] command [command options] [arguments...]

VERSION:
   0.0.6 (holochain 12)

COMMANDS:
     init, i           bootstrap the holochain service
     clone, cl, c      clone a holochain instance from a source
     test, t           run validation against test data for a chain in development
     genesis, gen, g   generate genesis entries or keys for a cloned holochain
     web, serve, w     serve a chain to the web on localhost:<port> (defaults to 3141)
     call, ca          call an exposed function
     dump, d           display a text dump of a chain
     dht               display a text dump of an app's dht
     join, c           joins a holochain by copying an instance from a source and generating genesis blocks
     reset, r          reset a chain. Warning this destroys all chain data!
     seed              seed calculates DNA hash and builds DNA file without generating genesis entries.  Useful only for testing and development.
     status, s         display information about installed chains
     template, dev, t  generate a configuration file template suitable for editing
     help, h           Shows a list of commands or help for one command

GLOBAL OPTIONS:
   --debug        debugging output
   --path value   path to holochain directory (default: ~/.holochain)
   --verbose, -V  verbose output
   --help, -h     show help
   --version, -v  print the version
```
You can also get help on any of the sub-commands by calling hc help structured like this: ```hc <command> help```

## hc init <id_string>
This command initializes the system with a default identity and generates default public/private keys for interacting with other networked peers. You provide a single string of identifying information which will be visible as your ```_agent_name```. This is often an email address.

Usage: ``` hc init `fred@flintstone.com' ```

## hc join <Source_Holochain>
This is how you join with an existing holochain that has already been created. You can specify a source by its unique DNA hash identifier (A string of characters that looks something like this: ```QmeHrPW2Y2xGeTLWv7vTYHNr9ViV1LYE3cFKgY2kskUf7G```) to retrieve it from the holochain of holochains (assuming it has been shared there by its original author). You can also point to a local file you've gotten from someone you trust.

## hc clone <Source_location> <New_Holochain_Name>
As a developer who is building or modifying a holochain application, you can clone a pre-existing holochain application configuration by specifying existing application files.

You can source from files anywhere such as from a git repo you've cloned, from a live chain you're already running in your .holochain directory, or one of the examples included in the holochain repository.

    hc clone <SOURCE_PATH> <NAME_FOR_NEW_HOLOCHAIN>```

For example: ```hc clone ./examples/sample sample```

Before you launch your chain, this is the chance for you to customize the application settings like the Application NAME, for example.

If you are developing a holochain application and need to destroy your running version of that holochain to test your new code, you can force it to overwrite with ```hc clone --force /programming/source/directory target-name```

## hc gen chain
Builds your genesis entries for starting your new local chain. It can also be used to generate new keys.

```hc gen chain <name>``` Creates genesis entries launching your new local chain
```hc gen keys <name>``` Creates new keys on this holochain

## hc status
To see what holochains are installed on your system, just type ```hc status```. You'll get a result showing each chain name and the the ID/hash of the DNA of the chain.

For example, results should look something like this:
```
installed holochains:
     escrow <not-started>  
     flack Qm9yPX4cX3hA9DNkx6kNjdKRmPLdDZeJrPcWChpLv6X7PG
```
## hc test <HOLOCHAIN_NAME>  (deprecated: will be moved to hcdev command)
This command runs the test harness for the specified holochain

``` hc test <HOLOCHAIN_NAME> ```

## hc serve <HOLOCHAIN NAME>
Launch UI services via web socket to exposed application functions. By default, the browser services are only available via localhost.

## hc call <fn_name> (deprecated: will be moved to hcdev command)
Call an exposed function from the hc command line instead of a web socket / browser UI.

## hc bs
Contact a bootstrap server specified in the chain's DNA to notify of your existence and search for peers to communicate with.

## hc dump <chain_name>
Display all the contents of the specified personal chain

## hc dev (deprecated: will be moved to hcdev command)
Secret undocumented developer feature which generates a skeletal holochain app to start developing with.
