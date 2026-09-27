use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedTx {
    pub txid: String,
    pub wtxid: String,
    pub version: i32,
    pub locktime: LocktimeInfo,
    pub is_segwit: bool,
    pub is_coinbase: bool,
    pub size_bytes: usize,
    pub weight_wu: u64,
    pub vsize_vb: u64,
    pub discount_ratio: f64,
    pub inputs: Vec<DecodedInput>,
    pub outputs: Vec<DecodedOutput>,
    pub fee_info: Option<FeeInfo>,
    pub rbf_status: RbfStatus,
    pub classification: TxClassification,
    pub network: String,
    pub confirmation_info: Option<ConfirmationInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocktimeInfo {
    pub raw: u32,
    pub locktime_type: LocktimeType,
    pub description: String,
    pub is_final: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocktimeType {
    None,
    BlockHeight,
    Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeInfo {
    pub total_input_sats: u64,
    pub total_output_sats: u64,
    pub fee_sats: u64,
    pub fee_rate_sat_per_vb: f64,
    pub fee_percentage: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RbfStatus {
    Signaling,
    NotSignaling,
    Mixed,
    Coinbase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxClassification {
    Coinbase,
    SimplePayment,
    Consolidation,
    BatchPayment,
    LightningChannelOpen,
    LightningChannelClose,
    OpReturnData,
    TaprootKeySpend,
    TaprootScriptSpend,
    MultisigSpend,
    Unknown,
}

impl std::fmt::Display for TxClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxClassification::Coinbase => write!(f, "Coinbase (Block Reward)"),
            TxClassification::SimplePayment => write!(f, "Simple Transfer (1 in, 2 out)"),
            TxClassification::Consolidation => write!(f, "UTXO Consolidation (Many in, 1 out)"),
            TxClassification::BatchPayment => write!(f, "Batch Payout (1 in, Many out)"),
            TxClassification::LightningChannelOpen => write!(f, "Lightning Channel Open"),
            TxClassification::LightningChannelClose => write!(f, "Lightning Channel Close"),
            TxClassification::OpReturnData => write!(f, "OP_RETURN Data Carrier"),
            TxClassification::TaprootKeySpend => write!(f, "Taproot Key-Path Spend"),
            TxClassification::TaprootScriptSpend => write!(f, "Taproot Script-Tree Spend"),
            TxClassification::MultisigSpend => write!(f, "Multi-Signature Spend"),
            TxClassification::Unknown => write!(f, "Standard Bitcoin Transaction"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationInfo {
    pub confirmed: bool,
    pub block_height: Option<u32>,
    pub block_hash: Option<String>,
    pub block_time: Option<u64>,
    pub confirmations: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedInput {
    pub index: usize,
    pub prevout_txid: String,
    pub prevout_vout: u32,
    pub sequence: u32,
    pub sequence_rbf: bool,
    pub relative_locktime: Option<RelativeLocktimeInfo>,
    pub coinbase_data: Option<CoinbaseData>,
    pub script_sig: Option<DecodedScript>,
    pub witness: Vec<WitnessItem>,
    pub spent_txout: Option<SpentTxOut>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelativeLocktimeInfo {
    pub raw: u32,
    pub is_time_based: bool,
    pub value: u32,
    pub human_readable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseData {
    pub hex: String,
    pub bip34_height: Option<u32>,
    pub text_representation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpentTxOut {
    pub value_sats: u64,
    pub script_pubkey: DecodedScript,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessItem {
    pub index: usize,
    pub hex: String,
    pub size_bytes: usize,
    pub inferred_type: WitnessItemType,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WitnessItemType {
    EcdsaSignature,
    SchnorrSignature,
    PublicKey,
    TaprootControlBlock,
    WitnessScript,
    Preimage,
    Empty,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedOutput {
    pub index: usize,
    pub value_sats: u64,
    pub value_btc: f64,
    pub script_pubkey: DecodedScript,
    pub address: Option<String>,
    pub script_type: ScriptType,
    pub op_return_payload: Option<OpReturnPayload>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptType {
    P2pkh,
    P2sh,
    P2wpkh,
    P2wsh,
    P2tr,
    OpReturn,
    Multisig,
    Anchor,
    NonStandard,
}

impl std::fmt::Display for ScriptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptType::P2pkh => write!(f, "P2PKH (Legacy Pubkey Hash)"),
            ScriptType::P2sh => write!(f, "P2SH (Pay-to-Script-Hash)"),
            ScriptType::P2wpkh => write!(f, "P2WPKH (Native SegWit v0)"),
            ScriptType::P2wsh => write!(f, "P2WSH (SegWit v0 Script Hash)"),
            ScriptType::P2tr => write!(f, "P2TR (Taproot v1)"),
            ScriptType::OpReturn => write!(f, "OP_RETURN (Provably Unspendable Data)"),
            ScriptType::Multisig => write!(f, "Bare MultiSig"),
            ScriptType::Anchor => write!(f, "P2A (Ephemeral Anchor)"),
            ScriptType::NonStandard => write!(f, "Non-Standard Script"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpReturnPayload {
    pub hex: String,
    pub ascii: Option<String>,
    pub protocol: OpReturnProtocol,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpReturnProtocol {
    PlainText(String),
    OmniLayer,
    OpenTimestamps,
    Runes,
    UnknownProtocol(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedScript {
    pub hex: String,
    pub asm: String,
    pub opcodes: Vec<ParsedOpcode>,
    pub script_type: ScriptType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedOpcode {
    pub offset: usize,
    pub name: String,
    pub hex: String,
    pub is_push: bool,
    pub push_data_hex: Option<String>,
    pub meaning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsbtAnalysis {
    pub num_inputs: usize,
    pub num_outputs: usize,
    pub fee_sats: Option<u64>,
    pub fee_rate_sat_per_vb: Option<f64>,
    pub is_finalized: bool,
    pub signatures_present: usize,
    pub signatures_required: usize,
    pub inputs_status: Vec<PsbtInputStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsbtInputStatus {
    pub index: usize,
    pub has_utxo: bool,
    pub value_sats: Option<u64>,
    pub has_partial_sigs: bool,
    pub partial_sigs_count: usize,
    pub sighash_type: Option<String>,
    pub has_witness_script: bool,
    pub has_redeem_script: bool,
    pub has_taproot_internal_key: bool,
    pub has_taproot_script_tree: bool,
    pub is_finalized: bool,
}
