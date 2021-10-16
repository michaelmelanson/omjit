# OMjit

This is a prototype of a JIT-compiled Javascript engine.

## Running

You'll need a working Rust environment, including Cargo. Once you have that, you can run Javascript programs with:

```
cargo run -- samples/add.js
```

This simple sample program calculates 3 + 2, and will print the result of `5`.

## Status

This is VERY rudimentary work. If you run anything beyond the sample program, you will run into "not yet implemented" crashes.

## Debugging

OMjit supports two useful command line flags:

* `-s`: Print the flow graph before executing program. This is useful to understand the compilation stage, where the flow graph and intermediate representation is generated.
* `-d`: Print generated code during execution. This is useful to understand what's going on in the code generation stage, or to debug crashes in the generated code.