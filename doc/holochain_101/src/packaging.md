# Building Holochain Apps: Packaging

The `hc package` command will automate the process of compiling your Zome code, encoding it, and inserting into the `.dna.json` file. In order to get these benefits, you just need to make sure that you have the right compilation tools installed on the machine you are using the command line tools from, and that you have the proper configuration files in your Zome folders.

`hc package` works with two special files called [`.hcignore` files](./hcignore_files.md) and [`.build` files](./build_files.md).