use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "txplore")]
#[command(author = "husteemah")]
#[command(version = "0.1.0")]
#[command(about = "A high-performance Bitcoin Transaction Explorer, Script Disassembler, and Analyzer in Rust", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inspect and decode a Bitcoin transaction from txid, raw hex, or file
    Inspect(InspectArgs),

    /// Launch interactive terminal dashboard (TUI) for deep transaction exploration
    Tui(TuiArgs),

    /// Launch embedded web dashboard with interactive visual DAG and REST API
    Serve(ServeArgs),

    /// Simulate step-by-step Bitcoin Script execution in VM
    EvalScript(EvalScriptArgs),

    /// Inspect and analyze a Partially Signed Bitcoin Transaction (PSBT)
    AnalyzePsbt(AnalyzePsbtArgs),
}

#[derive(Args, Debug)]
pub struct InspectArgs {
    /// Transaction ID (txid), raw hex, file path, or '-' for stdin
    #[arg(index = 1)]
    pub input: String,

    /// Bitcoin Network (bitcoin, testnet, signet, regtest)
    #[arg(short, long, default_value = "regtest")]
    pub network: String,

    /// Comma-separated spent input values in satoshis for offline fee calculation
    #[arg(long)]
    pub input_values: Option<String>,

    /// Bitcoin Core RPC URL
    #[arg(long)]
    pub rpc_url: Option<String>,

    /// Bitcoin Core RPC Username
    #[arg(long)]
    pub rpc_user: Option<String>,

    /// Bitcoin Core RPC Password
    #[arg(long)]
    pub rpc_pass: Option<String>,

    /// Bitcoin Core RPC Cookie File Path
    #[arg(long)]
    pub rpc_cookie: Option<PathBuf>,

    /// Custom Mempool.space / Esplora API URL
    #[arg(long)]
    pub esplora_url: Option<String>,

    /// Output full analysis as formatted JSON
    #[arg(long)]
    pub json: bool,

    /// Output full technical audit report in Markdown
    #[arg(long)]
    pub markdown: bool,

    /// Output Mermaid transaction flow graph
    #[arg(long)]
    pub mermaid: bool,
}

#[derive(Args, Debug)]
pub struct TuiArgs {
    /// Transaction ID (txid), raw hex, file path, or '-' for stdin
    #[arg(index = 1)]
    pub input: String,

    /// Bitcoin Network (bitcoin, testnet, signet, regtest)
    #[arg(short, long, default_value = "regtest")]
    pub network: String,

    /// Comma-separated spent input values in satoshis
    #[arg(long)]
    pub input_values: Option<String>,

    /// Custom Mempool.space / Esplora API URL
    #[arg(long)]
    pub esplora_url: Option<String>,
}

#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Port to bind web server to
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,

    /// Default Bitcoin network
    #[arg(short, long, default_value = "regtest")]
    pub network: String,

    /// Custom Mempool.space / Esplora API URL
    #[arg(long)]
    pub esplora_url: Option<String>,
}

#[derive(Args, Debug)]
pub struct EvalScriptArgs {
    /// Script in hex format
    #[arg(index = 1)]
    pub script_hex: String,

    /// Comma-separated initial stack hex elements
    #[arg(short, long)]
    pub stack: Option<String>,
}

#[derive(Args, Debug)]
pub struct AnalyzePsbtArgs {
    /// PSBT in Base64 or Hex format, or path to PSBT file
    #[arg(index = 1)]
    pub psbt: String,
}
