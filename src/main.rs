use anyhow::{anyhow, Context, Result};
use clap::Parser;

use bitcoin::Network;
use txplore::cli::{
    AnalyzePsbtArgs, BitRpcArgs, Cli, Commands, EvalScriptArgs, InspectArgs, ServeArgs, TuiArgs,
};
use txplore::decoder::evaluator::ScriptVm;
use txplore::decoder::psbt::analyze_psbt_str;
use txplore::fetcher::bitrpc::BitRpcClient;
use txplore::fetcher::esplora::EsploraClient;
use txplore::fetcher::offline::load_tx_from_source;
use txplore::fetcher::rpc::BitcoinCoreRpcClient;
use txplore::model::types::DecodedTx;
use txplore::render::json::to_json_pretty;
use txplore::render::markdown::generate_markdown_report;
use txplore::render::mermaid::generate_mermaid_diagram;
use txplore::render::table::print_transaction_summary;
use txplore::tui::app::run_tui;
use txplore::web::server::start_web_server;

fn parse_network(s: &str) -> Result<Network> {
    match s.to_lowercase().as_str() {
        "mainnet" | "bitcoin" => Ok(Network::Bitcoin),
        "testnet" | "testnet3" => Ok(Network::Testnet),
        "signet" => Ok(Network::Signet),
        "regtest" => Ok(Network::Regtest),
        other => Err(anyhow!(
            "Unsupported network: {}. Use bitcoin, testnet, signet, or regtest",
            other
        )),
    }
}

