// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The ratatui terminal UI: pick sources, watch live progress.
//!
//! The [`App`] state (source selection + per-source progress) is pure and unit
//! tested; the crossterm event loop and ratatui rendering are thin glue around
//! it (excluded from the coverage gate). Progress arrives as
//! [`crate::run::ProgressEvent`]s from the shared run loop over a channel.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::config::{Config, StoreBackend};
use crate::crawl::{CrawlOutcome, CrawlStats};
use crate::http::{Fetcher, HttpFetcher};
use crate::output::OutputWriter;
use crate::registry::build_adapters;
use crate::run::{ProgressEvent, run_all};

/// One selectable source row with its live tallies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRow {
    pub name: String,
    pub selected: bool,
    pub started: bool,
    pub stats: CrawlStats,
}

/// The TUI application state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App {
    pub rows: Vec<SourceRow>,
    pub cursor: usize,
    pub running: bool,
    pub done: bool,
    /// Whether the user asked to quit.
    pub quit: bool,
}

impl App {
    /// Builds the app over the known source names (none selected initially).
    pub fn new(source_names: &[&str]) -> Self {
        let rows = source_names
            .iter()
            .map(|n| SourceRow {
                name: (*n).to_string(),
                selected: false,
                started: false,
                stats: CrawlStats::default(),
            })
            .collect();
        Self {
            rows,
            cursor: 0,
            running: false,
            done: false,
            quit: false,
        }
    }

    /// Moves the cursor by `delta`, clamped to the row range.
    pub fn move_cursor(&mut self, delta: isize) {
        if self.rows.is_empty() {
            return;
        }
        let max = self.rows.len() as isize - 1;
        let next = (self.cursor as isize + delta).clamp(0, max);
        self.cursor = next as usize;
    }

    /// Toggles selection of the row under the cursor.
    pub fn toggle(&mut self) {
        if let Some(row) = self.rows.get_mut(self.cursor) {
            row.selected = !row.selected;
        }
    }

    /// Selects or clears every row.
    pub fn set_all(&mut self, selected: bool) {
        for row in &mut self.rows {
            row.selected = selected;
        }
    }

    /// The names of the currently-selected sources.
    pub fn selected_names(&self) -> Vec<String> {
        self.rows
            .iter()
            .filter(|r| r.selected)
            .map(|r| r.name.clone())
            .collect()
    }

    /// Marks the run started (once at least one source is selected).
    pub fn start(&mut self) -> bool {
        if self.running || self.selected_names().is_empty() {
            return false;
        }
        self.running = true;
        self.done = false;
        true
    }

    /// Applies a progress event to the state.
    pub fn apply(&mut self, event: ProgressEvent) {
        match event {
            ProgressEvent::Started(name) => {
                if let Some(r) = self.rows.iter_mut().find(|r| r.name == name) {
                    r.started = true;
                }
            }
            ProgressEvent::Finished(name, stats) => {
                if let Some(r) = self.rows.iter_mut().find(|r| r.name == name) {
                    r.stats = stats;
                }
            }
            ProgressEvent::AllDone => {
                self.running = false;
                self.done = true;
            }
        }
    }

    /// Totals across all rows, for the summary footer.
    pub fn totals(&self) -> CrawlStats {
        let mut t = CrawlStats::default();
        for r in &self.rows {
            t.accepted += r.stats.accepted;
            t.low_confidence += r.stats.low_confidence;
            t.rejected += r.stats.rejected;
            t.failed += r.stats.failed;
            t.deduped += r.stats.deduped;
            t.discovered += r.stats.discovered;
        }
        t
    }
}

// --- Terminal glue (excluded from the coverage gate) --------------------

/// Runs the interactive TUI: pick sources, start the crawl, watch progress,
/// then write the corpus on quit. Restores the terminal on every exit path.
pub async fn run_tui(
    config: Config,
    source_names: &[&'static str],
    limit: Option<usize>,
) -> Result<()> {
    let (root, safe_prefix, low_prefix) = match &config.store.backend {
        StoreBackend::LocalFs { root } => (
            root.clone(),
            config.store.safe_prefix.clone(),
            config.store.low_confidence_prefix.clone(),
        ),
        StoreBackend::S3 { .. } => anyhow::bail!("TUI requires a local_fs store for now"),
    };

    enable_raw_mode().context("enabling raw mode")?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen).context("entering alt screen")?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout)).context("terminal init")?;

    let result = event_loop(&mut terminal, &config, source_names, limit, &root).await;

    disable_raw_mode().ok();
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    let outcome = result?;
    if let Some(outcome) = outcome {
        let (summary, _entries) =
            OutputWriter::new(&root, safe_prefix, low_prefix).write(&outcome)?;
        println!(
            "Wrote safe: {}, low-confidence: {} to {}",
            summary.safe,
            summary.low_confidence,
            root.display()
        );
    }
    Ok(())
}

