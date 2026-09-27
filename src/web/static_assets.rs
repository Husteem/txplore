pub const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>txplore | Bitcoin Transaction Explorer</title>
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;600;700&family=Plus+Jakarta+Sans:wght@400;500;600;700;800&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg: #090d16;
            --surface: #111827;
            --surface-card: #182234;
            --border: #1f293d;
            --accent: #f59e0b;
            --accent-glow: rgba(245, 158, 11, 0.2);
            --primary: #38bdf8;
            --success: #10b981;
            --text-main: #f1f5f9;
            --text-muted: #94a3b8;
            --font-main: 'Plus Jakarta Sans', sans-serif;
            --font-mono: 'JetBrains Mono', monospace;
        }

        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            background-color: var(--bg);
            color: var(--text-main);
            font-family: var(--font-main);
            min-height: 100vh;
            display: flex;
            flex-direction: column;
        }

        header {
            border-bottom: 1px solid var(--border);
            background: rgba(17, 24, 39, 0.8);
            backdrop-filter: blur(12px);
            position: sticky;
            top: 0;
            z-index: 50;
            padding: 1rem 2rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }

        .logo {
            display: flex;
            align-items: center;
            gap: 0.75rem;
            font-weight: 800;
            font-size: 1.35rem;
            color: var(--accent);
            text-decoration: none;
        }

        .badge {
            display: inline-flex;
            align-items: center;
            padding: 0.25rem 0.65rem;
            border-radius: 9999px;
            font-size: 0.75rem;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }
        .badge-orange { background: rgba(245, 158, 11, 0.15); color: #fbbf24; border: 1px solid rgba(245, 158, 11, 0.3); }
        .badge-green { background: rgba(16, 185, 129, 0.15); color: #34d399; border: 1px solid rgba(16, 185, 129, 0.3); }
        .badge-blue { background: rgba(56, 189, 248, 0.15); color: #38bdf8; border: 1px solid rgba(56, 189, 248, 0.3); }

        .search-container {
            max-width: 900px;
            margin: 2rem auto 1rem auto;
            padding: 0 1.5rem;
            width: 100%;
        }

        .search-box {
            display: flex;
            gap: 0.5rem;
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 0.4rem;
            box-shadow: 0 8px 30px rgba(0,0,0,0.5);
        }

        input[type="text"] {
            flex: 1;
            background: transparent;
            border: none;
            color: var(--text-main);
            font-family: var(--font-mono);
            font-size: 0.95rem;
            padding: 0.6rem 1rem;
            outline: none;
        }

        button.btn {
            background: var(--accent);
            color: #000;
            border: none;
            font-weight: 700;
            font-size: 0.9rem;
            padding: 0.6rem 1.4rem;
            border-radius: 8px;
            cursor: pointer;
            transition: all 0.2s ease;
        }
        button.btn:hover { background: #d97706; transform: translateY(-1px); }

        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 1.5rem;
            flex: 1;
            width: 100%;
        }

        .card {
            background: var(--surface-card);
            border: 1px solid var(--border);
            border-radius: 14px;
            padding: 1.5rem;
            margin-bottom: 1.5rem;
            box-shadow: 0 4px 20px rgba(0,0,0,0.25);
        }

        .grid-4 {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
            gap: 1rem;
            margin-top: 1rem;
        }

        .stat-card {
            background: rgba(17, 24, 39, 0.6);
            border: 1px solid var(--border);
            border-radius: 10px;
            padding: 1rem;
        }
        .stat-label { font-size: 0.8rem; color: var(--text-muted); font-weight: 600; text-transform: uppercase; margin-bottom: 0.25rem; }
        .stat-value { font-size: 1.25rem; font-weight: 700; font-family: var(--font-mono); }

        .dag-canvas {
            width: 100%;
            height: 380px;
            background: rgba(10, 15, 26, 0.9);
            border: 1px solid var(--border);
            border-radius: 12px;
            overflow: hidden;
            position: relative;
        }

        .tabs-header {
            display: flex;
            gap: 0.5rem;
            border-bottom: 1px solid var(--border);
            margin-bottom: 1rem;
        }
        .tab-btn {
            background: transparent;
            color: var(--text-muted);
            border: none;
            padding: 0.75rem 1.25rem;
            font-weight: 600;
            cursor: pointer;
            border-bottom: 2px solid transparent;
            transition: all 0.2s;
        }
        .tab-btn.active {
            color: var(--accent);
            border-bottom-color: var(--accent);
        }

        table {
            width: 100%;
            border-collapse: collapse;
            font-family: var(--font-mono);
            font-size: 0.85rem;
        }
        th, td {
            text-align: left;
            padding: 0.75rem 1rem;
            border-bottom: 1px solid var(--border);
        }
        th { color: var(--text-muted); font-family: var(--font-main); font-weight: 600; }

        .opcode-chip {
            display: inline-block;
            background: #1e293b;
            color: #38bdf8;
            padding: 0.2rem 0.5rem;
            border-radius: 4px;
            font-size: 0.8rem;
            font-weight: 600;
            margin-right: 0.3rem;
            border: 1px solid #334155;
        }

        footer {
            border-top: 1px solid var(--border);
            padding: 1.5rem;
            text-align: center;
            font-size: 0.85rem;
            color: var(--text-muted);
        }
    </style>
</head>
<body>
    <header>
        <a href="/" class="logo">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <circle cx="12" cy="12" r="10"></circle>
                <path d="m9 12 2 2 4-4"></path>
            </svg>
            txplore
        </a>
        <div style="display:flex; gap:0.5rem; align-items:center;">
            <span class="badge badge-orange" id="networkBadge">Regtest</span>
            <span class="badge badge-green">v0.1.0 Capstone</span>
        </div>
    </header>

    <div class="search-container">
        <div class="search-box">
            <input type="text" id="txInput" placeholder="Enter Transaction ID (txid) or paste Raw Hex / PSBT..." />
            <button class="btn" onclick="analyzeTransaction()">Inspect</button>
        </div>
    </div>

    <div class="container" id="contentContainer">
        <!-- Overview Card -->
        <div class="card" id="overviewCard">
            <div style="display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:1rem;">
                <div>
                    <h2 style="font-size:1.4rem; font-weight:800;" id="txidTitle">Transaction Overview</h2>
                    <p style="color:var(--text-muted); font-family:var(--font-mono); font-size:0.85rem; word-break:break-all;" id="wtxidLabel"></p>
                </div>
                <div id="statusBadges" style="display:flex; gap:0.5rem;"></div>
            </div>

            <div class="grid-4">
                <div class="stat-card">
                    <div class="stat-label">Virtual Size (vsize)</div>
                    <div class="stat-value" id="vsizeVal">-</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">Total Weight</div>
                    <div class="stat-value" id="weightVal">-</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">Miner Fee</div>
                    <div class="stat-value" style="color:var(--success);" id="feeVal">-</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">Effective Fee Rate</div>
                    <div class="stat-value" style="color:var(--accent);" id="feeRateVal">-</div>
                </div>
            </div>
        </div>

        <!-- Visual Flow DAG Diagram -->
        <div class="card">
            <h3 style="margin-bottom:0.75rem; font-size:1.1rem; font-weight:700;">Transaction Flow Graph</h3>
            <div class="dag-canvas">
                <svg id="dagSvg" width="100%" height="100%"></svg>
            </div>
        </div>

        <!-- Details Tabbed Container -->
        <div class="card">
            <div class="tabs-header">
                <button class="tab-btn active" onclick="switchTab('inputs')">Inputs (<span id="inCount">0</span>)</button>
                <button class="tab-btn" onclick="switchTab('outputs')">Outputs (<span id="outCount">0</span>)</button>
                <button class="tab-btn" onclick="switchTab('scripts')">Script Disassembly</button>
                <button class="tab-btn" onclick="switchTab('simulator')">Script VM Simulator</button>
                <button class="tab-btn" onclick="switchTab('rawJson')">Raw JSON</button>
            </div>

            <div id="tabContent"></div>
        </div>
    </div>

    <footer>
        <p>Built with <strong>Rust (rust-bitcoin 0.32, Axum, Ratatui)</strong> for the Rust for Bitcoin Capstone Project</p>
        <p style="margin-top:0.25rem; font-size:0.75rem; color:#64748b;">Author: <code>@husteemah</code></p>
    </footer>

    <script>
        let currentTxData = null;

        async function analyzeTransaction() {
            const input = document.getElementById('txInput').value.trim();
            if (!input) return;

            try {
                let res;
                if (input.length === 64 && /^[0-9a-fA-F]+$/.test(input)) {
                    res = await fetch(`/api/tx/${input}`);
                } else {
                    res = await fetch('/api/decode', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ hex: input, network: 'regtest' })
                    });
                }

                if (!res.ok) {
                    alert('Error analyzing transaction: ' + await res.text());
                    return;
                }

                currentTxData = await res.json();
                renderAll(currentTxData);
            } catch (err) {
                alert('Request failed: ' + err.message);
            }
        }

        function renderAll(tx) {
            document.getElementById('networkBadge').innerText = tx.network.toUpperCase();
            document.getElementById('txidTitle').innerText = tx.txid;
            document.getElementById('wtxidLabel').innerText = 'Witness TXID: ' + tx.wtxid;

            const badges = document.getElementById('statusBadges');
            badges.innerHTML = `
                <span class="badge badge-green">${tx.is_segwit ? 'SegWit Active' : 'Legacy'}</span>
                <span class="badge badge-orange">${tx.rbf_status}</span>
                <span class="badge badge-blue">${tx.classification}</span>
            `;

            document.getElementById('vsizeVal').innerText = tx.vsize_vb + ' vB';
            document.getElementById('weightVal').innerText = tx.weight_wu + ' WU';

            if (tx.fee_info) {
                document.getElementById('feeVal').innerText = tx.fee_info.fee_sats.toLocaleString() + ' sats';
                document.getElementById('feeRateVal').innerText = tx.fee_info.fee_rate_sat_per_vb.toFixed(2) + ' sat/vB';
            } else {
                document.getElementById('feeVal').innerText = 'Unknown';
                document.getElementById('feeRateVal').innerText = 'Unknown';
            }

            document.getElementById('inCount').innerText = tx.inputs.length;
            document.getElementById('outCount').innerText = tx.outputs.length;

            renderDag(tx);
            switchTab('inputs');
        }

        function renderDag(tx) {
            const svg = document.getElementById('dagSvg');
            svg.innerHTML = '';

            const width = svg.clientWidth || 1000;
            const height = 380;
            const midY = height / 2;

            // Draw center transaction box
            const txX = width / 2 - 80;
            const txY = midY - 45;

            svg.innerHTML += `
                <rect x="${txX}" y="${txY}" width="160" height="90" rx="10" fill="#0f172a" stroke="#eab308" stroke-width="2"/>
                <text x="${txX + 80}" y="${txY + 28}" fill="#fef08a" font-weight="700" font-size="12" text-anchor="middle">TRANSACTION</text>
                <text x="${txX + 80}" y="${txY + 48}" fill="#f1f5f9" font-family="JetBrains Mono" font-size="10" text-anchor="middle">${tx.txid.substring(0,8)}...${tx.txid.substring(56)}</text>
                <text x="${txX + 80}" y="${txY + 68}" fill="#94a3b8" font-size="10" text-anchor="middle">${tx.vsize_vb} vB | ${tx.weight_wu} WU</text>
            `;

            // Draw Inputs (Left)
            const numIn = tx.inputs.length;
            tx.inputs.forEach((input, i) => {
                const inY = (height / (numIn + 1)) * (i + 1);
                const inX = 40;

                svg.innerHTML += `
                    <line x1="${inX + 160}" y1="${inY}" x2="${txX}" y2="${midY}" stroke="#38bdf8" stroke-width="2" stroke-dasharray="4"/>
                    <rect x="${inX}" y="${inY - 25}" width="160" height="50" rx="8" fill="#1e293b" stroke="#38bdf8" stroke-width="1.5"/>
                    <text x="${inX + 10}" y="${inY - 5}" fill="#38bdf8" font-size="11" font-weight="600">Input #${input.index}</text>
                    <text x="${inX + 10}" y="${inY + 12}" fill="#94a3b8" font-family="JetBrains Mono" font-size="9">${input.prevout_txid.substring(0,6)}...:${input.prevout_vout}</text>
                `;
            });

            // Draw Outputs (Right)
            const numOut = tx.outputs.length;
            tx.outputs.forEach((output, i) => {
                const outY = (height / (numOut + 1)) * (i + 1);
                const outX = width - 200;

                svg.innerHTML += `
                    <line x1="${txX + 160}" y1="${midY}" x2="${outX}" y2="${outY}" stroke="#22c55e" stroke-width="2"/>
                    <rect x="${outX}" y="${outY - 25}" width="160" height="50" rx="8" fill="#1e293b" stroke="#22c55e" stroke-width="1.5"/>
                    <text x="${outX + 10}" y="${outY - 5}" fill="#22c55e" font-size="11" font-weight="600">Output #${output.index}</text>
                    <text x="${outX + 10}" y="${outY + 12}" fill="#f1f5f9" font-family="JetBrains Mono" font-size="10">${output.value_sats.toLocaleString()} sats</text>
                `;
            });
        }

        function switchTab(tab) {
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            const container = document.getElementById('tabContent');
            if (!currentTxData) return;

            if (tab === 'inputs') {
                let html = '<table><thead><tr><th>#</th><th>Previous OutPoint</th><th>Value</th><th>Sequence</th><th>Witness / Script Details</th></tr></thead><tbody>';
                currentTxData.inputs.forEach(i => {
                    const val = i.spent_txout ? i.spent_txout.value_sats.toLocaleString() + ' sats' : 'Unknown';
                    const witnessSummary = i.witness.length ? `${i.witness.length} witness elements` : (i.script_sig ? i.script_sig.asm : 'None');
                    html += `<tr><td>${i.index}</td><td>${i.prevout_txid}:${i.prevout_vout}</td><td style="color:var(--success);">${val}</td><td>0x${i.sequence.toString(16)}</td><td><code>${witnessSummary}</code></td></tr>`;
                });
                html += '</tbody></table>';
                container.innerHTML = html;
            } else if (tab === 'outputs') {
                let html = '<table><thead><tr><th>#</th><th>Value (sats)</th><th>Script Type</th><th>Address / Payload</th><th>ASM</th></tr></thead><tbody>';
                currentTxData.outputs.forEach(o => {
                    const recipient = o.address || (o.op_return_payload ? 'OP_RETURN' : 'Non-standard');
                    html += `<tr><td>${o.index}</td><td style="color:var(--success); font-weight:700;">${o.value_sats.toLocaleString()}</td><td><span class="badge badge-blue">${o.script_type}</span></td><td><code>${recipient}</code></td><td><code>${o.script_pubkey.asm}</code></td></tr>`;
                });
                html += '</tbody></table>';
                container.innerHTML = html;
            } else if (tab === 'scripts') {
                let html = '<div>';
                currentTxData.outputs.forEach(o => {
                    html += `<div style="margin-bottom:1.5rem; background:#111827; padding:1rem; border-radius:8px;">
                        <h4 style="color:var(--accent); margin-bottom:0.5rem;">Output #${o.index} (${o.script_type})</h4>
                        <p style="margin-bottom:0.5rem; font-family:var(--font-mono); font-size:0.8rem; color:#64748b;">Raw: ${o.script_pubkey.hex}</p>
                        <div style="margin-top:0.5rem;">`;
                    o.script_pubkey.opcodes.forEach(op => {
                        html += `<div style="margin-bottom:0.4rem;"><span class="opcode-chip">${op.name}</span> <span style="font-size:0.85rem; color:#cbd5e1;">${op.meaning}</span></div>`;
                    });
                    html += '</div></div>';
                });
                html += '</div>';
                container.innerHTML = html;
            } else if (tab === 'simulator') {
                container.innerHTML = `
                    <div style="background:#0f172a; padding:1rem; border-radius:8px;">
                        <h4 style="color:var(--accent); margin-bottom:0.5rem;">Step-by-Step Script Execution Simulator</h4>
                        <p style="color:var(--text-muted); font-size:0.85rem; margin-bottom:1rem;">Simulate Forth stack execution for Output #0 script:</p>
                        <button class="btn" style="padding:0.4rem 1rem; font-size:0.8rem; margin-bottom:1rem;" onclick="runSimulation()">Step Execution</button>
                        <div id="simLog" style="font-family:var(--font-mono); font-size:0.85rem;">Click 'Step Execution' to begin trace.</div>
                    </div>
                `;
            } else if (tab === 'rawJson') {
                container.innerHTML = `<pre style="background:#0a0f1d; padding:1rem; border-radius:8px; overflow-x:auto; font-family:var(--font-mono); font-size:0.8rem;">${JSON.stringify(currentTxData, null, 2)}</pre>`;
            }
        }

        async function runSimulation() {
            if (!currentTxData || !currentTxData.outputs.length) return;
            const scriptHex = currentTxData.outputs[0].script_pubkey.hex;
            const res = await fetch('/api/simulate', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ script_hex: scriptHex, initial_stack: [] })
            });
            if (res.ok) {
                const trace = await res.json();
                let logHtml = `<div style="margin-bottom:0.5rem; color:var(--success);">Status: ${trace.success ? 'PASSED' : 'FAILED'}</div>`;
                trace.steps.forEach((s, idx) => {
                    logHtml += `<div style="margin-bottom:0.5rem; border-left:2px solid #38bdf8; padding-left:0.5rem;">
                        <strong>[Step ${idx + 1}]</strong> <span class="opcode-chip">${s.instruction}</span> ${s.description}<br/>
                        <span style="color:#64748b;">Stack after: [${s.stack_after.join(', ') || 'empty'}]</span>
                    </div>`;
                });
                document.getElementById('simLog').innerHTML = logHtml;
            }
        }

        window.onload = () => {
            // Load sample transaction on startup
            document.getElementById('txInput').value = "0200000000010101000000000000000000000000000000000000000000000000000000000000000000000000fdffffff01d07e010000000000160014531260aa2a199e228c537dfa42c82bea2c7c1f4d03483045022100cc48496d5378bdbaad964963ef3678f13cf676caf27e23391aac93824341243b022007524fb844a0358242f0f4e10dc25b55275e6326fa4612e1dd10c9b98aecde450123727573742d666f722d626974636f696e2d7765656b2d362d64656d6f2d73656372657446a8205d8ea7b116c5d45becb3b523ff762f385bc5a33c8494aadba0017411aae782888821034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aaac00000000";
            analyzeTransaction();
        };
    </script>
</body>
</html>
"##;
