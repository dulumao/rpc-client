use solana_client::client_error::Result as ClientResult;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::Signature;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use solana_sdk::{instruction::Instruction, signature::Keypair};
use solana_transaction_status::UiTransactionEncoding;

use crate::priority_fee;

const SEND_CFG: RpcSendTransactionConfig = RpcSendTransactionConfig {
    skip_preflight: false,
    preflight_commitment: Some(CommitmentLevel::Confirmed),
    encoding: Some(UiTransactionEncoding::Base64),
    max_retries: Some(0),
    min_context_slot: None,
};

pub async fn send_tx(
    rpc_client: &RpcClient,
    http_client: &reqwest::Client,
    ixs: &mut Vec<Instruction>,
    signer: &Keypair,
    cu_limit: u32,
    priority_fee: bool,
) -> ClientResult<Signature> {
    let mut final_ixs = Vec::with_capacity(ixs.len() + 2);
    let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(cu_limit);
    final_ixs.push(cu_limit_ix);
    if priority_fee {
        let pf = priority_fee::get_recent_priority_fee_estimate(http_client, rpc_client.url(), ixs)
            .await?;
        let cu_price_ix = ComputeBudgetInstruction::set_compute_unit_price(pf);
        final_ixs.push(cu_price_ix);
    };
    final_ixs.append(ixs);
    let mut tx = Transaction::new_with_payer(final_ixs.as_slice(), Some(&signer.pubkey()));
    let (hash, _) = rpc_client
        .get_latest_blockhash_with_commitment(rpc_client.commitment())
        .await?;
    tx.sign(&[signer], hash);
    rpc_client.send_transaction_with_config(&tx, SEND_CFG).await
}
