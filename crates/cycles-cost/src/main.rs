use ic_config::subnet_config::CyclesAccountManagerConfig;
use ic_cycles_account_manager::CyclesAccountManager;
use ic_registry_subnet_type::SubnetType;
use ic_replicated_state::canister_state::execution_state::WasmExecutionMode;
use ic_types::{
    batch::CanisterCyclesCostSchedule::Normal, ComputeAllocation, Cycles, NumBytes,
    NumInstructions, PrincipalId, SubnetId,
};
use std::time::Duration;

const SUBNET_SIZES: &[usize] = &[7, 13, 34];

// Fallback values when APIs are unreachable.
const FALLBACK_XDR_USD: f64 = 1.363390;
const FALLBACK_ICP_USD: f64 = 2.50;

fn main() {
    let config = CyclesAccountManagerConfig::application_subnet();
    let cam = CyclesAccountManager::new(
        NumInstructions::new(5_000_000_000),
        SubnetType::Application,
        SubnetId::from(PrincipalId::new_anonymous()),
        config,
    );

    let (xdr_usd, xdr_src) = fetch_xdr_usd()
        .map(|r| (r, "Yahoo Finance"))
        .unwrap_or_else(|| {
            eprintln!("Warning: Could not fetch XDR/USD rate, using fallback");
            (FALLBACK_XDR_USD, "fallback")
        });
    let (icp_usd, icp_src) = fetch_icp_usd()
        .map(|p| (p, "CoinGecko"))
        .unwrap_or_else(|| {
            eprintln!("Warning: Could not fetch ICP/USD price, using fallback");
            (FALLBACK_ICP_USD, "fallback")
        });

    let rows = compute_rows(&cam);

    print_table("Cycles Price Breakdown", &rows, fmt_cycles);

    println!();
    println!(
        "1T cycles = 1 XDR = ${:.6} ({}) | 1 ICP = ${:.2} ({})",
        xdr_usd, xdr_src, icp_usd, icp_src
    );

    println!();
    print_table("USD Cost", &rows, |v| fmt_usd(v as f64 / 1e12 * xdr_usd));

    println!();
    print_table("ICP Cost", &rows, |v| {
        fmt_icp(v as f64 / 1e12 * xdr_usd / icp_usd)
    });
}

// ---------------------------------------------------------------------------
// Row computation
// ---------------------------------------------------------------------------

enum Row {
    Data {
        name: &'static str,
        values: Vec<u128>,
    },
    Separator,
}

fn compute_rows(cam: &CyclesAccountManager) -> Vec<Row> {
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
                        - cam.execution_cost(NumInstructions::new(0), n, Normal, w32).get(),
                )
            }),
        )),
        Some((
            "1B executed instructions (wasm64)",
            Box::new(move |cam, n| {
                Cycles::new(
                    cam.execution_cost(NumInstructions::new(1_000_000_000), n, Normal, w64)
                        .get()
                        - cam.execution_cost(NumInstructions::new(0), n, Normal, w64).get(),
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
                        - cam.http_request_fee(
                            NumBytes::new(0),
                            Some(NumBytes::new(0)),
                            n,
                            Normal,
                        )
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
                        - cam.http_request_fee(
                            NumBytes::new(0),
                            Some(NumBytes::new(0)),
                            n,
                            Normal,
                        )
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
                let values = SUBNET_SIZES.iter().map(|&n| f(cam, n).get()).collect();
                Row::Data { name, values }
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Table printing
// ---------------------------------------------------------------------------

fn print_table(title: &str, rows: &[Row], fmt_val: impl Fn(u128) -> String) {
    let col_width = 22;
    let name_width = 40;
    let total_width = name_width + (1 + col_width) * SUBNET_SIZES.len();

    println!("{title}");
    println!("{}", "=".repeat(total_width));
    println!();
    print!("{:<name_width$}", "Transaction");
    for &n in SUBNET_SIZES {
        print!(" {:>col_width$}", format!("{n}-node app subnet"));
    }
    println!();
    println!("{}", "-".repeat(total_width));

    for entry in rows {
        match entry {
            Row::Separator => println!(),
            Row::Data { name, values } => {
                print!("{:<name_width$}", name);
                for v in values {
                    print!(" {:>col_width$}", fmt_val(*v));
                }
                println!();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Live price fetching
// ---------------------------------------------------------------------------

fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
        .build()
}

/// Fetch the latest XDR/USD rate from Yahoo Finance (ticker XDRUSD=X).
fn fetch_xdr_usd() -> Option<f64> {
    let body = http_agent()
        .get("https://query1.finance.yahoo.com/v8/finance/chart/XDRUSD=X?range=1d&interval=1d")
        .set("User-Agent", "Mozilla/5.0")
        .call()
        .ok()?
        .into_string()
        .ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    json["chart"]["result"][0]["meta"]["regularMarketPrice"].as_f64()
}

/// Fetch the latest ICP/USD price from CoinGecko.
fn fetch_icp_usd() -> Option<f64> {
    let body = http_agent()
        .get("https://api.coingecko.com/api/v3/simple/price?ids=internet-computer&vs_currencies=usd")
        .call()
        .ok()?
        .into_string()
        .ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    json["internet-computer"]["usd"].as_f64()
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

fn fmt_cycles(v: u128) -> String {
    let s = v.to_string();
    let mut result = String::new();
    let len = s.len();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push('_');
        }
        result.push(ch);
    }
    result
}

fn fmt_usd(usd: f64) -> String {
    if usd >= 1.0 {
        format!("${:.4}", usd)
    } else if usd >= 0.01 {
        format!("${:.6}", usd)
    } else if usd >= 0.0001 {
        format!("${:.8}", usd)
    } else {
        format!("${:.12}", usd)
    }
}

fn fmt_icp(icp: f64) -> String {
    if icp >= 1.0 {
        format!("{:.4} ICP", icp)
    } else if icp >= 0.01 {
        format!("{:.6} ICP", icp)
    } else if icp >= 0.0001 {
        format!("{:.8} ICP", icp)
    } else {
        format!("{:.12} ICP", icp)
    }
}
