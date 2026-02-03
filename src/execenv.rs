use ic_test_utilities_execution_environment::ExecutionTestBuilder;
use ic_types::Cycles;

pub fn run_instance(wasm: Vec<u8>, method: String) {
    let mut test = ExecutionTestBuilder::new().build();
    let canister_id = test.create_canister(Cycles::new(1_000_000_000_000));
    test.install_canister(canister_id, wasm)
        .expect("ExecEnv: failed to install canister");
    let _result = test.ingress(canister_id, method, vec![]);
}
