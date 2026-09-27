use anyhow::{Context, Result};
use axum::routing::{get, post};
use axum::Router;
use bitcoin::Network;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::fetcher::esplora::EsploraClient;
use crate::web::handlers::{
    decode_tx_handler, get_tx_handler, health_handler, index_handler, simulate_script_handler,
    AppState,
};

pub async fn start_web_server(
    port: u16,
    network: Network,
    esplora_url: Option<String>,
) -> Result<()> {
    let esplora = EsploraClient::new(esplora_url.as_deref(), network);
    let state = Arc::new(AppState { esplora, network });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/health", get(health_handler))
        .route("/api/tx/:txid", get(get_tx_handler))
        .route("/api/decode", post(decode_tx_handler))
        .route("/api/simulate", post(simulate_script_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(format!("Failed to bind TCP listener on {}", addr))?;

    println!("============================================================");
    println!(" txplore Web Dashboard Running!");
    println!(" URL: http://localhost:{}", port);
    println!(" Network: {}", network);
    println!(" REST Endpoints:");
    println!("   GET  /api/tx/:txid");
    println!("   POST /api/decode");
    println!("   POST /api/simulate");
    println!("============================================================");

    axum::serve(listener, app)
        .await
        .context("Web server encountered an error")?;

    Ok(())
}
