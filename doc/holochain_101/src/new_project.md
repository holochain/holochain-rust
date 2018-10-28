# Create A New Project

The command line tools discussed in the last article can be used to easily create a new folder on your computer, that contains all the initial folders and files needed for a Holochain application. 

You will typically want to start a new Holochain application this way, since creating them all by hand would waste your time.

In your terminal, change directories to one where you wish to initialize a new Holochain app. The command will create a new folder within the current directory for your app.

Come up with a name for your application, or at least for your project folder.

Copy or type the command below into your terminal, except replace `your_app_name` with the name you came up with. Press `Enter` to execute the command.

```shell
hc init your_app_name
```

`hc` specifies that you wish to use the Holochain command line tools. `init` specifies to use the command for initializing a new project folder. `your_app_name` is an argument you supply as the app, and folder name.

This has created a new folder in which you have the beginnings of a Holochain app.

It looks something like this, where the ellipses (`...`) indicate a folder
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
