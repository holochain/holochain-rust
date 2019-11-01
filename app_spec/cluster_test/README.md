# app_spec cluster test

The following environment variable needs to be set before running this,
to specify which binary to execute the DNA with.
```
EMULATION_HOLOCHAIN_BIN_PATH=/Path/to/holochain/binary
```

Then, run `node index.js`

Optionally, you can give it an argument, with the number of Conductors to start, like:
(default is 2)
```
node index.js 4
```

Optionally, you can give it a second argument, selecting whether to show debug logs, like:
(default is false)
```
node index.js 4 true
```
