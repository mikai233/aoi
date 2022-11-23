use std::fmt::{Debug, Display, Formatter};

use crate::tick::ScheduleEvent;

#[derive(Clone, Debug)]
pub struct ReceiveTimeoutEvent;

impl Display for ReceiveTimeoutEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReceiveTimeoutEvent")
    }
}

impl ScheduleEvent for ReceiveTimeoutEvent {}