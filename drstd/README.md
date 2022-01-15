# Introduction

The drstd crate provides a Rust standard library suitable for writing DynamoRIO clients.
While it is possible to use the default standard library, it is not recommended as it ends up invoking system calls as part of the application rather than as part of the client.
See [DynamoRIO's documentation](https://dynamorio.org/transparency.html) for more information about how client transparency is achieved.

The drstd crate instead builds the client in a `no_std` environment, such that it can then use the API provided by DynamoRIO to implement drstd on top, while trying to maintain compatibility with std where possible.
This currently provides the following features:

 - [x] `print!`, `println!`, `eprint!`, `eprintln!` macros to print using `dr_printf` and `dr_fprintf`.
 - [x] Implementation of a default allocator implementing `GlobalAlloc` by using `dr_global_alloc` and `dr_global_free`.
 - [x] `Path` and `PathBuf` thanks to the [unix\_path](https://docs.rs/unix_path/latest/unix_path/) crate.
