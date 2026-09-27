use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

use crate::decoder::evaluator::{ScriptExecutionTrace, ScriptVm};
use crate::model::types::DecodedTx;
use crate::tui::ui::render_ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiTab {
    Overview,
    Inputs,
    Outputs,
    Scripts,
    Simulator,
}

pub struct TuiApp {
    pub tx: DecodedTx,
    pub current_tab: TuiTab,
    pub selected_input_idx: usize,
    pub selected_output_idx: usize,
    pub simulator_trace: Option<ScriptExecutionTrace>,
    pub simulator_step: usize,
    pub should_quit: bool,
}

impl TuiApp {
    pub fn new(tx: DecodedTx) -> Self {
        // Initialize simulator trace for first output script or standard script
        let mut app = Self {
            tx,
            current_tab: TuiTab::Overview,
            selected_input_idx: 0,
            selected_output_idx: 0,
            simulator_trace: None,
            simulator_step: 0,
            should_quit: false,
        };
        app.recompute_simulation();
        app
    }

    pub fn recompute_simulation(&mut self) {
        if let Some(output) = self.tx.outputs.get(self.selected_output_idx) {
            if let Ok(script_bytes) = hex::decode(&output.script_pubkey.hex) {
                let script = bitcoin::ScriptBuf::from(script_bytes);
                let mut vm = ScriptVm::new();
                let trace = vm.simulate(&script);
                self.simulator_trace = Some(trace);
                self.simulator_step = 0;
            }
        }
    }

    pub fn run() -> Result<()> {
        Ok(())
    }
}

pub fn run_tui(tx: DecodedTx) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = TuiApp::new(tx);

    loop {
        terminal.draw(|f| render_ui(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('1') => app.current_tab = TuiTab::Overview,
                        KeyCode::Char('2') => app.current_tab = TuiTab::Inputs,
                        KeyCode::Char('3') => app.current_tab = TuiTab::Outputs,
                        KeyCode::Char('4') => app.current_tab = TuiTab::Scripts,
                        KeyCode::Char('5') => app.current_tab = TuiTab::Simulator,
                        KeyCode::Tab => {
                            app.current_tab = match app.current_tab {
                                TuiTab::Overview => TuiTab::Inputs,
                                TuiTab::Inputs => TuiTab::Outputs,
                                TuiTab::Outputs => TuiTab::Scripts,
                                TuiTab::Scripts => TuiTab::Simulator,
                                TuiTab::Simulator => TuiTab::Overview,
                            };
                        }
                        KeyCode::Down | KeyCode::Char('j') => match app.current_tab {
                            TuiTab::Inputs if app.selected_input_idx + 1 < app.tx.inputs.len() => {
                                app.selected_input_idx += 1;
                            }
                            TuiTab::Outputs | TuiTab::Scripts
                                if app.selected_output_idx + 1 < app.tx.outputs.len() =>
                            {
                                app.selected_output_idx += 1;
                                app.recompute_simulation();
                            }
                            TuiTab::Simulator => {
                                if let Some(ref trace) = app.simulator_trace {
                                    if app.simulator_step + 1 < trace.steps.len() {
                                        app.simulator_step += 1;
                                    }
                                }
                            }
                            _ => {}
                        },
                        KeyCode::Up | KeyCode::Char('k') => match app.current_tab {
                            TuiTab::Inputs if app.selected_input_idx > 0 => {
                                app.selected_input_idx -= 1;
                            }
                            TuiTab::Outputs | TuiTab::Scripts if app.selected_output_idx > 0 => {
                                app.selected_output_idx -= 1;
                                app.recompute_simulation();
                            }
                            TuiTab::Simulator if app.simulator_step > 0 => {
                                app.simulator_step -= 1;
                            }
                            _ => {}
                        },
                        KeyCode::Right | KeyCode::Char(' ')
                            if app.current_tab == TuiTab::Simulator =>
                        {
                            if let Some(ref trace) = app.simulator_trace {
                                if app.simulator_step + 1 < trace.steps.len() {
                                    app.simulator_step += 1;
                                }
                            }
                        }
                        KeyCode::Left
                            if app.current_tab == TuiTab::Simulator && app.simulator_step > 0 =>
                        {
                            app.simulator_step -= 1;
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
