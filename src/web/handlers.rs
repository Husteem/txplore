use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use bitcoin::Network;
use serde::Deserialize;
use std::sync::Arc;

use crate::decoder::evaluator::ScriptVm;
use crate::decoder::tx::decode_raw_tx_hex;
use crate::fetcher::bitrpc::BitRpcClient;
use crate::fetcher::esplora::EsploraClient;
use crate::web::static_assets::INDEX_HTML;

pub struct AppState {
    pub esplora: EsploraClient,
    pub network: Network,
}

#[derive(Debug, Deserialize)]
pub struct DecodeRequest {
    pub hex: String,
    pub network: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SimulateRequest {
    pub script_hex: String,
    pub initial_stack: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct BitRpcRequest {
    pub method: String,
    pub params: Option<serde_json::Value>,
}

pub async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub async fn health_handler() -> &'static str {
    "OK"
}

pub async fn get_tx_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(txid): Path<String>,
) -> Response {
    let bitrpc_key = headers
        .get("x-bitrpc-key")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .or_else(|| std::env::var("BITRPC_API_KEY").ok());

    if let Some(key) = bitrpc_key {
        let bitrpc = BitRpcClient::new(&key, state.network);
        match bitrpc.fetch_and_enrich(&txid).await {
            Ok(tx) => return Json(tx).into_response(),
            Err(e) => {
                eprintln!("BitRPC fetch encountered error (falling back to Esplora): {}", e);
            }
        }
    }

    match state.esplora.fetch_and_enrich(&txid).await {
        Ok(tx) => Json(tx).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            format!("Failed to retrieve transaction {}: {}", txid, e),
        )
            .into_response(),
    }
}

pub async fn decode_tx_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DecodeRequest>,
) -> Response {
    match decode_raw_tx_hex(&payload.hex, state.network) {
        Ok(mut tx) => {
            let _ = state.esplora.enrich_with_esplora_data(&mut tx).await;
            Json(tx).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Invalid transaction hex: {}", e),
        )
            .into_response(),
    }
}

pub async fn simulate_script_handler(Json(payload): Json<SimulateRequest>) -> Response {
    let script_bytes = match hex::decode(&payload.script_hex) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Invalid script hex: {}", e),
            )
                .into_response()
        }
    };

    let script = bitcoin::ScriptBuf::from(script_bytes);

    let mut initial_stack_bytes = Vec::new();
    if let Some(ref items) = payload.initial_stack {
        for it in items {
            if let Ok(b) = hex::decode(it) {
                initial_stack_bytes.push(b);
            }
        }
    }

    let mut vm = ScriptVm::with_initial_stack(&initial_stack_bytes);
    let trace = vm.simulate(&script);
    Json(trace).into_response()
}

pub async fn bitrpc_call_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<BitRpcRequest>,
) -> Response {
    let key = match headers
        .get("x-bitrpc-key")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .or_else(|| std::env::var("BITRPC_API_KEY").ok())
    {
        Some(k) => k,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "BitRPC API Key required. Pass header 'X-BitRPC-Key' or set environment variable BITRPC_API_KEY.",
            )
                .into_response()
        }
    };

    let client = BitRpcClient::new(&key, state.network);
    let params = payload.params.unwrap_or_else(|| serde_json::json!([]));

    match client.call::<serde_json::Value>(&payload.method, params).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            format!("BitRPC call failed: {}", e),
        )
            .into_response(),
    }
}
