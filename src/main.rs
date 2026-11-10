use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::{Frame, Terminal};

#[derive(Clone, Copy)]
enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    fn duration(self) -> Duration {
        match self {
            Self::Work => Duration::from_secs(25 * 60),
            Self::ShortBreak => Duration::from_secs(5 * 60),
            Self::LongBreak => Duration::from_secs(15 * 60),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Work => "FOCUS",
            Self::ShortBreak => "SHORT BREAK",
            Self::LongBreak => "LONG BREAK",
        }
    }
}

struct App {
    phase: Phase,
    completed_work: u32,
    remaining: Duration,
    running_since: Option<Instant>,
}

impl App {
    fn new() -> Self {
        Self {
            phase: Phase::Work,
            completed_work: 0,
            remaining: Phase::Work.duration(),
            running_since: None,
        }
    }

    fn remaining(&self) -> Duration {
        self.running_since
            .map(|start| self.remaining.saturating_sub(start.elapsed()))
            .unwrap_or(self.remaining)
    }

    fn tick(&mut self) {
        if self.running_since.is_some() && self.remaining().is_zero() {
            self.next_phase();
            self.running_since = Some(Instant::now());
        }
    }

    fn toggle(&mut self) {
        if let Some(start) = self.running_since.take() {
            self.remaining = self.remaining.saturating_sub(start.elapsed());
        } else {
            self.running_since = Some(Instant::now());
        }
    }

    fn reset(&mut self) {
        self.remaining = self.phase.duration();
        self.running_since = None;
    }

    fn next_phase(&mut self) {
        self.running_since = None;
        self.phase = match self.phase {
            Phase::Work => {
                self.completed_work += 1;
                if self.completed_work % 4 == 0 {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            }
            Phase::ShortBreak | Phase::LongBreak => Phase::Work,
        };
        self.remaining = self.phase.duration();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result?;
    Ok(())
}

fn run(
    terminal: &mut Terminal<impl ratatui::backend::Backend<Error = io::Error>>,
) -> io::Result<()> {
    let mut app = App::new();
    loop {
        app.tick();
        terminal.draw(|frame| draw(frame, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char(' ') => app.toggle(),
                    KeyCode::Char('r') => app.reset(),
                    KeyCode::Char('n') => app.next_phase(),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(7),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(3),
        ])
        .split(area);

    let remaining = app.remaining();
    let seconds = remaining.as_secs();
    let clock = format!("{:02}:{:02}", seconds / 60, seconds % 60);
    let running = app.running_since.is_some();
    let title = format!(
        " {} · pomodoro {} ",
        app.phase.label(),
        if running { "running" } else { "paused" }
    );

    let large_clock = large_digits(&clock);
    let timer = Paragraph::new(large_clock)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().title(title).borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(timer, chunks[0]);

    let total = app.phase.duration().as_secs();
    let elapsed = total.saturating_sub(seconds);
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(elapsed as f64 / total as f64);
    frame.render_widget(gauge, chunks[1]);

    let count = Paragraph::new(format!("Completed focus sessions: {}", app.completed_work))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(count, chunks[2]);

    let help = Paragraph::new("[space] start/pause   [r] reset   [n] next phase   [q] quit")
        .block(Block::default().borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(help, chunks[3]);
}

fn large_digits(clock: &str) -> String {
    let glyphs = clock.chars().map(|ch| match ch {
        '0' => ["███", "█ █", "█ █", "█ █", "███"],
        '1' => [" ██", "  █", "  █", "  █", "███"],
        '2' => ["███", "  █", "███", "█  ", "███"],
        '3' => ["███", "  █", "███", "  █", "███"],
        '4' => ["█ █", "█ █", "███", "  █", "  █"],
        '5' => ["███", "█  ", "███", "  █", "███"],
        '6' => ["███", "█  ", "███", "█ █", "███"],
        '7' => ["███", "  █", "  █", "  █", "  █"],
        '8' => ["███", "█ █", "███", "█ █", "███"],
        '9' => ["███", "█ █", "███", "  █", "███"],
        ':' => ["   ", " █ ", "   ", " █ ", "   "],
        _ => ["   "; 5],
    });

    let glyphs: Vec<_> = glyphs.collect();
    (0..5)
        .map(|row| {
            glyphs
                .iter()
                .map(|glyph| glyph[row])
                .collect::<Vec<_>>()
                .join("  ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
