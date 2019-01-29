# Intro to WebAssembly

What is WebAssembly exactly?

> "WebAssembly is a standard being developed by the W3C group for an efficient, lightweight instruction set. This means we can compile different types of programming languages ranging from C/C++, Go, Rust, and more into a single standard... WebAssembly, or WASM, for short, is memory-safe,platform independent, and maps well to all types of CPU architectures efficiently." - [source](https://medium.com/zkcapital/webassembly-the-future-of-blockchain-computing-1a0ae28f7e40)

Though initially designed for use by major browsers IE, Chrome, Firefox and Safari, WASM has quickly been taken up as a portable target for execution on native platforms as well.

 [WebAssembly.org](https://webassembly.org) describes it as a binary instruction format for a stack-based virtual machine.

 Despite being a binary format, "WebAssembly is designed to be pretty-printed in a textual format for debugging, testing, experimenting, optimizing, learning, teaching, and writing programs by hand."

 This textual format is called WAT.

Not because it needs to be understood, but so that you can get a glimpse of what WAT looks like, here's a little sample:

```
(module
    (memory (;0;) 17)
    (func (export "public_test_fn") (param $p0 i64) (result i64)
        i64.const 6
    )
    (data (i32.const 0)
        "1337.0"
    )
    (export "memory" (memory 0))
)
```

Once the above code is converted from WAT to binary WASM it is in the format that could be executed by the Holochain WASM interpreter.

Often times, for a language that compiles to WASM, you will have a configuration option to generate the (more) human readable WAT version of the code as well, while compiling it to WASM.

While the compilation to WASM mostly happens in the background for you as an app developer, having a basic understanding of the role of WebAssembly in this technology stack will no doubt help you along the way.
