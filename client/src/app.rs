use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{debug, trace};

use crate::{
    event::{Event, EventHandler},
    services::{MacAddr6, send_wol_packet},
    tui::Tui,
    ui,
    widgets::InputMacWidget,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub enum AppState {
    #[default]
    Idle,
    // Getting user input for the mac address for the WOL packet
    InputGetMacForWol,
    // Currently attempting to Wake the host PC
    Waking,
    // Host PC is awake but we aren't connected to the daemon
    Disconnected,
    // Successfully connected to the daemon
    Connected,
    // Some issue occured
    Error,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Action {
    Tick,
    OpenWolInputPopup,
    CloseWolInputPopup,
    SetMacAddr(MacAddr6),
    SendWOLPacket,
    Render,
    Quit,
    None,
}

#[derive(Clone, Copy, Debug)]
pub enum AppEvent {
    SendWOLPacket,
    Quit,
}

#[derive(Debug)]
pub struct App<'a> {
    pub app_state: AppState,
    pub should_quit: bool,
    pub tick_rate: u64,
    pub render_rate: u64,
    pub action_send: UnboundedSender<Action>,
    pub action_receive: UnboundedReceiver<Action>,
    pub curr_tick: u64,
    pub wol_mac: Option<MacAddr6>,
    pub mac_input: InputMacWidget<'a>,
}

impl Default for App<'_> {
    fn default() -> Self {
        let (action_send, action_receive) = mpsc::unbounded_channel::<Action>();

        Self {
            app_state: AppState::Idle,
            should_quit: false,
            tick_rate: 250,
            render_rate: 250,
            action_send,
            action_receive,
            curr_tick: 0,
            wol_mac: None,
            mac_input: InputMacWidget::default(),
        }
    }
}

impl App<'_> {
    // #[instrument]
    pub async fn run(&mut self) -> anyhow::Result<()> {
        debug!("Entering run");
        let backend = CrosstermBackend::new(std::io::stderr());
        let terminal = Terminal::new(backend)?;
        let events = EventHandler::new(self.tick_rate);
        let mut tui = Tui::new(terminal, events);

        tui.enter()?;
        // Where do I build and register stateful widgets?
        trace!("Entering main app loop");
        loop {
            if let Ok(event) = tui.events.next().await {
                self.handle_event(&event).await?;
            }
            // trace!("Draw called");
            while let Ok(action) = self.action_receive.try_recv() {
                self.handle_action(&action, &mut tui).await?;
            }
            if self.should_quit {
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    pub fn draw(&mut self, tui: &mut Tui) -> anyhow::Result<()> {
        tui.terminal.draw(|frame| ui::render(self, frame))?;
        Ok(())
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    async fn handle_event(&mut self, event: &Event) -> anyhow::Result<()> {
        let action_send = self.action_send.clone();
        match event {
            Event::Tick => {
                action_send.send(Action::Tick)?;
                if (self.curr_tick * self.tick_rate).is_multiple_of(self.render_rate) {
                    action_send.send(Action::Render)?;
                }
            }
            Event::Crossterm(evt) => match evt {
                event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event).await?;
                }
                _ => {}
            },
            Event::App(app_evt) => self.handle_app_event(app_evt).await,
        }

        Ok(())
    }

    pub async fn handle_key_event(&mut self, key_event: &KeyEvent) -> anyhow::Result<()> {
        let action_send = self.action_send.clone();
        match self.app_state {
            AppState::InputGetMacForWol => self
                .mac_input
                .handle_key_event(key_event, action_send.clone())?,
            _ => {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => action_send.send(Action::Quit)?,
                    KeyCode::Char('c') | KeyCode::Char('C')
                        if key_event.modifiers == KeyModifiers::CONTROL =>
                    {
                        action_send.send(Action::Quit)?
                    }
                    KeyCode::Char('w') => {
                        if self.wol_mac.is_some() {
                            action_send.send(Action::SendWOLPacket)?
                        } else {
                            action_send.send(Action::OpenWolInputPopup)?
                        }
                    }
                    _ => {}
                };
            }
        }
        Ok(())
    }

    async fn handle_action(&mut self, action: &Action, tui: &mut Tui) -> anyhow::Result<()> {
        if action != &Action::Tick && action != &Action::Render {
            tracing::debug!("Rx action: {action:?}");
        }

        match action {
            Action::Tick => {
                self.curr_tick = self.curr_tick.wrapping_add(1);
            }
            Action::OpenWolInputPopup => {
                self.app_state = AppState::InputGetMacForWol;
            }
            Action::CloseWolInputPopup => {
                self.app_state = AppState::Idle;
            }
            Action::SetMacAddr(addr) => {
                self.wol_mac = Some(*addr);
                if self.app_state == AppState::InputGetMacForWol {
                    self.action_send.send(Action::CloseWolInputPopup)?
                }
            }
            Action::SendWOLPacket => {
                self.app_state = AppState::Waking;
                if let Some(server_mac) = self.wol_mac {
                    self.try_wake(&server_mac).await?;
                }
            }
            Action::Render => {
                self.draw(tui)?;
            }
            Action::Quit => self.should_quit = true,
            Action::None => {}
        }

        Ok(())
    }

    async fn handle_app_event(&mut self, app_event: &AppEvent) {
        todo!()
    }

    async fn try_wake(&mut self, server_mac: &MacAddr6) -> anyhow::Result<()> {
        match send_wol_packet(server_mac).await {
            Ok(_) => {
                debug!("Packet sent");
                Ok(())
            }
            Err(e) => {
                tracing::error!("Packet Error: {}", e);
                Err(e)
            }
        }
    }
}
