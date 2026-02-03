use candid::{CandidType, Decode, Encode, Nat};
use futures::StreamExt;
use futures::stream::FuturesOrdered;
use ic_agent::export::reqwest::Url;
use ic_agent::{Agent, export::Principal, identity::AnonymousIdentity};
use ic_management_canister_types::{
    CanisterInstallMode, ChunkHash, InstallChunkedCodeArgs, UploadChunkArgs,
};
use serde::Deserialize;
use sha2::Sha256;
use sha2::digest::Digest;
use tokio::runtime::Runtime;

#[derive(CandidType)]
struct Argument {
    amount: Option<Nat>,
}

#[derive(CandidType, Deserialize)]
struct CreateCanisterResult {
    canister_id: Principal,
}

pub fn run_instance(url: String, use_mainnet: bool, wasm: Vec<u8>, method: String) {
    let rt: Runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(3)
        .max_blocking_threads(1)
        .thread_stack_size(8 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap_or_else(|err| panic!("Testnet: Could not create tokio runtime: {}", err));

    rt.block_on(async {
        let url = Url::parse(&url).unwrap();
        let agent = create_agent(url, use_mainnet).await;

        let management_canister_id = Principal::from_text("aaaaa-aa").unwrap();
        let effective_canister_id = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();

        let response = agent
            .update(
                &management_canister_id,
                "provisional_create_canister_with_cycles",
            )
            .with_effective_canister_id(effective_canister_id)
            .with_arg(
                Encode!(&Argument {
                    amount: Some(Nat::from(u64::MAX / 2))
                })
                .unwrap(),
            )
            .call_and_wait()
            .await
            .map(|result| {
                let r = Decode!(result.as_slice(), CreateCanisterResult).unwrap();
                r.canister_id
            });

        assert!(response.is_ok());
        let target_canister_id = response.unwrap();
        println!("canister id {}", target_canister_id);

        let chunks = wasm.chunks(900_000);
        let mut hash_fut = FuturesOrdered::new();

        for chunk in chunks {
            let result = agent
                .update(&management_canister_id, "upload_chunk")
                .with_effective_canister_id(effective_canister_id)
                .with_arg(
                    Encode!(&UploadChunkArgs {
                        canister_id: target_canister_id,
                        chunk: chunk.to_vec()
                    })
                    .unwrap(),
                )
                .call_and_wait();
            hash_fut.push_back(result);
        }

        let mut hashes: Vec<ChunkHash> = vec![];
        while let Some(result) = hash_fut.next().await {
            let r = Decode!(result.as_ref().unwrap().as_slice(), ChunkHash).unwrap();
            hashes.push(r);
        }

        let mut hasher = Sha256::new();
        hasher.update(wasm);
        let full_hash = hasher.finalize();

        let _result = agent
            .update(&management_canister_id, "install_chunked_code")
            .with_effective_canister_id(effective_canister_id)
            .with_arg(
                Encode!(&InstallChunkedCodeArgs {
                    mode: CanisterInstallMode::Reinstall,
                    target_canister: target_canister_id,
                    store_canister: Some(target_canister_id),
                    chunk_hashes_list: hashes.clone(),
                    wasm_module_hash: full_hash.to_vec(),
                    arg: vec![],
                    sender_canister_version: None,
                })
                .unwrap(),
            )
            .call_and_wait()
            .await
            .expect("Testnet: Failed to install canister");

        let _result = agent
            .update(&target_canister_id, method)
            .with_effective_canister_id(target_canister_id)
            .with_arg(Encode!(&()).unwrap())
            .call_and_wait()
            .await;
    })
}

pub async fn create_agent(url: Url, use_mainnet: bool) -> Agent {
    let agent = Agent::builder()
        .with_url(url)
        .with_identity(AnonymousIdentity)
        .build()
        .unwrap();

    if !use_mainnet {
        agent.fetch_root_key().await.unwrap();
    }
    agent
}
