use crate::sentinel;
use crate::sentinel::timing::{self, Timing};

#[derive(Debug, Clone, Default)]
pub struct Timeline {
    events: Vec<sentinel::Event>,
}

impl Timeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: sentinel::Event) {
        self.events.push(event);
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
