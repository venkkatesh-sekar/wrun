(module
  (func $noop)
  ;; The wasmtime runner prefixes the method name with "canister_update "
  (export "canister_update noop" (func $noop))
)