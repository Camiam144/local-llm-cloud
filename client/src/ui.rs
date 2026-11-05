use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{App, AppState};

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title_block = Block::default()
        .borders(Borders::all())
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "Local LLM Connector",
        Style::default().fg(Color::Yellow),
    ))
    .block(title_block);
    frame.render_widget(title, chunks[0]);

    match app.app_state {
        AppState::Idle => {
            render_idle_screen(app, chunks[1], frame);
        }
        AppState::Waking => {
            render_wake_screen(app, chunks[1], frame);
        }
        AppState::Disconnected => todo!(),
        AppState::Connected => todo!(),
        AppState::Error => todo!(),
        AppState::InputGetMacForWol => render_mac_input_popup(app, chunks[1], frame),
    }

    let footer = Paragraph::new(Text::styled("Footer info", Style::default()));
    frame.render_widget(footer, chunks[2]);
}

fn render_idle_screen(_app: &App, area: Rect, frame: &mut Frame) {
    let text = Paragraph::new(Text::styled(
        "Push W to send wake event to PC",
        Style::default().fg(Color::Gray),
    ));
    frame.render_widget(text, area);
}

fn render_wake_screen(app: &App, area: Rect, frame: &mut Frame) {
    let spinner = ['⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇'];
    // Calculate if we should advance
    let spinner_frame_dur = 125;
    let i = ((app.curr_tick * app.tick_rate) / spinner_frame_dur) as usize % spinner.len();
    let text = Paragraph::new(Text::styled(
        format!("{} Waking Server", spinner[i]),
        Style::default().fg(Color::Gray),
    ));

    frame.render_widget(text, area);
}

fn render_mac_input_popup(app: &App, area: Rect, frame: &mut Frame) {
    // fakes a popup to take input.
    frame.render_widget(&app.mac_input.text_area, area);
}