/// The draw/input loop. Returns the crawl outcome if one ran to completion.
async fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    config: &Config,
    source_names: &[&'static str],
    limit: Option<usize>,
    root: &std::path::Path,
) -> Result<Option<CrawlOutcome>> {
    let mut app = App::new(source_names);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut handle: Option<tokio::task::JoinHandle<CrawlOutcome>> = None;

    loop {
        terminal.draw(|f| render(f, &app)).context("draw")?;
        while let Ok(ev) = rx.try_recv() {
            app.apply(ev);
        }
        if event::poll(Duration::from_millis(120)).context("poll")?
            && let Event::Key(key) = event::read().context("read")?
        {
            match key.code {
                KeyCode::Char('q') => app.quit = true,
                KeyCode::Up => app.move_cursor(-1),
                KeyCode::Down => app.move_cursor(1),
                KeyCode::Char(' ') => app.toggle(),
                KeyCode::Char('a') => app.set_all(true),
                KeyCode::Enter if app.start() => {
                    let fetcher: Arc<dyn Fetcher> = Arc::new(HttpFetcher::new(
                        config.user_agent(),
                        Duration::from_millis(config.delay_ms),
                    )?);
                    let built =
                        build_adapters(&app.selected_names(), fetcher, &root.join(".checkouts"));
                    let adapters = built.adapters;
                    let txc = tx.clone();
                    handle = Some(tokio::spawn(async move {
                        run_all(&adapters, limit, Some(txc)).await
                    }));
                }
                _ => {}
            }
        }
        if app.quit {
            break;
        }
    }

    match handle {
        Some(h) if h.is_finished() => Ok(Some(h.await.context("crawl task")?)),
        Some(h) => {
            h.abort();
            Ok(None)
        }
        None => Ok(None),
    }
}

/// Renders the source list + a totals/footer.
fn render(frame: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(4)])
        .split(frame.area());

    let items: Vec<ListItem> = app
        .rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let mark = if r.selected { "[x]" } else { "[ ]" };
            let cursor = if i == app.cursor { "›" } else { " " };
            let s = &r.stats;
            let line = format!(
                "{cursor} {mark} {:<14} accepted {}  low {}  rejected {}  failed {}",
                r.name, s.accepted, s.low_confidence, s.rejected, s.failed
            );
            let style = if i == app.cursor {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::styled(line, style))
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .title(" score-crawler ")
            .borders(Borders::ALL),
    );
    frame.render_widget(list, chunks[0]);

    let t = app.totals();
    let status = if app.done {
        "done"
    } else if app.running {
        "running…"
    } else {
        "select sources"
    };
    let footer = Paragraph::new(vec![
        Line::from(format!(
            "TOTAL  accepted {}  low {}  rejected {}  failed {}  [{}]",
            t.accepted, t.low_confidence, t.rejected, t.failed, status
        )),
        Line::from("↑/↓ move · space select · a all · enter start · q quit"),
    ])
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App::new(&["openscore", "cpdl", "pdmx"])
    }

    #[test]
    fn cursor_stays_in_range() {
        let mut a = app();
        a.move_cursor(-1);
        assert_eq!(a.cursor, 0);
        a.move_cursor(10);
        assert_eq!(a.cursor, 2);
    }

    #[test]
    fn toggle_and_select_all() {
        let mut a = app();
        a.toggle(); // openscore on
        a.move_cursor(2);
        a.toggle(); // pdmx on
        assert_eq!(a.selected_names(), vec!["openscore", "pdmx"]);
        a.set_all(true);
        assert_eq!(a.selected_names().len(), 3);
        a.set_all(false);
        assert!(a.selected_names().is_empty());
    }

    #[test]
    fn start_requires_a_selection() {
        let mut a = app();
        assert!(!a.start()); // nothing selected
        assert!(!a.running);
        a.toggle();
        assert!(a.start());
        assert!(a.running);
        assert!(!a.start()); // already running
    }

    #[test]
    fn progress_events_update_rows_and_totals() {
        let mut a = app();
        a.toggle();
        a.start();
        a.apply(ProgressEvent::Started("openscore".into()));
        assert!(a.rows[0].started);

        let stats = CrawlStats {
            accepted: 3,
            low_confidence: 1,
            rejected: 2,
            ..CrawlStats::default()
        };
        a.apply(ProgressEvent::Finished("openscore".into(), stats));
        assert_eq!(a.rows[0].stats.accepted, 3);
        assert_eq!(a.totals().accepted, 3);
        assert_eq!(a.totals().rejected, 2);

        a.apply(ProgressEvent::AllDone);
        assert!(a.done && !a.running);
    }
}
