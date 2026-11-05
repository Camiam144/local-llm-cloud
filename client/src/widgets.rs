use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{
        Color, Style,
        palette::tailwind::{GREEN, RED, SLATE, YELLOW},
    },
    widgets::{Block, Borders, Widget},
};
use ratatui_textarea::TextArea;
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;

use crate::{app::Action, services::MacAddr6};

pub struct ConnStatusWidget {
    current_state: ConnStatusWidgetState,
}

impl ConnStatusWidget {
    pub fn set_state(&mut self, state: ConnStatusWidgetState) {
        self.current_state = state;
    }
}

pub enum ConnStatusWidgetState {
    Disconnected,
    Waking,
    Connected,
    Error(String),
}

impl Widget for &ConnStatusWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (color, text) = match &self.current_state {
            ConnStatusWidgetState::Disconnected => (SLATE, "Disconnected".to_string()),
            ConnStatusWidgetState::Waking => (YELLOW, "Waking...".to_string()),
            ConnStatusWidgetState::Connected => (GREEN, "Connected".to_string()),
            ConnStatusWidgetState::Error(e) => (RED, format!("Error: {}", e)),
        };
    }
}

#[derive(Debug)]
pub struct InputMacWidget<'a> {
    pub text_area: TextArea<'a>,
    pub is_valid: bool,
}

impl Default for InputMacWidget<'_> {
    fn default() -> Self {
        let mut text_area = TextArea::default();
        text_area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Mac 6 Address"),
        );
        text_area.set_cursor_line_style(Style::default());
        text_area.set_placeholder_text("Enter a valid Mac6 address (ex FF:FF:FF:FF:FF:FF)");
        Self {
            text_area,
            is_valid: false,
        }
    }
}

impl InputMacWidget<'_> {
    fn validate(&mut self) -> bool {
        // The example has the styling in here, could easily break this out
        // into another function that runs in the render loop or input handling loop.
        if let Some(mac) = self.text_area.lines().first() {
            if let Err(err) = MacAddr6::try_from(mac.as_str()) {
                self.is_valid = false;
                // Set Affirmative styling
                self.text_area
                    .set_style(Style::default().fg(Color::LightRed));
                self.text_area.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Color::LightRed)
                        .title(format!("Error: {}", err)),
                );
                return false;
            } else {
                self.is_valid = true;
                self.text_area
                    .set_style(Style::default().fg(Color::LightGreen));
                self.text_area.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Color::LightGreen)
                        .title("OK"),
                );
                return true;
            }
        }
        self.is_valid = false;
        false
    }

    pub fn handle_key_event(
        &mut self,
        key_event: &KeyEvent,
        action_send: UnboundedSender<Action>,
    ) -> anyhow::Result<()> {
        tracing::trace!("Mac Input got keycode {:?}", key_event);
        match key_event.code {
            KeyCode::Esc => {
                self.text_area.clear();
                action_send.send(Action::CloseWolInputPopup)?
            }
            KeyCode::Enter => {
                if self.validate() {
                    let addr = self.text_area.lines().first().unwrap();
                    tracing::debug!("Parsed Addr: {}", &addr);
                    let mac = MacAddr6::try_from(addr.clone().as_str())?;
                    action_send.send(Action::SetMacAddr(mac))?
                }
            }
            _ => {
                let changed = self.text_area.input(*key_event);
                if changed {
                    self.validate();
                }
            }
        }
        Ok(())
    }
}
