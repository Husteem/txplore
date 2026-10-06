# txplore User & Operation Guide

Welcome to the comprehensive operational manual for **txplore**. This guide details each of the 3 primary operating modes, test vectors, command options, keyboard shortcuts, and REST API integration patterns.

---

## Table of Contents

1. [Quickstart & Binary Location](#1-quickstart--binary-location)
2. [Mode 1: Command-Line Interface (CLI) Inspection](#2-mode-1-command-line-interface-cli-inspection)
   - [Basic Hex & OutPoint Inspection](#basic-hex--outpoint-inspection)
   - [Offline Fee Estimation with Input Values](#offline-fee-estimation-with-input-values)
   - [Reading from File or Stdin Pipe](#reading-from-file-or-stdin-pipe)
   - [Exporting Technical Reports (Markdown, Mermaid, JSON)](#exporting-technical-reports-markdown-mermaid-json)
   - [Live Network Queries (Esplora & Bitcoin Core RPC)](#live-network-queries-esplora--bitcoin-core-rpc)
3. [Mode 2: Interactive Terminal UI (TUI)](#3-mode-2-interactive-terminal-ui-tui)
   - [Launching the TUI](#launching-the-tui)
   - [Tab Breakdown & Visual Panels](#tab-breakdown--visual-panels)
   - [Keyboard Navigation & Shortcuts](#keyboard-navigation--shortcuts)
   - [Live Script VM Debugging in the Terminal](#live-script-vm-debugging-in-the-terminal)
4. [Mode 3: Web Server & Interactive Dashboard](#4-mode-3-web-server--interactive-dashboard)
   - [Starting the Embedded Server](#starting-the-embedded-server)
   - [Visual Transaction DAG Flow Graph](#visual-transaction-dag-flow-graph)
   - [REST API Endpoints & Curl Examples](#rest-api-endpoints--curl-examples)
5. [Additional Tools: Script VM & PSBT Analyzer](#5-additional-tools-script-vm--psbt-analyzer)
   - [Direct Script Execution (eval-script)](#direct-script-execution-eval-script)
   - [Partially Signed Bitcoin Transactions (analyze-psbt)](#partially-signed-bitcoin-transactions-analyze-psbt)
6. [Curated Test Vectors](#6-curated-test-vectors)

---

## 1. Quickstart & Installation

Clone the repository and enter the project directory:

```bash
git clone https://github.com/Husteem/txplore.git
cd txplore
```

### Option A: Global Installation via Cargo (Recommended)

Install the `txplore` binary directly into your Cargo environment (`~/.cargo/bin`):

```bash
cargo install --path .
```

Verify that `txplore` is available in your PATH:

```bash
txplore --version
# txplore 0.1.0
```

*(Note: Ensure `~/.cargo/bin` is in your `$PATH`. If running for the first time, run `source ~/.cargo/env` or add `export PATH="$HOME/.cargo/bin:$PATH"` to your `~/.bashrc`)*

### Option B: Local Release Binary Execution

Build the optimized local release binary:

```bash
cargo build --release
```

Run directly from the repository root:

```bash
./target/release/txplore --version
# txplore 0.1.0
```

Or execute commands via Cargo:

```bash
cargo run --release -- inspect <TXID>
```

---

## 2. Mode 1: Command-Line Interface (CLI) Inspection

The CLI mode gives you terminal visibility into consensus fields, script types, witness items, and fee economics.

### Basic Hex & OutPoint Inspection

Inspect a raw hex string directly in your terminal:

```bash
txplore inspect <RAW_HEX>
```

Sample output displays:
- Core metadata: Transaction ID, Witness ID, Version, Classification.
- Protocol badges: SegWit Active / Legacy, BIP125 RBF Signaling status.
- Size and weight: Virtual Size (vB), Weight Units (WU), Serialized Size (bytes), and Witness Discount percentage.
- Formatted Inputs Table: Previous OutPoints, Sequence numbers, ScriptSig, and Witness stack items.
- Formatted Outputs Table: Satoshi and BTC values, Script standard classification, derived address, and ASM disassembly.

### Offline Fee Estimation with Input Values

When inspecting raw transactions offline, previous output values are unknown unless specified. Use `--input-values` (in satoshis) to calculate exact miner fees, sat/vB fee rates, and fee percentages:

```bash
txplore inspect <RAW_HEX> --input-values 100000 50000
```

### Reading from File or Stdin Pipe

Pass a text file path or stream directly from stdin using `-`:

```bash
# Read from file
txplore inspect tx_hex.txt

# Stream from standard input
cat tx_hex.txt | txplore inspect -

# Query Bitcoin Core via CLI and pipe into txplore
bitcoin-cli getrawtransaction <txid> | txplore inspect -
```

### Exporting Technical Reports (Markdown, Mermaid, JSON)

```bash
# Generate a complete Markdown security & audit report
txplore inspect <RAW_HEX> --markdown > audit_report.md

# Generate a visual Mermaid DAG diagram (compatible with GitHub Markdown)
txplore inspect <RAW_HEX> --mermaid

# Output structured JSON for automation or scripting
txplore inspect <RAW_HEX> --json | jq .
```

### Live Network Queries (Esplora & Bitcoin Core RPC)

Query live transactions directly using a TXID:

```bash
# Fetch from Bitcoin Mainnet via public Esplora / Mempool.space
txplore inspect 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b --network bitcoin

# Fetch from Testnet
txplore inspect <txid> --network testnet

# Fetch from a private Bitcoin Core node via JSON-RPC
txplore inspect <txid> \
  --rpc-url http://127.0.0.1:8332 \
  --rpc-user bitcoinrpc \
  --rpc-pass secretpassword
```

---

## 3. Mode 2: Interactive Terminal UI (TUI)

The TUI mode delivers a high-performance, full-screen dashboard powered by `ratatui` and `crossterm`.

### Launching the TUI

```bash
# Launch with raw hex
txplore tui <RAW_HEX>

# Launch with live TXID on Mainnet
txplore tui <TXID> --network bitcoin
```

### Tab Breakdown & Visual Panels

The TUI features a top navigation bar with 5 specialized tabs:

1. **[1] Overview Tab**:
   - Transaction ID, Witness TXID, Version, Network.
   - Status indicators: SegWit status, BIP125 RBF signaling badge.
   - Consensus Metrics: Virtual Size, Total Weight, Serialized Size, Witness Savings.
   - Locktime breakdown: Block height / UNIX timestamp / Immediate execution status.
   - Confirmation status: Block height and confirmation count (when querying live networks).

2. **[2] Inputs Tab**:
   - Scrollable list of all inputs with previous OutPoint (`txid:vout`).
   - Sequence number, BIP68 relative locktime analysis.
   - Full witness item inspector showing inferred types (Schnorr signature, ECDSA signature, public key, control block).

3. **[3] Outputs Tab**:
   - Output allocation table: Index, Satoshis, BTC value, Script standard.
   - Recipient address derivation (Base58check for Legacy, Bech32/Bech32m for SegWit and Taproot).
   - OP_RETURN payload inspector: Automatic protocol decoding for Runes, Omni Layer, and OpenTimestamps.

4. **[4] Script Disassembly Tab**:
   - Complete script disassembly in Assembly (ASM) format.
   - Educational descriptions explaining what each opcode does.

5. **[5] Script VM Simulator Tab**:
   - Interactive Forth-like virtual machine running the selected output script.
   - Left pane: Step-by-step instruction log with descriptions.
   - Right pane: Active stack inspector displaying stack state before and after each opcode execution.

### Keyboard Navigation & Shortcuts

| Key | Action |
| :--- | :--- |
| `1` - `5` | Switch directly to Tab 1 through 5 |
| `Tab` | Cycle forward through tabs |
| `Down` / `j` | Move selection down to next input or output |
| `Up` / `k` | Move selection up to previous input or output |
| `Space` / `Right` | Step forward one instruction in the Script VM (Tab 5) |
| `Left` | Step backward one instruction in the Script VM (Tab 5) |
| `r` | Reset the Script VM simulation back to Step 1 (Tab 5) |
| `q` / `Esc` | Exit the TUI and restore terminal state |

---

## 4. Mode 3: Web Server & Interactive Dashboard

The Web mode runs an embedded HTTP server hosting an interactive single-page application with dark-mode styling and zero external frontend dependencies.

### Starting the Embedded Server

```bash
# Start server on default port (8080)
txplore serve

# Start server on custom port (e.g., 3000) for Mainnet
txplore serve --port 3000 --network bitcoin
```

Console confirmation:
```
============================================================
 txplore Web Dashboard Running!
 URL: http://localhost:3000
 Network: bitcoin
 REST Endpoints:
   GET  /api/tx/:txid
   POST /api/decode
   POST /api/simulate
============================================================
```

Open `http://localhost:3000` in your web browser.

### Visual Transaction DAG Flow Graph

The web dashboard renders a dynamic, interactive SVG Directed Acyclic Graph (DAG) visualizing transaction mechanics:
- **Input Nodes (Cyan)**: Displays OutPoints, sequence numbers, and spent values.
- **Central Transaction Node (Gold)**: Displays short TXID, virtual size, weight units, and total miner fee.
- **Output Nodes (Green)**: Displays recipient addresses, satoshi allocations, and script standards.
- **Data Carrier Nodes (Purple)**: Displays OP_RETURN protocol tags (e.g., Runes, Omni, Text).

### REST API Endpoints & Curl Examples

The web server exposes REST API endpoints for external integrations:

#### Health Check: `GET /api/health`
```bash
curl http://localhost:3000/api/health
# OK
```

#### Decode Transaction Hex: `POST /api/decode`
```bash
curl -X POST http://localhost:3000/api/decode \
  -H "Content-Type: application/json" \
  -d '{"raw_hex":"<RAW_HEX>"}'
```

#### Simulate Bitcoin Script: `POST /api/simulate`
```bash
curl -X POST http://localhost:3000/api/simulate \
  -H "Content-Type: application/json" \
  -d '{"script_hex":"515293","initial_stack":[]}'
```

---

## 5. Additional Tools: Script VM & PSBT Analyzer

### Direct Script Execution (`eval-script`)

Simulate Bitcoin Script execution without building a full transaction:

```bash
# Execute arithmetic script: 1 + 2 = 3
txplore eval-script "515293"

# Execute hash script with initial stack data
txplore eval-script "a8" --stack "68656c6c6f"
```

Output trace:
```
============================================================
 Bitcoin Script Stack VM Simulator
============================================================
Script Hex: 0x515293
Status:     SUCCESS (VALID)
------------------------------------------------------------
Step-by-Step Execution Trace:
[Step 01] OP_PUSHNUM_1             -> Pushed 1 (OP_1 / true) onto stack
         Stack After: [01]
[Step 02] OP_PUSHNUM_2             -> Pushed 2 onto stack
         Stack After: [01, 02]
[Step 03] OP_ADD                   -> Added 1 + 2 = 3 (OP_ADD)
         Stack After: [03]
------------------------------------------------------------
Final Stack: [03]
============================================================
```

### Partially Signed Bitcoin Transactions (`analyze-psbt`)

Analyze unfinalized BIP174/BIP370 PSBTs from Base64 or Hex:

```bash
txplore analyze-psbt <BASE64_OR_HEX_PSBT>
```

Displays:
- Number of inputs and outputs.
- Signatures present vs signatures required.
- Finalization status.
- Estimated fee and fee rate (sat/vB).
- Per-input UTXO completeness, sighash types, and taproot script trees.

---

## 6. Curated Test Vectors

Here are real-world Bitcoin transaction test vectors to explore:

### Vector 1: First Bitcoin Transfer (Satoshi to Hal Finney, Block 170)
- **Type**: Legacy P2PKH / Bare Pubkey transfer
- **TXID**: `f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16`
- **Command**:
```bash
txplore inspect 0100000001c997a5e56e104102fa209c6a852dd90660a20b2d01514e87e377753130188723000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c218d40b55ac0d94a22793adc1dee17eb9c80d76aac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000
```

### Vector 2: Genesis Block Coinbase
- **Type**: Genesis Block Reward (50 BTC)
- **TXID**: `4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b`
- **Command**:
```bash
txplore inspect 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b --network bitcoin
```

### Vector 3: SegWit v0 Native P2WPKH Transfer
- **Command**:
```bash
txplore inspect 020000000100000000000000000000000000000000000000000000000000000000000000010000000000fdffffff01207f010000000000160014798939634d0b13488f28fa6eb722c6081da2cf3f024730440220677df982845c43d2c943df4d8525b6a70e7e17ae93566ea7247a32e185c8801902206085a73e654eb640e79fb4ab40989326e5e8e7a6ad4256ebaa5a5c6579c29d0f012103f679dc6eb949cefc9529deeeeaae6e4ee64bfd4e1bfba1659ca2a49aa5231c6a00000000
```
- **Inspect**: Note the `SEGWIT ACTIVE` and `BIP125 RBF Signaling` badges, the witness discount calculation, and Bech32 address generation (`bc1q...`).

### Vector 4: Taproot P2TR Key-Path Spend
- **Command**:
```bash
txplore inspect 020000000100000000000000000000000000000000000000000000000000000000000000020000000000ffffffff0140420f0000000000225120a60869f0dbcf1dc659c9cecbaf8050135ea9e8cdc487053f1dc6d60f4e0f3d64014028c25785a9df67d60e7e0e7a2b9d033efb573a908eb5501867160914eec89e9f905cffecba32d02a5a54db68d4f0d611894a9a0dc1b4aa634d9a4897087e5b0200000000
```
- **Inspect**: Note the 64-byte Schnorr signature identification in the witness stack and P2TR Bech32m address.
