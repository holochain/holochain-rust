import {
    debug,
    commit_entry,
    get_entry,
    serialize,
    deserialize,
    stringify
} from "./node_modules/hdk-assemblyscript"

/*

There are decorators available to simplify development.

You can delete the following examples, or modify them to get started.

The can_stringify decorator enables an object of a particular class to be converted to a string by calling .toString() on it.
It also enables debug and commit_entry to implicitly convert parameters of this type into strings.
*/

@can_stringify
class TestClass {
    key: string
    otherKey: i32
}

/*
Use the zome_function decorator to expose this function as a zome function.
It enables automatic serialization/deserialization of arguments and return values.
*/

@zome_function
function testfunction(param1: string): string {
    let myTest: TestClass = {
        key: "hello",
        otherKey: 23
    };
    debug(myTest);
    return "something";
}
