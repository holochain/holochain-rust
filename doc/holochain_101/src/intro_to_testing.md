# Building Holochain Apps: Testing

By default, when you use `hc init` to create a new project folder, it creates a sub-directory called `test`. The files in that folder are equipped for testing your project.

Once you have a project folder initiated, you can run `hc test` to execute your tests. This combines the following steps:
  1. Packaging your files into a DNA file, located at `dist/bundle.json`. This step will fail if your packaging step fails.
  2. Installing build and testing dependencies, if they're not installed (`npm install`)
  4. Executing (with [holochain-nodejs](https://www.npmjs.com/package/@holochain/holochain-nodejs)) the test file found at `test/index.js`

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

You have the flexibility to write tests in quite a variety of ways, open to you to explore.