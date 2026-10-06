pub const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>txplore | Bitcoin Transaction Explorer & Script VM</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:ital,wght@0,300;0,400;0,500;0,600;0,700;1,400&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg: #09090b;
            --surface: #101114;
            --surface-elevated: #16181d;
            --surface-hover: #1c1f26;
            --border: #27272a;
            --border-subtle: #1e1e24;
            --border-focus: #52525b;
            --accent: #f59e0b;
            --accent-dim: #b45309;
            --green: #10b981;
            --green-dim: #064e3b;
            --blue: #38bdf8;
            --blue-dim: #0c4a6e;
            --purple: #c084fc;
            --red: #ef4444;
            --red-dim: #450a0a;
            --text: #fafafa;
            --text-secondary: #a1a1aa;
            --text-muted: #71717a;
            --text-dim: #52525b;
            --font-mono: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
        }

        * { margin: 0; padding: 0; box-sizing: border-box; }

        body {
            background-color: var(--bg);
            color: var(--text);
            font-family: var(--font-mono);
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            line-height: 1.5;
            -webkit-font-smoothing: antialiased;
        }

        /* Top Developer Nav */
        header {
            border-bottom: 1px solid var(--border);
            background: var(--surface);
            padding: 0.75rem 1.5rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
            position: sticky;
            top: 0;
            z-index: 50;
        }

        .brand {
            display: flex;
            align-items: center;
            gap: 0.6rem;
            text-decoration: none;
            color: var(--text);
            font-weight: 700;
            font-size: 0.95rem;
            letter-spacing: -0.02em;
        }
        .brand-dot {
            width: 7px;
            height: 7px;
            border-radius: 50%;
            background-color: var(--green);
            box-shadow: 0 0 6px var(--green);
        }
        .brand-sub {
            color: var(--text-dim);
            font-size: 0.75rem;
            font-weight: 400;
            margin-left: 0.2rem;
        }

        .header-meta {
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }

        .tag {
            font-size: 0.7rem;
            padding: 0.2rem 0.55rem;
            border-radius: 3px;
            border: 1px solid var(--border);
            background: var(--surface-elevated);
            color: var(--text-secondary);
            font-weight: 500;
            text-transform: uppercase;
            letter-spacing: 0.04em;
        }
        .tag-accent {
            border-color: rgba(245, 158, 11, 0.4);
            color: var(--accent);
            background: rgba(245, 158, 11, 0.08);
        }
        .tag-green {
            border-color: rgba(16, 185, 129, 0.4);
            color: var(--green);
            background: rgba(16, 185, 129, 0.08);
        }
        .tag-blue {
            border-color: rgba(56, 189, 248, 0.4);
            color: var(--blue);
            background: rgba(56, 189, 248, 0.08);
        }

        /* Search Section */
        .search-wrap {
            max-width: 1200px;
            margin: 1.5rem auto 0 auto;
            padding: 0 1.5rem;
            width: 100%;
        }

        .terminal-prompt-box {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 4px;
            padding: 0.4rem 0.5rem;
            display: flex;
            align-items: center;
            gap: 0.5rem;
            transition: border-color 0.15s ease;
        }
        .terminal-prompt-box:focus-within {
            border-color: var(--border-focus);
        }

        .prompt-symbol {
            color: var(--accent);
            font-size: 0.85rem;
            font-weight: 700;
            padding-left: 0.5rem;
            user-select: none;
        }

        input[type="text"] {
            flex: 1;
            background: transparent;
            border: none;
            color: var(--text);
            font-family: var(--font-mono);
            font-size: 0.85rem;
            padding: 0.4rem 0.25rem;
            outline: none;
        }
        input[type="text"]::placeholder {
            color: var(--text-dim);
        }

        .btn {
            background: var(--surface-elevated);
            color: var(--text);
            border: 1px solid var(--border);
            border-radius: 3px;
            font-family: var(--font-mono);
            font-size: 0.75rem;
            font-weight: 600;
            padding: 0.45rem 0.9rem;
            cursor: pointer;
            transition: all 0.15s ease;
            white-space: nowrap;
        }
        .btn:hover {
            background: var(--surface-hover);
            border-color: var(--border-focus);
        }
        .btn-primary {
            background: var(--accent);
            color: #000;
            border-color: var(--accent);
            font-weight: 700;
        }
        .btn-primary:hover {
            background: #fbbf24;
            border-color: #fbbf24;
        }

        /* Preset Chips */
        .preset-row {
            display: flex;
            flex-wrap: wrap;
            gap: 0.4rem;
            margin-top: 0.6rem;
            align-items: center;
        }
        .preset-label {
            font-size: 0.7rem;
            color: var(--text-dim);
            text-transform: uppercase;
            letter-spacing: 0.05em;
            margin-right: 0.25rem;
        }
        .chip {
            background: transparent;
            border: 1px solid var(--border-subtle);
            color: var(--text-secondary);
            font-family: var(--font-mono);
            font-size: 0.7rem;
            padding: 0.2rem 0.5rem;
            border-radius: 3px;
            cursor: pointer;
            transition: all 0.15s ease;
        }
        .chip:hover {
            border-color: var(--border-focus);
            color: var(--text);
            background: var(--surface-elevated);
        }

        /* Developer Notification Banner */
        .banner {
            display: none;
            max-width: 1200px;
            margin: 1rem auto 0 auto;
            padding: 0.75rem 1rem;
            border-radius: 4px;
            font-size: 0.8rem;
            width: calc(100% - 3rem);
            align-items: center;
            justify-content: space-between;
            border: 1px solid var(--border);
        }
        .banner-error {
            background: var(--red-dim);
            border-color: rgba(239, 68, 68, 0.4);
            color: #fca5a5;
        }
        .banner-info {
            background: var(--blue-dim);
            border-color: rgba(56, 189, 248, 0.4);
            color: #bae6fd;
        }
        .banner-success {
            background: var(--green-dim);
            border-color: rgba(16, 185, 129, 0.4);
            color: #a7f3d0;
        }
        .banner-close {
            background: transparent;
            border: none;
            color: inherit;
            cursor: pointer;
            font-family: var(--font-mono);
            font-size: 0.8rem;
            padding-left: 0.75rem;
        }

        /* Main Workspace */
        .workspace {
            max-width: 1200px;
            margin: 1.25rem auto 3rem auto;
            padding: 0 1.5rem;
            width: 100%;
            flex: 1;
        }

        .panel {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 4px;
            padding: 1.25rem;
            margin-bottom: 1.25rem;
        }

        .panel-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 1rem;
            padding-bottom: 0.75rem;
            border-bottom: 1px solid var(--border-subtle);
        }
        .panel-title {
            font-size: 0.8rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-secondary);
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }

        /* Metric Grid */
        .metrics-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
            gap: 0.75rem;
        }
        .metric-cell {
            background: var(--surface-elevated);
            border: 1px solid var(--border-subtle);
            border-radius: 3px;
            padding: 0.75rem;
        }
        .metric-label {
            font-size: 0.68rem;
            color: var(--text-dim);
            text-transform: uppercase;
            letter-spacing: 0.05em;
            margin-bottom: 0.25rem;
        }
        .metric-value {
            font-size: 1.05rem;
            font-weight: 600;
            color: var(--text);
            word-break: break-all;
        }
        .metric-value-accent { color: var(--accent); }
        .metric-value-green { color: var(--green); }
        .metric-value-blue { color: var(--blue); }

        /* TXID Hash Header */
        .hash-display {
            display: flex;
            align-items: center;
            justify-content: space-between;
            flex-wrap: wrap;
            gap: 0.5rem;
            padding-bottom: 0.75rem;
            margin-bottom: 0.75rem;
            border-bottom: 1px solid var(--border-subtle);
        }
        .hash-title {
            font-size: 0.75rem;
            color: var(--text-dim);
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }
        .hash-string {
            font-size: 0.88rem;
            color: var(--text);
            font-weight: 600;
            word-break: break-all;
        }
        .copy-btn {
            background: var(--surface-elevated);
            border: 1px solid var(--border);
            color: var(--text-secondary);
            padding: 0.2rem 0.5rem;
            font-size: 0.7rem;
            border-radius: 3px;
            cursor: pointer;
            font-family: var(--font-mono);
        }
        .copy-btn:hover { color: var(--text); border-color: var(--border-focus); }

        /* DAG Circuit Flow */
        .dag-viewport {
            width: 100%;
            height: 380px;
            background-color: #0b0c0e;
            background-image: radial-gradient(#1f2026 1px, transparent 1px);
            background-size: 14px 14px;
            border: 1px solid var(--border-subtle);
            border-radius: 3px;
            overflow: hidden;
            position: relative;
        }

        /* Tabbed Workspace */
        .tabs-bar {
            display: flex;
            border-bottom: 1px solid var(--border);
            gap: 0.25rem;
            background: var(--surface);
            padding: 0 0.5rem;
            overflow-x: auto;
        }
        .tab-item {
            background: transparent;
            color: var(--text-muted);
            border: none;
            padding: 0.7rem 1rem;
            font-family: var(--font-mono);
            font-size: 0.78rem;
            font-weight: 500;
            cursor: pointer;
            border-bottom: 2px solid transparent;
            transition: all 0.15s ease;
            white-space: nowrap;
        }
        .tab-item:hover {
            color: var(--text);
        }
        .tab-item.active {
            color: var(--accent);
            border-bottom-color: var(--accent);
            background: rgba(245, 158, 11, 0.04);
        }

        .tab-pane {
            display: none;
            padding: 1rem 0 0 0;
        }
        .tab-pane.active {
            display: block;
        }

        /* Tabular Data */
        table {
            width: 100%;
            border-collapse: collapse;
            font-size: 0.78rem;
        }
        th {
            text-align: left;
            padding: 0.6rem 0.75rem;
            border-bottom: 1px solid var(--border);
            color: var(--text-dim);
            font-weight: 600;
            text-transform: uppercase;
            font-size: 0.68rem;
            letter-spacing: 0.05em;
        }
        td {
            padding: 0.65rem 0.75rem;
            border-bottom: 1px solid var(--border-subtle);
            color: var(--text-secondary);
            vertical-align: middle;
        }
        tr:hover td {
            background: var(--surface-elevated);
            color: var(--text);
        }

        .op-chip {
            display: inline-block;
            background: #151b26;
            color: var(--blue);
            padding: 0.15rem 0.4rem;
            border-radius: 2px;
            font-size: 0.72rem;
            font-weight: 600;
            border: 1px solid #1e293b;
            margin-right: 0.25rem;
        }

        /* Script VM Simulator */
        .vm-box {
            background: var(--surface-elevated);
            border: 1px solid var(--border-subtle);
            border-radius: 3px;
            padding: 1rem;
        }
        .vm-controls {
            display: flex;
            gap: 0.5rem;
            margin-bottom: 1rem;
            flex-wrap: wrap;
        }
        .stack-display {
            background: #0b0c0e;
            border: 1px solid var(--border);
            border-radius: 3px;
            padding: 0.75rem;
            min-height: 50px;
            display: flex;
            align-items: center;
            gap: 0.4rem;
            overflow-x: auto;
            margin-bottom: 1rem;
        }
        .stack-item {
            background: var(--surface-hover);
            border: 1px solid var(--border-focus);
            color: var(--text);
            padding: 0.3rem 0.6rem;
            border-radius: 2px;
            font-size: 0.75rem;
            white-space: nowrap;
        }

        /* Code & Pre */
        pre {
            background: #0b0c0e;
            border: 1px solid var(--border-subtle);
            border-radius: 3px;
            padding: 1rem;
            font-family: var(--font-mono);
            font-size: 0.75rem;
            color: #d4d4d8;
            overflow-x: auto;
            max-height: 500px;
        }

        /* Footer */
        footer {
            border-top: 1px solid var(--border);
            padding: 1.25rem 1.5rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
            font-size: 0.72rem;
            color: var(--text-dim);
            background: var(--surface);
        }
        footer a { color: var(--text-secondary); text-decoration: none; }
        footer a:hover { color: var(--accent); }
    </style>
</head>
<body>

    <header>
        <div class="brand">
            <span class="brand-dot"></span>
            <span>txplore</span>
            <span class="brand-sub">/ bitcoin explorer & stack vm</span>
        </div>
        <div class="header-meta">
            <span class="tag tag-accent" id="netTag">MAINNET</span>
            <span class="tag tag-green">MIRROR FAILOVER</span>
            <span class="tag">v0.1.0</span>
            <a href="https://github.com/Husteem/txplore" target="_blank" class="tag" style="text-decoration:none;">GITHUB</a>
        </div>
    </header>

    <div class="search-wrap">
        <div class="terminal-prompt-box">
            <span class="prompt-symbol">$ txplore inspect</span>
            <input type="text" id="txInput" spellcheck="false" placeholder="Paste 64-char TXID, raw hex, or PSBT..." />
            <button class="btn btn-primary" id="inspectBtn" onclick="analyzeTransaction()">INSPECT ↵</button>
            <button class="btn" onclick="clearInput()">CLEAR</button>
        </div>

        <div class="preset-row">
            <span class="preset-label">Test Vectors:</span>
            <button class="chip" onclick="loadPreset('live_mainnet')">Live Mainnet (bbfa1951...)</button>
            <button class="chip" onclick="loadPreset('hal_finney')">Hal Finney (Block 170)</button>
            <button class="chip" onclick="loadPreset('genesis')">Genesis Coinbase</button>
            <button class="chip" onclick="loadPreset('segwit_v0')">Native SegWit P2WPKH</button>
            <button class="chip" onclick="loadPreset('taproot')">Taproot Key-Path Spend</button>
            <button class="chip" onclick="loadPreset('runes')">OP_RETURN Runes Carrier</button>
        </div>
    </div>

    <!-- In-App Notification Banner (Replaces harsh browser alert) -->
    <div class="banner" id="statusBanner">
        <span id="bannerText"></span>
        <button class="banner-close" onclick="dismissBanner()">✕</button>
    </div>

    <div class="workspace" id="mainWorkspace">

        <!-- Spec Header & Overview -->
        <div class="panel">
            <div class="hash-display">
                <div>
                    <div class="hash-title">TRANSACTION IDENTIFIER</div>
                    <div class="hash-string" id="txidLabel">Loading...</div>
                </div>
                <div style="display:flex; gap:0.4rem; align-items:center;">
                    <button class="copy-btn" onclick="copyTxid()">COPY TXID</button>
                    <div id="statusBadges" style="display:flex; gap:0.4rem;"></div>
                </div>
            </div>

            <div class="metrics-grid">
                <div class="metric-cell">
                    <div class="metric-label">VIRTUAL SIZE</div>
                    <div class="metric-value" id="vsizeVal">-</div>
                </div>
                <div class="metric-cell">
                    <div class="metric-label">TOTAL WEIGHT</div>
                    <div class="metric-value" id="weightVal">-</div>
                </div>
                <div class="metric-cell">
                    <div class="metric-label">MINER FEE</div>
                    <div class="metric-value metric-value-green" id="feeVal">-</div>
                </div>
                <div class="metric-cell">
                    <div class="metric-label">EFFECTIVE FEE RATE</div>
                    <div class="metric-value metric-value-accent" id="feeRateVal">-</div>
                </div>
                <div class="metric-cell">
                    <div class="metric-label">CONFIRMATIONS</div>
                    <div class="metric-value metric-value-blue" id="confVal">-</div>
                </div>
                <div class="metric-cell">
                    <div class="metric-label">VERSION / LOCKTIME</div>
                    <div class="metric-value" id="verLockVal">-</div>
                </div>
            </div>
        </div>

        <!-- Directed Acyclic Graph (DAG) -->
        <div class="panel">
            <div class="panel-header">
                <span class="panel-title">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="6" cy="6" r="3"></circle>
                        <circle cx="18" cy="18" r="3"></circle>
                        <path d="M8.5 8.5l7 7"></path>
                    </svg>
                    TRANSACTION FLOW CIRCUIT (DAG)
                </span>
                <span style="font-size:0.7rem; color:var(--text-dim);">INPUTS (CYAN) → CORE (GOLD) → OUTPUTS (GREEN)</span>
            </div>
            <div class="dag-viewport">
                <svg id="dagSvg" width="100%" height="100%"></svg>
            </div>
        </div>

        <!-- Tabbed Developer Workspace -->
        <div class="panel" style="padding: 0;">
            <div class="tabs-bar">
                <button class="tab-item active" onclick="setTab('tab-io')">[1] INPUTS & OUTPUTS</button>
                <button class="tab-item" onclick="setTab('tab-vm')">[2] STACK VM SIMULATOR</button>
                <button class="tab-item" onclick="setTab('tab-asm')">[3] SCRIPT DISASSEMBLY</button>
                <button class="tab-item" onclick="setTab('tab-bytes')">[4] CONSENSUS BYTE MAP</button>
                <button class="tab-item" onclick="setTab('tab-export')">[5] EXPORT & REPORTS</button>
            </div>

            <div style="padding: 1.25rem;">
                <!-- Tab 1: Inputs & Outputs -->
                <div class="tab-pane active" id="tab-io">
                    <h4 style="font-size:0.75rem; text-transform:uppercase; color:var(--text-dim); margin-bottom:0.75rem;">INPUTS (<span id="inCount">0</span>)</h4>
                    <div style="overflow-x:auto; margin-bottom:1.5rem;">
                        <table id="inputsTable">
                            <thead>
                                <tr>
                                    <th>#</th>
                                    <th>PREV OUTPOINT</th>
                                    <th>VALUE (SATS)</th>
                                    <th>SEQUENCE / RBF</th>
                                    <th>WITNESS STACK</th>
                                </tr>
                            </thead>
                            <tbody id="inputsBody"></tbody>
                        </table>
                    </div>

                    <h4 style="font-size:0.75rem; text-transform:uppercase; color:var(--text-dim); margin-bottom:0.75rem;">OUTPUTS (<span id="outCount">0</span>)</h4>
                    <div style="overflow-x:auto;">
                        <table id="outputsTable">
                            <thead>
                                <tr>
                                    <th>#</th>
                                    <th>VALUE (SATS / BTC)</th>
                                    <th>STANDARD</th>
                                    <th>RECIPIENT ADDRESS / PAYLOAD</th>
                                    <th>SCRIPTPUBKEY ASM</th>
                                </tr>
                            </thead>
                            <tbody id="outputsBody"></tbody>
                        </table>
                    </div>
                </div>

                <!-- Tab 2: Stack VM Simulator -->
                <div class="tab-pane" id="tab-vm">
                    <div class="vm-box">
                        <div class="vm-controls">
                            <button class="btn btn-primary" onclick="stepVm()">STEP FORWARD (SPACE)</button>
                            <button class="btn" onclick="prevVm()">STEP BACK</button>
                            <button class="btn" onclick="runAllVm()">EXECUTE ALL</button>
                            <button class="btn" onclick="resetVm()">RESET VM</button>
                            <span style="font-size:0.75rem; color:var(--text-dim); margin-left:auto; align-self:center;" id="vmStepCounter">Step 0 / 0</span>
                        </div>

                        <div style="font-size:0.7rem; color:var(--text-dim); margin-bottom:0.4rem; text-transform:uppercase;">ACTIVE STACK STATE:</div>
                        <div class="stack-display" id="vmStackView">
                            <span style="color:var(--text-dim); font-size:0.75rem;">[ Stack is currently empty ]</span>
                        </div>

                        <div style="font-size:0.7rem; color:var(--text-dim); margin-bottom:0.4rem; text-transform:uppercase;">EXECUTION TRACE LOG:</div>
                        <div id="vmLog" style="display:flex; flex-direction:column; gap:0.4rem; max-height:280px; overflow-y:auto;"></div>
                    </div>
                </div>

                <!-- Tab 3: Script Disassembly -->
                <div class="tab-pane" id="tab-asm">
                    <div id="disassemblyList" style="display:flex; flex-direction:column; gap:1rem;"></div>
                </div>

                <!-- Tab 4: Consensus Byte Map -->
                <div class="tab-pane" id="tab-bytes">
                    <div style="margin-bottom:1rem; font-size:0.75rem; color:var(--text-muted);">
                        Raw serialized transaction bytes decoded into consensus boundaries (BIP141 / BIP144).
                    </div>
                    <div id="byteMapContainer" style="line-height:2; word-break:break-all; font-size:0.8rem;"></div>
                </div>

                <!-- Tab 5: Exports & Reports -->
                <div class="tab-pane" id="tab-export">
                    <div style="display:flex; gap:0.5rem; margin-bottom:1rem;">
                        <button class="btn" onclick="copyExport('json')">COPY JSON</button>
                        <button class="btn" onclick="copyExport('markdown')">COPY MARKDOWN AUDIT</button>
                        <button class="btn" onclick="copyExport('mermaid')">COPY MERMAID DIAGRAM</button>
                    </div>
                    <pre id="jsonPreview">// Transaction JSON will appear here...</pre>
                </div>
            </div>
        </div>

    </div>

    <footer>
        <div>txplore v0.1.0 • Built with Rust (rust-bitcoin 0.32, Axum, Ratatui)</div>
        <div>Author: <code>Naim Hussain (@husteemah)</code></div>
    </footer>

    <script>
        let currentTx = null;
        let vmTrace = null;
        let currentVmStep = 0;

        function showBanner(msg, type = 'info') {
            const b = document.getElementById('statusBanner');
            const t = document.getElementById('bannerText');
            b.className = 'banner banner-' + type;
            b.style.display = 'flex';
            t.innerText = msg;
        }

        function dismissBanner() {
            document.getElementById('statusBanner').style.display = 'none';
        }

        function clearInput() {
            document.getElementById('txInput').value = '';
            document.getElementById('txInput').focus();
        }

        function copyTxid() {
            if (currentTx && currentTx.txid) {
                navigator.clipboard.writeText(currentTx.txid);
                showBanner('Copied TXID to clipboard: ' + currentTx.txid, 'success');
            }
        }

        async function analyzeTransaction() {
            const input = document.getElementById('txInput').value.trim();
            if (!input) return;

            showBanner('Querying transaction from Esplora mirrors (mempool.space / blockstream.info)...', 'info');

            try {
                let res;
                if (input.length === 64 && /^[0-9a-fA-F]+$/.test(input)) {
                    res = await fetch(`/api/tx/${input}`);
                } else {
                    res = await fetch('/api/decode', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ hex: input, network: 'bitcoin' })
                    });
                }

                if (!res.ok) {
                    const errText = await res.text();
                    showBanner('[✗] Analysis Error: ' + errText, 'error');
                    return;
                }

                currentTx = await res.json();
                dismissBanner();
                renderAll(currentTx);
            } catch (err) {
                showBanner('[✗] Network Connection Failure: ' + err.message, 'error');
            }
        }

        function renderAll(tx) {
            document.getElementById('txidLabel').innerText = tx.txid;

            // Badges
            const badgeContainer = document.getElementById('statusBadges');
            badgeContainer.innerHTML = '';
            if (tx.is_segwit) badgeContainer.innerHTML += '<span class="tag tag-green">SEGWIT ACTIVE</span>';
            if (tx.bip69_compliant) badgeContainer.innerHTML += '<span class="tag tag-blue">BIP69 COMPLIANT</span>';
            if (tx.has_rbf_signaling) badgeContainer.innerHTML += '<span class="tag tag-accent">RBF SIGNALING</span>';

            // Metrics
            document.getElementById('vsizeVal').innerText = `${tx.vsize_vb.toLocaleString()} vB`;
            document.getElementById('weightVal').innerText = `${tx.weight_wu.toLocaleString()} WU`;
            
            const feeSats = tx.fee_sats !== null ? `${tx.fee_sats.toLocaleString()} sats` : 'Unknown';
            document.getElementById('feeVal').innerText = feeSats;

            const feeRate = tx.fee_rate_sat_per_vb !== null ? `${tx.fee_rate_sat_per_vb.toFixed(1)} sat/vB` : 'Unknown';
            document.getElementById('feeRateVal').innerText = feeRate;

            if (tx.confirmation_info && tx.confirmation_info.confirmed) {
                const height = tx.confirmation_info.block_height || 'Block';
                document.getElementById('confVal').innerText = `CONFIRMED #${height}`;
            } else {
                document.getElementById('confVal').innerText = 'MEMPOOL (0 CONF)';
            }

            document.getElementById('verLockVal').innerText = `v${tx.version} / Lock: ${tx.locktime}`;

            // Tables
            renderTables(tx);
            renderDag(tx);
            renderDisassembly(tx);
            renderByteMap(tx);
            initVm(tx);

            document.getElementById('jsonPreview').innerText = JSON.stringify(tx, null, 2);
        }

        function renderTables(tx) {
            document.getElementById('inCount').innerText = tx.inputs.length;
            document.getElementById('outCount').innerText = tx.outputs.length;

            const inBody = document.getElementById('inputsBody');
            inBody.innerHTML = '';
            tx.inputs.forEach(i => {
                const val = i.spent_txout ? `${i.spent_txout.value_sats.toLocaleString()} sats` : '<span style="color:var(--text-dim)">Unknown (Offline)</span>';
                const witnessDesc = i.witness.length ? `${i.witness.length} witness items` : (i.script_sig ? i.script_sig.asm : 'None');
                inBody.innerHTML += `
                    <tr>
                        <td><strong>${i.index}</strong></td>
                        <td><code>${i.prevout_txid.substring(0,8)}...${i.prevout_txid.substring(56)}:${i.prevout_vout}</code></td>
                        <td>${val}</td>
                        <td><code>0x${i.sequence.toString(16)}</code></td>
                        <td><span style="font-size:0.72rem; color:var(--text-secondary);">${witnessDesc}</span></td>
                    </tr>
                `;
            });

            const outBody = document.getElementById('outputsBody');
            outBody.innerHTML = '';
            tx.outputs.forEach(o => {
                const btc = (o.value_sats / 100000000).toFixed(8);
                const addr = o.address || (o.op_return_payload ? `<span style="color:var(--purple);">OP_RETURN [${o.op_return_payload.protocol}]</span>` : 'Non-Standard');
                outBody.innerHTML += `
                    <tr>
                        <td><strong>${o.index}</strong></td>
                        <td><span style="color:var(--green); font-weight:600;">${o.value_sats.toLocaleString()} sats</span> <span style="font-size:0.68rem; color:var(--text-dim);">(${btc} BTC)</span></td>
                        <td><span class="tag tag-blue">${o.script_type}</span></td>
                        <td><code>${addr}</code></td>
                        <td><code style="font-size:0.72rem; color:var(--text-dim);">${o.script_pubkey.asm.substring(0, 40)}${o.script_pubkey.asm.length > 40 ? '...' : ''}</code></td>
                    </tr>
                `;
            });
        }

        function renderDag(tx) {
            const svg = document.getElementById('dagSvg');
            svg.innerHTML = '';
            const width = svg.clientWidth || 1100;
            const height = 380;
            const midY = height / 2;
            const txX = width / 2 - 90;
            const txY = midY - 45;

            // Draw Core Box
            svg.innerHTML += `
                <rect x="${txX}" y="${txY}" width="180" height="90" rx="3" fill="#14151a" stroke="#f59e0b" stroke-width="1.5"/>
                <text x="${txX + 90}" y="${txY + 24}" fill="#f59e0b" font-weight="700" font-size="11" text-anchor="middle" font-family="JetBrains Mono">TX CORE</text>
                <text x="${txX + 90}" y="${txY + 44}" fill="#fafafa" font-size="10" text-anchor="middle" font-family="JetBrains Mono">${tx.txid.substring(0,6)}...${tx.txid.substring(58)}</text>
                <text x="${txX + 90}" y="${txY + 62}" fill="#a1a1aa" font-size="9" text-anchor="middle" font-family="JetBrains Mono">${tx.vsize_vb.toLocaleString()} vB • ${tx.weight_wu.toLocaleString()} WU</text>
                <text x="${txX + 90}" y="${txY + 77}" fill="#10b981" font-size="9" text-anchor="middle" font-family="JetBrains Mono">${tx.fee_rate_sat_per_vb ? tx.fee_rate_sat_per_vb.toFixed(1) + ' sat/vB' : ''}</text>
            `;

            // Draw Inputs (Left)
            const numIn = Math.min(tx.inputs.length, 6);
            for (let i = 0; i < numIn; i++) {
                const input = tx.inputs[i];
                const inY = (height / (numIn + 1)) * (i + 1);
                const inX = 30;

                svg.innerHTML += `
                    <path d="M ${inX + 170} ${inY} C ${txX - 40} ${inY}, ${txX - 40} ${midY}, ${txX} ${midY}" fill="none" stroke="#38bdf8" stroke-width="1.5" stroke-dasharray="3"/>
                    <rect x="${inX}" y="${inY - 20}" width="170" height="40" rx="3" fill="#10131a" stroke="#38bdf8" stroke-width="1"/>
                    <text x="${inX + 8}" y="${inY - 5}" fill="#38bdf8" font-size="10" font-weight="600" font-family="JetBrains Mono">IN #${input.index}</text>
                    <text x="${inX + 8}" y="${inY + 10}" fill="#a1a1aa" font-size="8" font-family="JetBrains Mono">${input.prevout_txid.substring(0,6)}...:${input.prevout_vout}</text>
                `;
            }

            // Draw Outputs (Right)
            const numOut = Math.min(tx.outputs.length, 6);
            for (let i = 0; i < numOut; i++) {
                const output = tx.outputs[i];
                const outY = (height / (numOut + 1)) * (i + 1);
                const outX = width - 200;
                const isOpReturn = output.script_type === 'OP_RETURN' || !!output.op_return_payload;
                const strokeColor = isOpReturn ? '#c084fc' : '#10b981';

                svg.innerHTML += `
                    <path d="M ${txX + 180} ${midY} C ${outX - 40} ${midY}, ${outX - 40} ${outY}, ${outX} ${outY}" fill="none" stroke="${strokeColor}" stroke-width="1.5"/>
                    <rect x="${outX}" y="${outY - 20}" width="170" height="40" rx="3" fill="#101412" stroke="${strokeColor}" stroke-width="1"/>
                    <text x="${outX + 8}" y="${outY - 5}" fill="${strokeColor}" font-size="10" font-weight="600" font-family="JetBrains Mono">OUT #${output.index} (${output.script_type})</text>
                    <text x="${outX + 8}" y="${outY + 10}" fill="#fafafa" font-size="9" font-family="JetBrains Mono">${output.value_sats.toLocaleString()} sats</text>
                `;
            }
        }

        function setTab(tabId) {
            document.querySelectorAll('.tab-item').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab-pane').forEach(p => p.classList.remove('active'));
            event.target.classList.add('active');
            document.getElementById(tabId).classList.add('active');
        }

        function renderDisassembly(tx) {
            const list = document.getElementById('disassemblyList');
            list.innerHTML = '';
            tx.outputs.forEach(o => {
                let ops = '';
                o.script_pubkey.opcodes.forEach(op => {
                    ops += `<div style="margin-bottom:0.3rem;"><span class="op-chip">${op.name}</span> <span style="font-size:0.75rem; color:var(--text-secondary);">${op.meaning}</span></div>`;
                });
                list.innerHTML += `
                    <div style="background:var(--surface-elevated); border:1px solid var(--border-subtle); padding:1rem; border-radius:3px;">
                        <div style="font-size:0.75rem; font-weight:700; color:var(--accent); margin-bottom:0.3rem;">OUTPUT #${o.index} — ${o.script_type}</div>
                        <div style="font-size:0.72rem; color:var(--text-dim); margin-bottom:0.6rem; word-break:break-all;">HEX: ${o.script_pubkey.hex}</div>
                        <div>${ops}</div>
                    </div>
                `;
            });
        }

        function renderByteMap(tx) {
            const container = document.getElementById('byteMapContainer');
            const hex = tx.raw_hex;
            if (!hex) {
                container.innerHTML = '<span style="color:var(--text-dim)">Raw hex not available for this query</span>';
                return;
            }
            // Colorized consensus segments
            let html = '';
            html += `<span style="background:rgba(56,189,248,0.2); color:#38bdf8; padding:2px 4px; border-radius:2px;" title="Version [4 Bytes]">${hex.substring(0,8)}</span>`;
            if (tx.is_segwit && hex.length > 12) {
                html += `<span style="background:rgba(245,158,11,0.2); color:#f59e0b; padding:2px 4px; border-radius:2px;" title="Marker & Flag [2 Bytes]">${hex.substring(8,12)}</span>`;
                html += `<span style="color:var(--text-secondary); padding:2px 4px;" title="Payload Chunks">${hex.substring(12, hex.length - 8)}</span>`;
            } else {
                html += `<span style="color:var(--text-secondary); padding:2px 4px;">${hex.substring(8, hex.length - 8)}</span>`;
            }
            html += `<span style="background:rgba(239,68,68,0.2); color:#ef4444; padding:2px 4px; border-radius:2px;" title="LockTime [4 Bytes]">${hex.substring(hex.length - 8)}</span>`;
            container.innerHTML = html;
        }

        async function initVm(tx) {
            if (!tx.outputs.length) return;
            const scriptHex = tx.outputs[0].script_pubkey.hex;
            try {
                const res = await fetch('/api/simulate', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ script_hex: scriptHex, initial_stack: [] })
                });
                if (res.ok) {
                    vmTrace = await res.json();
                    currentVmStep = 0;
                    updateVmDisplay();
                }
            } catch (e) {
                console.error(e);
            }
        }

        function updateVmDisplay() {
            if (!vmTrace) return;
            const steps = vmTrace.steps || [];
            document.getElementById('vmStepCounter').innerText = `Step ${currentVmStep} / ${steps.length}`;

            const stackView = document.getElementById('vmStackView');
            const logView = document.getElementById('vmLog');

            if (currentVmStep === 0) {
                stackView.innerHTML = '<span style="color:var(--text-dim); font-size:0.75rem;">[ Initial Stack: empty ]</span>';
                logView.innerHTML = '<div style="font-size:0.75rem; color:var(--text-dim);">Click Step Forward or Press Space to begin Forth VM execution.</div>';
                return;
            }

            const step = steps[currentVmStep - 1];
            stackView.innerHTML = '';
            if (step.stack_after.length === 0) {
                stackView.innerHTML = '<span style="color:var(--text-dim); font-size:0.75rem;">[ Empty Stack ]</span>';
            } else {
                step.stack_after.forEach(it => {
                    stackView.innerHTML += `<div class="stack-item">0x${it}</div>`;
                });
            }

            let logHtml = '';
            for (let i = 0; i < currentVmStep; i++) {
                const s = steps[i];
                logHtml += `
                    <div style="background:#111317; border-left:2px solid var(--accent); padding:0.4rem 0.6rem; font-size:0.75rem;">
                        <strong>[Step ${i + 1}]</strong> <span class="op-chip">${s.instruction}</span> ${s.description}
                    </div>
                `;
            }
            logView.innerHTML = logHtml;
        }

        function stepVm() {
            if (!vmTrace) return;
            if (currentVmStep < vmTrace.steps.length) {
                currentVmStep++;
                updateVmDisplay();
            }
        }

        function prevVm() {
            if (currentVmStep > 0) {
                currentVmStep--;
                updateVmDisplay();
            }
        }

        function runAllVm() {
            if (!vmTrace) return;
            currentVmStep = vmTrace.steps.length;
            updateVmDisplay();
        }

        function resetVm() {
            currentVmStep = 0;
            updateVmDisplay();
        }

        function copyExport(type) {
            if (!currentTx) return;
            let text = '';
            if (type === 'json') {
                text = JSON.stringify(currentTx, null, 2);
            } else if (type === 'markdown') {
                text = `# Bitcoin Transaction ${currentTx.txid}\n\n- Virtual Size: ${currentTx.vsize_vb} vB\n- Weight: ${currentTx.weight_wu} WU\n- Inputs: ${currentTx.inputs.length}\n- Outputs: ${currentTx.outputs.length}`;
            } else if (type === 'mermaid') {
                text = `graph LR\n  subgraph Inputs\n  end\n  TX[${currentTx.txid.substring(0,8)}]\n  subgraph Outputs\n  end`;
            }
            navigator.clipboard.writeText(text);
            showBanner(`Copied ${type.toUpperCase()} export to clipboard!`, 'success');
        }

        const presets = {
            live_mainnet: 'bbfa19515e20878bb4ae58fd33187e61689f8a52aa1840c847ebebc846503425',
            hal_finney: 'f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16',
            genesis: '4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b',
            segwit_v0: '020000000100000000000000000000000000000000000000000000000000000000000000010000000000fdffffff01207f010000000000160014798939634d0b13488f28fa6eb722c6081da2cf3f024730440220677df982845c43d2c943df4d8525b6a70e7e17ae93566ea7247a32e185c8801902206085a73e654eb640e79fb4ab40989326e5e8e7a6ad4256ebaa5a5c6579c29d0f012103f679dc6eb949cefc9529deeeeaae6e4ee64bfd4e1bfba1659ca2a49aa5231c6a00000000',
            taproot: '020000000100000000000000000000000000000000000000000000000000000000000000020000000000ffffffff0140420f0000000000225120a60869f0dbcf1dc659c9cecbaf8050135ea9e8cdc487053f1dc6d60f4e0f3d64014028c25785a9df67d60e7e0e7a2b9d033efb573a908eb5501867160914eec89e9f905cffecba32d02a5a54db68d4f0d611894a9a0dc1b4aa634d9a4897087e5b0200000000',
            runes: '020000000100000000000000000000000000000000000000000000000000000000000000010000000000ffffffff0200000000000000000d6a0b52554e45535f44454d4f0101d07e010000000000160014531260aa2a199e228c537dfa42c82bea2c7c1f4d00000000'
        };

        function loadPreset(key) {
            document.getElementById('txInput').value = presets[key];
            analyzeTransaction();
        }

        // Keyboard Shortcuts
        window.addEventListener('keydown', (e) => {
            if (e.key === '/' && document.activeElement !== document.getElementById('txInput')) {
                e.preventDefault();
                document.getElementById('txInput').focus();
            } else if (e.key === 'Escape') {
                dismissBanner();
            } else if (e.key === ' ' && document.getElementById('tab-vm').classList.contains('active')) {
                if (document.activeElement.tagName !== 'INPUT') {
                    e.preventDefault();
                    stepVm();
                }
            }
        });

        window.onload = () => {
            loadPreset('live_mainnet');
        };
    </script>
</body>
</html>
"##;
