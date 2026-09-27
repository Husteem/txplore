# Capstone Project Presentation: txplore

**Project Title**: `txplore`: High-Performance Bitcoin Transaction Explorer, Script Disassembler & Stack VM Simulator  
**Author**: Naim Hussain (Discord: `@husteemah`)  
**Cohort**: Rust for Bitcoin 2.0  
**Repository**: `https://github.com/Husteem/txplore`  

---

## 1. Project Overview & Motivation

### The Problem
Most blockchain explorers (Mempool, Blockstream, Blockchain.com) cater to end-users: they show whether a transaction confirmed, total fees, and fiat approximations. However, for Bitcoin engineers, protocol researchers, and security auditors:
- Standard explorers hide consensus details (e.g. byte-by-byte witness discounts, BIP68 sequence encoding masks, locktime types).
- Script disassembly is superficial, lacking explanation of opcode semantics.
- Complex scripts (Multisig, Miniscript, Lightning HTLCs, Taproot trees) cannot be dynamically tested or stepped through.
- Relying on web explorers leaks query IP addresses and financial metadata.

### The Solution: txplore
`txplore` is a standalone, local-first Bitcoin transaction inspection tool written in safe Rust. It offers:
1. **Consensus-Exact Metrics**: Raw bytes, Weight Units, Virtual Size, and Witness Discount ratio.
2. **Deep Script Analysis**: Full opcode breakdown, educational explanations, standard classification (P2PKH through Taproot P2TR and BIP388 P2A), and OP_RETURN protocol detection (Runes, Omni, OTS).
3. **Step-by-Step Script Stack VM**: An interactive Forth stack engine that executes scripts, visualizes stack mutations, and tests branch conditions.
4. **BIP174/BIP370 PSBT Audit**: Inspects unfinalized transactions, signature readiness, and fee rate.
5. **Universal Interfaces**: Single-command CLI, formatted tables, GitHub-ready Markdown reports, Mermaid DAG diagrams, full-screen Terminal UI (Ratatui), and an embedded Web Dashboard (Axum) featuring dynamic SVG DAG flow graphs.

---

## 2. Architecture & Technical Highlights

### Modern Idiomatic Rust Architecture
- **Consensus Decoding (`bitcoin` v0.32)**: Robust zero-copy consensus deserialization and strict standard validation.
- **Pluggable Fetching Engine**: Seamlessly switches between air-gapped offline hex strings/files, direct Bitcoin Core JSON-RPC (`bitcoincore-rpc`), and public Esplora REST endpoints (`reqwest`).
- **Interactive Stack VM**: Designed with an explicit state machine tracking execution condition stacks (for `OP_IF`/`OP_ELSE`/`OP_ENDIF`), primary stack, and alt stack with full arithmetic and cryptographic opcode support.
- **Dual UI Layer**:
  - *Ratatui TUI*: Ultra-responsive terminal experience with 5 tabs and keyboard controls.
  - *Axum Web Dashboard*: Zero external assets, embedded dark-mode UI with interactive SVG transaction DAG flow graph.

---

## 3. Five-Minute Live Demo Script (For Cohort Mentors)

### Minute 1: The Pitch & Offline Inspection
> "Good day mentors and fellow builders. Today I am presenting txplore, a developer-centric Bitcoin transaction explorer and Script VM debugger written in Rust.
> Let us start with an offline inspection of a raw transaction hex string without touching the internet."

```bash
# Run CLI inspection with input values for instant fee calculation
txplore inspect 0200000001... --input-values 100000
```
*Highlight*: Show the clean terminal table output, the witness discount calculation (e.g. 48.2% fee savings), locktime analysis, and protocol classification.

### Minute 2: Markdown & Mermaid Diagram Export
> "When preparing security audit reports or documenting transaction architectures, engineers need visual diagrams and technical reports."

```bash
# Generate visual Mermaid diagram
txplore inspect 0200000001... --mermaid

# Generate complete technical audit report in Markdown
txplore inspect 0200000001... --markdown > audit_report.md
```
*Highlight*: Point out how the Mermaid graph accurately depicts inputs pointing into the transaction node, splitting into recipient outputs and miner fees.

### Minute 3: The Interactive Script Stack VM Simulator
> "Next, let us look at the core differentiator: our Bitcoin Script Stack VM Simulator. Let us evaluate an arithmetic and hashing script directly."

```bash
txplore eval-script "OP_10 OP_20 OP_ADD OP_SHA256"
```
*Highlight*: Walk through the step-by-step trace showing the initial stack, step indices, opcodes, stack before and after, and final success status.

### Minute 4: The Ratatui Terminal Dashboard (TUI)
> "For power users in terminal environments, txplore includes a full-screen interactive TUI."

```bash
txplore tui 0200000001...
```
*Highlight*: Use `Tab` to navigate through:
- Tab 1: Overview
- Tab 2: Inputs (demonstrating OutPoints, sequence numbers, and relative locktimes)
- Tab 3: Outputs (demonstrating address derivation and script types)
- Tab 4: Script Disassembly (demonstrating opcode explanations)
- Tab 5: Stack VM Simulator (press `Space` to step through instructions live)

### Minute 5: The Axum Web Server & Dynamic SVG Flow Graph
> "Finally, txplore can be launched as an embedded local web server."

```bash
txplore serve --port 3000
```
*Highlight*: Open `http://localhost:3000`. Show the dark-mode dashboard, paste a transaction hex, and show the interactive SVG transaction DAG flow graph dynamically rendered with satoshi flows and fee breakdown.

---

## 4. Key Learnings & Future Roadmap

### Technical Learnings from the Cohort
- Deep mastery of Bitcoin consensus serialization rules and weight calculation nuances.
- Understanding BIP68 relative locktimes and BIP125 opt-in replace-by-fee mechanisms.
- Taproot witness structure (Schnorr signature sizes, control blocks, script path vs key path).
- Managing async I/O in Rust with Tokio, Axum, and terminal event loops with Crossterm/Ratatui.

### Future Roadmap
- Miniscript compiler integration for policy analysis.
- Silent Payments (BIP352) detection and scanning.
- Hardware wallet PSBT signing flow integration directly inside the TUI.