fn parse_input_values(s: Option<&str>) -> Result<Option<Vec<u64>>> {
    if let Some(val_str) = s {
        let mut list = Vec::new();
        for item in val_str.split(',') {
            let num: u64 = item
                .trim()
                .parse()
                .context(format!("Failed to parse input satoshi amount: '{}'", item))?;
            list.push(num);
        }
        Ok(Some(list))
    } else {
        Ok(None)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inspect(args) => handle_inspect(args).await?,
        Commands::Tui(args) => handle_tui(args).await?,
        Commands::Serve(args) => handle_serve(args).await?,
        Commands::EvalScript(args) => handle_eval_script(args)?,
        Commands::AnalyzePsbt(args) => handle_analyze_psbt(args)?,
        Commands::Bitrpc(args) => handle_bitrpc(args).await?,
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn resolve_transaction(
    input: &str,
    network: Network,
    input_values: Option<&[u64]>,
    rpc_url: Option<&str>,
    rpc_user: Option<String>,
    rpc_pass: Option<String>,
    rpc_cookie: Option<std::path::PathBuf>,
    esplora_url: Option<&str>,
    bitrpc_key: Option<&str>,
) -> Result<DecodedTx> {
    let clean = input.trim();

    if clean.len() == 64 && clean.chars().all(|c| c.is_ascii_hexdigit()) {
        if let Some(key) = bitrpc_key {
            let bitrpc = BitRpcClient::new(key, network);
            bitrpc.fetch_and_enrich(clean).await
        } else if let Some(url) = rpc_url {
            let rpc = BitcoinCoreRpcClient::new(url, rpc_user, rpc_pass, rpc_cookie, network)?;
            rpc.fetch_and_enrich(clean)
        } else {
            let esplora = EsploraClient::new(esplora_url, network);
            esplora.fetch_and_enrich(clean).await
        }
    } else {
        load_tx_from_source(clean, network, input_values)
    }
}

async fn handle_inspect(args: InspectArgs) -> Result<()> {
    let network = parse_network(&args.network)?;
    let input_values = parse_input_values(args.input_values.as_deref())?;

    let tx = resolve_transaction(
        &args.input,
        network,
        input_values.as_deref(),
        args.rpc_url.as_deref(),
        args.rpc_user,
        args.rpc_pass,
        args.rpc_cookie,
        args.esplora_url.as_deref(),
        args.bitrpc_key.as_deref(),
    )
    .await?;

    if args.json {
        println!("{}", to_json_pretty(&tx)?);
    } else if args.markdown {
        println!("{}", generate_markdown_report(&tx));
    } else if args.mermaid {
        println!("{}", generate_mermaid_diagram(&tx));
    } else {
        print_transaction_summary(&tx);
    }

    Ok(())
}

async fn handle_tui(args: TuiArgs) -> Result<()> {
    let network = parse_network(&args.network)?;
    let input_values = parse_input_values(args.input_values.as_deref())?;

    let tx = resolve_transaction(
        &args.input,
        network,
        input_values.as_deref(),
        None,
        None,
        None,
        None,
        args.esplora_url.as_deref(),
        args.bitrpc_key.as_deref(),
    )
    .await?;

    run_tui(tx)?;
    Ok(())
}

async fn handle_serve(args: ServeArgs) -> Result<()> {
    let network = parse_network(&args.network)?;
    start_web_server(args.port, network, args.esplora_url).await
}

fn handle_eval_script(args: EvalScriptArgs) -> Result<()> {
    let script_bytes = hex::decode(args.script_hex.trim()).context("Invalid script hex string")?;
    let script = bitcoin::ScriptBuf::from(script_bytes);

    let mut initial_stack = Vec::new();
    if let Some(ref st) = args.stack {
        for it in st.split(',') {
            let b = hex::decode(it.trim()).context(format!("Invalid stack hex item: '{}'", it))?;
            initial_stack.push(b);
        }
    }

    let mut vm = ScriptVm::with_initial_stack(&initial_stack);
    let trace = vm.simulate(&script);

    println!("============================================================");
    println!(" Bitcoin Script Stack VM Simulator");
    println!("============================================================");
    println!("Script Hex: 0x{}", args.script_hex);
    println!(
        "Status:     {}",
        if trace.success {
            "SUCCESS (VALID)"
        } else {
            "FAILED (INVALID)"
        }
    );
    if let Some(ref err) = trace.error_message {
        println!("Error:      {}", err);
    }
    println!("------------------------------------------------------------");
    println!("Step-by-Step Execution Trace:");
    for (i, step) in trace.steps.iter().enumerate() {
        println!(
            "[Step {:02}] {:<24} -> {}",
            i + 1,
            step.instruction,
            step.description
        );
        println!("         Stack After: [{}]", step.stack_after.join(", "));
    }
    println!("------------------------------------------------------------");
    println!("Final Stack: [{}]", trace.final_stack.join(", "));
    println!("============================================================");

    Ok(())
}

fn handle_analyze_psbt(args: AnalyzePsbtArgs) -> Result<()> {
    let analysis = analyze_psbt_str(&args.psbt)?;

    println!("============================================================");
    println!(" Partially Signed Bitcoin Transaction (PSBT) Analysis");
    println!("============================================================");
    println!("Inputs Count:       {}", analysis.num_inputs);
    println!("Outputs Count:      {}", analysis.num_outputs);
    println!(
        "Signatures Present: {} / {}",
        analysis.signatures_present, analysis.signatures_required
    );
    println!(
        "Finalized Status:   {}",
        if analysis.is_finalized {
            "Fully Signed & Finalized"
        } else {
            "Incomplete (Awaiting Signatures)"
        }
    );

    if let Some(fee) = analysis.fee_sats {
        println!("Estimated Fee:      {} sats", fee);
    }
    if let Some(rate) = analysis.fee_rate_sat_per_vb {
        println!("Estimated Fee Rate: {:.2} sat/vB", rate);
    }

    println!("------------------------------------------------------------");
    println!("Inputs Signing Status:");
    for input in &analysis.inputs_status {
        let utxo_str = if let Some(v) = input.value_sats {
            format!("{} sats", v)
        } else {
            "Missing UTXO".to_string()
        };
        println!(
            "  Input #{}: UTXO: {} | Partial Sigs: {} | Finalized: {}",
            input.index, utxo_str, input.partial_sigs_count, input.is_finalized
        );
    }
    println!("============================================================");

    Ok(())
}

async fn handle_bitrpc(args: BitRpcArgs) -> Result<()> {
    let key = args.api_key.or_else(|| std::env::var("BITRPC_API_KEY").ok()).context(
        "BitRPC API key required. Pass --api-key <KEY> or set export BITRPC_API_KEY='<KEY>'",
    )?;

    let client = BitRpcClient::new(&key, Network::Bitcoin);
    let params: serde_json::Value = if let Some(ref p) = args.params {
        serde_json::from_str(p).unwrap_or_else(|_| serde_json::json!([p]))
    } else {
        serde_json::json!([])
    };

    let result: serde_json::Value = client.call(&args.method, params).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
