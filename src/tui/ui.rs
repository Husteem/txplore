use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap};
use ratatui::Frame;

use crate::tui::app::{TuiApp, TuiTab};

pub fn render_ui(frame: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header & Tabs
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    render_header_tabs(frame, app, chunks[0]);

    match app.current_tab {
        TuiTab::Overview => render_overview_tab(frame, app, chunks[1]),
        TuiTab::Inputs => render_inputs_tab(frame, app, chunks[1]),
        TuiTab::Outputs => render_outputs_tab(frame, app, chunks[1]),
        TuiTab::Scripts => render_scripts_tab(frame, app, chunks[1]),
        TuiTab::Simulator => render_simulator_tab(frame, app, chunks[1]),
    }

    render_footer(frame, chunks[2]);
}

fn render_header_tabs(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let tab_titles = vec![
        "[1] Overview",
        "[2] Inputs",
        "[3] Outputs",
        "[4] Scripts",
        "[5] Script VM Simulator",
    ];

    let active_index = match app.current_tab {
        TuiTab::Overview => 0,
        TuiTab::Inputs => 1,
        TuiTab::Outputs => 2,
        TuiTab::Scripts => 3,
        TuiTab::Simulator => 4,
    };

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " Bitcoin Transaction Explorer (txplore) - Network: {} ",
                    app.tx.network
                ))
                .style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        )
        .select(active_index);

    frame.render_widget(tabs, area);
}

