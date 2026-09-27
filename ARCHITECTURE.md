# Architecture Design Document: txplore

**Author**: akshola00  
**Project**: Rust for Bitcoin Cohort Capstone - Option 4: Transaction Explorer  
**Repository**: `/home/x0/transaction-explorer`  

---

## 1. System Overview

`txplore` is an end-to-end Bitcoin transaction analysis framework engineered in modern idiomatic Rust. It is architectured around a strict separation of concerns, decoupling raw consensus deserialization, script disassembly, stack VM simulation, and multi-source fetching from presentation engines (CLI tables, Markdown reports, Mermaid diagrams, Ratatui TUI, and an Axum HTTP REST/Web dashboard).

```
                      +---------------------------------------+
                      |         Data Acquisition Layer        |
                      | - Offline Hex / File / Pipe           |
                      | - Mempool / Esplora REST API          |
                      | - Bitcoin Core JSON-RPC               |
                      +-------------------+-------------------+
                                          |
                                          v
                      +---------------------------------------+
                      |          Consensus Decoding           |
                      | - Deserialization (bitcoin v0.32)     |
                      | - TXID & WTXID double-SHA256          |
                      | - Vsize, Weight Units, Discount Ratio |
                      | - Locktime & BIP68 / BIP125 RBF       |
                      +-------------------+-------------------+
                                          |
        +---------------------------------+---------------------------------+
        |                                 |                                 |
        v                                 v                                 v
+-----------------------+     +-----------------------+     +-----------------------+
|  Script Disassembly   |     | Script Stack VM Trace |     |     PSBT Analyzer     |
| - Opcode Classifier   |     | - Forth Stack Engine  |     | - BIP174 / BIP370     |
| - Standard Script Dts |     | - Conditional Logic   |     | - UTXO Completeness   |
| - OP_RETURN Protocols |     | - Crypto & Math Exec  |     | - Fee Rate Estimation |
+-----------+-----------+     +-----------+-----------+     +-----------+-----------+
            |                             |                             |
            +-----------------------------+-----------------------------+
                                          |
                                          v
                      +---------------------------------------+
                      |        Canonical Domain Model         |
                      |  (DecodedTx, DecodedInput, etc.)      |
                      +-------------------+-------------------+
                                          |
        +------------------+--------------+--------------+------------------+
        |                  |                             |                  |
        v                  v                             v                  v
+---------------+  +---------------+              +---------------+  +---------------+
| CLI Tables    |  | Reports &     |              | Ratatui TUI   |  | Axum Web App  |
| (comfy-table) |  | Serialization |              | - 5-Tab Nav   |  | - REST API    |
|               |  | (MD/JSON/MMD) |              | - Live VM Step|  | - SVG DAG Flow|
+---------------+  +---------------+              +---------------+  +---------------+
```

---

## 2. Core Modules and Data Pipelines

### 2.1 Domain Model (`src/model/types.rs`)

The domain model acts as the single source of truth across all renderers and inspection tools. Key structures include:
- `DecodedTx`: Encapsulates TXID, WTXID, version, raw byte size, weight units (WU), virtual size (vB), witness discount ratio, locktime details, RBF status, and transaction classification.
- `DecodedInput`: Encapsulates OutPoint (TXID:vout), sequence number, BIP68 relative locktime analysis, coinbase scriptSig height extraction (BIP34), scriptSig disassembly, and witness items.
- `DecodedOutput`: Contains output index, satoshi and BTC amounts, `DecodedScript` disassembly, standard script classification, derived network addresses, and structured `OpReturnPayload`.
- `FeeInfo`: Captures aggregate input satoshis, output satoshis, miner fee, fee rate in sat/vB, and fee percentage of total value transferred.
- `ScriptExecutionTrace`: Records the step-by-step state of the main stack and alt stack during VM simulation.
- `PsbtAnalysis`: Captures input UTXO completeness, partial signatures, sighash types, and signing progress.

### 2.2 Consensus Deserialization Engine (`src/decoder/tx.rs`)

Bitcoin transactions are deserialized using `bitcoin::consensus::deserialize` from `rust-bitcoin` v0.32:
- **TXID and WTXID Calculation**: Double-SHA256 hash of consensus-serialized bytes. For transactions containing SegWit witness data, WTXID accounts for the marker, flag, and witness stack.
- **Weight and Virtual Size**: Computed using standard consensus rules:
  $$\text{Weight} = (\text{Base Size} \times 3) + \text{Total Size}$$
  $$\text{Virtual Size} = \lceil \text{Weight} / 4 \rceil$$
- **Witness Discount Ratio**: Accurately measures the economic savings achieved by SegWit witness serialization compared to legacy serialization.
- **BIP68 Relative Locktimes**: Decodes input `nSequence` field. Checks `DISABLE_FLAG` (bit 31), `TYPE_FLAG` (bit 22 for time-based 512-second intervals vs block-height-based), and 16-bit value masks.
- **BIP125 Opt-In RBF**: Signals RBF if any input sequence is strictly less than `0xfffffffe`.

### 2.3 Script Disassembler & Protocol Classifier (`src/decoder/script.rs`)

