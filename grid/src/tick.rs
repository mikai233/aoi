use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Display};
use std::time::Duration;

use tokio::task::JoinHandle;

use crate::message::{EventMessage, EventMessageReceiver, EventMessageSender};

pub struct Ticker {
    events: HashMap<String, JoinHandle<()>>,
    event_sender: EventMessageSender,
    event_receiver: EventMessageReceiver,
}

impl Ticker {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = tokio::sync::mpsc::unbounded_channel();
        Self {
            events: HashMap::new(),
            event_sender,
            event_receiver,
        }
    }

    pub fn schedule_once(&mut self, delay: Duration, key: String, event: Box<dyn ScheduleEvent>) {
        let sender = self.event_sender.clone();
        let j = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let _ = sender.send(EventMessage(event));
        });
        self.events.insert(key, j);
    }

    pub fn cancel(&mut self, key: String) -> bool {
        if let Some(j) = self.events.remove(&key) {
            j.abort();
            true
        } else {
            false
        }
    }

    pub async fn handle_event(&mut self) -> Option<EventMessage> {
        self.event_receiver.recv().await
    }
}

pub trait ScheduleEvent: EventClone + Any + fmt::Debug + fmt::Display + Send + Sync + 'static {}

impl Clone for Box<dyn ScheduleEvent> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait EventClone {
    fn clone_box(&self) -> Box<dyn ScheduleEvent>;
}

impl<T> EventClone for T where T: 'static + Clone + ScheduleEvent {
    fn clone_box(&self) -> Box<dyn ScheduleEvent> {
        Box::new(self.clone())
    }
}