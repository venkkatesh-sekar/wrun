use std::time::Duration;

fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
        .build()
}

/// Fetch the latest XDR/USD rate from Yahoo Finance (ticker XDRUSD=X).
pub fn fetch_xdr_usd() -> Option<f64> {
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
pub fn fetch_icp_usd() -> Option<f64> {
    let body = http_agent()
        .get(
            "https://api.coingecko.com/api/v3/simple/price?ids=internet-computer&vs_currencies=usd",
        )
        .call()
        .ok()?
        .into_string()
        .ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    json["internet-computer"]["usd"].as_f64()
}
