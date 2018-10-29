# Create A New Project

The command line tools discussed in the last article can be used to easily create a new folder on your computer, that contains all the initial folders and files needed for a Holochain application. 

You will typically want to create a new project folder for a Holochain application this way.  This one approach will suit the creation of a new holochain powered app or adding holochain into an existing application. 

In your terminal, change directories to one where you wish to initialize a new Holochain app. The command will create a new folder within the current directory for your app.

Come up with a name for your application, or at least for your project folder.

Copy or type the command below into your terminal, except replace `your_app_name` with the name you came up with. Press `Enter` to execute the command.

```shell
hc init your_app_name
```

`hc` specifies that you wish to use the Holochain command line tools. `init` specifies to use the command for initializing a new project folder. `your_app_name` is an argument you supply as the app, and folder name.

This has created a new folder in which you have the beginnings of a Holochain app.

TODO: document clarified folder structure here and use to illustrate an overview of app development

clarification of how to add a holochain back end to an existing project
