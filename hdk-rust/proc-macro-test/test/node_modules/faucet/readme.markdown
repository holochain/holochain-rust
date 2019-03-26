# faucet

human-readable TAP summarizer

# example

Pipe TAP text into the `faucet` command, get back pretty results:

![piping tap](images/test.gif)

You can use any runner you want to generate the TAP output. Here we'll use
[tape](https://npmjs.org/package/tape):

![tape runner](images/falafel.gif)

You can give the `faucet` command a list of files:

![list of files](images/gutter.gif)

or if you just type `faucet`, any js files in `test/` or `tests/` will be run
using the `tape` command internally:

![implicit test/ directory](images/dnode.gif)

# install

To get the `faucet` command, with [npm](https://npmjs.org) do:

```
npm install -g faucet
```

# generating TAP

The great thing about TAP is that it's inherently serializable on stdout, so you
can use whichever libraries you wish to generate it.

Many test libraries have ways to get TAP output.

[tape](https://npmjs.org/package/tape) and [tap](https://npmjs.org/package/tap)
will give you TAP output by default.

With a [tape](https://npmjs.org/package/tape) test, you would just write
a `test.js` like:

``` js
var test = require('tape');

test('beep boop', function (t) {
    t.plan(2);
    
    t.equal(1 + 1, 2);
    setTimeout(function () {
        t.deepEqual(
            'ABC'.toLowerCase().split(''),
            ['a','b','c']
        );
    });
});
```

and then just run the file with `node test.js` to get the TAP output:

```
TAP version 13
# beep boop
ok 1 should be equal
ok 2 should be equivalent

1..2
# tests 2
# pass  2

# ok
```

or if you have a directory of files, you can use the `tape` command that you get
when you `npm install -g tape`:

```
$ tape test/*.js
TAP version 13
# stream in a stream
ok 1 should be equivalent
# expand a streams1 stream
ok 2 should be equivalent
# expand a streams2 stream
ok 3 should be equivalent
# expand a streams2 stream with delay
ok 4 should be equivalent

1..4
# tests 4
# pass  4

# ok

```

To get TAP out of [mocha](https://npmjs.org/package/mocha), do `mocha -R tap`:

```
$ mocha -R tap
1..17
ok 1  shim found
ok 2  core shim not found
ok 3  false file
ok 4  false module
ok 5  local
ok 6  index.js of module dir
ok 7  alternate main
ok 8  string browser field as main
ok 9  string browser field as main - require subfile
ok 10  object browser field as main
ok 11  object browser field replace file
ok 12  object browser field replace file - no paths
ok 13  replace module in browser field object
ok 14  replace module in object browser field with subdirectory
ok 15  replace module in object browser field with subdirectory containing
package.json
ok 16  replace module in object browser field with subdirectory containing
package.json with string browser field as main
ok 17  replace module in object browser field with subdirectory containing
package.json with object browser field as main
# tests 17
# pass 17
# fail 0
```

Once you've got a way to get TAP out of your tests, just pipe into `faucet`:

![mocha pipe](images/mocha.gif)

# usage

```
usage:
  faucet [FILES]
  command | faucet
```

# license

MIT
