use crate::model::types::DecodedTx;
use anyhow::Result;

pub fn to_json_pretty(tx: &DecodedTx) -> Result<String> {
    serde_json::to_string_pretty(tx).map_err(|e| anyhow::anyhow!("Failed to serialize JSON: {}", e))
}
