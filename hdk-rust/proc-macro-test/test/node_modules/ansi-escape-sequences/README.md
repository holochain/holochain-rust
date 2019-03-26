[![view on npm](http://img.shields.io/npm/v/ansi-escape-sequences.svg)](https://www.npmjs.org/package/ansi-escape-sequences)
[![npm module downloads](http://img.shields.io/npm/dt/ansi-escape-sequences.svg)](https://www.npmjs.org/package/ansi-escape-sequences)
[![Build Status](https://travis-ci.org/75lb/ansi-escape-sequences.svg?branch=master)](https://travis-ci.org/75lb/ansi-escape-sequences)
[![Dependency Status](https://david-dm.org/75lb/ansi-escape-sequences.svg)](https://david-dm.org/75lb/ansi-escape-sequences)
[![js-standard-style](https://img.shields.io/badge/code%20style-standard-brightgreen.svg)](https://github.com/feross/standard)

# ansi-escape-sequences
A simple library containing all known terminal [ansi escape codes and sequences](http://en.wikipedia.org/wiki/ANSI_escape_code). Useful for adding colour to your command-line output, or building a dynamic text user interface.

## API Reference
**Example**  
```js
const ansi = require('ansi-escape-sequences')
```

* [ansi-escape-sequences](#module_ansi-escape-sequences)
    * [.cursor](#module_ansi-escape-sequences.cursor)
        * [.hide](#module_ansi-escape-sequences.cursor.hide)
        * [.show](#module_ansi-escape-sequences.cursor.show)
        * [.up([lines])](#module_ansi-escape-sequences.cursor.up) ⇒ <code>string</code>
        * [.down([lines])](#module_ansi-escape-sequences.cursor.down) ⇒ <code>string</code>
        * [.forward([lines])](#module_ansi-escape-sequences.cursor.forward) ⇒ <code>string</code>
        * [.back([lines])](#module_ansi-escape-sequences.cursor.back) ⇒ <code>string</code>
        * [.nextLine([lines])](#module_ansi-escape-sequences.cursor.nextLine) ⇒ <code>string</code>
        * [.previousLine([lines])](#module_ansi-escape-sequences.cursor.previousLine) ⇒ <code>string</code>
        * [.horizontalAbsolute(n)](#module_ansi-escape-sequences.cursor.horizontalAbsolute) ⇒ <code>string</code>
        * [.position(n, m)](#module_ansi-escape-sequences.cursor.position) ⇒ <code>string</code>
    * [.erase](#module_ansi-escape-sequences.erase)
        * [.display(n)](#module_ansi-escape-sequences.erase.display) ⇒ <code>string</code>
        * [.inLine(n)](#module_ansi-escape-sequences.erase.inLine) ⇒ <code>string</code>
    * [.style](#module_ansi-escape-sequences.style) : <code>enum</code>
    * [.styles(effectArray)](#module_ansi-escape-sequences.styles) ⇒ <code>string</code>
    * [.format(str, [styleArray])](#module_ansi-escape-sequences.format) ⇒ <code>string</code>

<a name="module_ansi-escape-sequences.cursor"></a>

## ansi.cursor
cursor-related sequences

**Kind**: static property of [<code>ansi-escape-sequences</code>](#module_ansi-escape-sequences)  

* [.cursor](#module_ansi-escape-sequences.cursor)
    * [.hide](#module_ansi-escape-sequences.cursor.hide)
    * [.show](#module_ansi-escape-sequences.cursor.show)
    * [.up([lines])](#module_ansi-escape-sequences.cursor.up) ⇒ <code>string</code>
    * [.down([lines])](#module_ansi-escape-sequences.cursor.down) ⇒ <code>string</code>
    * [.forward([lines])](#module_ansi-escape-sequences.cursor.forward) ⇒ <code>string</code>
    * [.back([lines])](#module_ansi-escape-sequences.cursor.back) ⇒ <code>string</code>
    * [.nextLine([lines])](#module_ansi-escape-sequences.cursor.nextLine) ⇒ <code>string</code>
    * [.previousLine([lines])](#module_ansi-escape-sequences.cursor.previousLine) ⇒ <code>string</code>
    * [.horizontalAbsolute(n)](#module_ansi-escape-sequences.cursor.horizontalAbsolute) ⇒ <code>string</code>
    * [.position(n, m)](#module_ansi-escape-sequences.cursor.position) ⇒ <code>string</code>

<a name="module_ansi-escape-sequences.cursor.hide"></a>

### cursor.hide
Hides the cursor

**Kind**: static property of [<code>cursor</code>](#module_ansi-escape-sequences.cursor)  
<a name="module_ansi-escape-sequences.cursor.show"></a>

### cursor.show
Shows the cursor

**Kind**: static property of [<code>cursor</code>](#module_ansi-escape-sequences.cursor)  
<a name="module_ansi-escape-sequences.cursor.up"></a>

### cursor.up([lines]) ⇒ <code>string</code>
Moves the cursor `lines` cells up. If the cursor is already at the edge of the screen, this has no effect

**Kind**: static method of [<code>cursor</code>](#module_ansi-escape-sequences.cursor)  

| Param | Type | Default |
| --- | --- | --- |
| [lines] | <code>number</code> | <code>1</code> | 

<a name="module_ansi-escape-sequences.cursor.down"></a>

### cursor.down([lines]) ⇒ <code>string</code>
Moves the cursor `lines` cells down. If the cursor is already at the edge of the screen, this has no effect

**Kind**: static method of [<code>cursor</code>](#module_ansi-escape-sequences.cursor)  

| Param | Type | Default |
| --- | --- | --- |
| [lines] | <code>number</code> | <code>1</code> | 

<a name="module_ansi-escape-sequences.cursor.forward"></a>

### cursor.forward([lines]) ⇒ <code>string</code>
Moves the cursor `lines` cells forward. If the cursor is already at the edge of the screen, this has no effect

**Kind**: static method of [<code>cursor</code>](#module_ansi-escape-sequences.cursor)  

| Param | Type | Default |
| --- | --- | --- |
| [lines] | <code>number</code> | <code>1</code> | 

<a name="module_ansi-escape-sequences.cursor.back"></a>

### cursor.back([lines]) ⇒ <code>string</code>
Moves the cursor `lines` cells back. If the cursor is already at the edge of the screen, this has no effect

**Kind**: static method of [<code>cursor</code>](#module_ansi-escape-sequences.cursor)  

| Param | Type | Default |
| --- | --- | --- |
| [lines] | <code>number</code> | <code>1</code> | 

<a name="module_ansi-escape-sequences.cursor.nextLine"></a>

### cursor.nextLine([lines]) ⇒ <code>string</code>
Moves cursor to beginning of the line n lines down.

**Kind**: static method of [<code>cursor</code>](#module_ansi-escape-sequences.cursor)  

| Param | Type | Default |
| --- | --- | --- |
| [lines] | <code>number</code> | <code>1</code> | 

<a name="module_ansi-escape-sequences.cursor.previousLine"></a>

### cursor.previousLine([lines]) ⇒ <code>string</code>
Moves cursor to beginning of the line n lines up.

**Kind**: static method of [<code>cursor</code>](#module_ansi-escape-sequences.cursor)  

| Param | Type | Default |
| --- | --- | --- |
| [lines] | <code>number</code> | <code>1</code> | 

<a name="module_ansi-escape-sequences.cursor.horizontalAbsolute"></a>

### cursor.horizontalAbsolute(n) ⇒ <code>string</code>
Moves the cursor to column n.

**Kind**: static method of [<code>cursor</code>](#module_ansi-escape-sequences.cursor)  

| Param | Type | Description |
| --- | --- | --- |
| n | <code>number</code> | column number |

<a name="module_ansi-escape-sequences.cursor.position"></a>

### cursor.position(n, m) ⇒ <code>string</code>
Moves the cursor to row n, column m. The values are 1-based, and default to 1 (top left corner) if omitted.

**Kind**: static method of [<code>cursor</code>](#module_ansi-escape-sequences.cursor)  

| Param | Type | Description |
| --- | --- | --- |
| n | <code>number</code> | row number |
| m | <code>number</code> | column number |

<a name="module_ansi-escape-sequences.erase"></a>

## ansi.erase
erase sequences

**Kind**: static property of [<code>ansi-escape-sequences</code>](#module_ansi-escape-sequences)  

* [.erase](#module_ansi-escape-sequences.erase)
    * [.display(n)](#module_ansi-escape-sequences.erase.display) ⇒ <code>string</code>
    * [.inLine(n)](#module_ansi-escape-sequences.erase.inLine) ⇒ <code>string</code>

<a name="module_ansi-escape-sequences.erase.display"></a>

### erase.display(n) ⇒ <code>string</code>
Clears part of the screen. If n is 0 (or missing), clear from cursor to end of screen. If n is 1, clear from cursor to beginning of the screen. If n is 2, clear entire screen.

**Kind**: static method of [<code>erase</code>](#module_ansi-escape-sequences.erase)  

| Param | Type |
| --- | --- |
| n | <code>number</code> | 

<a name="module_ansi-escape-sequences.erase.inLine"></a>

### erase.inLine(n) ⇒ <code>string</code>
Erases part of the line. If n is zero (or missing), clear from cursor to the end of the line. If n is one, clear from cursor to beginning of the line. If n is two, clear entire line. Cursor position does not change.

**Kind**: static method of [<code>erase</code>](#module_ansi-escape-sequences.erase)  

| Param | Type |
| --- | --- |
| n | <code>number</code> | 

<a name="module_ansi-escape-sequences.style"></a>

## ansi.style : <code>enum</code>
Various formatting styles (aka Select Graphic Rendition codes).

**Kind**: static enum of [<code>ansi-escape-sequences</code>](#module_ansi-escape-sequences)  
**Properties**

| Name | Type | Default |
| --- | --- | --- |
| reset | <code>string</code> | <code>&quot;\u001b[0m&quot;</code> | 
| bold | <code>string</code> | <code>&quot;\u001b[1m&quot;</code> | 
| italic | <code>string</code> | <code>&quot;\u001b[3m&quot;</code> | 
| underline | <code>string</code> | <code>&quot;\u001b[4m&quot;</code> | 
| fontDefault | <code>string</code> | <code>&quot;\u001b[10m&quot;</code> | 
| font2 | <code>string</code> | <code>&quot;\u001b[11m&quot;</code> | 
| font3 | <code>string</code> | <code>&quot;\u001b[12m&quot;</code> | 
| font4 | <code>string</code> | <code>&quot;\u001b[13m&quot;</code> | 
| font5 | <code>string</code> | <code>&quot;\u001b[14m&quot;</code> | 
| font6 | <code>string</code> | <code>&quot;\u001b[15m&quot;</code> | 
| imageNegative | <code>string</code> | <code>&quot;\u001b[7m&quot;</code> | 
| imagePositive | <code>string</code> | <code>&quot;\u001b[27m&quot;</code> | 
| black | <code>string</code> | <code>&quot;\u001b[30m&quot;</code> | 
| red | <code>string</code> | <code>&quot;\u001b[31m&quot;</code> | 
| green | <code>string</code> | <code>&quot;\u001b[32m&quot;</code> | 
| yellow | <code>string</code> | <code>&quot;\u001b[33m&quot;</code> | 
| blue | <code>string</code> | <code>&quot;\u001b[34m&quot;</code> | 
| magenta | <code>string</code> | <code>&quot;\u001b[35m&quot;</code> | 
| cyan | <code>string</code> | <code>&quot;\u001b[36m&quot;</code> | 
| white | <code>string</code> | <code>&quot;\u001b[37m&quot;</code> | 
| grey | <code>string</code> | <code>&quot;\u001b[90m&quot;</code> | 
| gray | <code>string</code> | <code>&quot;\u001b[90m&quot;</code> | 
| "bg-black" | <code>string</code> | <code>&quot;\u001b[40m&quot;</code> | 
| "bg-red" | <code>string</code> | <code>&quot;\u001b[41m&quot;</code> | 
| "bg-green" | <code>string</code> | <code>&quot;\u001b[42m&quot;</code> | 
| "bg-yellow" | <code>string</code> | <code>&quot;\u001b[43m&quot;</code> | 
| "bg-blue" | <code>string</code> | <code>&quot;\u001b[44m&quot;</code> | 
| "bg-magenta" | <code>string</code> | <code>&quot;\u001b[45m&quot;</code> | 
| "bg-cyan" | <code>string</code> | <code>&quot;\u001b[46m&quot;</code> | 
| "bg-white" | <code>string</code> | <code>&quot;\u001b[47m&quot;</code> | 
| "bg-grey" | <code>string</code> | <code>&quot;\u001b[100m&quot;</code> | 
| "bg-gray" | <code>string</code> | <code>&quot;\u001b[100m&quot;</code> | 

**Example**  
```js
console.log(ansi.style.red + 'this is red' + ansi.style.reset)
```
<a name="module_ansi-escape-sequences.styles"></a>

## ansi.styles(effectArray) ⇒ <code>string</code>
Returns an ansi sequence setting one or more effects

**Kind**: static method of [<code>ansi-escape-sequences</code>](#module_ansi-escape-sequences)  

| Param | Type | Description |
| --- | --- | --- |
| effectArray | <code>string</code> \| <code>Array.&lt;string&gt;</code> | a style, or list or styles |

**Example**  
```js
> ansi.styles('green')
'\u001b[32m'

> ansi.styles([ 'green', 'underline' ])
'\u001b[32;4m'
```
<a name="module_ansi-escape-sequences.format"></a>

## ansi.format(str, [styleArray]) ⇒ <code>string</code>
A convenience function, applying the provided styles to the input string and then resetting.

Inline styling can be applied using the syntax `[style-list]{text to format}`, where `style-list` is a space-separated list of styles from [ansi.style](#module_ansi-escape-sequences.style). For example `[bold white bg-red]{bold white text on a red background}`.

**Kind**: static method of [<code>ansi-escape-sequences</code>](#module_ansi-escape-sequences)  

| Param | Type | Description |
| --- | --- | --- |
| str | <code>string</code> | the string to format |
| [styleArray] | <code>Array.&lt;string&gt;</code> | a list of styles to add to the input string |

**Example**  
```js
> ansi.format('what?', 'green')
'\u001b[32mwhat?\u001b[0m'

> ansi.format('what?', ['green', 'bold'])
'\u001b[32;1mwhat?\u001b[0m'

> ansi.format('[green bold]{what?}')
'\u001b[32;1mwhat?\u001b[0m'
```

## Load anywhere

This library is compatible with Node.js, the Web and any style of module loader. It can be loaded anywhere, natively without transpilation.

Node.js:

```js
const ansi = require('ansi-escape-sequences')
```

Within Node.js with ECMAScript Module support enabled:

```js
import ansi from 'ansi-escape-sequences'
```

Within a modern browser ECMAScript Module:

```js
import ansi from './node_modules/ansi-escape-sequences/dist/index.mjs'
```

Old browser (adds `window.ansi`):

```html
<script nomodule src="./node_modules/ansi-escape-sequences/dist/index.js"></script>
```

* * *

&copy; 2014-19 Lloyd Brookes \<75pound@gmail.com\>. Documented by [jsdoc-to-markdown](https://github.com/jsdoc2md/jsdoc-to-markdown).
