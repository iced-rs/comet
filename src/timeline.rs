use crate::sentinel;
use crate::sentinel::timing::{self, Timing};

use std::collections::VecDeque;
use std::ops::RangeInclusive;

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

    pub fn range(&self) -> RangeInclusive<Index> {
        Index(0)..=Index(self.events.len())
    }

    pub fn is_live(&self, index: Index) -> bool {
        self.events.len() == index.0
    }

    pub fn push(&mut self, event: sentinel::Event) -> Index {
        self.events.push_back(event);

        if self.events.len() > Self::MAX_SIZE {
            self.events.pop_front();
        }

        Index(self.len())
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn timings<'a>(
        &'a self,
        stage: &'a timing::Stage,
        index: Index,
    ) -> impl DoubleEndedIterator<Item = &Timing> + Clone + 'a {
        self.events
            .iter()
            .rev()
            .skip(self.events.len().saturating_sub(index.0))
            .filter_map(move |event| match event {
                sentinel::Event::TimingMeasured(timing) if &timing.stage == stage => Some(timing),
                _ => None,
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Index(usize);

impl From<u8> for Index {
    fn from(n: u8) -> Self {
        Self(usize::from(n))
    }
}

impl From<Index> for f64 {
    fn from(index: Index) -> Self {
        index.0 as Self
    }
}

impl num_traits::FromPrimitive for Index {
    fn from_i64(n: i64) -> Option<Self> {
        if n < 0 {
            None
        } else {
            Some(Self(n as usize))
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(Self(n as usize))
    }
}
