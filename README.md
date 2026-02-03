# Wasm Runner (`wrun`)

`wrun` is a command-line utility for executing a method within a WebAssembly (Wasm) module using various Internet Computer (IC) execution environments. It can compile a Wasm Text Format (`.wat`) file into a binary Wasm module on the fly and run it in one of the supported instance types, ranging from a pure `wasmtime` environment to a live IC testnet deployment.

## Features

- Compile `.wat` to Wasm automatically.
- Execute Wasm modules in 6 different environments.
- Simple command-line interface.

## Prerequisites

Before you can build and run `wrun`, you need to have the Rust toolchain installed. You can get it from rustup.rs.

## Building

The project depends on several crates from the DFINITY `ic` repository. Cargo will fetch and build them automatically.

1.  Clone the repository (if you haven't already).
2.  Build the project:

    ```sh
    cargo build --release
    ```

3.  The executable will be available at `target/release/wrun`.

## Usage

The tool requires three arguments: the path to the `.wat` file, the name of the method to execute, and the instance type to use.

```
wrun <WAT_FILE> <METHOD> --instance-type <INSTANCE_TYPE>
```

### Arguments

-   `<WAT_FILE>`: Path to the Wasm Text Format file.
-   `<METHOD>`: The name of the update method to call within the Wasm module.

### Options

-   `--url <URL>`: The testnet URL to use (only for the `testnet` instance type).
-   `-i, --instance-type <INSTANCE_TYPE>`: The execution environment to use.
-   `-h, --help`: Print help information.
-   `-V, --version`: Print version information.

## Instance Types

`wrun` supports the following instance types, each providing a different execution context:

-   `wasmtime`: Runs the Wasm using the pure `wasmtime` engine. This environment does **not** provide any IC System API imports. Wasm modules with IC-specific imports will fail.
-   `embedder`: Uses the IC's `wasmtime-embedder`, which provides a mocked IC System API. This is suitable for testing Wasm that interacts with system-level features like time, cycles, and call contexts.
-   `execenv`: Uses the IC's `execution-environment`, a higher-level simulation that manages canister state and inter-canister calls locally.
-   `pocket-ic`: Runs the Wasm on a local PocketIC server, a powerful tool for testing canister smart contracts in a deterministic, multi-canister environment.
-   `testnet`: Deploys the Wasm as a new canister to a specified IC testnet, installs the code, and calls the method. The testnet URL can be provided via the `--url` flag, otherwise a default is used.

## Examples

Here are a couple of example `.wat` files and how to run them.

### Example 1: Simple Wasm for `wasmtime`

Since the `wasmtime` runner does not provide IC System API imports, we need a self-contained module.

**`noop.wat`**
```wat
(module
  (func $noop)
  ;; The wasmtime runner prefixes the method name with "canister_update "
  (export "canister_update noop" (func $noop))
)
```

**To run:**
```sh
./target/release/wrun noop.wat noop --instance-type wasmtime
```

### Example 2: Counter Canister for IC Environments

This example uses IC System API functions to maintain a counter in memory. It will work with `embedder`, `execenv`, `pocket-ic`, and `testnet`.

**`counter.wat`**
```wat
(module
  (import "ic0" "msg_reply" (func $msg_reply))
  (import "ic0" "msg_reply_data_append" (func $msg_reply_data_append (param i32 i32)))

  (memory (export "memory") 1)
  (data (i32.const 0) "\00\00\00\00") ;; A 32-bit counter at memory address 0

  (func $read (export "read")
    ;; Reply with the 4 bytes of the counter
    (call $msg_reply_data_append
      (i32.const 0) ;; src: memory address 0
      (i32.const 4) ;; size: 4 bytes
    )
    (call $msg_reply)
  )

  (func $inc (export "inc")
    ;; Load current value from memory
    (i32.load (i32.const 0))
    ;; Increment it
    (i32.const 1)
    (i32.add)
    ;; Store the new value back to memory
    (i32.store (i32.const 0))
    ;; Reply
    (call $msg_reply)
  )

  ;; Also export with the wasmtime prefix for compatibility, though it won't work there.
  (export "canister_update inc" (func $inc))
  (export "canister_update read" (func $read))
)
```

**To run with PocketIC:**
```sh
./target/release/wrun counter.wat inc --instance-type pocket-ic
```

**To run on a testnet:**
```sh
./target/release/wrun counter.wat inc --instance-type testnet
```
