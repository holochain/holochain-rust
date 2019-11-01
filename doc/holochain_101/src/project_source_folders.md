# Project Source Folders

The source code folder for a Holochain DNA project looks something like this, where the ellipses (...) indicate a folder
- test
    - ...
- zomes
    - ...
- .gitignore
- .hcignore
- app.json

`test` contains some starter code for writing tests.

`zomes` will contain sub-folders, each of which represents a "Zome", which can be thought of as a submodule of the source code of your DNA.

`.gitignore` contains useful defaults for ignoring files when using GIT version control.

`.hcignore` is utilized by the packaging commands of the `hc` command line tools

`app.json` is the top level configuration of your DNA.

Carry on to the next article to see about making changes to the configuration of a new project.

