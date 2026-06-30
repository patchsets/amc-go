use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct TurnstilePayload {
    r#type: String,
    site_key: String,
    url: String,
    action: String,
}

#[derive(Deserialize)]
struct TurnstileResponse {
    success: bool,
    token: Option<String>,
    error: Option<String>,
}

pub async fn solve_turnstile() -> Result<String> {
    let payload = TurnstilePayload {
        r#type: "turnstile".into(),
        site_key: "0x4AAAAAAA9oPHboisPr8cag".into(),
        url: "https://graph.amctheatres.com/".into(),
        action: "login".into(),
    };

    let client = wreq::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let resp = client
        .post("https://api.nslsolver.com/solve")
        .header("X-API-Key", "nsl_d93dfc0eebe91ad945bec902f57e4e4ec85fe6af018352d4")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    let result: TurnstileResponse = resp.json().await?;

    if !result.success {
        bail!(
            "solver error: {}",
            result.error.unwrap_or_else(|| "unknown".into())
        );
    }

    result
        .token
        .ok_or_else(|| anyhow::anyhow!("solver returned success but no token"))
}
