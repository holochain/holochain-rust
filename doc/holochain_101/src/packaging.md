<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Contents**

- [Building Holochain Apps: Packaging](#building-holochain-apps-packaging)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

# Building Holochain Apps: Packaging

The `hc package` command will automate the process of compiling your Zome code, encoding it, and inserting into the `.dna.json` file. In order to get these benefits, you just need to make sure that you have the right compilation tools installed on the machine you are using the command line tools from, and that you have the proper configuration files in your Zome folders.

`hc package` works with two special files called [`.hcignore` files](./hcignore_files.md) and [`.hcbuild` files](./build_files.md).
