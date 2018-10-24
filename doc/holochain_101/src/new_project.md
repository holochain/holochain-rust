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

Thus, this tells the Holochain command line tools to initialize a new project with the name you have specified.

Upon running this command, you will see a new folder generated containing the beginnings of your Holochain app.

TODO: document clarified folder structure here and use to illustrate an overview of app development
