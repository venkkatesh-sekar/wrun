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

## Cycles Cost Tool

The `cycles-cost` binary prints the IC cycles price breakdown table, matching the [official docs](https://docs.internetcomputer.org/references/cycles-cost-formulas#cycles-price-breakdown). It uses `CyclesAccountManager` fee methods directly to compute costs for different subnet sizes, with both wasm32 and wasm64 execution modes.

It also fetches live exchange rates to show equivalent USD and ICP costs:
- **XDR/USD** from [Yahoo Finance](https://finance.yahoo.com/quote/XDRUSD=X/) (1 trillion cycles = 1 XDR)
- **ICP/USD** from [CoinGecko](https://www.coingecko.com/en/coins/internet-computer)

Both fall back to built-in constants if the APIs are unreachable.

```sh
cargo run -p cycles-cost
```

Subnet sizes are configured via the `SUBNET_SIZES` constant in `crates/cycles-cost/src/main.rs`.

**Sample output:**
```
Cycles Price Breakdown
=============================================================================================================

Transaction                                   7-node app subnet     13-node app subnet     34-node app subnet
-------------------------------------------------------------------------------------------------------------
Canister creation                               269_230_769_230        500_000_000_000      1_307_692_307_692
Compute 1% allocated per second                       5_384_615             10_000_000             26_153_846
Update message execution (wasm32)                     2_692_307              5_000_000             13_076_923
Update message execution (wasm64)                     2_692_307              5_000_000             13_076_923
1B executed instructions (wasm32)                   538_461_539          1_000_000_000          2_615_384_615
1B executed instructions (wasm64)                 1_076_923_077          2_000_000_000          5_230_769_230
Xnet call                                               140_000                260_000                680_000
Xnet byte transmission                                      538                  1_000                  2_615
Ingress message reception                               646_153              1_200_000              3_138_461
Ingress byte reception                                    1_076                  2_000                  5_230
GiB storage per second                                   68_384                127_000                332_153

HTTPS outcall (per call)                             23_940_000             49_140_000            171_360_000
HTTPS outcall request (per byte)                          2_800                  5_200                 13_600
HTTPS outcall response (per byte)                         5_600                 10_400                 27_200

tECDSA signing (secp256k1)                        5_384_615_384         10_000_000_000         26_153_846_153
tSchnorr signing (bip340secp256k1)                5_384_615_384         10_000_000_000         26_153_846_153
vetKD key derivation (bls12_381_g2)               5_384_615_384         10_000_000_000         26_153_846_153

1T cycles = 1 XDR = $1.363400 (Yahoo Finance) | 1 ICP = $2.55 (CoinGecko)

USD Cost
=============================================================================================================

Transaction                                   7-node app subnet     13-node app subnet     34-node app subnet
-------------------------------------------------------------------------------------------------------------
Canister creation                                     $0.367069              $0.681700                $1.7829
Compute 1% allocated per second                 $0.000007341384        $0.000013634000        $0.000035658154
Update message execution (wasm32)               $0.000003670691        $0.000006817000        $0.000017829077
Update message execution (wasm64)               $0.000003670691        $0.000006817000        $0.000017829077
1B executed instructions (wasm32)                   $0.00073414            $0.00136340            $0.00356582
1B executed instructions (wasm64)                   $0.00146828            $0.00272680            $0.00713163
Xnet call                                       $0.000000190876        $0.000000354484        $0.000000927112
Xnet byte transmission                          $0.000000000734        $0.000000001363        $0.000000003565
Ingress message reception                       $0.000000880965        $0.000001636080        $0.000004278978
Ingress byte reception                          $0.000000001467        $0.000000002727        $0.000000007131
GiB storage per second                          $0.000000093235        $0.000000173152        $0.000000452857
...

ICP Cost
=============================================================================================================

Transaction                                   7-node app subnet     13-node app subnet     34-node app subnet
-------------------------------------------------------------------------------------------------------------
Canister creation                                  0.143949 ICP           0.267333 ICP           0.699179 ICP
Compute 1% allocated per second              0.000002878974 ICP     0.000005346667 ICP     0.000013983590 ICP
...
tECDSA signing (secp256k1)                       0.00287897 ICP         0.00534667 ICP           0.013984 ICP
tSchnorr signing (bip340secp256k1)               0.00287897 ICP         0.00534667 ICP           0.013984 ICP
vetKD key derivation (bls12_381_g2)              0.00287897 ICP         0.00534667 ICP           0.013984 ICP
```
