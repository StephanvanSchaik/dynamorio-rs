# Introduction

The dynamorio-rs crate provides safe Rust bindings to the DynamoRIO dynamic binary instrumentation framework, essentially allowing you to write DynamoRIO clients in Rust.

# Getting Started

Examples can be found in the [examples](./examples) directory.
Check out this repository:

```
git clone https://github.com/StephanvanSchaik/dynamorio-rs
```

Then go to the directory containing the empty client:

```
cd dynamorio-rs/examples/empty
```

Finally, build it as follows:

```
cargo build
```

This should produce `target/debug/libempty.so`.

Assuming you have dynamorio installed system-wide, you can then run the client as follows:

```
drrun -c target/debug/libempty.so -- ls
```

It is also possible to build DynamoRIO out-of-tree.
See [building from source](https://dynamorio.org/page_building.html) for instructions on how to build DynamoRIO from source.
Assuming you checked out DynamoRIO's repository to `~/dynamorio` and built it into `~/dynamorio/build`, you can also run the following command to build the client:

```
DRIO_BUILD_DIR=~/dynamorio/build cargo build
```

You can then run the client as follows:

```
~/dynamorio/build/bin64/drrun -c target/debug/libempty.so --ls
```
