# txplore: Bitcoin Transaction Explorer, Script Disassembler & Stack VM Simulator

[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org)
[![Bitcoin](https://img.shields.io/badge/bitcoin-v0.32-yellow.svg)](https://crates.io/crates/bitcoin)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

**txplore** is a production-grade, zero-dependency-runtime Bitcoin transaction explorer, script disassembler, and execution debugger written in Rust. Built as the Capstone Project for the *Rust for Bitcoin* cohort by **Naim Hussain (@husteemah)**.

It delivers deep visibility into Bitcoin transactions, moving beyond surface-level block explorers to provide byte-level consensus verification, step-by-step Script VM stack simulation, BIP174/BIP370 PSBT analysis, and multi-interface rendering (CLI tables, Markdown audit reports, Mermaid DAG diagrams, interactive Ratatui TUI, and an embedded Axum web dashboard).

---

## Key Features

1. **Complete Consensus Deserialization & Metrics**
   - Precise computation of Raw Size (bytes), Weight (WU), Virtual Size (vB), and Witness Discount ratio.
   - BIP68 relative locktime inspection (block-based and 512-second epoch-based).
   - BIP113 Median Time Past (MTP) locktime analysis and BIP125 Replace-By-Fee (RBF) signaling detection.
   - Coinbase transaction identification and BIP34 height extraction.

2. **Advanced Script Disassembly & Protocol Classification**
   - Full human-readable opcode disassembly paired with explanatory educational tooltips.
   - Address derivation across Mainnet, Testnet, Signet, and Regtest.
   - Standards classification:
     - Legacy: P2PKH, P2SH, Bare Multisig
     - SegWit v0: P2WPKH, P2WSH
     - SegWit v1: Taproot (P2TR key path and script path)
     - Ephemeral / Anchor: P2A (BIP388)
     - Null Data / OP_RETURN protocol identification: Runes, Omni Layer, OpenTimestamps, and UTF-8 strings.
   - Witness item inspection with automatic identification of Schnorr signatures, ECDSA DER signatures, public keys, and redeem/witness scripts.

3. **Step-by-Step Bitcoin Script Stack VM Simulator**
   - Interactive Forth-like virtual machine simulating Bitcoin Script execution.
   - Traces mutation of the main stack and alt stack opcode by opcode.
   - Supports stack manipulation (`OP_DUP`, `OP_DROP`, `OP_SWAP`, `OP_ROT`, `OP_OVER`), arithmetic (`OP_ADD`, `OP_SUB`, `OP_1ADD`, `OP_1SUB`), bitwise/comparison logic (`OP_EQUAL`, `OP_EQUALVERIFY`, `OP_NUMEQUAL`), conditionals (`OP_IF`, `OP_ELSE`, `OP_ENDIF`), and cryptographic hashing (`OP_SHA256`, `OP_HASH160`, `OP_RIPEMD160`).

4. **BIP174 & BIP370 PSBT Deep Inspection**
   - Parses Partial Transactions from Hex or Base64.
   - Inspects input UTXO completeness (Witness UTXO vs Non-Witness UTXO).
   - Tracks partial signatures, sighash types, redeem scripts, witness scripts, Taproot internal keys, and taproot script trees.
   - Computes transaction fee and fee rate before finalization.

5. **Multi-Source Fetching Engine**
   - **Offline Mode**: Inspects raw hex strings, files, or standard input (`-`), with `--input-values` support for offline fee and fee-rate calculation.
   - **Esplora / Mempool.space REST API**: Live network queries resolving raw transaction bytes, confirmation status, block heights, timestamps, and input prevouts.
   - **Bitcoin Core RPC**: Direct JSON-RPC connection (`getrawtransaction`, `gettxout`) for private node queries.

6. **Multiple Output & Rendering Formats**
   - **CLI Tables**: Colored summaries formatted with `comfy-table`.
   - **Mermaid Graph**: Generates clean visual input-transaction-output DAG diagrams.
   - **Markdown Technical Audit**: Exports comprehensive Markdown reports ready for security and auditing documentation.
   - **JSON**: Machine-readable structured output.
   - **Interactive Ratatui TUI**: Terminal dashboard with 5 tabs and real-time keyboard navigation.
   - **Embedded Web Dashboard**: Fast Axum HTTP server hosting a responsive dark-mode dashboard with interactive SVG transaction flow graphs.

---

## Installation & Requirements

Ensure you have Rust 1.75+ installed:

```bash
git clone https://github.com/Husteem/txplore.git
cd txplore
cargo build --release
```

The compiled binary will be located at `target/release/txplore`.

---

## CLI Usage & Examples

### 1. Offline Raw Hex Inspection

Inspect a raw transaction hex string directly:

```bash
# Print colored summary tables to terminal
txplore inspect 0200000001...

# Provide input values in satoshis to compute fees offline
txplore inspect 0200000001... --input-values 100000

# Export Markdown technical audit report
txplore inspect 0200000001... --markdown > tx_audit.md

# Export Mermaid flow graph
txplore inspect 0200000001... --mermaid

# Export structured JSON
txplore inspect 0200000001... --json
```

### 2. Reading from File or Stdin

```bash
# From file
txplore inspect tx_hex.txt

# From stdin pipe
cat tx_hex.txt | txplore inspect -
```

### 3. Fetching from Live Networks (Esplora / Mempool.space)

```bash
# Fetch from Bitcoin Mainnet
txplore inspect 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b --network mainnet

# Fetch from Testnet
txplore inspect <txid> --network testnet

# Fetch using a custom Esplora endpoint
txplore inspect <txid> --esplora-url https://mempool.space/api
```

### 4. Fetching from Bitcoin Core RPC

```bash
txplore inspect <txid> \
  --rpc-url http://127.0.0.1:8332 \
  --rpc-user bitcoinrpc \
  --rpc-pass secretpassword
```

### 5. Interactive Terminal UI (TUI)

Launch the full-screen terminal dashboard:

```bash
txplore tui <hex-or-txid>
```

#### TUI Keyboard Shortcuts:
- `Tab` / `1-5`: Switch tabs (Overview, Inputs, Outputs, Script Disassembly, Script VM)
- `Up` / `Down` / `j` / `k`: Scroll list items
- `Space`: Execute next step in Script VM simulator
- `r`: Reset Script VM simulator
- `q` / `Esc`: Exit application

### 6. Interactive Script VM Simulation

Simulate arbitrary Bitcoin scripts directly from the command line:

```bash
# Simulate a simple math script: 10 + 20
txplore eval-script "OP_10 OP_20 OP_ADD"

# Simulate with initial stack elements (hex encoded)
txplore eval-script "OP_DUP OP_HASH160" --initial-stack "02d69de24b245ad0605d5000710079707e753e4f7d2126c70c2274708a0a4d71e4"
```

### 7. BIP174 / BIP370 PSBT Analysis

Analyze unfinalized or partially signed transactions:

```bash
txplore analyze-psbt cHNidP8BAFICAAAAAZ...
```

### 8. Web Server & Visual Dashboard

Launch the embedded web server:

```bash
txplore serve --port 3000 --bind 127.0.0.1
```

Visit `http://localhost:3000` in your browser. The web dashboard includes:
- Live transaction lookup by TXID or raw hex paste
- Interactive SVG transaction DAG flow graph
- Real-time step-by-step Script VM visual debugger
- REST API endpoints for external integrations:
  - `GET /api/health`
  - `GET /api/tx/:txid`
  - `POST /api/decode`
  - `POST /api/simulate`

---

## Architecture

```
                    +--------------------------------+
                    |           Data Source          |
                    | Offline Hex / Esplora / RPC   |
                    +---------------+----------------+
                                    |
                                    v
                    +--------------------------------+
                    |        Consensus Decoder       |
                    | (Weight, Vsize, Locktime, RBF) |
                    +---------------+----------------+
                                    |
          +-------------------------+-------------------------+
          |                         |                         |
          v                         v                         v
+-------------------+     +-------------------+     +-------------------+
| Script Disasm     |     | Script Stack VM   |     | PSBT Inspector    |
| Type Classify     |     | Step Simulator    |     | BIP174 / BIP370   |
+---------+---------+     +---------+---------+     +---------+---------+
          |                         |                         |
          +-------------------------+-------------------------+
                                    |
                                    v
                    +--------------------------------+
                    |        Rendering Layer         |
                    | Tables | MD | Mermaid | JSON   |
                    | Ratatui TUI | Axum Web Engine  |
                    +--------------------------------+
```

---

## Test Suite

The project includes an extensive automated test suite covering edge cases across Legacy, SegWit v0, Taproot, OP_RETURN protocols, Script VM execution, PSBT analysis, and rendering outputs.

Run all tests:

```bash
cargo test
```

Run linter checks:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

---

## License

This project is licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
