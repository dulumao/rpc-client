use solana_client::client_error::{ClientError, ClientErrorKind, Result as ClientResult};
use solana_sdk::instruction::Instruction;
use solana_transaction_status::UiTransactionEncoding;

pub async fn get_recent_priority_fee_estimate(
    http_client: &reqwest::Client,
    rpc_url: String,
    ixs: &[Instruction],
) -> ClientResult<u64> {
    let account_keys: Vec<Vec<String>> = ixs.iter().map(account_keys).collect();
    let account_keys = account_keys.concat();
    let body = GetPriorityFeeEstimateRequest {
        transaction: None,
        account_keys: Some(account_keys),
        options: Some(GetPriorityFeeEstimateOptions {
            priority_level: Some(PriorityLevel::Medium),
            include_all_priority_fee_levels: Some(false),
            transaction_encoding: Some(UiTransactionEncoding::Base64),
            lookback_slots: None,
            recommended: Some(true),
            include_vote: Some(true),
        }),
    };
    let json = serde_json::to_value(body)?;
    log::info!("json: {:?}", json);
    let res = http_client
        .post(rpc_url)
        .json(&json)
        .send()
        .await?
        .json::<GetPriorityFeeEstimateResponse>()
        .await?;
    log::info!("response: {:?}", res);
    match res.priority_fee_estimate {
        Some(f64) => {
            log::info!("priority fee: {}", f64);
            Ok(f64 as u64)
        }
        None => Err(ClientError {
            kind: ClientErrorKind::Custom("EmptyPriorityFeeError".to_string()),
            request: None,
        }),
    }
}

fn account_keys(ix: &Instruction) -> Vec<String> {
    ix.accounts.iter().map(|a| a.pubkey.to_string()).collect()
}

#[derive(serde::Serialize)]
struct GetPriorityFeeEstimateRequest {
    transaction: Option<String>,       // estimate fee for a serialized txn
    account_keys: Option<Vec<String>>, // estimate fee for a list of accounts
    options: Option<GetPriorityFeeEstimateOptions>,
}

#[derive(serde::Serialize)]
struct GetPriorityFeeEstimateOptions {
    priority_level: Option<PriorityLevel>, // Default to MEDIUM
    include_all_priority_fee_levels: Option<bool>, // Include all priority level estimates in the response
    transaction_encoding: Option<UiTransactionEncoding>, // Default Base58
    lookback_slots: Option<u8>, // The number of slots to look back to calculate the estimate. Valid numbers are 1-150, default is 150
    recommended: Option<bool>,  // The Helius recommended fee for landing transactions
    include_vote: Option<bool>, // Include vote transactions in the priority fee estimate calculation. Default to true
}

#[derive(serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum PriorityLevel {
    _Min,      // 0th percentile
    _Low,      // 25th percentile
    Medium,    // 50th percentile
    _High,     // 75th percentile
    _VeryHigh, // 95th percentile
    // labelled unsafe to prevent people from using and draining their funds by accident
    _UnsafeMax, // 100th percentile
    _Default,   // 50th percentile
}

#[derive(serde::Deserialize, Debug)]
struct GetPriorityFeeEstimateResponse {
    priority_fee_estimate: Option<f64>,
}
