use ic_config::embedders::Config as EmbeddersConfig;
use ic_embedders::wasmtime_embedder::system_api::ApiType;
use ic_test_utilities_embedders::WasmtimeInstanceBuilder;
use ic_test_utilities_types::ids::user_test_id;
use ic_types::methods::{FuncRef, WasmMethod};
use ic_types::{Cycles, messages::CallContextId, time::UNIX_EPOCH};

pub fn run_instance(wasm: Vec<u8>, method: String) {
    let config = EmbeddersConfig::default();
    let instance_result = WasmtimeInstanceBuilder::new()
        .with_wasm(wasm)
        .with_config(config)
        .with_api_type(ApiType::update(
            UNIX_EPOCH,
            vec![1_u8; 10],
            Cycles::new(10_000_000_000),
            user_test_id(24).get(),
            CallContextId::from(0),
        ))
        .try_build();

    let mut instance = match instance_result {
        Ok(instance) => instance,
        Err((_, _)) => {
            panic!("WasmtimeEmbedder: Failed to build instance");
        }
    };

    let func_ref = FuncRef::Method(WasmMethod::Update(method));
    let _result = instance.run(func_ref);
}
