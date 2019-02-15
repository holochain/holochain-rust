# Running Tests

By default, when you use `hc init` to [create a new project folder](./new_project.md), it creates a sub-directory called `test`. The files in that folder are equipped for testing your project. The contents of the folder represent a simple nodejs package (in that they have a `index.js` file and a `package.json` file).

Tools to help with testing are also built right into the [development command line tools](./intro_to_command_line_tools.md).

## hc test

Once you have a project folder initiated, you can run `hc test` to execute your tests. This combines the following steps:
  1. Packaging your files into a DNA file, located at `dist/bundle.json`. This step will fail if your packaging step fails.
  2. Installing build and testing dependencies, if they're not installed (`npm install`)
  3. Executing the test file found at `test/index.js` (`node test/index.js`)

The tests can of course be called manually using nodejs, but you will find that using the convenience of the `hc test` command makes the process much smoother, since it includes the packaging step for when you change the DNA source files.

`hc test` also has some configurable options.

If you want to run it without repackaging the DNA, run it with
```shell
hc test --skip-package
```

If your tests are in a different folder than `test`, run it with
```shell
hc test --dir tests
```
 where `tests` is the name of the folder.

If the file you wish to actually execute is somewhere besides `test/index.js` then run it with
```shell
hc test --testfile test/test.js
```
where `test/test.js` is the path of the file.