The script disassembler iterates through raw script bytes without crashing on malformed opcodes:
- **Standard Script Recognition**: Recognizes P2PKH, P2SH, Native SegWit v0 P2WPKH, SegWit v0 P2WSH, Taproot P2TR (BIP341), Bare MultiSig, and Ephemeral Anchors (P2A).
- **OP_RETURN Protocol Parsing**: Extracts and classifies data carrier payloads:
  - *Runes*: Detected via `OP_RETURN OP_13` prefix.
  - *Omni Layer*: Detected via `6f6d6e69` prefix.
  - *OpenTimestamps*: Detected via `03004f54` prefix.
  - *Plain Text*: Cleanly falls back to UTF-8 extraction if bytes form readable characters.
- **Witness Item Semantic Inference**: Uses byte lengths and signature framing to identify 64-byte Schnorr signatures, ECDSA DER signatures (70 to 73 bytes), compressed public keys (33 bytes), and Taproot control blocks.

### 2.4 Bitcoin Script Stack VM Simulator (`src/decoder/evaluator.rs`)

To give developers and auditors deep visibility into Bitcoin Script execution, `txplore` implements an in-memory stack virtual machine:
- **Dual Stack Architecture**: Implements both the primary execution stack and the secondary `alt_stack`.
- **Conditional Branching Tree**: Tracks conditional depth via an execution condition stack, handling nested `OP_IF`, `OP_NOTIF`, `OP_ELSE`, and `OP_ENDIF` blocks.
- **Stack Primitives**: Implements exact Forth semantics for `OP_DUP`, `OP_DROP`, `OP_SWAP`, `OP_ROT`, `OP_OVER`, `OP_NIP`, `OP_TUCK`, and `OP_2DUP`.
- **Arithmetic Engine**: Implements Bitcoin Script 32-bit signed integer semantics with little-endian encoding, including `OP_ADD`, `OP_SUB`, `OP_1ADD`, `OP_1SUB`, `OP_NEGATE`, `OP_ABS`, and `OP_MIN`/`OP_MAX`.
- **Cryptographic Hashing**: Executes real hash transformations on stack items using `sha256`, `ripemd160`, and `hash160` from `bitcoin::hashes`.
- **Execution Tracing**: Yields an ordered `ScriptExecutionTrace` containing every instruction, human-readable explanations, and exact stack snapshots before and after execution.

### 2.5 Multi-Source Fetching Layer (`src/fetcher/`)

1. **Offline Mode (`offline.rs`)**:
   - Parses raw hex strings directly, reads from local files, or streams from standard input (`-`).
   - Accepts `--input-values` to compute miner fees and sat/vB fee rates in completely air-gapped environments.
2. **Esplora / Mempool.space REST Client (`esplora.rs`)**:
   - Asynchronously queries `/tx/:txid/hex` for raw consensus bytes.
   - Queries `/tx/:txid/status` to extract confirmation height, block hash, block time, and confirmation count.
   - Queries `/tx/:txid` to resolve spent prevouts, providing exact input satoshi values for automatic fee calculations.
3. **Bitcoin Core RPC Client (`rpc.rs`)**:
   - Communicates via authenticated JSON-RPC using `bitcoincore-rpc`.
   - Calls `getrawtransaction` with verbose decoding fallback.
   - Calls `gettxout` to query live UTXO set for unconfirmed or recent transactions.

---

## 3. User Interfaces and Export Engines

### 3.1 Terminal UI (`src/tui/`)
Built with `ratatui` and `crossterm`. Features an interactive 5-tab layout:
- **Tab 1: Overview**: Core metadata, fee economics, locktime details, and RBF status.
- **Tab 2: Inputs**: Detailed input list, OutPoints, sequence numbers, relative locktimes, and witness item inspector.
- **Tab 3: Outputs**: Value distributions in satoshis and BTC, script types, derived addresses, and OP_RETURN data payloads.
- **Tab 4: Script Disassembly**: Opcode-by-opcode breakdown with educational descriptions.
- **Tab 5: Stack VM Simulator**: Interactive debugger allowing single-step execution (`Space`) and reset (`r`).

### 3.2 Web Dashboard & REST Server (`src/web/`)
Built with `axum` and `tokio`. Zero external frontend assets required:
- Embedded responsive single-page application with modern dark-mode aesthetic.
- Dynamic SVG Transaction DAG Flow Graph visualizing inputs, transaction node, and outputs with satoshi flows and fee badges.
- Interactive Script VM panel providing step-by-step stack inspection in the browser.
- REST API exposing `/api/health`, `/api/tx/:txid`, `/api/decode`, and `/api/simulate`.

### 3.3 Static Exporters (`src/render/`)
- **CLI Tables (`table.rs`)**: Colored formatting with `comfy-table` for terminal output.
- **Mermaid DAG (`mermaid.rs`)**: Visual flow diagram export compatible with GitHub markdown.
- **Markdown Audit Report (`markdown.rs`)**: Comprehensive technical security report suitable for compliance, audits, and documentation.
- **JSON (`json.rs`)**: Standardized machine-readable serialization.

---

## 4. Verification and Quality Assurance

- **Zero Clippy Warnings**: Strict adherence to Rust compiler safety guidelines (`-D warnings`).
- **Comprehensive Unit Tests**: Full coverage across script execution, PSBT parsing, consensus decoding, and rendering formats.
- **Memory Safety**: Completely written in safe Rust without `unsafe` blocks.
