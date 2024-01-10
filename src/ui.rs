use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{sync::mpsc::Receiver, time::Duration};
use std::{error::Error, io, sync::RwLock};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame, Terminal,
};

use crate::metrics::{MetricsSummary, RequestMetric};

pub(crate) fn run_ui(
    total_requests_expected: usize,
    metrics_receiver: &mut Receiver<RequestMetric>,
) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let metrics_summary = RwLock::new(MetricsSummary::new(total_requests_expected));

    // create app and run it
    let res = run_app(&mut terminal, metrics_summary, metrics_receiver);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    metrics_summary: RwLock<MetricsSummary>,
    metrics_receiver: &mut Receiver<RequestMetric>,
) -> io::Result<()> {
    while let Ok(request_metric) = metrics_receiver.recv() {
        let mut metrics_summary_mut = metrics_summary.write().unwrap();
        metrics_summary_mut.record(request_metric);
        drop(metrics_summary_mut);
        let metrics_summary = metrics_summary.read().unwrap();
        terminal.draw(|f| ui(f, &metrics_summary))?;
        if metrics_summary.is_completed() {
            break;
        }
        let timeout = Duration::from_millis(50);
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, metrics: &MetricsSummary) {
    let completed = metrics.ok + metrics.errors;
    let total = metrics.total_expected;
    // main
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let label = format!("{}/{}", completed, total);
    let gauge = Gauge::default()
        .block(Block::default().title("Progress ").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan))
        .label(label)
        .percent((completed as f64 / total as f64 * 100f64) as u16);
    f.render_widget(gauge, chunks[0]);

    // details
    let details_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(chunks[1]);

    // detals left
    let left_details_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(7), Constraint::Min(3)].as_ref())
        .split(details_chunks[0]);

    let counts_block = Block::default()
        .title(vec![Span::from("Counts")])
        .borders(Borders::ALL);
    f.render_widget(counts_block, left_details_chunks[0]);

    let counts_chunks = Layout::default()
        .margin(2)
        .direction(Direction::Vertical)
        .constraints([Constraint::Max(1), Constraint::Max(1), Constraint::Max(1)].as_ref())
        .split(left_details_chunks[0]);

    let in_progress_span = Span::styled(
        format!("In Progress : \t{}", metrics.in_progress),
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Cyan),
    );
    f.render_widget(Paragraph::new(in_progress_span), counts_chunks[0]);

    let ok_span = Span::styled(
        format!("Ok          : \t{}", metrics.ok),
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Green),
    );
    f.render_widget(Paragraph::new(ok_span), counts_chunks[1]);

    let errors_span = Span::styled(
        format!("Errors      : \t{}", metrics.errors),
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
    );
    f.render_widget(Paragraph::new(errors_span), counts_chunks[2]);

    // details rigth
    // detals left
    let right_details_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(details_chunks[1]);

    let width = right_details_chunks[0].width;
    let rel: usize = total / width as usize;

    let ok_durations: Vec<_> = metrics
        .ok_durations_ms
        .chunks(rel)
        .map(|w| {
            let sum: f32 = w.iter().sum();
            let len = w.len() as f32;
            (sum / len) as u64
        })
        .collect();

    let ok_durations_sparkline = Sparkline::default()
        .block(Block::default().title("OK durations").borders(Borders::ALL))
        .data(&ok_durations)
        .style(Style::default().fg(Color::Green));
    f.render_widget(ok_durations_sparkline, right_details_chunks[0]);

    let error_durations: Vec<_> = metrics
        .error_durations_ms
        .chunks(rel)
        .map(|w| {
            let sum: f32 = w.iter().sum();
            let len = w.len() as f32;
            (sum / len) as u64
        })
        .collect();

    let error_durations_sparkline = Sparkline::default()
        .block(
            Block::default()
                .title("Error durations")
                .borders(Borders::ALL),
        )
        .data(&error_durations)
        .style(Style::default().fg(Color::Red));
    f.render_widget(error_durations_sparkline, right_details_chunks[1]);
}
