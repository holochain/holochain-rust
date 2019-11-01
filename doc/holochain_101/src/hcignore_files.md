# Ignoring Files Using A .hcignore File

Sometimes, you'll want to exclude files and folders in your project directory to get a straight `.dna.json` file that can be understood by Holochain. In order to do that, just create a `.hcignore` file. It has a similar structure to `.gitignore` files:

```
README.md
dist
.DS_Store
```

The `hc package` command includes patterns inside `.gitignore` files automatically, so you don't have to write everything twice. Also *hidden* files are ignored by default as well.

Because `hc package` will attempt to package everything in the directory that is not explicitly ignored, Holochain will return an error if the DNA package is malformed. It is a common mistake to forget to exclude files or folders in the .hcignore file, so that your DNA will be valid.
