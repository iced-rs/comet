use crate::beacon;
use crate::beacon::span;
use crate::core::time::{Duration, SystemTime};

use std::collections::VecDeque;
use std::ops::RangeInclusive;

#[derive(Debug, Clone, Default)]
pub struct Timeline {
    events: VecDeque<beacon::Event>,
}

impl Timeline {
    // TODO: Make configurable
    const MAX_SIZE: usize = 1_000_000;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn capacity(&self) -> usize {
        Self::MAX_SIZE
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn range(&self) -> RangeInclusive<Index> {
        Index(0)..=Index(self.events.len())
    }

    pub fn is_live(&self, index: Index) -> bool {
        self.events.len() == index.0
    }

    pub fn push(&mut self, event: beacon::Event) -> Index {
        self.events.push_back(event);

        if self.events.len() > Self::MAX_SIZE {
            self.events.pop_front();
        }

        Index(self.len())
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn seek(
        &self,
        index: Index,
    ) -> impl DoubleEndedIterator<Item = &beacon::Event> + Clone + '_ {
        self.events
            .iter()
            .rev()
            .skip(self.events.len().saturating_sub(index.0))
    }

    pub fn timeframes<'a>(
        &'a self,
        index: Index,
        stage: &'a span::Stage,
    ) -> impl DoubleEndedIterator<Item = Timeframe> + Clone + '_ {
        self.seek(index).filter_map(move |event| match event {
            beacon::Event::SpanFinished { at, duration, span } if &span.stage() == stage => {
                Some(Timeframe {
                    at: *at,
                    duration: *duration,
                })
            }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeframe {
    pub at: SystemTime,
    pub duration: Duration,
}
