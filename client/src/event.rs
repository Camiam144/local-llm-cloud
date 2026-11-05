use futures::{FutureExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;

use crossterm::event::{Event as CrosstermEvent, EventStream};

use crate::app::AppEvent;

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Crossterm(CrosstermEvent),
    App(AppEvent),
}

#[allow(dead_code)]
pub struct EventHandler {
    sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Construct a new event handler.
    /// This handler runs at the given tick rate in ms. For example, passing
    /// a value of 250 will make the event loop send a tick event every
    /// 250 milliseconds.
    pub fn new(tick_rate: u64) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = EventTask::new(sender.clone());
        tokio::spawn(async move { actor.run(tick_rate).await });
        Self { sender, receiver }
    }

    /// Receive an event from the sender.
    ///
    /// This will block until an event is received.
    ///
    /// # Errors
    ///
    /// This function returns an error if the sender channel is disconnected.
    /// This can happen if an error occurs in the event thread. In practice, this
    /// shouldn't happen unless there's a problem with the underlying terminal.
    pub async fn next(&mut self) -> anyhow::Result<Event> {
        // trace!("event receiver receiving");
        self.receiver
            .recv()
            .await
            .ok_or(anyhow::anyhow!("Failed to receive event."))
    }

    /// Queue an event to be sent to the receiver.
    pub fn send(&mut self, app_event: AppEvent) {
        // Ignore the result because if we shut down the app the receiever will
        // be dropped and this will fail but that's expected behavior.
        let _ = self.sender.send(Event::App(app_event));
    }
}

/// A thread that handles the crossterm and tick events.
struct EventTask {
    /// Event transmitter channel
    sender: mpsc::UnboundedSender<Event>,
}

impl EventTask {
    fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self { sender }
    }
    async fn run(self, tick_rate: u64) {
        let reader = EventStream::new();
        let tick_rate = Duration::from_millis(tick_rate);
        let mut tick = tokio::time::interval(tick_rate);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        tokio::pin!(reader);

        loop {
            let tick_delay = tick.tick().fuse();
            let maybe_ct_event = reader.next().fuse();
            tokio::select! {
                _ = self.sender.closed() => {break;}
                _ = tick_delay => { self.send(Event::Tick);}
                Some(Ok(e)) = maybe_ct_event => {
                    self.send(Event::Crossterm(e));
                }
            };
        }
    }

    fn send(&self, event: Event) {
        let _ = self.sender.send(event);
    }
}
