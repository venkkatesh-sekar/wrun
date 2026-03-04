mod costs;
mod format;
mod prices;

const SUBNET_SIZES: &[usize] = &[7, 13, 34];

// Fallback values when APIs are unreachable.
const FALLBACK_XDR_USD: f64 = 1.363390;
const FALLBACK_ICP_USD: f64 = 2.50;

fn main() {
    let cam = costs::new_cycles_account_manager();

    let (xdr_usd, xdr_src) = prices::fetch_xdr_usd()
        .map(|r| (r, "Yahoo Finance"))
        .unwrap_or_else(|| {
            eprintln!("Warning: Could not fetch XDR/USD rate, using fallback");
            (FALLBACK_XDR_USD, "fallback")
        });
    let (icp_usd, icp_src) = prices::fetch_icp_usd()
        .map(|p| (p, "CoinGecko"))
        .unwrap_or_else(|| {
            eprintln!("Warning: Could not fetch ICP/USD price, using fallback");
            (FALLBACK_ICP_USD, "fallback")
        });

    let rows = costs::compute_rows(&cam, SUBNET_SIZES);

    format::print_table(
        "Cycles Price Breakdown",
        &rows,
        SUBNET_SIZES,
        format::fmt_cycles,
    );

    println!();
    println!(
        "1T cycles = 1 XDR = ${:.6} ({}) | 1 ICP = ${:.2} ({})",
        xdr_usd, xdr_src, icp_usd, icp_src
    );

    println!();
    format::print_table("USD Cost", &rows, SUBNET_SIZES, |v| {
        format::fmt_usd(v as f64 / 1e12 * xdr_usd)
    });

    println!();
    format::print_table("ICP Cost", &rows, SUBNET_SIZES, |v| {
        format::fmt_icp(v as f64 / 1e12 * xdr_usd / icp_usd)
    });
}
