use wasmtime::*;

pub fn run_instance(wasm: Vec<u8>, method: String) {
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).expect("Wasmtime: failed to create module");

    // No System API functionality for now.
    let linker = Linker::new(&engine);
    let mut store: Store<u32> = Store::new(&engine, 4);
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Wasmtime: failed to instantiate module");
    let function = format!("canister_update {}", method);
    let _result = instance
        .get_typed_func::<(), ()>(&mut store, function.as_str())
        .expect("Wasmtime: failed to get function: {function}")
        .call(&mut store, ());
}
