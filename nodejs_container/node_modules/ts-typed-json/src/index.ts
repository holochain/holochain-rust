import Dict from 'ts-dict';
import * as fs from 'fs';
import RSVP = require('rsvp');

export { Object_ as Object, Array_ as Array };

/** JSON data, as returned by `JSON.parse()`. */
export type Value = null | boolean | number | string | Object_ | Array_;

/** JSON object values. */
interface Object_ extends Dict<Value> {}

/** JSON array values. */
interface Array_ extends Array<Value> {}

/** Tests a JSON value to see if it is `null`. */
export function isNull(x: Value): x is null {
    return x === null;
}

/** Cast a JSON value to `null`, throwing a `TypeError` if the cast fails. */
export function asNull(x: Value): null {
    if (x !== null) {
        throw new TypeError("not null");
    }
    return null;
}

/** Tests a JSON value to see if it is a boolean. */
export function isBoolean(x: Value): x is boolean {
    return typeof x === 'boolean';
}

/** Cast a JSON value to boolean, throwing a `TypeError` if the cast fails. */
export function asBoolean(x: Value): boolean {
    if (typeof x !== 'boolean') {
        throw new TypeError("not boolean");
    }
    return x;
}

/** Tests a JSON value to see if it is a number. */
export function isNumber(x: Value): x is number {
    return typeof x === 'number';
}

/** Cast a JSON value to number, throwing a `TypeError` if the cast fails. */
export function asNumber(x: Value): number {
    if (typeof x !== 'number') {
        throw new TypeError("not number");
    }
    return x;
}

/** Tests a JSON value to see if it is a string. */
export function isString(x: Value): x is string {
    return typeof x === 'string';
}

/** Cast a JSON value to string, throwing a `TypeError` if the cast fails. */
export function asString(x: Value): string {
    if (typeof x !== 'string') {
        throw new TypeError("not string");
    }
    return x;
}

/** Tests a JSON value to see if it is a JSON object. */
export function isObject(x: Value): x is Object_ {
    return !!x && typeof x === 'object' && !Array.isArray(x);
}

/** Cast a JSON value to `Object`, throwing a `TypeError` if the cast fails. */
export function asObject(x: Value): Object_ {
    if (!isObject(x)) {
        throw new TypeError("not a JSON object");
    }
    return x;
}

/** Tests a JSON value to see if it is a JSON array. */
export function isArray(x: Value): x is Array_ {
    return Array.isArray(x);
}

/** Cast a JSON value to `Array`, throwing a `TypeError` if the cast fails. */
export function asArray(x: Value): Array_ {
    if (!isArray(x)) {
        throw new TypeError("not a JSON array");
    }
    return x;
}

/** A more safely typed version of `JSON.parse()`. */
export function parse(source: string): Value {
    return JSON.parse(source);
}

/** A more safely typed version of `JSON.stringify()`. */
export function stringify(value: Value): string {
    return JSON.stringify(value);
}

/** Synchronously reads a text file and parses it as JSON. */
export function loadSync(path: string, encoding: string = 'utf8'): Value {
    return parse(fs.readFileSync(path, encoding));
}

const readFile = RSVP.denodeify<string>(fs.readFile);

export async function load(path: string, encoding: string = 'utf8'): Promise<Value> {
    let source = await readFile(path, { encoding: encoding });
    return parse(source);
}
