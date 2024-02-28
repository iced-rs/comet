use crate::sentinel;
use crate::sentinel::timing::{self, Timing};

use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
pub struct Timeline {
    events: VecDeque<sentinel::Event>,
}

impl Timeline {
    // TODO: Make configurable
    const MAX_SIZE: usize = 1_000_000;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: sentinel::Event) {
        self.events.push_back(event);

        if self.events.len() > Self::MAX_SIZE {
            self.events.pop_front();
        }
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn timings<'a>(
        &'a self,
        stage: &'a timing::Stage,
    ) -> impl DoubleEndedIterator<Item = &Timing> + Clone + 'a {
        self.events.iter().filter_map(move |event| match event {
            sentinel::Event::TimingMeasured(timing) if &timing.stage == stage => Some(timing),
            _ => None,
        })
    }
}
