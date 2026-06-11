# DATIR: Dynamic Abstract Type Inference in Rust
This repository contains all the code necessary to compile arbitrary Rust source code, adding in instrumentation which captures information regarding the abstract types of formal and return values from each function. This is an implementation of abstract type inference (ATI), based on [dynamic inference of abstract types](https://dl.acm.org/doi/10.1145/1146238.1146268).

## Building and running from source
This instrumentation relies on `rustc`'s query system to execute callbacks. This requires linking against `rustc`'s nightly build. To do so, run the following bash [commands](https://rust-lang.github.io/rustup/installation/index.html):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none -y;

echo ". \"$HOME/.cargo/env\"" >> ~/.bashrc;

rustup toolchain install nightly --allow-downgrade --profile minimal --component clippy;

rustup component add rust-src rustc-dev llvm-tools-preview;
```

At this point, you should be able to compile and run this project with:
```sh
cargo +nightly run -- INPUT_MAIN_RS [DATIR-ARGS] [PASSTHROUGH-OPTIONAL-ARGS]
```

For more usage information, run `cargo +nightly run -- --help`.

If typing `+nightly` becomes tedious, feel free to run `rustup default nightly` to default to the nightly compiler build. After executing that command, you can simply omit the nightly flag. To switch back to the stable build as the default, use `rustup default stable`.

All rust toolchains assume a C-linker is available. If you run into a `linker 'cc' not found` error upon running a cargo command, install the tried and true `gcc` toolchain with `sudo apt install build-essential` (on Linux).

### Building a standalone binary
DATIR provides a `build.rs` and `rust-toolchain.toml` files which include the appropriate `rpath` entries to properly load dynamic dependencies.
A release-optimized binary can be built with:
```sh
cargo build -r
```

Which can then be executed with:
```sh
./target/release/datir
```

Note this the current setup does not bundle the necessary dylibs.
This means this output binary cannot be moved to an arbitrary location.

## Command Line Arguments
DATIR supports the following command line arguments, in the `[DATIR-ARGS]` position:
1. `-d <PATH> / --decls-path <PATH>`: if specified, loads the `.decls` file at `<PATH>` to perform instrumentation, rather than generating a brand new one. `<PATH>` must point to a `.decls` file which corresponds to the program being instrumented (defines the appropriate program points, with the correct set of variables at each).
2. `-rd <INT_DEPTH> / --rec-depth <INT_DEPTH>`: If `-d` is specified, the recursive depth to which variables are expanded during generation of the `.decls` file. If `-d` is unspecified, does nothing.
3. `-o <FILE> / --output <FILE>`: if specified, places the resulting instrumented binary at `<FILE>`. Otherwise, the output binary is placed in the current working directory.
4. `-r <ATI_OUTPUT_DIR> / --release <ATI_OUTPUT_DIR>`: specifies the directory within which output `.ati` files are placed. See the Output section below for more information. If this flag is specified, no debug output is produced in the `./logs/` directory.
5. `--test`: Enables test mode, which disables all debug output, but does not produce `.ati` files. This is used by all unit tests.
6. `-h / --help`: Prints usage information.

## File Description
The following files make up the majority of the implementation:

1. `src/ati/*`: Contains the ATI runtime library that is used at runtime to dynamically keep track of value interactions. All files within this directory are injected into the target crate.
2. `src/callbacks/*`: Defines the callbacks used by various compiler invocations. DATIR currently relies on being able to perform two compilations, one to generally gather some information (`src/callbacks/gather/`), another to perform the actual instrumentation (`src/callbacks/instrument/`). Following instrumentation, some extra code has to be generated and inserted into the crate. This is done by code contained within `src/callbacks/codegen`.
    - `src/callbacks/file_loader/*`: Defines a custom `rustc`-compatible `FileLoader` which is capable of performing AST-level mutations before the file contents even make it to the compiler parser. This allows instrumentation of all files, not just the crate root.
7. `tests/*`: Unit tests, which invoke the compiler on input files and checks the ATI output against an expected partition.

## Output
By default, using DATIR to compile a program results in a `main.decls` file being generated alongside the `main.rs` file.
This file declares the program points (and variables) at which an abstract type partition will be inferred.
If a Rust-Daikon Frontend program already generated this file, it can instead be loaded using the `-d` command line argument.
This will skip producing a duplicate `main.decls` file.

DATIR can produce abstract type partitions in two formats, based on what flags are used to invoke it. If `--release ATI_OUTPUT_DIR` is specified, then the produced target binary will write a file to the output directory every time it is invoked, in the `.ati` format that is compatible with the `decls-merger`.

If `--release` is unspecified, then executing the produced target binary will instead simple print the comparability report to stdout, in the following format:

```
===ATI-ANALYSIS-START===
tests/simple/main.rs::foo:::ENTER
x -> 1
y -> 1
z -> 2
---
tests/simple/main.rs::foo:::EXIT
return -> 0
x -> 0
y -> 0
z -> 2
---
tests/simple/main.rs::main:::ENTER
---
tests/simple/main.rs::main:::EXIT
---
```

If `--release ATI_OUTPUT_DIR` is specified, then executing the instrumented binary will produce a `ATI_OUTPUT_DIR/<RANDOM>.ati` file, in the following format:
```
ppt tests/simple/main.rs::foo:::ENTER
var x 1
var y 1
var z 2

ppt tests/simple/main.rs::foo:::EXIT
var return 0
var x 0
var y 0
var z 2

ppt tests/simple/main.rs::main:::ENTER

ppt tests/simple/main.rs::main:::EXIT

```

`.ati` files can be merged with the corresponding `.decls` files using the accompanying, standalone [DECLS-MERGER](https://github.com/olegian/datir-decls-merger) tool.