fn render_overview_tab(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(
            "Txid: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(&app.tx.txid, Style::default().fg(Color::Green)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Wtxid: ", Style::default().fg(Color::Yellow)),
        Span::raw(&app.tx.wtxid),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Classification: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{}", app.tx.classification),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Version: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}", app.tx.version)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("SegWit Active: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            if app.tx.is_segwit {
                "Yes (v0 / v1 Taproot)"
            } else {
                "No (Legacy)"
            },
            Style::default().fg(if app.tx.is_segwit {
                Color::Green
            } else {
                Color::Blue
            }),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("BIP125 RBF: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{:?}", app.tx.rbf_status)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Locktime: ", Style::default().fg(Color::Yellow)),
        Span::raw(&app.tx.locktime.description),
    ]));

    if let Some(ref conf) = app.tx.confirmation_info {
        lines.push(Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                if conf.confirmed {
                    format!("Confirmed in block {}", conf.block_height.unwrap_or(0))
                } else {
                    "Unconfirmed (Mempool)".to_string()
                },
                Style::default().fg(if conf.confirmed {
                    Color::Green
                } else {
                    Color::Yellow
                }),
            ),
        ]));
    }

    let left_para = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Core Attributes "),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(left_para, chunks[0]);

    // Right Column: Economics & Metrics
    let mut metric_lines = Vec::new();
    metric_lines.push(Line::from(vec![
        Span::styled("Virtual Size: ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{} vB", app.tx.vsize_vb),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]));
    metric_lines.push(Line::from(vec![
        Span::styled("Weight Units: ", Style::default().fg(Color::Cyan)),
        Span::raw(format!("{} WU", app.tx.weight_wu)),
    ]));
    metric_lines.push(Line::from(vec![
        Span::styled("Serialized Size: ", Style::default().fg(Color::Cyan)),
        Span::raw(format!("{} bytes", app.tx.size_bytes)),
    ]));
    metric_lines.push(Line::from(vec![
        Span::styled("Witness Discount: ", Style::default().fg(Color::Cyan)),
        Span::raw(format!("{:.1}%", app.tx.discount_ratio * 100.0)),
    ]));

    metric_lines.push(Line::from(""));
    metric_lines.push(Line::from(Span::styled(
        "Economic Summary",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::UNDERLINED),
    )));

    if let Some(ref fee) = app.tx.fee_info {
        metric_lines.push(Line::from(vec![
            Span::styled("Total Inputs: ", Style::default().fg(Color::White)),
            Span::raw(format!(
                "{} sats ({:.6} BTC)",
                fee.total_input_sats,
                fee.total_input_sats as f64 / 1e8
            )),
        ]));
        metric_lines.push(Line::from(vec![
            Span::styled("Total Outputs: ", Style::default().fg(Color::White)),
            Span::raw(format!(
                "{} sats ({:.6} BTC)",
                fee.total_output_sats,
                fee.total_output_sats as f64 / 1e8
            )),
        ]));
        metric_lines.push(Line::from(vec![
            Span::styled("Miner Fee: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{} sats", fee.fee_sats),
                Style::default().fg(Color::Green),
            ),
        ]));
        metric_lines.push(Line::from(vec![
            Span::styled("Fee Rate: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:.2} sat/vB", fee.fee_rate_sat_per_vb),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    } else {
        metric_lines.push(Line::from(Span::styled(
            "Input amounts not provided (Offline Mode)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let right_para = Paragraph::new(metric_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Metrics & Fees "),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(right_para, chunks[1]);
}

fn render_inputs_tab(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let items: Vec<ListItem> = app
        .tx
        .inputs
        .iter()
        .map(|input| {
            let prefix = if input.index == app.selected_input_idx {
                "> "
            } else {
                "  "
            };
            let text = format!(
                "{}Input #{}: {}:{}",
                prefix, input.index, input.prevout_txid, input.prevout_vout
            );
            let style = if input.index == app.selected_input_idx {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Inputs List "),
    );
    frame.render_widget(list, chunks[0]);

    // Detail pane
    if let Some(input) = app.tx.inputs.get(app.selected_input_idx) {
        let mut details = Vec::new();
        details.push(Line::from(vec![
            Span::styled("Input Index: ", Style::default().fg(Color::Yellow)),
            Span::raw(input.index.to_string()),
        ]));
        details.push(Line::from(vec![
            Span::styled("Previous Outpoint: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}:{}", input.prevout_txid, input.prevout_vout)),
        ]));
        details.push(Line::from(vec![
            Span::styled("Sequence: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("0x{:08x}", input.sequence)),
        ]));

        if let Some(ref rel) = input.relative_locktime {
            details.push(Line::from(vec![
                Span::styled("BIP68 Relative Lock: ", Style::default().fg(Color::Yellow)),
                Span::raw(&rel.human_readable),
            ]));
        }

        if let Some(ref spent) = input.spent_txout {
            details.push(Line::from(vec![
                Span::styled("Spent Value: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{} sats", spent.value_sats)),
            ]));
        }

        if let Some(ref ss) = input.script_sig {
            details.push(Line::from(""));
            details.push(Line::from(Span::styled(
                "ScriptSig:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            details.push(Line::from(format!("  Hex: {}", ss.hex)));
            details.push(Line::from(format!("  ASM: {}", ss.asm)));
        }

        if !input.witness.is_empty() {
            details.push(Line::from(""));
            details.push(Line::from(Span::styled(
                format!("Witness Stack ({} elements):", input.witness.len()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            for w in &input.witness {
                details.push(Line::from(format!(
                    "  [{}] {} ({} bytes)",
                    w.index, w.description, w.size_bytes
                )));
                details.push(Line::from(Span::styled(
                    format!("       0x{}", w.hex),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        let detail_para = Paragraph::new(details)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Selected Input Details "),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(detail_para, chunks[1]);
    }
}

fn render_outputs_tab(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let items: Vec<ListItem> = app
        .tx
        .outputs
        .iter()
        .map(|output| {
            let prefix = if output.index == app.selected_output_idx {
                "> "
            } else {
                "  "
            };
            let text = format!(
                "{}Output #{}: {} sats ({})",
                prefix, output.index, output.value_sats, output.script_type
            );
            let style = if output.index == app.selected_output_idx {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Outputs List "),
    );
    frame.render_widget(list, chunks[0]);

    if let Some(output) = app.tx.outputs.get(app.selected_output_idx) {
        let mut details = Vec::new();
        details.push(Line::from(vec![
            Span::styled("Output Index: ", Style::default().fg(Color::Yellow)),
            Span::raw(output.index.to_string()),
        ]));
        details.push(Line::from(vec![
            Span::styled("Value: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{} sats ({:.8} BTC)", output.value_sats, output.value_btc),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        details.push(Line::from(vec![
            Span::styled("Script Type: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", output.script_type)),
        ]));

        if let Some(ref addr) = output.address {
            details.push(Line::from(vec![
                Span::styled("Address: ", Style::default().fg(Color::Yellow)),
                Span::styled(addr, Style::default().fg(Color::Cyan)),
            ]));
        }

        if let Some(ref op) = output.op_return_payload {
            details.push(Line::from(""));
            details.push(Line::from(Span::styled(
                "OP_RETURN Data Carrier:",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )));
            details.push(Line::from(format!("  Protocol: {:?}", op.protocol)));
            if let Some(ref ascii) = op.ascii {
                details.push(Line::from(format!("  Decoded Text: \"{}\"", ascii)));
            }
            details.push(Line::from(format!("  Hex: 0x{}", op.hex)));
        }

        details.push(Line::from(""));
        details.push(Line::from(Span::styled(
            "ScriptPubKey:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        details.push(Line::from(format!("  Hex: {}", output.script_pubkey.hex)));
        details.push(Line::from(format!("  ASM: {}", output.script_pubkey.asm)));

        let detail_para = Paragraph::new(details)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Selected Output Details "),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(detail_para, chunks[1]);
    }
}

fn render_scripts_tab(frame: &mut Frame, app: &TuiApp, area: Rect) {
    if let Some(output) = app.tx.outputs.get(app.selected_output_idx) {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled(
                "Inspecting Script for Output #",
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                output.index.to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" ({})", output.script_type)),
        ]));
        lines.push(Line::from(format!("Raw Hex: {}", output.script_pubkey.hex)));
        lines.push(Line::from(format!(
            "Disassembly (ASM): {}",
            output.script_pubkey.asm
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Opcode Instructions & Meaning:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        )));

        for op in &output.script_pubkey.opcodes {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("[+{:03}] ", op.offset),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    &op.name,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(&op.meaning, Style::default().fg(Color::White)),
            ]));
            if let Some(ref data) = op.push_data_hex {
                lines.push(Line::from(vec![
                    Span::raw("       Push Bytes: "),
                    Span::styled(data, Style::default().fg(Color::Yellow)),
                ]));
            }
        }

        let para = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Script Disassembler & Semantics "),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(para, area);
    }
}

fn render_simulator_tab(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(8)])
        .split(area);

    let info_text = vec![
        Line::from(vec![
            Span::styled(
                "Bitcoin Script Stack Execution Simulator",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Stepping through script execution in a simulated Forth-like VM"),
        ]),
        Line::from(vec![
            Span::raw("Controls: "),
            Span::styled(
                "[Space / Right Arrow]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Next Step  "),
            Span::styled(
                "[Left Arrow]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Previous Step"),
        ]),
    ];

    let info_para = Paragraph::new(info_text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(info_para, chunks[0]);

    if let Some(ref trace) = app.simulator_trace {
        let total_steps = trace.steps.len();
        let current_step_idx = app.simulator_step.min(total_steps.saturating_sub(1));

        let sim_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Left Pane: Step Timeline
        let step_items: Vec<ListItem> = trace
            .steps
            .iter()
            .enumerate()
            .map(|(idx, step)| {
                let prefix = if idx == current_step_idx {
                    "▶ "
                } else {
                    "  "
                };
                let text = format!(
                    "{}[Step {}] {}",
                    prefix,
                    step.step_index + 1,
                    step.instruction
                );
                let style = if idx == current_step_idx {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(text).style(style)
            })
            .collect();

        let steps_list =
            List::new(step_items).block(Block::default().borders(Borders::ALL).title(format!(
                " Execution Steps ({}/{}) ",
                current_step_idx + 1,
                total_steps
            )));
        frame.render_widget(steps_list, sim_chunks[0]);

        // Right Pane: Active Step Stack Inspection
        if let Some(step) = trace.steps.get(current_step_idx) {
            let mut stack_lines = vec![
                Line::from(vec![
                    Span::styled("Current Instruction: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        &step.instruction,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Description: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&step.description),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "--- Stack Before Execution ---",
                    Style::default().fg(Color::DarkGray),
                )),
            ];
            if step.stack_before.is_empty() {
                stack_lines.push(Line::from("  (empty)"));
            } else {
                for (s_idx, item) in step.stack_before.iter().enumerate() {
                    stack_lines.push(Line::from(format!("  [{}] 0x{}", s_idx, item)));
                }
            }

            stack_lines.push(Line::from(""));
            stack_lines.push(Line::from(Span::styled(
                "--- Stack After Execution ---",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )));
            if step.stack_after.is_empty() {
                stack_lines.push(Line::from("  (empty)"));
            } else {
                for (s_idx, item) in step.stack_after.iter().enumerate() {
                    stack_lines.push(Line::from(vec![
                        Span::styled(format!("  [{}] ", s_idx), Style::default().fg(Color::Green)),
                        Span::raw(format!("0x{}", item)),
                    ]));
                }
            }

            let stack_para = Paragraph::new(stack_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Live VM Stack State "),
                )
                .wrap(Wrap { trim: true });
            frame.render_widget(stack_para, sim_chunks[1]);
        }
    }
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer_text = Line::from(vec![
        Span::styled(
            "[1-5]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Switch Tab  "),
        Span::styled(
            "[Tab]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Cycle Tabs  "),
        Span::styled(
            "[j/k / Down/Up]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Select Item  "),
        Span::styled(
            "[q / Esc]",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Exit"),
    ]);

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}
