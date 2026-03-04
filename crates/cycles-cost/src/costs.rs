use ic_config::subnet_config::CyclesAccountManagerConfig;
use ic_cycles_account_manager::CyclesAccountManager;
use ic_registry_subnet_type::SubnetType;
use ic_replicated_state::canister_state::execution_state::WasmExecutionMode;
use ic_types::{
    ComputeAllocation, Cycles, NumBytes, NumInstructions, PrincipalId, SubnetId,
    batch::CanisterCyclesCostSchedule::Normal,
};
use std::time::Duration;

pub enum Row {
    Data {
        name: &'static str,
        values: Vec<u128>,
    },
    Separator,
}

pub fn new_cycles_account_manager() -> CyclesAccountManager {
    let config = CyclesAccountManagerConfig::application_subnet();
    CyclesAccountManager::new(
        NumInstructions::new(5_000_000_000),
        SubnetType::Application,
        SubnetId::from(PrincipalId::new_anonymous()),
        config,
    )
}

pub fn compute_rows(cam: &CyclesAccountManager, subnet_sizes: &[usize]) -> Vec<Row> {
    let w32 = WasmExecutionMode::Wasm32;
    let w64 = WasmExecutionMode::Wasm64;
    let alloc1 = ComputeAllocation::try_from(1).unwrap();

    type F = Box<dyn Fn(&CyclesAccountManager, usize) -> Cycles>;

    let defs: Vec<Option<(&str, F)>> = vec![
        Some((
            "Canister creation",
            Box::new(move |cam, n| cam.canister_creation_fee(n, Normal)),
        )),
        Some((
            "Compute 1% allocated per second",
            Box::new(move |cam, n| {
                cam.compute_allocation_cost(alloc1, Duration::from_secs(1), n, Normal)
            }),
        )),
        Some((
            "Update message execution (wasm32)",
            Box::new(move |cam, n| cam.execution_cost(NumInstructions::new(0), n, Normal, w32)),
        )),
        Some((
            "Update message execution (wasm64)",
            Box::new(move |cam, n| cam.execution_cost(NumInstructions::new(0), n, Normal, w64)),
        )),
        Some((
            "1B executed instructions (wasm32)",
            Box::new(move |cam, n| {
                Cycles::new(
                    cam.execution_cost(NumInstructions::new(1_000_000_000), n, Normal, w32)
                        .get()
                        - cam
                            .execution_cost(NumInstructions::new(0), n, Normal, w32)
                            .get(),
                )
            }),
        )),
        Some((
            "1B executed instructions (wasm64)",
            Box::new(move |cam, n| {
                Cycles::new(
                    cam.execution_cost(NumInstructions::new(1_000_000_000), n, Normal, w64)
                        .get()
                        - cam
                            .execution_cost(NumInstructions::new(0), n, Normal, w64)
                            .get(),
                )
            }),
        )),
        Some((
            "Xnet call",
            Box::new(move |cam, n| cam.xnet_call_performed_fee(n, Normal)),
        )),
        Some((
            "Xnet byte transmission",
            Box::new(move |cam, n| {
                cam.xnet_call_bytes_transmitted_fee(NumBytes::new(1), n, Normal)
            }),
        )),
        Some((
            "Ingress message reception",
            Box::new(move |cam, n| cam.ingress_message_received_fee(n, Normal)),
        )),
        Some((
            "Ingress byte reception",
            Box::new(move |cam, n| cam.ingress_byte_received_fee(n, Normal)),
        )),
        Some((
            "GiB storage per second",
            Box::new(move |cam, n| cam.gib_storage_per_second_fee(n, Normal)),
        )),
        None,
        Some((
            "HTTPS outcall (per call)",
            Box::new(move |cam, n| {
                cam.http_request_fee(NumBytes::new(0), Some(NumBytes::new(0)), n, Normal)
            }),
        )),
        Some((
            "HTTPS outcall request (per byte)",
            Box::new(move |cam, n| {
                Cycles::new(
                    cam.http_request_fee(NumBytes::new(1), Some(NumBytes::new(0)), n, Normal)
                        .get()
                        - cam
                            .http_request_fee(NumBytes::new(0), Some(NumBytes::new(0)), n, Normal)
                            .get(),
                )
            }),
        )),
        Some((
            "HTTPS outcall response (per byte)",
            Box::new(move |cam, n| {
                Cycles::new(
                    cam.http_request_fee(NumBytes::new(0), Some(NumBytes::new(1)), n, Normal)
                        .get()
                        - cam
                            .http_request_fee(NumBytes::new(0), Some(NumBytes::new(0)), n, Normal)
                            .get(),
                )
            }),
        )),
        None,
        Some((
            "tECDSA signing (secp256k1)",
            Box::new(move |cam, n| cam.ecdsa_signature_fee(n, Normal)),
        )),
        Some((
            "tSchnorr signing (bip340secp256k1)",
            Box::new(move |cam, n| cam.schnorr_signature_fee(n, Normal)),
        )),
        Some((
            "vetKD key derivation (bls12_381_g2)",
            Box::new(move |cam, n| cam.vetkd_fee(n, Normal)),
        )),
    ];

    defs.into_iter()
        .map(|entry| match entry {
            None => Row::Separator,
            Some((name, f)) => {
                let values = subnet_sizes.iter().map(|&n| f(cam, n).get()).collect();
                Row::Data { name, values }
            }
        })
        .collect()
}
