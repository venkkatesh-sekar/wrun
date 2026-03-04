use candid::Principal;
use pocket_ic::PocketIc;

pub fn run_instance(wasm: Vec<u8>, method: String) {
    let pic = PocketIc::new();
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, u128::MAX / 2);
    pic.install_canister(canister_id, wasm, vec![], None);
    let _result = pic.update_call(canister_id, Principal::anonymous(), method.as_str(), vec![]);
}
