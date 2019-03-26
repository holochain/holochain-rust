#!/usr/bin/env node
var faucet = require('../');
var minimist = require('minimist');
var defined = require('defined');
var tapeCmd = require.resolve('tape/bin/tape');

var spawn = require('child_process').spawn;
var fs = require('fs');
var path = require('path');

var argv = minimist(process.argv.slice(2));
var tap = faucet({
    width: defined(argv.w, argv.width, process.stdout.isTTY
        ? process.stdout.columns - 5
        : 0
    )
});
process.on('exit', function (code) {
    if (code === 0 && tap.exitCode !== 0) {
        process.exit(tap.exitCode);
    }
});
process.stdout.on('error', function () {});

if (!process.stdin.isTTY || argv._[0] === '-') {
    process.stdin.pipe(tap).pipe(process.stdout);
    return;
}

var files = argv._.reduce(function (acc, file) {
    if (fs.statSync(file).isDirectory()) {
        return acc.concat(fs.readdirSync(file).map(function (x) {
            return path.join(file, x);
        }).filter(jsFile));
    }
    else return acc.concat(file);
}, []);

if (files.length === 0 && fs.existsSync('test')) {
    files.push.apply(files, fs.readdirSync('test').map(function (x) {
        return path.join('test', x);
    }).filter(jsFile));
}
if (files.length === 0 && fs.existsSync('tests')) {
    files.push.apply(files, fs.readdirSync('tests').map(function (x) {
        return path.join('tests', x);
    }).filter(jsFile));
}

if (files.length === 0) {
    console.error('usage: `faucet [FILES]` or `| faucet`\n');
    console.error(
        'No test files or stdin provided and no files in test/ or tests/'
        + ' directories found.'
    );
    return process.exit(1);
}

var tape = spawn(tapeCmd, files);
tape.stderr.pipe(process.stderr);
tape.stdout.pipe(tap).pipe(process.stdout);

var tapeCode;
tape.on('exit', function (code) { tapeCode = code });
process.on('exit', function (code) {
    if (code === 0 && tapeCode !== 0) {
        console.error('# non-zero exit from the `tape` command');
        process.exit(tapeCode);
    }
});

function jsFile (x) { return /\.js$/i.test(x) }
